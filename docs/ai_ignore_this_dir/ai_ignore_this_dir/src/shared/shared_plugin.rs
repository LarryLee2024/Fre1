//! SharedPlugin — Shared 层统一入口
//!
//! Layer 3 职责：零外部依赖，不含业务逻辑。
//! 注册所有 Shared 模块的基础设施（ids, error, events, audit 等）。

use bevy::app::{App, Plugin};

/// Shared 层统一 Plugin
///
/// 注册顺序：基础工具 → ID 系统 → 错误工具 → 事件白名单 → 审计
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, _app: &mut App) {
        // 目前 Shared 层模块为纯类型定义，无需注册 Resource/System。
        // 后续模块迁移时将在此处逐步添加：
        // - app.init_resource::<EventWhitelist>()
        // - app.add_event::<DomainEvent>() 等
    }
}
