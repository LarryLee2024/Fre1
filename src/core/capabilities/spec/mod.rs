//! spec — Spec（规格/配置）能力领域
//!
//! 负责 Spec（Def→Spec→Instance 三层分离的中间桥梁），管理实体身上的
//! AbilitySpec 和 EffectSpec 配置实例，提供 SpecRegistry 工厂转换和
//! SpecContainer 容器组件。
//!
//! 分层结构：
//! - foundation/: 纯数据类型（AbilitySpec, EffectSpec, SpecId, etc.）
//! - mechanism/:  ECS 组件（SpecContainer）+ 生命周期管理（SpecRegistry）
//! - events/:     领域事件（SpecGranted, SpecRemoved, etc.）
//!
//! 详见 docs/02-domain/spec_domain.md

pub mod events;
pub mod foundation;
pub mod mechanism;

mod plugin;
pub use plugin::*;
