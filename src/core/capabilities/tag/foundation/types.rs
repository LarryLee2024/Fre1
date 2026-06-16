//! Tag 领域基础类型与枚举
//!
//! 仅包含纯数据定义，零行为逻辑。

use std::fmt;

/// 标签唯一标识符，强类型 newtype。
/// 格式: `tag_<6位数字>`（如 `tag_000001`）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagId(pub String);

impl TagId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// 返回 ID 的字符串切片（不含 `tag_` 前缀）
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TagId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 标签命名空间枚举。
/// 用于强制命名空间一致性，禁止跨域引用。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagQueryMode {
    /// 至少匹配一个目标标签
    Any,
    /// 匹配全部目标标签
    All,
    /// 不匹配任何目标标签（用于免疫/排除检查）
    None,
}
