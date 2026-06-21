//! Convenience macros for generating trait implementations.
//!
//! These macros eliminate repetitive `impl` blocks for common patterns:
//!
//! - `impl_domain_event!(TypeName)` — generates `impl DomainEvent for TypeName {}`
//! - `impl_rule_failure!(TypeName, ...)` — generates `impl RuleFailure for TypeName`
//!   with per-variant error codes
//!
//! These are used instead of proc-macro derives to avoid the complexity
//! of a separate proc-macro crate in this project.

// ============================================================================
// impl_domain_event!
// ============================================================================

/// Generate a `DomainEvent` marker trait implementation.
///
/// # Example
///
/// ```ignore
/// impl_domain_event!(TurnEnded);
/// impl_domain_event!(LevelUp);
/// ```
#[macro_export]
macro_rules! impl_domain_event {
    ($ty:ty) => {
        impl $crate::shared::diagnostics::DomainEvent for $ty {}
    };
}

// ============================================================================
// impl_rule_failure!
// ============================================================================

/// Generate a `RuleFailure` implementation with specified error codes.
///
/// Each variant maps to its error code via a pattern-match arm.
///
/// Also generates `impl sealed::Sealed for TypeName` to satisfy the
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
