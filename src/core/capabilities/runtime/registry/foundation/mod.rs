//! Registry Foundation — 注册中心基础类型与值对象

pub(crate) mod error;
pub(crate) mod types;
pub(crate) mod values;

pub use error::RegistryError;
pub use types::{AllocatorState, IdAllocator, IdType, RegistryEntry};
pub use values::{BrokenReference, CrossReferenceReport, DefRegistry};
