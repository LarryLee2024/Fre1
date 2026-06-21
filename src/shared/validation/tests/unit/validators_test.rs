use crate::shared::validation::{MinLength, NotEmpty, Range, ValidationError, Validator};

// ─── NotEmpty ────────────────────────────────────────────────────────────

#[test]
fn not_empty_str_valid() {
    assert!(NotEmpty.validate("hello").is_ok());
}

#[test]
fn not_empty_str_invalid() {
    let result = NotEmpty.validate("");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "value must not be empty"
    );
}

#[test]
fn not_empty_string_valid() {
    let s = String::from("hello");
    assert!(NotEmpty.validate(&s).is_ok());
}

#[test]
fn not_empty_string_invalid() {
    let s = String::new();
    let result = NotEmpty.validate(&s);
    assert!(result.is_err());
}

#[test]
fn not_empty_slice_valid() {
    let data = [1, 2, 3];
    assert!(NotEmpty.validate(data.as_slice()).is_ok());
}

#[test]
fn not_empty_slice_invalid() {
    let data: [i32; 0] = [];
    let result = NotEmpty.validate(data.as_slice());
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "collection must not be empty"
    );
}

#[test]
fn not_empty_vec_valid() {
    let v = vec![1, 2, 3];
    assert!(NotEmpty.validate(&v).is_ok());
}

#[test]
fn not_empty_vec_invalid() {
    let v: Vec<i32> = vec![];
    let result = NotEmpty.validate(&v);
    assert!(result.is_err());
}

// ─── Range ───────────────────────────────────────────────────────────────

#[test]
fn range_int_valid_within_bounds() {
    let range = Range::new(0, 100);
    assert!(range.validate(&50).is_ok());
}

#[test]
fn range_int_valid_at_min_bound() {
    let range = Range::new(0, 100);
    assert!(range.validate(&0).is_ok());
}

#[test]
fn range_int_valid_at_max_bound() {
    let range = Range::new(0, 100);
    assert!(range.validate(&100).is_ok());
}

#[test]
fn range_int_invalid_below_min() {
    let range = Range::new(0, 100);
    let result = range.validate(&(-1));
    assert!(result.is_err());
}

#[test]
fn range_int_invalid_above_max() {
    let range = Range::new(0, 100);
    let result = range.validate(&101);
    assert!(result.is_err());
}

#[test]
fn range_float_valid() {
    let range = Range::new(0.0, 1.0);
    assert!(range.validate(&0.5).is_ok());
}

#[test]
fn range_float_invalid() {
    let range = Range::new(0.0, 1.0);
    assert!(range.validate(&1.5).is_err());
}

#[test]
fn range_error_message_contains_bounds() {
    let range = Range::new(0, 100);
    let err = range.validate(&200).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("0"), "error should mention min bound");
    assert!(msg.contains("100"), "error should mention max bound");
}

// ─── MinLength ───────────────────────────────────────────────────────────

#[test]
fn min_length_str_valid() {
    let checker = MinLength::new(3);
    assert!(checker.validate("abc").is_ok());
}

#[test]
fn min_length_str_exact_match() {
    let checker = MinLength::new(3);
    assert!(checker.validate("abc").is_ok());
}

#[test]
fn min_length_str_invalid() {
    let checker = MinLength::new(5);
    let result = checker.validate("ab");
    assert!(result.is_err());
}

#[test]
fn min_length_str_error_message() {
    let checker = MinLength::new(5);
    let err = checker.validate("ab").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("5"), "error should mention minimum length");
    assert!(msg.contains("2"), "error should mention actual length");
}

#[test]
fn min_length_string_valid() {
    let s = String::from("hello");
    let checker = MinLength::new(3);
    assert!(checker.validate(&s).is_ok());
}

#[test]
fn min_length_string_invalid() {
    let s = String::from("hi");
    let checker = MinLength::new(5);
    assert!(checker.validate(&s).is_err());
}

#[test]
fn min_length_slice_valid() {
    let data = [1, 2, 3, 4, 5];
    let checker = MinLength::new(3);
    assert!(checker.validate(data.as_slice()).is_ok());
}

#[test]
fn min_length_slice_invalid() {
    let data = [1, 2];
    let checker = MinLength::new(5);
    assert!(checker.validate(data.as_slice()).is_err());
}

#[test]
fn min_length_vec_valid() {
    let v = vec![1, 2, 3];
    let checker = MinLength::new(2);
    assert!(checker.validate(&v).is_ok());
}

#[test]
fn min_length_vec_invalid() {
    let v: Vec<i32> = vec![1];
    let checker = MinLength::new(5);
    assert!(checker.validate(&v).is_err());
}

// ─── ValidationError ─────────────────────────────────────────────────────

#[test]
fn validation_error_new() {
    let err = ValidationError::new("something went wrong");
    assert_eq!(err.field, None);
    assert_eq!(err.message, "something went wrong");
}

#[test]
fn validation_error_with_field() {
    let err = ValidationError::with_field("level", "must be at least 1");
    assert_eq!(err.field(), Some("level"));
    assert_eq!(err.message(), "must be at least 1");
}

#[test]
fn validation_error_display_without_field() {
    let err = ValidationError::new("test error");
    assert_eq!(err.to_string(), "test error");
}

#[test]
fn validation_error_display_with_field() {
    let err = ValidationError::with_field("name", "too short");
    assert_eq!(err.to_string(), "[name] too short");
}
