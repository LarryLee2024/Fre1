/// 审计事件数据结构
///
/// ADR-006: 领域事件白名单与审计轨迹架构
/// AuditEvent 是审计轨迹中的单个事件记录，支持序列化/反序列化。
use bevy::prelude::Entity;
use serde::{Deserialize, Serialize};

/// 审计事件元数据
///
/// 包含事件发生时的上下文信息，用于追踪事件在游戏流程中的位置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMetadata {
    /// 事件发生的回合数
    pub turn_number: u32,
    /// 事件发生的阶段（如 "player_turn", "enemy_turn", "resolution"）
    pub phase: String,
    /// 事件来源标识（如 "combat_system", "buff_system", "skill_system"）
    pub source: String,
}

/// 单个审计事件
///
/// 记录一个领域事件的完整快照，用于审计轨迹收集。
/// ADR-006 §决策: 审计轨迹数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// 事件发生的时间戳（使用 Bevy 的 Time::elapsed_secs 或类似机制）
    pub timestamp: u64,
    /// 事件类型名称（必须在事件白名单中）
    pub event_type: String,
    /// 相关实体（可选，某些事件可能不关联特定实体）
    pub entity: Option<Entity>,
    /// 事件的序列化数据（JSON 格式，支持任意结构）
    pub data: serde_json::Value,
    /// 事件元数据
    pub metadata: AuditMetadata,
}
