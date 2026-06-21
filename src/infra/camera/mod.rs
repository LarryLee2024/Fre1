//! Camera — 镜头基础设施模块（L2 Infra）
//!
//! Camera 位于 Infra 层，与 registry/pipeline/replay/save/input 平级。
//! 提供 State Machine 驱动的镜头系统，所有外部请求通过 CameraRequest Event 触发。
//!
//! ## 模块结构
//! - `foundation/` — 纯类型定义（CameraPose, CameraTarget, CameraRequest, CameraState, CameraCommand）
//! - `systems/` — Bevy Systems（input_handler, state_machine, movement, shake, bounds）
//! - `components.rs` — ECS Component 类型（CameraBounds, CameraShake, CameraInputBlock, IdleTimeout, MainCamera）
//! - `resources.rs` — ECS Resource 类型（UnitPositionResolver, TileSize）
//! - `query.rs` — 公开只读查询 API（CameraQuery）
//! - `plugin.rs` — CameraPlugin + spawn_camera/despawn_camera 工厂函数
//!
//! ## 外部通信
//! - 外部系统通过 `commands.trigger(CameraRequest::...)` 请求镜头变化
//! - 外部系统通过 `CameraQuery` 只读查询镜头信息
//! - 场景系统在 OnEnter 时 spawn Camera、可选插入 CameraBounds
//!
//! ## 禁止
//! - 外部系统直接修改 Camera Transform/Projection
//! - Camera 模块依赖任何 core::domains::* 类型
//!
//! 详见 `docs/01-architecture/40-cross-cutting/ADR-064-camera-architecture.md`
//! 详见 `docs/02-domain/infrastructure/camera_domain.md`

pub mod components;
pub mod foundation;
mod plugin;
pub mod query;
pub mod resources;
pub mod systems;

pub use plugin::*;

#[cfg(test)]
mod tests;
