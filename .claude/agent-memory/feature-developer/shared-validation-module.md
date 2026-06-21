---
name: shared-validation-module
description: Location and structure of the shared/validation/ module (chain validator utility, ValidationResult, Validator trait, built-in validators)
metadata:
  type: reference
---

Location: `src/shared/validation/mod.rs` — 605 lines, one file, no submodule split needed.

Provides:
- `ValidationError` — struct with optional `field` name + `message` string
- `ValidationResult<T>` — enum Valid(T) / Invalid(Vec<ValidationError>), with is_valid/is_invalid/errors/into_result/unwrap/map/map_err, From<ValidationResult<T>> for Result<T, Vec<ValidationError>>, and FromIterator<ValidationResult<T>> for ValidationResult<Vec<T>>
- `Validator<T: ?Sized>` trait — with `type Error: Display` and `fn validate(&self, value: &T)`
- `ValidationChain<T, E = ValidationError>` — builder with new/check/value/errors/validate/validate_all
- Built-in: `NotEmpty` (for str, String, &str, [T], Vec<T>), `Range<T: PartialOrd + Display>` (for [min,max] validation), `MinLength` (for str, String, &str, [T], Vec<T>)

Tests at `src/shared/validation/tests/unit/` with 3 test files: validation_result_test (128 lines), chain_test (108 lines), validators_test (216 lines).

Key design decisions:
- Default E = ValidationError for ergonomic chain usage
- Validator<T: ?Sized> enables str/unsized impls
- All built-in validators share ValidationError as Error type for chain composability
- Tests cannot run due to pre-existing compilation errors in other modules (hashing, collections)
