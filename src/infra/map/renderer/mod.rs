//! Renderer — 地图表现层（V1: Entity-per-Tile）
//!
//! V1 使用 Entity-per-Tile 快速路径，每个 Tile 一个 Entity + SpriteSheetBundle。
//! 未来 V2 将迁移到 Material2D 批处理渲染。
//!
//! 子模块：
//! - spawn: Tile Entity 生成与初始化
//! - overlay: 覆盖层（光标高亮、移动范围、交互区域）
//! - cleanup: 渲染实体清理

pub mod cleanup;
pub mod overlay;
pub mod spawn;

pub use cleanup::*;
pub use overlay::*;
pub use spawn::*;
