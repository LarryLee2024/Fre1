//! ECS Components — 法术领域组件与类型
//!
//! 定义法术相关的 ID 类型、值类型、ECS 组件。
//! 详见 docs/02-domain/domains/spell_domain.md
//! 详见 docs/04-data/domains/spell_schema.md

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ─── ID 类型 ──────────────────────────────────────────────────────

/// 法术定义标识符（前缀: `spl_`）。
///
/// 统一使用 shared::ids::SpellId。
pub use crate::shared::ids::SpellId as SpellDefId;

// ─── 值类型 ────────────────────────────────────────────────────────

/// 法术环阶（0 = 戏法, 1-9 = 法术环阶）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum SpellLevel {
    Cantrip,
    L1,
    L2,
    L3,
    L4,
    L5,
    L6,
    L7,
    L8,
    L9,
}

impl SpellLevel {
    /// 获取环阶的数值表示（0-9）。
    pub fn as_u8(&self) -> u8 {
        match self {
            SpellLevel::Cantrip => 0,
            SpellLevel::L1 => 1,
            SpellLevel::L2 => 2,
            SpellLevel::L3 => 3,
            SpellLevel::L4 => 4,
            SpellLevel::L5 => 5,
            SpellLevel::L6 => 6,
            SpellLevel::L7 => 7,
            SpellLevel::L8 => 8,
            SpellLevel::L9 => 9,
        }
    }

    /// 从数值创建环阶（0-9），越界返回 None。
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(SpellLevel::Cantrip),
            1 => Some(SpellLevel::L1),
            2 => Some(SpellLevel::L2),
            3 => Some(SpellLevel::L3),
            4 => Some(SpellLevel::L4),
            5 => Some(SpellLevel::L5),
            6 => Some(SpellLevel::L6),
            7 => Some(SpellLevel::L7),
            8 => Some(SpellLevel::L8),
            9 => Some(SpellLevel::L9),
            _ => None,
        }
    }
}

impl Default for SpellLevel {
    fn default() -> Self {
        SpellLevel::Cantrip
    }
}

/// 施法时间类型。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub enum CastingTime {
    /// 1 个标准动作。
    Action,
    /// 1 个附赠动作。
    BonusAction,
    /// 反应（在特定时机触发）。
    Reaction,
    /// 长施法时间（分钟）。
    Longer { minutes: u32 },
}

/// 施法组件需求。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct SpellComponents {
    /// 需要语言成分（沉默时不可施法）。
    pub verbal: bool,
    /// 需要姿势成分（束缚时不可施法）。
    pub somatic: bool,
    /// 材料成分（可选）。
    pub material: Option<MaterialComponent>,
}

/// 材料成分定义。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct MaterialComponent {
    /// 材料描述本地化 Key。
    pub description: String,
    /// 材料是否被消耗。
    pub consumed: bool,
    /// 材料是否有金币价值要求。
    pub cost_gold: Option<u32>,
}

/// 法术射程。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub enum SpellRange {
    Self_,
    Touch,
    Ranged { base: u32, max: Option<u32> },
    Radius { center: RangeCenter, radius: u32 },
    Cone { length: u32 },
    Line { length: u32, width: u32 },
    Unlimited,
    Special,
}

/// 范围中心点。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub enum RangeCenter {
    Self_,
    Point,
}

/// 法术持续时间。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub enum SpellDuration {
    /// 瞬时生效。
    Instant,
    /// 专注维持，最多持续 max_turns 回合。
    Concentration { max_turns: u32 },
    /// 固定持续回合数。
    Timed { turns: u32 },
    /// 永久。
    Permanent,
}

/// 法术豁免类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum SaveType {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

/// 豁免检定结果。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum SaveResult {
    /// 豁免成功。
    Success,
    /// 豁免失败。
    Failure,
    /// 需要进一步处理（如减半伤害）。
    Partial,
}

// ─── 定义层结构 ──────────────────────────────────────────────────

/// 法术的静态定义（Definition 层）。
///
/// Spell 是 Ability 的子类型，复用 Ability 生命周期。
/// 详见 ADR-023 §1.1
#[derive(Debug, Clone, Asset, Serialize, Deserialize, Reflect)]
pub struct SpellDef {
    /// 法术唯一标识（前缀: `spl_`）。
    pub id: SpellDefId,
    /// 法术名称本地化 Key。
    pub name_key: String,
    /// 法术描述本地化 Key。
    pub desc_key: String,
    /// 法术环阶（0 = 戏法, 1-9）。
    pub level: SpellLevel,
    /// 施法时间。
    pub casting_time: CastingTime,
    /// 施法组件需求。
    pub components: SpellComponents,
    /// 法术射程。
    pub range: SpellRange,
    /// 持续时间。
    pub duration: SpellDuration,
    /// 是否需要专注。
    pub requires_concentration: bool,
    /// 豁免类型（如不需要豁免则为 None）。
    pub saving_throw: Option<SaveType>,
    /// 法术是否可升环施法。
    pub can_upcast: bool,
    /// 基础效果 ID 列表。
    pub effects: Vec<String>,
}

// ─── Instance 层组件 ─────────────────────────────────────────────

/// 法术位条目。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct SpellSlotEntry {
    /// 该环级的最大法术位数。
    pub total: u32,
    /// 已使用的法术位数。
    pub used: u32,
}

impl SpellSlotEntry {
    /// 剩余可用法术位数。
    pub fn remaining(&self) -> u32 {
        self.total.saturating_sub(self.used)
    }

    /// 是否还有可用法术位。
    pub fn has_available(&self) -> bool {
        self.used < self.total
    }
}

/// 法术位池组件。
///
/// 记录每个环阶的法术位总数与已用量。
/// 详见 docs/04-data/domains/spell_schema.md §1.2
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct SpellSlotPool {
    /// 各环阶法术位配置（仅 1-9 环有配置，戏法无消耗）。
    pub slots_by_level: Vec<SpellSlotEntry>,
}

impl SpellSlotPool {
    /// 创建新的法术位池（默认全空）。
    pub fn new() -> Self {
        Self {
            slots_by_level: (0..9)
                .map(|_| SpellSlotEntry { total: 0, used: 0 })
                .collect(),
        }
    }

    /// 获取指定环阶的可用法术位数（level: 1-9, index: 0=L1）。
    pub fn remaining(&self, level: SpellLevel) -> u32 {
        let idx = (level.as_u8().saturating_sub(1)) as usize;
        self.slots_by_level.get(idx).map_or(0, |e| e.remaining())
    }

    /// 消耗一个指定环阶的法术位。
    ///
    /// 返回是否成功消耗。
    pub fn consume(&mut self, level: SpellLevel) -> bool {
        let idx = (level.as_u8().saturating_sub(1)) as usize;
        if let Some(entry) = self.slots_by_level.get_mut(idx) {
            if entry.has_available() {
                entry.used += 1;
                return true;
            }
        }
        false
    }

    /// 恢复所有法术位（长休）。
    pub fn restore_all(&mut self) {
        for entry in &mut self.slots_by_level {
            entry.used = 0;
        }
    }

    /// 恢复指定环阶的一个法术位。
    pub fn restore_one(&mut self, level: SpellLevel) {
        let idx = (level.as_u8().saturating_sub(1)) as usize;
        if let Some(entry) = self.slots_by_level.get_mut(idx) {
            entry.used = entry.used.saturating_sub(1);
        }
    }
}

impl Default for SpellSlotPool {
    fn default() -> Self {
        Self::new()
    }
}

/// 法术书/法术列表组件。
///
/// 记录角色已知和已准备的法术。
/// 详见 docs/04-data/domains/spell_schema.md §1.3
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct Spellbook {
    /// 所有已习得的法术。
    pub known_spells: Vec<SpellDefId>,
    /// 当前已准备的法术（长休后可更换）。
    pub prepared_spells: Vec<SpellDefId>,
    /// 最大可准备法术数量。
    pub max_prepared: u32,
}

impl Spellbook {
    /// 创建空法术书。
    pub fn new(max_prepared: u32) -> Self {
        Self {
            known_spells: Vec::new(),
            prepared_spells: Vec::new(),
            max_prepared,
        }
    }

    /// 是否已习得指定法术。
    pub fn knows(&self, spell_id: &SpellDefId) -> bool {
        self.known_spells.contains(spell_id)
    }

    /// 是否已准备指定法术。
    pub fn has_prepared(&self, spell_id: &SpellDefId) -> bool {
        self.prepared_spells.contains(spell_id)
    }
}

/// 专注快照。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct ConcentrationSnapshot {
    /// 施法者的体质调整值（用于专注检定）。
    pub con_modifier: i32,
}

/// 专注状态组件。
///
/// 同一时间最多只能维持一个专注法术。
/// 详见 docs/04-data/domains/spell_schema.md §1.4
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct Concentration {
    /// 当前专注的法术 ID。
    pub spell_id: SpellDefId,
    /// 专注持续的总回合数。
    pub total_duration: u32,
    /// 已持续的回合数。
    pub elapsed_rounds: u32,
    /// 专注建立时的快照（用于打断检定）。
    pub snapshot: ConcentrationSnapshot,
}

impl Concentration {
    /// 创建新的专注状态。
    pub fn new(spell_id: SpellDefId, total_duration: u32, con_modifier: i32) -> Self {
        Self {
            spell_id,
            total_duration,
            elapsed_rounds: 0,
            snapshot: ConcentrationSnapshot { con_modifier },
        }
    }
}

/// 法术位变化的来源。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum SlotChangeSource {
    /// 长休恢复。
    LongRest,
    /// 特殊能力恢复。
    Ability,
    /// 施法消耗。
    SpellCast,
}

// ─── Resources ─────────────────────────────────────────────────────

/// 法术系统配置 Resource。
#[derive(Resource, Debug, Clone, Reflect, Deserialize)]
#[reflect(Resource)]
pub struct SpellConfig {
    /// 专注打断检定 DC 的基础值（取 max(10, 伤害/2)）。
    pub concentration_base_dc: u32,
    /// 默认每角色最大专注法术数（通常为 1）。
    pub max_concentration: u32,
    /// 戏法是否计入已知法术上限。
    pub cantrips_count_against_known: bool,
}

impl Default for SpellConfig {
    fn default() -> Self {
        Self {
            concentration_base_dc: 10,
            max_concentration: 1,
            cantrips_count_against_known: false,
        }
    }
}
