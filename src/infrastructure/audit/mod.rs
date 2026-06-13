/// 审计轨迹模块（ADR-006）
///
/// 领域事件白名单与审计轨迹架构。
/// 业务代码触发 DomainEvent → AuditTrail 收集 → 下游系统（Replay/LogUI/Achievement）消费。
///
/// 核心原则：
/// - 事件是唯一事实源：日志、回放、UI、成就共用同一套事件
/// - 白名单管理：新增事件必须先更新白名单
/// - 审计轨迹不影响业务逻辑执行
pub mod event;
mod trail;
mod whitelist;

pub use trail::AuditTrail;
pub use whitelist::{EventWhitelist, WhitelistEntry, WhitelistStatus};

use bevy::prelude::*;

/// 审计插件
///
/// ADR-006 §Module Design: 注册 AuditTrail Resource 和审计收集系统
pub struct AuditPlugin;

impl Plugin for AuditPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AuditTrail::new())
            .insert_resource(EventWhitelist::default());
    }
}
