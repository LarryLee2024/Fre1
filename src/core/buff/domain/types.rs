//! Buff 数据模型类型定义（ADR-020 + ADR-021）
//!
//! - `BuffData`：运行时 Buff 数据（定义态）
//! - `BuffDef`：RON 反序列化用（定义态，TagName 替代 GameplayTag）
//! - `DurationPolicy`：持续策略（运行时）
//! - `StackPolicy`：叠层策略（运行时）
//! - `DurationDef`：持续策略 RON 表示
//! - `StackDef`：叠层策略 RON 表示
//! - `BuffCondition`：触发条件（占位）

use crate::core::attribute::AttributeModifierDef;
use crate::core::effect::EffectDef;
use crate::core::tag::{GameplayTag, TagName};
use serde::Deserialize;

// ──────────────────────────── DurationPolicy（ADR-021） ────────────────────────────

/// 持续策略 — Buff 持续多久（运行时）
#[derive(Clone, Debug, PartialEq, Eq, bevy::reflect::Reflect)]
pub enum DurationPolicy {
    /// 持续 N 回合（最常见），tick 递减
    Turns(u32),
    /// 直到死亡才消失
    UntilDeath,
    /// 移动后消失
    UntilMove,
    /// 攻击后消失
    UntilAttack,
    /// 受伤后消失（如护盾）
    UntilDamaged,
    /// 战斗结束消失
    BattleEnd,
    /// 永久（直到手动移除）
    Permanent,
}

impl Default for DurationPolicy {
    fn default() -> Self {
        DurationPolicy::Turns(1)
    }
}

// ──────────────────────────── StackPolicy（ADR-021） ────────────────────────────────

/// 叠层策略 — Buff 如何叠加（运行时）
#[derive(Clone, Debug, PartialEq, Eq, bevy::reflect::Reflect)]
pub enum StackPolicy {
    /// 不可叠加，重复施加刷新持续时间
    NoStack,
    /// 可叠加 N 层，达到上限后刷新最旧层的持续时间
    Stackable(u32),
    /// 可叠加 N 层，达到上限后不再接受新叠加
    StackableNoRefresh(u32),
}

impl Default for StackPolicy {
    fn default() -> Self {
        StackPolicy::NoStack
    }
}

// ──────────────────────── DurationDef（RON 反序列化用） ────────────────────────────

/// 持续策略 RON 表示（用于 BuffDef 中反序列化）
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum DurationDef {
    Turns(u32),
    UntilDeath,
    UntilMove,
    UntilAttack,
    UntilDamaged,
    BattleEnd,
    Permanent,
}

impl Default for DurationDef {
    fn default() -> Self {
        DurationDef::Turns(1)
    }
}

impl From<DurationDef> for DurationPolicy {
    fn from(def: DurationDef) -> Self {
        match def {
            DurationDef::Turns(n) => DurationPolicy::Turns(n),
            DurationDef::UntilDeath => DurationPolicy::UntilDeath,
            DurationDef::UntilMove => DurationPolicy::UntilMove,
            DurationDef::UntilAttack => DurationPolicy::UntilAttack,
            DurationDef::UntilDamaged => DurationPolicy::UntilDamaged,
            DurationDef::BattleEnd => DurationPolicy::BattleEnd,
            DurationDef::Permanent => DurationPolicy::Permanent,
        }
    }
}

// ───────────────────────── StackDef（RON 反序列化用） ──────────────────────────────

/// 叠层策略 RON 表示（用于 BuffDef 中反序列化）
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum StackDef {
    NoStack,
    Stackable(u32),
    StackableNoRefresh(u32),
}

impl Default for StackDef {
    fn default() -> Self {
        StackDef::NoStack
    }
}

impl From<StackDef> for StackPolicy {
    fn from(def: StackDef) -> Self {
        match def {
            StackDef::NoStack => StackPolicy::NoStack,
            StackDef::Stackable(n) => StackPolicy::Stackable(n),
            StackDef::StackableNoRefresh(n) => StackPolicy::StackableNoRefresh(n),
        }
    }
}

// ──────────────────────────── BuffCondition（占位） ────────────────────────────────

/// Buff 触发条件（占位，Phase 2+ 实现）
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum BuffCondition {}

// ──────────────────────────── BuffData（运行时） ────────────────────────────────────

/// Buff 数据定义（运行时，ADR-020 Phase 1）
///
/// 新增字段 `description`、`effects`、`duration`、`stack`、`conditions`，
/// 与旧扁平字段（`dot_damage`、`hot_heal`、`is_stun`、`is_cleanse`）共存。
/// 旧字段将在 Phase 3 移除。
#[derive(Clone, Debug)]
pub struct BuffData {
    pub id: String,
    /// 旧字段：直接文本（向后兼容）
    pub name: String,
    /// 新字段：本地化 Key（优先使用）
    pub name_key: Option<String>,
    /// 新字段（ADR-020）：描述文本
    pub description: String,
    /// 新字段（ADR-020）：效果列表（替代 dot_damage/hot_heal）
    pub effects: Vec<EffectDef>,
    /// 新字段（ADR-021）：持续策略
    pub duration: DurationPolicy,
    /// 新字段（ADR-021）：叠层策略
    pub stack: StackPolicy,
    /// 新字段（ADR-020）：触发条件
    pub conditions: Vec<BuffCondition>,
    // ── 旧字段（Phase 3 移除） ──
    pub default_duration: u32,
    pub modifiers: Vec<AttributeModifierDef>,
    pub tags: Vec<GameplayTag>,
    pub dot_damage: i32,
    pub hot_heal: i32,
    pub is_stun: bool,
    pub is_cleanse: bool,
    pub is_buff: bool,
}

impl Default for BuffData {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            name_key: None,
            description: String::new(),
            effects: vec![],
            duration: DurationPolicy::default(),
            stack: StackPolicy::default(),
            conditions: vec![],
            default_duration: 1,
            modifiers: vec![],
            tags: vec![],
            dot_damage: 0,
            hot_heal: 0,
            is_stun: false,
            is_cleanse: false,
            is_buff: false,
        }
    }
}

impl BuffData {
    pub fn is_debuff(&self) -> bool {
        !self.is_buff
    }
}

// ──────────────────────────── BuffDef（RON 反序列化） ──────────────────────────────

/// Buff 数据定义（RON 反序列化用，TagName 替代 GameplayTag）
///
/// 新增字段 `description`、`effects`、`duration`、`stack`、`conditions`，
/// 均使用 `#[serde(default)]` 保证旧配置兼容。
#[derive(Clone, Debug, Deserialize)]
pub struct BuffDef {
    #[serde(default)]
    pub version: u32,
    pub id: String,
    /// 旧字段：直接文本（向后兼容）
    #[serde(default)]
    pub name: String,
    /// 新字段：本地化 Key（优先使用）
    #[serde(default)]
    pub name_key: Option<String>,
    /// 新字段（ADR-020）：描述文本
    #[serde(default)]
    pub description: String,
    /// 新字段（ADR-020）：效果列表
    #[serde(default)]
    pub effects: Vec<EffectDef>,
    /// 新字段（ADR-021）：持续策略
    #[serde(default)]
    pub duration: DurationDef,
    /// 新字段（ADR-021）：叠层策略
    #[serde(default)]
    pub stack: StackDef,
    /// 新字段（ADR-020）：触发条件（暂未反序列化，留空）
    #[serde(default)]
    pub conditions: Vec<BuffCondition>,
    // ── 旧字段 ──
    pub default_duration: u32,
    pub modifiers: Vec<AttributeModifierDef>,
    pub tags: Vec<TagName>,
    pub dot_damage: i32,
    pub hot_heal: i32,
    pub is_stun: bool,
    pub is_cleanse: bool,
    pub is_buff: bool,
}

impl From<BuffDef> for BuffData {
    fn from(def: BuffDef) -> Self {
        // 从 duration: DurationDef 计算运行时策略
        let duration_policy: DurationPolicy = def.duration.into();
        // 若 duration 是 Turns(0) 且 default_duration > 0，回退到 default_duration
        // （兼容旧 RON 中没有 duration 字段的情况）
        let duration_policy = match duration_policy {
            DurationPolicy::Turns(0) if def.default_duration > 0 => {
                DurationPolicy::Turns(def.default_duration)
            }
            other => other,
        };

        BuffData {
            id: def.id,
            name: def.name,
            name_key: def.name_key,
            description: def.description,
            effects: def.effects,
            duration: duration_policy,
            stack: def.stack.into(),
            conditions: def.conditions,
            default_duration: def.default_duration,
            modifiers: def.modifiers,
            tags: def.tags.iter().map(|t| t.to_tag()).collect(),
            dot_damage: def.dot_damage,
            hot_heal: def.hot_heal,
            is_stun: def.is_stun,
            is_cleanse: def.is_cleanse,
            is_buff: def.is_buff,
        }
    }
}
