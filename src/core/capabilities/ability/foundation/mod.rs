//! C1: 纯数据定义层 — Ability 基础类型、枚举、值对象
//!
//! 包含 ConstAbilityMetadata trait（#12 编译期元数据模式）

// [ADR-045] pub(crate) — crate 内共享，测试可访问，外部不可访问
pub(crate) mod def;
pub(crate) mod error;
pub(crate) mod failure;
pub(crate) mod spec_consts;
pub(crate) mod types;
pub(crate) mod values;

pub use def::AbilityDef;
pub use error::AbilityError;
pub use failure::AbilityFailure;
pub use spec_consts::ConstAbilityMetadata;
pub use types::*;
pub use values::*;
