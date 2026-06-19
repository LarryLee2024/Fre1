//! ECS Components — 队伍领域组件与类型
//!
//! 定义队伍管理相关的 ID 类型、值类型、ECS 组件。
//! 详见 docs/02-domain/domains/party_domain.md
//! 详见 docs/04-data/domains/party_schema.md

use std::collections::HashMap;

use bevy::asset::Asset;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ─── ID 类型 ──────────────────────────────────────────────────────

/// 羁绊模板标识符（前缀: `bnd_`）。
///
/// 统一使用 shared::ids::BondDefId。
pub use crate::shared::ids::BondDefId;

/// 阵型模板标识符（前缀: `fmd_`）。
///
/// 统一使用 shared::ids::FormationDefId。
pub use crate::shared::ids::FormationDefId;
use crate::shared::localization_key::LocalizationKey;

// ─── 值类型 ────────────────────────────────────────────────────────

/// 阵型枚举。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum FormationType {
    /// 一字排开（默认）。
    Line,
    /// 前 3 后 2。
    Vanguard,
    /// 前后各 2。
    Defensive,
    /// 自定义偏移量。
    Custom(Vec<Vec2>),
}

impl FormationType {
    /// 获取默认阵型偏移量。
    ///
    /// 返回 4 个位置的相对偏移（用于战场部署）。
    pub fn default_offsets(&self) -> Vec<Vec2> {
        match self {
            FormationType::Line => vec![
                Vec2::new(-1.0, 0.5),
                Vec2::new(-1.0, -0.5),
                Vec2::new(-2.0, 0.5),
                Vec2::new(-2.0, -0.5),
            ],
            FormationType::Vanguard => vec![
                Vec2::new(-0.5, 1.0),
                Vec2::new(-0.5, -1.0),
                Vec2::new(-1.5, 0.0),
                Vec2::new(-2.0, 0.5),
                Vec2::new(-2.0, -0.5),
            ],
            FormationType::Defensive => vec![
                Vec2::new(-1.0, 1.0),
                Vec2::new(-1.0, -1.0),
                Vec2::new(-1.5, 0.5),
                Vec2::new(-1.5, -0.5),
            ],
            FormationType::Custom(offsets) => offsets.clone(),
        }
    }
}

impl Default for FormationType {
    fn default() -> Self {
        FormationType::Line
    }
}

/// 羁绊匹配模式。
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub enum BondMatchMode {
    /// 同时满足所有条件。
    All,
    /// 满足任一条件即可。
    Any,
}

/// 羁绊条件要求。
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct BondRequirement {
    /// 特定角色 ID（可选，config 中为字符串，运行时 resolve 为 Entity）。
    #[serde(skip)]
    pub specific_entity: Option<Entity>,
    /// 需要的标签列表（可选）。
    pub required_tags: Vec<String>,
    /// 匹配模式。
    pub match_mode: BondMatchMode,
}

/// 羁绊模板定义（Definition 层）。
///
/// 描述特定角色组合的羁绊条件和效果。
#[derive(Debug, Clone, Asset, Reflect, Serialize, Deserialize)]
pub struct BondDef {
    /// 羁绊唯一标识。
    pub id: BondDefId,
    /// 羁绊名称本地化 Key。
    #[reflect(ignore)]
    pub name_key: LocalizationKey,
    /// 羁绊描述本地化 Key。
    #[reflect(ignore)]
    pub desc_key: LocalizationKey,
    /// 激活条件：需要哪些角色/标签同时在活跃队伍中。
    pub required_members: Vec<BondRequirement>,
    /// 各等级的效果描述（ModifierDefId 占位）。
    pub level_effects: HashMap<u32, Vec<String>>,
    /// 最大羁绊等级。
    pub max_level: u32,
}

/// 当前激活的羁绊实例。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct ActiveBond {
    /// 羁绊模板 ID。
    pub bond_id: BondDefId,
    /// 当前等级。
    pub level: u32,
    /// 参与的角色。
    pub participants: Vec<Entity>,
    /// 积累的战斗次数（用于升级）。
    pub accumulated_battles: u32,
}

/// 队伍成员信息。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct PartyMember {
    /// 角色实体。
    pub entity: Entity,
    /// 队伍槽位索引。
    pub slot_index: u32,
    /// 阵型偏移。
    pub formation_offset: Vec2,
    /// 是否活跃（出战状态）。
    pub is_active: bool,
}

impl PartyMember {
    /// 创建新的队伍成员。
    pub fn new(entity: Entity, slot_index: u32) -> Self {
        Self {
            entity,
            slot_index,
            formation_offset: Vec2::ZERO,
            is_active: true,
        }
    }
}

// ─── Resources ─────────────────────────────────────────────────────

/// 队伍资源（Instance 层）。
///
/// 管理所有可加入队伍的角色和当前活跃成员。
/// 详见 docs/02-domain/domains/party_domain.md §1
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct Party {
    /// 当前活跃（上场战斗）成员。
    pub members: Vec<PartyMember>,
    /// 预备队员（不在战斗中）。
    pub reserve_members: Vec<Entity>,
    /// 当前阵型。
    pub formation: FormationType,
    /// 当前选中成员索引。
    pub active_member: Option<usize>,
    /// 活跃成员上限（默认 4）。
    pub max_active: u32,
    /// 队伍总人数上限（活跃 + 预备，默认 12）。
    pub max_total: u32,
}

impl Party {
    /// 创建空队伍。
    pub fn new() -> Self {
        Self {
            members: Vec::new(),
            reserve_members: Vec::new(),
            formation: FormationType::Line,
            active_member: None,
            max_active: 4,
            max_total: 12,
        }
    }

    /// 获取活跃成员数。
    pub fn active_count(&self) -> usize {
        self.members.iter().filter(|m| m.is_active).count()
    }

    /// 获取预备队员数。
    pub fn reserve_count(&self) -> usize {
        self.reserve_members.len()
    }

    /// 获取总人数（活跃 + 预备）。
    pub fn total_count(&self) -> usize {
        self.active_count() + self.reserve_count()
    }

    /// 检查队伍是否已满。
    pub fn is_full(&self) -> bool {
        self.total_count() >= self.max_total as usize
    }

    /// 检查活跃成员是否已达上限。
    pub fn is_active_full(&self) -> bool {
        self.active_count() >= self.max_active as usize
    }
}

impl Default for Party {
    fn default() -> Self {
        Self::new()
    }
}

/// 羁绊状态资源（Instance 层）。
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct BondState {
    /// 当前激活的羁绊。
    pub active_bonds: Vec<ActiveBond>,
    /// 已定义的羁绊模板。
    pub defs: HashMap<BondDefId, BondDef>,
}

impl BondState {
    /// 创建空的羁绊状态。
    pub fn new() -> Self {
        Self {
            active_bonds: Vec::new(),
            defs: HashMap::new(),
        }
    }
}

impl Default for BondState {
    fn default() -> Self {
        Self::new()
    }
}

/// 队伍标记组件。
///
/// 标记具有队伍管理系统的实体（通常是玩家队伍的代表实体）。
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub struct PartyMarker;
