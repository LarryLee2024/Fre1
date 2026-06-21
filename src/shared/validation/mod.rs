//! 链式校验工具
//!
//! 零业务语义的通用校验基础设施。
//! 提供链式校验器（ValidationChain）和简单的内建校验器。
//!
//! 与 `content/` 层配置校验系统不同——本模块是通用的纯 Rust 校验工具，
//! 任何领域模块均可用于业务规则校验。
//!
//! # 核心类型
//!
//! | 类型 | 用途 |
//! |------|------|
//! | [`ValidationResult<T>`] | 支持 Valid/Invalid 双态的校验结果 |
//! | [`ValidationError`]     | 含字段名 + 错误消息的校验错误 |
//! | [`ValidationChain<T>`]  | 链式校验器建造者 |
//! | [`Validator<T>`]        | 自定义校验器 trait |
//!
//! # 内建校验器
//!
//! | 校验器 | 用途 |
//! |--------|------|
//! | [`NotEmpty`]  | 验证字符串 / 切片不为空 |
//! | [`Range<T>`]  | 验证值在 `[min, max]` 闭区间内 |
//! | [`MinLength`] | 验证最小长度 |

use std::fmt;
use std::iter::FromIterator;

// ═══════════════════════════════════════════════════════════════════════════
// ValidationError
// ═══════════════════════════════════════════════════════════════════════════

/// 校验错误。
///
/// 包含可选的字段名和错误描述消息。字段名用于关联校验失败的具体字段，
/// 错误消息用于向用户展示。
///
/// # 构造
///
/// ```ignore
/// // 仅消息
/// let err = ValidationError::new("value out of range");
///
/// // 带字段名
/// let err = ValidationError::with_field("level", "must be at least 1");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    /// 校验失败的字段名（可选）。
    pub field: Option<String>,
    /// 错误描述消息。
    pub message: String,
}

impl ValidationError {
    /// 创建仅含消息的校验错误（无字段名）。
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            field: None,
            message: message.into(),
        }
    }

    /// 创建带字段名的校验错误。
    pub fn with_field(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: Some(field.into()),
            message: message.into(),
        }
    }

    /// 返回字段名的引用（如有）。
    pub fn field(&self) -> Option<&str> {
        self.field.as_deref()
    }

    /// 返回错误消息的引用。
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.field {
            Some(field) => write!(f, "[{field}] {}", self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ValidationResult
// ═══════════════════════════════════════════════════════════════════════════

/// 校验结果。
///
/// 区别于 Rust 标准 `Result<T, E>`，`Invalid` 状态下携带**所有**累积错误。
///
/// # 语义
///
/// - `Valid(T)` —— 校验通过，包含原始值
/// - `Invalid(Vec<ValidationError>)` —— 校验失败，包含所有累积错误
///
/// # 示例
///
/// ```ignore
/// let result = validate_something(value);
/// if result.is_valid() {
///     let val = result.into_result().unwrap();
///     println!("valid: {val}");
/// } else {
///     for err in result.errors() {
///         println!("error: {err}");
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult<T> {
    /// 校验通过，包含通过校验的值。
    Valid(T),
    /// 校验失败，包含所有累积错误。
    Invalid(Vec<ValidationError>),
}

impl<T> ValidationResult<T> {
    /// 校验是否通过。
    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationResult::Valid(_))
    }

    /// 校验是否失败。
    pub fn is_invalid(&self) -> bool {
        matches!(self, ValidationResult::Invalid(_))
    }

    /// 获取错误列表。校验通过时返回空切片。
    pub fn errors(&self) -> &[ValidationError] {
        match self {
            ValidationResult::Valid(_) => &[],
            ValidationResult::Invalid(errs) => errs.as_slice(),
        }
    }

    /// 转换为标准 `Result<T, Vec<ValidationError>>`。
    ///
    /// - `Valid(v)` → `Ok(v)`
    /// - `Invalid(e)` → `Err(e)`
    pub fn into_result(self) -> Result<T, Vec<ValidationError>> {
        match self {
            ValidationResult::Valid(val) => Ok(val),
            ValidationResult::Invalid(errs) => Err(errs),
        }
    }

    /// 消费 self，解包出 T。
    ///
    /// # Panics
    ///
    /// 当 `self` 为 `Invalid` 时 panic，附带错误详情。
    pub fn unwrap(self) -> T {
        match self {
            ValidationResult::Valid(val) => val,
            ValidationResult::Invalid(errs) => {
                panic!("called `ValidationResult::unwrap()` on an `Invalid` value: {errs:?}")
            }
        }
    }

    /// 将 Valid 中的值通过映射函数转换，Invalid 保持不变。
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> ValidationResult<U> {
        match self {
            ValidationResult::Valid(val) => ValidationResult::Valid(f(val)),
            ValidationResult::Invalid(errs) => ValidationResult::Invalid(errs),
        }
    }

    /// 将 Valid 中的值和错误类型同时映射。
    pub fn map_err<F: FnOnce(Vec<ValidationError>) -> Vec<ValidationError>>(
        self,
        f: F,
    ) -> ValidationResult<T> {
        match self {
            ValidationResult::Valid(val) => ValidationResult::Valid(val),
            ValidationResult::Invalid(errs) => ValidationResult::Invalid(f(errs)),
        }
    }
}

impl<T> From<ValidationResult<T>> for Result<T, Vec<ValidationError>> {
    fn from(result: ValidationResult<T>) -> Self {
        result.into_result()
    }
}

impl<T> FromIterator<ValidationResult<T>> for ValidationResult<Vec<T>> {
    /// 从多个 [`ValidationResult`] 迭代器聚合为单个结果。
    ///
    /// - 所有元素均为 `Valid` → 收集为 `Valid(Vec<T>)`
    /// - 任一元素为 `Invalid` → 聚合所有错误为 `Invalid(Vec<ValidationError>)`
    fn from_iter<I: IntoIterator<Item = ValidationResult<T>>>(iter: I) -> Self {
        let mut items = Vec::new();
        let mut errors = Vec::new();

        for result in iter {
            match result {
                ValidationResult::Valid(item) => items.push(item),
                ValidationResult::Invalid(errs) => errors.extend(errs),
            }
        }

        if errors.is_empty() {
            ValidationResult::Valid(items)
        } else {
            ValidationResult::Invalid(errors)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Validator Trait
// ═══════════════════════════════════════════════════════════════════════════

/// 校验器 trait。
///
/// 定义对类型 `T` 的单条校验规则。通过 [`ValidationChain`] 可将多个校验器
/// 串联为校验管道，累积所有错误后统一返回。
///
/// # 内建实现
///
/// | 校验器 | 校验类型 |
/// |--------|----------|
/// | [`NotEmpty`]  | [`str`]、[`String`]、切片、[`Vec`] |
/// | [`Range<T>`]  | 实现 `PartialOrd + Display` 的类型 |
/// | [`MinLength`] | [`str`]、[`String`]、切片、[`Vec`] |
///
/// # 自定义校验器示例
///
/// ```ignore
/// struct Positive;
///
/// impl Validator<i32> for Positive {
///     type Error = String;
///     fn validate(&self, value: &i32) -> Result<(), Self::Error> {
///         if *value <= 0 {
///             Err("value must be positive".into())
///         } else {
///             Ok(())
///         }
///     }
/// }
/// ```
pub trait Validator<T: ?Sized> {
    /// 校验失败时的错误类型。
    type Error: fmt::Display;

    /// 执行校验。
    ///
    /// - `Ok(())` — 校验通过
    /// - `Err(e)` — 校验失败，返回错误
    fn validate(&self, value: &T) -> Result<(), Self::Error>;
}

// ═══════════════════════════════════════════════════════════════════════════
// ValidationChain
// ═══════════════════════════════════════════════════════════════════════════

/// 链式校验器建造者。
///
/// 串联执行多个校验规则，**累积**所有错误后统一返回。
/// 不短路——即使某个校验失败，仍会继续执行后续校验器。
///
/// # 泛型参数
///
/// - `T` — 被校验的值类型
/// - `E` — 校验错误类型（必须实现 [`fmt::Display`]）
///
/// # 使用
///
/// ```ignore
/// use fre_shared::validation::{ValidationChain, NotEmpty, MinLength};
///
/// let result = ValidationChain::new("hello")
///     .check(NotEmpty)
///     .check(MinLength::new(2))
///     .validate();
///
/// assert!(result.is_ok());
/// ```
#[derive(Debug)]
pub struct ValidationChain<T, E = ValidationError> {
    /// 被校验的值。
    value: T,
    /// 累积的校验错误。
    errors: Vec<E>,
}

impl<T, E> ValidationChain<T, E> {
    /// 创建新链，包装待校验的值。
    pub fn new(value: T) -> Self {
        Self {
            value,
            errors: Vec::new(),
        }
    }

    /// 添加并执行一个校验器。
    ///
    /// 校验失败时累积错误，继续执行后续校验器。
    pub fn check<V: Validator<T, Error = E>>(mut self, validator: V) -> Self {
        if let Err(e) = validator.validate(&self.value) {
            self.errors.push(e);
        }
        self
    }

    /// 获取被校验值的引用。
    pub fn value(&self) -> &T {
        &self.value
    }

    /// 获取当前累积的错误列表。
    pub fn errors(&self) -> &[E] {
        &self.errors
    }
}

impl<T, E: fmt::Display> ValidationChain<T, E> {
    /// 最终校验，返回标准 `Result<T, Vec<E>>`。
    ///
    /// - 无错误 → `Ok(value)`
    /// - 有错误 → `Err(errors)`
    pub fn validate(self) -> Result<T, Vec<E>> {
        if self.errors.is_empty() {
            Ok(self.value)
        } else {
            Err(self.errors)
        }
    }

    /// 最终校验，返回 [`ValidationResult<T>`]。
    ///
    /// 将 `E` 通过 `Display` 转换为 [`ValidationError`]，
    /// 合并为统一的校验结果类型。
    pub fn validate_all(self) -> ValidationResult<T> {
        if self.errors.is_empty() {
            ValidationResult::Valid(self.value)
        } else {
            let errs = self
                .errors
                .into_iter()
                .map(|e| ValidationError::new(e.to_string()))
                .collect();
            ValidationResult::Invalid(errs)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// NotEmpty — 非空校验器
// ═══════════════════════════════════════════════════════════════════════════

/// 非空校验器。
///
/// 验证字符串或切片不为空。支持 [`str`]、[`String`]、切片 `[T]`、[`Vec<T>`]。
///
/// # 示例
///
/// ```ignore
/// assert!(NotEmpty.validate("hello").is_ok());
/// assert!(NotEmpty.validate("").is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotEmpty;

impl Validator<str> for NotEmpty {
    type Error = ValidationError;

    fn validate(&self, value: &str) -> Result<(), Self::Error> {
        if value.is_empty() {
            Err(ValidationError::new("value must not be empty"))
        } else {
            Ok(())
        }
    }
}

impl Validator<String> for NotEmpty {
    type Error = ValidationError;

    fn validate(&self, value: &String) -> Result<(), Self::Error> {
        if value.is_empty() {
            Err(ValidationError::new("value must not be empty"))
        } else {
            Ok(())
        }
    }
}

impl Validator<&str> for NotEmpty {
    type Error = ValidationError;

    fn validate(&self, value: &&str) -> Result<(), Self::Error> {
        if value.is_empty() {
            Err(ValidationError::new("value must not be empty"))
        } else {
            Ok(())
        }
    }
}

impl<T> Validator<[T]> for NotEmpty {
    type Error = ValidationError;

    fn validate(&self, value: &[T]) -> Result<(), Self::Error> {
        if value.is_empty() {
            Err(ValidationError::new("collection must not be empty"))
        } else {
            Ok(())
        }
    }
}

impl<T> Validator<Vec<T>> for NotEmpty {
    type Error = ValidationError;

    fn validate(&self, value: &Vec<T>) -> Result<(), Self::Error> {
        if value.is_empty() {
            Err(ValidationError::new("collection must not be empty"))
        } else {
            Ok(())
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Range — 范围校验器
// ═══════════════════════════════════════════════════════════════════════════

/// 范围校验器。
///
/// 验证值是否在 `[min, max]` 闭区间内。适用于实现了 [`PartialOrd`] 和 [`Display`] 的类型。
///
/// # 示例
///
/// ```ignore
/// let range = Range::new(0, 100);
/// assert!(range.validate(&50).is_ok());
/// assert!(range.validate(&(-1)).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Range<T: PartialOrd> {
    /// 区间最小值（包含）。
    pub min: T,
    /// 区间最大值（包含）。
    pub max: T,
}

impl<T: PartialOrd + std::fmt::Debug> Range<T> {
    /// 创建 `[min, max]` 闭区间校验器。
    ///
    /// # Panics
    ///
    /// 当 `min > max` 时 panic。
    pub fn new(min: T, max: T) -> Self {
        assert!(
            min <= max,
            "Range: min must be <= max, got min={min:?} max={max:?}"
        );
        Self { min, max }
    }
}

impl<T: PartialOrd + fmt::Display> Validator<T> for Range<T> {
    type Error = ValidationError;

    fn validate(&self, value: &T) -> Result<(), Self::Error> {
        if *value < self.min || *value > self.max {
            Err(ValidationError::new(format!(
                "value must be between {} and {}",
                self.min, self.max
            )))
        } else {
            Ok(())
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MinLength — 最小长度校验器
// ═══════════════════════════════════════════════════════════════════════════

/// 最小长度校验器。
///
/// 验证字符串或切片的长度是否达到指定最小值。
/// 支持 [`str`]、[`String`]、切片、[`Vec<T>`]。
///
/// # 示例
///
/// ```ignore
/// let min_len = MinLength::new(3);
/// assert!(min_len.validate("abc").is_ok());
/// assert!(min_len.validate("ab").is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinLength {
    /// 最小长度。
    pub min: usize,
}

impl MinLength {
    /// 创建最小长度校验器。
    pub fn new(min: usize) -> Self {
        Self { min }
    }
}

impl Validator<str> for MinLength {
    type Error = ValidationError;

    fn validate(&self, value: &str) -> Result<(), Self::Error> {
        if value.len() < self.min {
            Err(ValidationError::new(format!(
                "length must be at least {}, got {}",
                self.min,
                value.len()
            )))
        } else {
            Ok(())
        }
    }
}

impl Validator<String> for MinLength {
    type Error = ValidationError;

    fn validate(&self, value: &String) -> Result<(), Self::Error> {
        if value.len() < self.min {
            Err(ValidationError::new(format!(
                "length must be at least {}, got {}",
                self.min,
                value.len()
            )))
        } else {
            Ok(())
        }
    }
}

impl Validator<&str> for MinLength {
    type Error = ValidationError;

    fn validate(&self, value: &&str) -> Result<(), Self::Error> {
        if value.len() < self.min {
            Err(ValidationError::new(format!(
                "length must be at least {}, got {}",
                self.min,
                value.len()
            )))
        } else {
            Ok(())
        }
    }
}

impl<T> Validator<[T]> for MinLength {
    type Error = ValidationError;

    fn validate(&self, value: &[T]) -> Result<(), Self::Error> {
        if value.len() < self.min {
            Err(ValidationError::new(format!(
                "length must be at least {}, got {}",
                self.min,
                value.len()
            )))
        } else {
            Ok(())
        }
    }
}

impl<T> Validator<Vec<T>> for MinLength {
    type Error = ValidationError;

    fn validate(&self, value: &Vec<T>) -> Result<(), Self::Error> {
        if value.len() < self.min {
            Err(ValidationError::new(format!(
                "length must be at least {}, got {}",
                self.min,
                value.len()
            )))
        } else {
            Ok(())
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests;
