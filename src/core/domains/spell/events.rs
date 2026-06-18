//! 领域事件 — Spell 域对外发布的事件
//!
//! 所有跨域通信必须通过 Event，禁止直接引用对方数据结构（Data Law 012）。
//!
//! 事件订阅关系详见 docs/02-domain/domains/spell_domain.md §6

use bevy::prelude::*;

use super::components::{SaveResult, SaveType, SpellDefId, SpellLevel};

/// 施法请求事件（输入）。
///
/// 由玩家/AI 触发，进入施法流程。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct SpellCastRequest {
    /// 施法者实体。
    pub caster: Entity,
    /// 要施放的法术 ID。
    pub spell_id: SpellDefId,
    /// 目标实体列表（根据法术类型可能为空）。
    pub targets: Vec<Entity>,
    /// 目标位置（范围法术时使用）。
    pub target_position: Option<Vec2>,
    /// 升环施法时的环级（None = 使用默认环级）。
    pub upcast_level: Option<SpellLevel>,
}

/// 施法结果事件（输出）。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct SpellCastResult {
    /// 施法者实体。
    pub caster: Entity,
    /// 施放的法术 ID。
    pub spell_id: SpellDefId,
    /// 实际使用的环级。
    pub effective_level: SpellLevel,
    /// 施法结果。
    pub result: CastOutcome,
}

/// 施法结果枚举。
#[derive(Debug, Clone, PartialEq)]
pub enum CastOutcome {
    /// 施法成功。
    Success,
    /// 施法失败（原因描述）。
    Failed { reason: String },
    /// 开始引导（长施法时间）。
    CastingStarted { remaining_minutes: u32 },
}

/// 法术位变化事件。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct SpellSlotChanged {
    /// 拥有者实体。
    pub entity: Entity,
    /// 变化的环级。
    pub level: SpellLevel,
    /// 当前剩余法术位数。
    pub remaining: u32,
    /// 当前总法术位数。
    pub total: u32,
    /// 变化来源。
    pub source: String,
}

/// 专注建立事件。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct ConcentrationStarted {
    /// 专注者实体。
    pub entity: Entity,
    /// 专注的法术 ID。
    pub spell_id: SpellDefId,
    /// 最大持续回合数。
    pub max_duration: u32,
}

/// 专注打断事件。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct ConcentrationBroken {
    /// 专注者实体。
    pub entity: Entity,
    /// 被打断的法术 ID。
    pub spell_id: SpellDefId,
    /// 打断原因。
    pub reason: ConcentrationBreakReason,
}

/// 专注打断原因。
#[derive(Debug, Clone, PartialEq)]
pub enum ConcentrationBreakReason {
    /// 受到伤害导致检定失败。
    DamageTaken { damage: u32, dc: u32, roll: i32 },
    /// 主动结束专注。
    ManualEnd,
    /// 持续时间到。
    DurationExpired,
    /// 施放新的专注法术。
    ReplacedByNewSpell { new_spell_id: SpellDefId },
}

/// 豁免检定事件。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct SaveRolled {
    /// 目标实体。
    pub target: Entity,
    /// 豁免类型。
    pub save_type: SaveType,
    /// 骰子结果（d20）。
    pub roll: i32,
    /// 总加值。
    pub modifier: i32,
    /// 豁免 DC。
    pub dc: u32,
    /// 豁免结果。
    pub result: SaveResult,
}
