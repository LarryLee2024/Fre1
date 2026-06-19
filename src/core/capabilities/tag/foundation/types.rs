//! Tag 领域基础类型与枚举
//!
//! 仅包含纯数据定义，零行为逻辑。

use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

crate::define_string_id! {
    pub TagId,
    prefix: "tag",
}

/// 标签命名空间枚举。
/// 用于强制命名空间一致性，禁止跨域引用。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TagNamespace {
    DamageType,
    StatusEffect,
    SkillType,
    EquipmentSlot,
    EquipmentCategory,
    WeaponCategory,
    ArmorCategory,
    ItemCategory,
    Faction,
    CombatState,
    MovementType,
    TerrainType,
    BuffCategory,
    Immune,
    Cooldown,
    SpellSchool,
    QuestTag,
    DialogueTag,
    /// 允许扩展的命名空间，但必须注册
    Custom(String),
}

/// 标签查询模式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TagQueryMode {
    /// 至少匹配一个目标标签
    Any,
    /// 匹配全部目标标签
    All,
    /// 不匹配任何目标标签（用于免疫/排除检查）
    None,
}
