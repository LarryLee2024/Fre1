//! Registry Foundation — 注册中心基础类型与值对象

pub mod types;
pub mod values;

pub use types::{AllocatorState, IdAllocator, IdType, RegistryEntry, RegistryError};
pub use values::{BrokenReference, CrossReferenceReport, DefRegistry};
