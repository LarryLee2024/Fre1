/// 审计轨迹收集器
///
/// ADR-006: 领域事件白名单与审计轨迹架构
/// AuditTrail 是一个 Bevy Resource，负责收集和存储 AuditEvent 实例。
/// 业务代码触发 DomainEvent → AuditTrail 监听并记录 → 下游系统（Replay/LogUI/Achievement）消费。
use bevy::prelude::Resource;

use super::event::AuditEvent;

/// 审计轨迹收集器
///
/// 作为 Bevy Resource 注册，存储所有收集到的审计事件。
/// ADR-006 §Definition/Instance: Instance（运行时状态）
#[derive(Debug, Clone, Resource)]
pub struct AuditTrail {
    /// 已收集的审计事件列表
    events: Vec<AuditEvent>,
}

impl Default for AuditTrail {
    fn default() -> Self {
        Self { events: Vec::new() }
    }
}

impl AuditTrail {
    /// 创建空的审计轨迹
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加一个审计事件到轨迹中
    pub fn push(&mut self, event: AuditEvent) {
        self.events.push(event);
    }

    /// 获取所有已收集的审计事件（只读引用）
    pub fn events(&self) -> &[AuditEvent] {
        &self.events
    }

    /// 获取已收集的事件数量
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// 检查审计轨迹是否为空
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// 清空审计轨迹（用于新战斗开始时重置）
    pub fn clear(&mut self) {
        self.events.clear();
    }
}
