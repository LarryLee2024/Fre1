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
/// 顶级命名空间控制在 15个以内，新增需架构评审。
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
    /// 领域归属分类（Party, Progression, Inventory, CampRest 等）
    Domain,
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

/// 标签内容分类（Content Layer 概念）
///
/// 决定标签的可见性、参与层级继承的行为、以及内容管线的处理方式。
/// 对应 docs/03-content/definitions/vocabulary/tag-def.md §TagCategory
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum TagCategory {
    /// 游戏玩法标签 — 玩家可见，参与层级继承，用于运行时逻辑
    Gameplay,
    /// 内容管理标签 — 编辑器/工具链使用，不参与运行时逻辑
    Semantic,
    /// 系统内部标签 — 系统标记用，通常仅代码注册
    System,
}

impl Default for TagCategory {
    fn default() -> Self {
        Self::Gameplay
    }
}
