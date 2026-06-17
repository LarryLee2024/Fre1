use crate::shared::error::*;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
struct TestError(String);

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for TestError {}

#[test]
fn domain_error_adds_tag() {
    let result: Result<i32, TestError> = Err(TestError("missing target".into()));
    let err = result.domain("combat").unwrap_err();
    assert_eq!(err.domain, "combat");
    assert_eq!(err.source.0, "missing target");
    assert!(err.context.is_none());
}

#[test]
fn context_adds_message() {
    let result: Result<i32, TestError> = Err(TestError("roll failed".into()));
    let err = result
        .with_context("combat", "during crit check")
        .unwrap_err();
    assert_eq!(err.domain, "combat");
    assert_eq!(err.context.as_deref(), Some("during crit check"));
}

#[test]
fn ok_value_passthrough() {
    let result: Result<i32, TestError> = Ok(42);
    let val = result.domain("combat").unwrap();
    assert_eq!(val, 42);
}

#[test]
fn display_shows_domain_info() {
    let err = ErrorContext {
        domain: "inventory",
        source: TestError("bag full".into()),
        context: None,
    };
    let msg = err.to_string();
    assert!(msg.contains("[inventory]"));
    assert!(msg.contains("bag full"));
}

#[test]
fn display_shows_context_info() {
    let err = ErrorContext {
        domain: "combat",
        source: TestError("crit fail".into()),
        context: Some("during attack roll".into()),
    };
    let msg = err.to_string();
    assert!(msg.contains("crit fail"));
    assert!(msg.contains("during attack roll"));
}
