//! Macros for the shared traits module.
//!
//! Provides `impl_rule_failure!` for generating RuleFailure implementations
//! with sealed trait support.

/// Generate a `RuleFailure` implementation with specified error codes.
///
/// Each variant maps to its error code via a pattern-match arm.
/// Also generates `impl sealed::Sealed for $ty` to satisfy the
/// Sealed Trait constraint (see ADR-057).
///
/// # Example
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
