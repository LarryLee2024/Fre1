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
/// 顶级命名空间控制在 12 个以内，新增需架构评审。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TagNamespace {
    /// 角色/单位分类（Human, Elf, Boss, Undead 等）
    Character,
    /// 技能/能力分类（Fire, Healing, Active, Passive 等）
    Ability,
    /// 状态/增减益/免疫分类（Poisoned, Burning, Immune.Fire 等）
    Status,
    /// 装备分类（Weapon, Armor, Slot 等）
    Equipment,
    /// 物品分类（Consumable, QuestItem 等）
    Item,
    /// 伤害类型分类（Physical, Fire, Ice 等）
    Damage,
    /// 地形分类（Water, Lava, Forest 等）
    Terrain,
    /// 阵营分类（Player, Enemy, Neutral 等）
    Faction,
    /// 任务分类（Main, Side, Faction 等）
    Quest,
    /// 战斗状态分类（InCombat, OutOfCombat 等）
    Combat,
    /// 触发条件分类（OnDamaged, OnHealed 等）
    Trigger,
    /// 表现信号分类（OnApply, OnTick 等）
    Cue,
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
