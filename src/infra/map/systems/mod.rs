//! Systems — ECS 地图接入系统
//!
//! 连接 MapAsset 加载/卸载事件与 ECS 世界状态。
//! 通信模式：使用 Bevy 0.19 trigger() + Observer 模式。
//!
//! 子模块：
//! - map_loader_system: MapAsset → GridMap 转换与资源注册
//! - map_cleanup_system: 地图卸载时的资源与实体清理
//! - object_instantiator: MapObject → ECS Entity 实例化

mod map_cleanup_system;
mod map_loader_system;
mod object_instantiator;

pub use map_cleanup_system::*;
pub use map_loader_system::*;
pub use object_instantiator::*;
