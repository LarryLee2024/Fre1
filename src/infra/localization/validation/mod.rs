//! validation — Startup validation and runtime audit for localization.
//!
//! Contains the startup key-integrity validator and the periodic coverage auditor.

pub(crate) mod audit;
pub(crate) mod validator;

pub use audit::{AuditTimer, audit_system};
pub use validator::validation_system;
