//! 领域事件 — Party 域对外发布的事件
//!
//! 所有跨域通信必须通过 Event，禁止直接引用对方数据结构（Data Law 012）。
//!
//! 事件订阅关系详见 docs/02-domain/domains/party_domain.md §6

use bevy::prelude::*;

use super::components::BondDefId;
use crate::shared::localization_key::LocalizationKey;

/// 新成员加入队伍时触发。
///
/// 订阅者：
/// - Party：重新评估所有羁绊条件
/// - UI：更新队伍面板
/// - Quest：检查"特定成员在队"的任务条件
#[derive(Event, Debug, Clone, PartialEq)]
pub struct MemberJoined {
    /// 队伍标识（预留，当前使用单一队伍）。
    pub party_id: Option<Entity>,
    /// 加入的角色实体。
    pub entity: Entity,
    /// 角色类型（active/reserve）。
    pub role: String,
}

/// 成员离开队伍时触发。
///
/// 订阅者：
/// - Party：重新评估羁绊
/// - Quest：检查任务条件
#[derive(Event, Debug, Clone, PartialEq)]
pub struct MemberRemoved {
    /// 队伍标识。
    pub party_id: Option<Entity>,
    /// 离开的角色实体。
    pub entity: Entity,
    /// 离开原因。
    pub reason: String,
}

/// 战斗中换人时触发。
///
/// 订阅者：
/// - Combat：更新参与者列表
/// - Party：重新评估羁绊
/// - UI：换人动画
#[derive(Event, Debug, Clone, PartialEq)]
pub struct MemberSwapped {
    /// 队伍标识。
    pub party_id: Option<Entity>,
    /// 离场的成员。
    pub outgoing: Entity,
    /// 上场的成员。
    pub incoming: Entity,
}

/// 羁绊激活时触发。
///
/// 订阅者：
/// - Modifier：应用羁绊加成
/// - UI：显示羁绊激活通知
#[derive(Event, Debug, Clone, PartialEq)]
pub struct BondActivated {
    /// 队伍标识。
    pub party_id: Option<Entity>,
    /// 羁绊 ID。
    pub bond_id: BondDefId,
    /// 参与的角色列表。
    pub members: Vec<Entity>,
    /// 效果描述。
    pub effect_description: LocalizationKey,
}

/// 羁绊解除时触发。
///
/// 订阅者：
/// - Modifier：移除羁绊加成
/// - UI：显示羁绊解除通知
#[derive(Event, Debug, Clone, PartialEq)]
pub struct BondDeactivated {
    /// 队伍标识。
    pub party_id: Option<Entity>,
    /// 羁绊 ID。
    pub bond_id: BondDefId,
    /// 解除原因。
    pub reason: String,
}
