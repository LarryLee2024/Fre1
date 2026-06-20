//! Entity ↔ ID 映射模块。
//!
//! 提供 EntityMapper 通用双向映射器，用于隔离 Domain 层和 ECS Entity。

mod entity_mapper;

pub use entity_mapper::*;
