//! Registry Foundation — 注册中心基础类型与值对象

pub(crate) mod types;
pub(crate) mod values;

pub use types::{AllocatorState, IdAllocator, IdType, RegistryEntry, RegistryError};
pub use values::{BrokenReference, CrossReferenceReport, DefRegistry};
