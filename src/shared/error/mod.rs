//! 错误上下文工具
//!
//! 零业务语义的错误基础设施。各领域使用自己的错误枚举。
//! 详见 `docs/00-governance/coding-rules.md` §错误处理
//!
//! # 核心类型
//! - [`ErrorContext<E>`]: 带领域标签的错误包装
//! - [`ContextExt`]: 为 `Result<T, E>` 提供 `.domain()` / `.with_context()` 扩展

use std::error::Error;
use std::fmt;

/// 带领域上下文的错误包装。
///
/// 用于在错误传播过程中附加来源领域标识和人类可读上下文，
/// 不引入全局 AppError 或 anyhow。
///
/// # 使用
///
/// ```ignore
/// use fre_shared::error::ContextExt;
///
/// fn do_combat() -> Result<(), ErrorContext<CombatError>> {
///     let result = risky_operation().domain("combat")?;
///     let result2 = other_op().with_context("combat", "during damage calculation")?;
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct ErrorContext<E> {
    /// 来源领域标识（如 "combat"、"inventory"）
    pub domain: &'static str,

    /// 原始错误
    pub source: E,

    /// 额外上下文说明
    pub context: Option<String>,
}

impl<E: fmt::Display> fmt::Display for ErrorContext<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.domain, self.source)?;
        if let Some(ctx) = &self.context {
            write!(f, " ({})", ctx)?;
        }
        Ok(())
    }
}

impl<E: Error + 'static> Error for ErrorContext<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

/// 为 `Result<T, E>` 提供添加领域上下文的扩展 trait。
///
/// 类似 anyhow::Context 但限域到领域标签，不引入全局错误类型。
///
/// # 方法
///
/// | 方法 | 用途 |
/// |------|------|
/// | `.domain(tag)` | 添加领域标签 |
/// | `.with_context(tag, msg)` | 添加领域标签和上下文信息 |
pub trait ContextExt<T, E> {
    /// 添加领域标签，将 `Err(e)` 转换为 `Err(ErrorContext { domain: tag, source: e })`。
    fn domain(self, tag: &'static str) -> Result<T, ErrorContext<E>>;

    /// 添加领域标签和上下文信息。
    fn with_context(self, tag: &'static str, msg: impl Into<String>) -> Result<T, ErrorContext<E>>;
}

impl<T, E> ContextExt<T, E> for Result<T, E> {
    fn domain(self, tag: &'static str) -> Result<T, ErrorContext<E>> {
        self.map_err(|e| ErrorContext {
            domain: tag,
            source: e,
            context: None,
        })
    }

    fn with_context(self, tag: &'static str, msg: impl Into<String>) -> Result<T, ErrorContext<E>> {
        self.map_err(|e| ErrorContext {
            domain: tag,
            source: e,
            context: Some(msg.into()),
        })
    }
}

#[cfg(test)]
mod tests;
