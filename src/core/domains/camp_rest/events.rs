//! 领域事件 — CampRest 域对外发布的事件
//!
//! 所有跨域通信必须通过 Event，禁止直接引用对方数据结构（Data Law 012）。
//!
//! 事件订阅关系详见 docs/02-domain/domains/camp_rest_domain.md §6

use bevy::prelude::*;

use super::components::CampEventId;

/// 短休完成时触发。
///
/// 订阅者：
/// - Ability：重置标记为"短休恢复"的能力
/// - UI：显示短休结果
#[derive(Event, Debug, Clone, PartialEq)]
pub struct ShortRestCompleted {
    /// 参与休息的角色实体列表。
    pub entities: Vec<Entity>,
    /// 消耗的生命骰数量。
    pub hit_dice_used: u32,
    /// 恢复的 HP 总量。
    pub hp_healed: u32,
    /// 已恢复的能力列表。
    pub abilities_restored: Vec<String>,
}

/// 长休开始时触发。
///
/// 订阅者：
/// - Narrative：准备营地事件
/// - NPC：激活营地 NPC
#[derive(Event, Debug, Clone, PartialEq)]
pub struct LongRestStarted {
    /// 参与休息的角色实体列表。
    pub entities: Vec<Entity>,
    /// 营地位置描述。
    pub camp_location: String,
}

/// 长休完成时触发。
///
/// 订阅者：
/// - Spell：恢复全部法术位
/// - Ability：重置所有能力
/// - Effect：移除临时效果
/// - UI：显示长休结果
#[derive(Event, Debug, Clone, PartialEq)]
pub struct LongRestCompleted {
    /// 参与休息的角色实体列表。
    pub entities: Vec<Entity>,
    /// HP 恢复总量。
    pub hp_restored: u32,
    /// 法术位是否已恢复。
    pub spell_slots_restored: bool,
    /// 生命骰恢复量。
    pub hit_dice_restored: u32,
    /// 触发的营地事件列表。
    pub events_triggered: Vec<String>,
}

/// 长休被中断时触发。
///
/// 订阅者：
/// - Combat：如中断原因为战斗，开始战斗
/// - UI：显示中断警告
#[derive(Event, Debug, Clone, PartialEq)]
pub struct LongRestInterrupted {
    /// 参与休息的角色实体列表。
    pub entities: Vec<Entity>,
    /// 中断来源描述。
    pub interruption_source: String,
    /// 累计中断时间（分钟）。
    pub cumulative_interrupt_time: u32,
}

/// 营地事件触发时触发。
///
/// 订阅者：
/// - Narrative：推动剧情
/// - Quest：检查任务进度
/// - UI：显示事件界面
#[derive(Event, Debug, Clone, PartialEq)]
pub struct CampEventTriggered {
    /// 事件 ID。
    pub event_id: CampEventId,
    /// 事件类型。
    pub event_type: String,
    /// 参与者列表。
    pub participants: Vec<Entity>,
    /// 可用选项列表。
    pub choices_available: Vec<String>,
}
