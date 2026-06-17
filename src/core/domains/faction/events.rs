//! 领域事件 — Faction 域对外发布的事件
//!
//! 所有跨域通信必须通过 Event，禁止直接引用对方数据结构（Data Law 012）。
//!
//! 事件订阅关系详见 docs/02-domain/domains/faction_domain.md §6

use bevy::prelude::*;

use super::components::{FactionId, FactionRelationType, ReputationLevel};

/// 角色在某阵营的声望变化时触发。
///
/// 订阅者：
/// - Narrative：更新对话选项
/// - Economy：更新价格折扣
/// - Quest：检查任务条件
/// - UI：显示声望变化通知
#[derive(Event, Debug, Clone, PartialEq)]
pub struct ReputationChanged {
    /// 声望变化的实体
    pub entity: Entity,
    /// 目标阵营
    pub faction_id: FactionId,
    /// 变化前的声望值
    pub old_value: i32,
    /// 变化后的声望值
    pub new_value: i32,
    /// 变化后的声望等级
    pub new_level: ReputationLevel,
    /// 变化原因描述
    pub reason: String,
}

/// 两个阵营间的关系变化时触发。
///
/// 订阅者：
/// - Combat：更新战场敌对关系
/// - Narrative：更新剧情走向
/// - Faction：通知阵营成员
/// - UI：显示阵营关系变化公告
#[derive(Event, Debug, Clone, PartialEq)]
pub struct FactionRelationChanged {
    /// 阵营 A
    pub faction_a: FactionId,
    /// 阵营 B
    pub faction_b: FactionId,
    /// 变化前的关系
    pub old_relation: FactionRelationType,
    /// 变化后的关系
    pub new_relation: FactionRelationType,
    /// 变化原因/触发事件
    pub cause: String,
}

/// 声望等级提升时触发。
///
/// 订阅者：
/// - Narrative：解锁新对话
/// - Quest：解锁新任务
/// - UI：显示达成的消息
#[derive(Event, Debug, Clone, PartialEq)]
pub struct ReputationLevelUp {
    /// 声望变化的实体
    pub entity: Entity,
    /// 目标阵营
    pub faction_id: FactionId,
    /// 变化前的等级
    pub old_level: ReputationLevel,
    /// 变化后的等级
    pub new_level: ReputationLevel,
}

/// 关系判定完成时触发（调试用/状态面板更新）。
///
/// 订阅者：
/// - UI：更新关系面板
/// - 调试工具：记录关系判定结果
#[derive(Event, Debug, Clone, PartialEq)]
pub struct RelationshipEvaluated {
    /// 评估主体实体
    pub entity: Entity,
    /// 目标实体（如为阵营则 target_id 为 None，faction_id 有值）
    pub target_entity: Option<Entity>,
    /// 目标阵营
    pub faction_id: FactionId,
    /// 阵营间基础关系
    pub base_relation: FactionRelationType,
    /// 声望修正后的等级
    pub reputation_level: ReputationLevel,
    /// 最终关系状态
    pub final_state: super::components::RelationshipState,
}
