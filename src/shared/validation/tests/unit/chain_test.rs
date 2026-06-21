use crate::shared::validation::{MinLength, NotEmpty, Range, ValidationChain};

#[test]
fn chain_new_contains_value() {
    let chain = ValidationChain::new(42);
    assert_eq!(*chain.value(), 42);
    assert!(chain.errors().is_empty());
}

#[test]
fn chain_passes_with_validators_succeeding() {
    let result = ValidationChain::new(50)
        .check(Range::new(0, 100))
        .validate();
    assert_eq!(result, Ok(50));
}

#[test]
fn chain_fails_with_single_validator_failing() {
    let result = ValidationChain::new(150)
        .check(Range::new(0, 100))
        .validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().len(), 1);
}

#[test]
fn chain_accumulates_all_errors() {
    let result = ValidationChain::new(-5)
        .check(Range::new(0, 100))
        .check(Range::new(10, 50))
        .validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().len(), 2);
}

#[test]
fn chain_does_not_short_circuit_on_first_failure() {
    // Even though first validator fails, the chain continues and accumulates both errors.
    let result = ValidationChain::new("")
        .check(NotEmpty)
        .check(MinLength::new(3))
        .validate::<Vec<crate::shared::validation::ValidationError>>();
    // Both should fail because empty string fails both NotEmpty and MinLength
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().len(), 2);
}

#[test]
fn chain_validate_all_returns_valid() {
    let result = ValidationChain::new(50)
        .check(Range::new(0, 100))
        .validate_all();
    assert!(result.is_valid());
}

#[test]
fn chain_validate_all_returns_invalid() {
    let result = ValidationChain::new(500)
        .check(Range::new(0, 100))
        .validate_all();
    assert!(result.is_invalid());
}

#[test]
fn chain_validate_all_with_multiple_errors() {
    let result = ValidationChain::new(-1)
        .check(Range::new(0, 10))
        .check(Range::new(20, 30))
        .validate_all();
    assert!(result.is_invalid());
    assert_eq!(result.errors().len(), 2);
}

#[test]
fn chain_with_string_validators() {
    let result = ValidationChain::new("hello")
        .check(NotEmpty)
        .check(MinLength::new(2))
        .validate::<Vec<crate::shared::validation::ValidationError>>();
    assert_eq!(result, Ok("hello"));
}

#[test]
fn chain_with_different_error_types() {
    // Using ValidationError as the common error type
    let result = ValidationChain::new(42)
        .check(Range::new(0, 100))
        .check(Range::new(10, 50))
        .validate();
    assert_eq!(result, Ok(42));
}

#[test]
fn chain_value_accessor_reflects_wrapped_value() {
    let chain: ValidationChain<i32, crate::shared::validation::ValidationError> =
        ValidationChain::new(100);
    assert_eq!(*chain.value(), 100);
}

#[test]
fn chain_errors_accessor_returns_accumulated_count() {
    let chain = ValidationChain::new(-5)
        .check(Range::new(0, 100))
        .check(Range::new(10, 50));
    assert_eq!(chain.errors().len(), 2);
}
