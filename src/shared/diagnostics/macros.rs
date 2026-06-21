//! Macros for the diagnostics module.
//!
//! Provides `impl_domain_event!` for generating DomainEvent marker trait
//! implementations.

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
