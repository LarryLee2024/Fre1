//! 共享 trait 模块的宏。
//!
//! 提供 `impl_rule_failure!` 用于生成 RuleFailure 实现，
//! 支持 sealed trait。

/// 生成带有指定错误码的 `RuleFailure` 实现。
///
/// 每个变体通过模式匹配分支映射到其错误码。
/// 同时生成 `impl sealed::Sealed for $ty` 以满足
/// Sealed Trait 约束（参见 ADR-057）。
///
/// # 示例
///
/// ```ignore
/// impl_rule_failure!(CombatFailure,
///     Self::InsufficientParticipants { .. } => "COMBAT_INSUFFICIENT_PARTICIPANTS",
///     Self::NotYourTurn => "COMBAT_NOT_YOUR_TURN",
/// );
/// ```
#[macro_export]
macro_rules! impl_rule_failure {
    ($ty:ty, $( $variant:pat => $code:expr ),+ $(,)?) => {
        impl $crate::shared::traits::sealed::Sealed for $ty {}
        impl $crate::shared::traits::RuleFailure for $ty {
            fn code(&self) -> &'static str {
                match self {
                    $( $variant => $code, )+
                }
            }
        }
    };
}
