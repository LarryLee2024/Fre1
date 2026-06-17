//! Registry Mechanism — 注册校验逻辑

pub mod validator;

pub use validator::{validate_cross_references, validate_id_format};
