//! C1: 纯数据定义层 — Effect 基础类型、枚举、值对象

mod types;
#[cfg(test)]
pub mod values;

pub use types::*;
pub use values::*;
