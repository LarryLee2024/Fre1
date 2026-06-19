//! C1: 纯数据定义层 — Ability 基础类型、枚举、值对象

// [ADR-045] pub(crate) — crate 内共享，测试可访问，外部不可访问
pub(crate) mod types;
pub(crate) mod values;
pub(crate) mod def;

pub use def::AbilityDef;
pub use types::*;
pub use values::*;
