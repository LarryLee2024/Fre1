use crate::shared::validation::ValidationResult;

#[test]
fn valid_state() {
    let result: ValidationResult<i32> = ValidationResult::Valid(42);
    assert!(result.is_valid());
    assert!(!result.is_invalid());
    assert!(result.errors().is_empty());
}

#[test]
fn invalid_state() {
    let err = crate::shared::validation::ValidationError::new("something wrong");
    let result: ValidationResult<i32> = ValidationResult::Invalid(vec![err]);
    assert!(!result.is_valid());
    assert!(result.is_invalid());
    assert_eq!(result.errors().len(), 1);
}

#[test]
fn invalid_with_multiple_errors() {
    let result: ValidationResult<i32> = ValidationResult::Invalid(vec![
        crate::shared::validation::ValidationError::new("error one"),
        crate::shared::validation::ValidationError::new("error two"),
    ]);
    assert_eq!(result.errors().len(), 2);
}

#[test]
fn into_result_returns_ok_for_valid() {
    let result: ValidationResult<i32> = ValidationResult::Valid(42);
    let std_result: Result<i32, Vec<crate::shared::validation::ValidationError>> =
        result.into_result();
    assert_eq!(std_result, Ok(42));
}

#[test]
fn into_result_returns_err_for_invalid() {
    let err = crate::shared::validation::ValidationError::new("oops");
    let result: ValidationResult<i32> = ValidationResult::Invalid(vec![err]);
    let std_result: Result<i32, Vec<crate::shared::validation::ValidationError>> =
        result.into_result();
    assert!(std_result.is_err());
}

#[test]
fn from_conversion_to_std_result() {
    let result: ValidationResult<i32> = ValidationResult::Valid(99);
    let std_result: Result<i32, Vec<crate::shared::validation::ValidationError>> = result.into();
    assert_eq!(std_result, Ok(99));
}

#[test]
#[should_panic(expected = "ValidationResult::unwrap()")]
fn unwrap_panics_on_invalid() {
    let result: ValidationResult<i32> =
        ValidationResult::Invalid(vec![crate::shared::validation::ValidationError::new("fail")]);
    let _ = result.unwrap();
}

#[test]
fn unwrap_returns_value_on_valid() {
    let result: ValidationResult<i32> = ValidationResult::Valid(42);
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn map_transforms_valid_value() {
    let result: ValidationResult<i32> = ValidationResult::Valid(21);
    let mapped = result.map(|v| v * 2);
    assert_eq!(mapped, ValidationResult::Valid(42));
}

#[test]
fn map_preserves_invalid() {
    let err = crate::shared::validation::ValidationError::new("oops");
    let result: ValidationResult<i32> = ValidationResult::Invalid(vec![err]);
    let mapped = result.map(|v| v * 2);
    assert!(mapped.is_invalid());
    assert_eq!(mapped.errors().len(), 1);
}

#[test]
fn map_err_transforms_errors() {
    let result: ValidationResult<i32> = ValidationResult::Invalid(vec![
        crate::shared::validation::ValidationError::new("original"),
    ]);
    let mapped = result.map_err(|mut errs| {
        errs.push(crate::shared::validation::ValidationError::new("extra"));
        errs
    });
    assert_eq!(mapped.errors().len(), 2);
}

#[test]
fn from_iter_collects_all_valid() {
    let results = vec![
        ValidationResult::Valid(1),
        ValidationResult::Valid(2),
        ValidationResult::Valid(3),
    ];
    let collected: ValidationResult<Vec<i32>> = results.into_iter().collect();
    assert_eq!(collected, ValidationResult::Valid(vec![1, 2, 3]));
}

#[test]
fn from_iter_aggregates_errors() {
    let results = vec![
        ValidationResult::Valid(1),
        ValidationResult::Invalid(vec![crate::shared::validation::ValidationError::new(
            "error at position 2",
        )]),
        ValidationResult::Valid(3),
        ValidationResult::Invalid(vec![crate::shared::validation::ValidationError::new(
            "error at position 4",
        )]),
    ];
    let collected: ValidationResult<Vec<i32>> = results.into_iter().collect();
    assert!(collected.is_invalid());
    assert_eq!(collected.errors().len(), 2);
}

#[test]
fn from_iter_empty_yields_valid_empty_vec() {
    let results: Vec<ValidationResult<i32>> = vec![];
    let collected: ValidationResult<Vec<i32>> = results.into_iter().collect();
    assert_eq!(collected, ValidationResult::Valid(vec![]));
}
