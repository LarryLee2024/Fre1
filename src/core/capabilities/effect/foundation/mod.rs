//! C1: 纯数据定义层 — Effect 基础类型、枚举、值对象

mod types;
// [ADR-045] pub(crate) — crate 内共享，测试可访问，外部不可访问
pub(crate) mod def;
pub(crate) mod error;
pub(crate) mod values;

pub use def::EffectDef;
pub use error::EffectError;
pub use types::*;
pub use values::*;
