//! ECS Components — 阵营领域组件
//!
//! 定义阵营归属、声望、阵营关系等 ECS 组件。
//! 详见 docs/02-domain/domains/faction_domain.md

use std::collections::HashMap;

use bevy::prelude::*;

// ─── ID 类型 ──────────────────────────────────────────────────────

/// 阵营标识符。
///
/// 当前使用 String，内容系统接入后可通过 Registry 的 DefinitionId 桥接。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct FactionId(pub String);

impl FactionId {
    /// 从字符串创建 FactionId。
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// 返回内部字符串引用。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for FactionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for FactionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for FactionId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

// ─── 枚举类型 ─────────────────────────────────────────────────────

/// 声望等级 — 将数值声望映射为业务层面的关系等级。
///
/// 详见 faction_domain.md §1 声望等级体系。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub enum ReputationLevel {
    /// 仇恨：-100 ~ -51
    Hated,
    /// 敌对：-50 ~ -11
    Hostile,
    /// 中立（默认）：-10 ~ +9
    #[default]
    Neutral,
    /// 友好：+10 ~ +49
    Friendly,
    /// 尊敬：+50 ~ +89
    Honored,
    /// 崇敬：+90 ~ +100
    Revered,
}

impl ReputationLevel {
    /// 从声望值计算等级。
    ///
    /// 不变量 3.1：声望值被 clamp 到 [-100, +100]。
    pub fn from_value(value: i32) -> Self {
        let clamped = value.clamp(-100, 100);
        match clamped {
            -100..=-51 => Self::Hated,
            -50..=-11 => Self::Hostile,
            -10..=9 => Self::Neutral,
            10..=49 => Self::Friendly,
            50..=89 => Self::Honored,
            90..=100 => Self::Revered,
            _ => unreachable!(),
        }
    }

    /// 返回该等级的最小声望阈值。
    pub fn min_value(self) -> i32 {
        match self {
            Self::Hated => -100,
            Self::Hostile => -50,
            Self::Neutral => -10,
            Self::Friendly => 10,
            Self::Honored => 50,
            Self::Revered => 90,
        }
    }

    /// 返回该等级的最大声望阈值。
    pub fn max_value(self) -> i32 {
        match self {
            Self::Hated => -51,
            Self::Hostile => -11,
            Self::Neutral => 9,
            Self::Friendly => 49,
            Self::Honored => 89,
            Self::Revered => 100,
        }
    }
}

/// 阵营间基础关系。
///
/// 由剧情/设定决定，相对稳定（禁止频繁切换）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub enum FactionRelationType {
    /// 盟友
    Allied,
    /// 中立（默认）
    #[default]
    Neutral,
    /// 敌对
    Hostile,
    /// 战争
    War,
}

/// 个体对阵营的综合关系状态。
///
/// 由 FactionRelationType + Reputation 共同决定。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub enum RelationshipState {
    /// 盟友 — 不攻击，可交易/对话
    Allied,
    /// 中立（默认）— 不主动攻击，标准交互
    #[default]
    Neutral,
    /// 敌对 — 可攻击，交互受限
    Hostile,
    /// 战争 — 无差别攻击
    War,
}

// ─── ECS Components ───────────────────────────────────────────────

/// 角色的阵营归属列表。
///
/// 一个角色可以属于多个阵营。
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct FactionMembership {
    /// 所属阵营 ID 列表
    pub factions: Vec<FactionId>,
}

impl FactionMembership {
    /// 创建空的阵营归属。
    pub fn new() -> Self {
        Self {
            factions: Vec::new(),
        }
    }

    /// 创建带初始阵营的归属。
    pub fn with_faction(faction: impl Into<FactionId>) -> Self {
        Self {
            factions: vec![faction.into()],
        }
    }

    /// 添加阵营归属。
    pub fn join(&mut self, faction: impl Into<FactionId>) {
        let id = faction.into();
        if !self.factions.contains(&id) {
            self.factions.push(id);
        }
    }

    /// 离开阵营。
    pub fn leave(&mut self, faction: &FactionId) {
        self.factions.retain(|f| f != faction);
    }

    /// 是否属于指定阵营。
    pub fn is_member(&self, faction: &FactionId) -> bool {
        self.factions.contains(faction)
    }
}

impl Default for FactionMembership {
    fn default() -> Self {
        Self::new()
    }
}

/// 角色在各阵营的声望值。
///
/// 存储为 HashMap<FactionId, i32>，未记录的阵营默认为 0（Neutral）。
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct Reputation {
    /// 阵营 → 声望值映射
    pub values: HashMap<FactionId, i32>,
}

impl Reputation {
    /// 创建空的声望记录。
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// 获取对指定阵营的声望值（默认 0）。
    pub fn get(&self, faction: &FactionId) -> i32 {
        self.values.get(faction).copied().unwrap_or(0)
    }

    /// 设置声望值（自动 clamp 到 [-100, +100]，不变量 3.1）。
    pub fn set(&mut self, faction: FactionId, value: i32) -> i32 {
        let clamped = value.clamp(-100, 100);
        self.values.insert(faction, clamped);
        clamped
    }

    /// 修改声望值（delta 可为负），返回新值。
    ///
    /// 不变量 3.2：调用方应提供变更原因（通过 Event 传递）。
    pub fn modify(&mut self, faction: &FactionId, delta: i32) -> i32 {
        let current = self.get(faction);
        let new_value = (current + delta).clamp(-100, 100);
        self.values.insert(faction.clone(), new_value);
        new_value
    }

    /// 获取指定阵营的声望等级。
    pub fn level(&self, faction: &FactionId) -> ReputationLevel {
        ReputationLevel::from_value(self.get(faction))
    }
}

impl Default for Reputation {
    fn default() -> Self {
        Self::new()
    }
}

/// 关键角色标记 — 受最低声望保护（不变量 3.5）。
///
/// 带有此组件的角色的声望不能降到导致永久敌对/离开队伍的阈值以下，
/// 除非剧情明确允许。
#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component)]
pub struct KeyCharacter;

// ─── 资源 ─────────────────────────────────────────────────────────

/// 全局阵营关系表（Resource）。
///
/// 存储所有阵营对之间的基础关系。
/// 使用 (FactionId, FactionId) 作为键，且保证 (A,B) 与 (B,A) 值相同（不变量 3.3）。
#[derive(Resource, Debug, Clone, Default, Reflect)]
#[reflect(Resource)]
pub struct FactionRelationTable {
    /// 阵营对关系映射
    pub relations: HashMap<(FactionId, FactionId), FactionRelationType>,
}

impl FactionRelationTable {
    /// 创建空的关系表。
    pub fn new() -> Self {
        Self {
            relations: HashMap::new(),
        }
    }

    /// 设置两个阵营之间的关系（自动维护对称性，不变量 3.3）。
    pub fn set_relation(&mut self, a: FactionId, b: FactionId, relation: FactionRelationType) {
        if a == b {
            return;
        }
        // 使用字典序保证键的一致性
        let key = if a.0 <= b.0 {
            (a.clone(), b.clone())
        } else {
            (b, a)
        };
        self.relations.insert(key, relation);
    }

    /// 查询两个阵营之间的关系（默认 Neutral）。
    pub fn get_relation(&self, a: &FactionId, b: &FactionId) -> FactionRelationType {
        if a == b {
            return FactionRelationType::Allied;
        }
        let key = if a.0 <= b.0 {
            (a.clone(), b.clone())
        } else {
            (b.clone(), a.clone())
        };
        self.relations.get(&key).copied().unwrap_or_default()
    }
}
