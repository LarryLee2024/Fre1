//! 召唤领域 — 组件定义
//!
//! 详见 docs/02-domain/domains/summon_domain.md
//! Schema: docs/04-data/domains/summon_schema.md

use crate::shared::localization_key::LocalizationKey;
use bevy::asset::Asset;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ─── 召唤物网格大小 ────────────────────────────────────────────

/// 召唤物占用的网格大小。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum GridSize {
    Small,
    Medium,
    Large,
    Huge,
}

// ─── 召唤物 AI 模式 ───────────────────────────────────────────

/// 召唤物 AI 模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum SummonAIMode {
    Autonomous,
    Follow,
    Guard,
    Defensive,
}

// ─── 召唤消耗 ──────────────────────────────────────────────────

/// 召唤消耗。
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct SummonCost {
    pub ability_id: Option<String>,
    pub spell_level: Option<u32>,
    pub requires_concentration: bool,
}

// ─── 召唤物模板定义 ───────────────────────────────────────────

/// 召唤物模板定义。运行时只读。
#[derive(Debug, Clone, Asset, Reflect, Serialize, Deserialize)]
pub struct SummonTemplateDef {
    pub id: String,
    #[reflect(ignore)]
    pub name_key: LocalizationKey,
    pub base_attributes: Vec<(String, f32)>,
    pub tags: Vec<String>,
    pub abilities: Vec<String>,
    pub modifiers: Vec<String>,
    pub grid_size: GridSize,
    pub default_ai_mode: SummonAIMode,
    pub summon_cost: SummonCost,
}

// ─── 召唤绑定 ──────────────────────────────────────────────────

/// 召唤物与召唤者的绑定关系。
#[derive(Debug, Clone, Component, Reflect)]
pub struct SummonBond {
    pub caster: Entity,
    pub template_id: String,
    pub ai_mode: SummonAIMode,
    pub summoned_at: f64,
}

// ─── 召唤槽位管理 ─────────────────────────────────────────────

/// 召唤者的召唤槽位管理。
#[derive(Debug, Clone, Component, Reflect)]
pub struct SummonSlotManager {
    pub active_summons: Vec<Entity>,
    pub max_slots: u32,
}

impl SummonSlotManager {
    /// 创建指定最大槽位数的召唤槽管理器。
    pub fn new(max_slots: u32) -> Self {
        Self {
            active_summons: Vec::new(),
            max_slots,
        }
    }

    /// 检查是否有空闲槽位（当前召唤物数量 < 最大槽位数）。
    pub fn has_free_slot(&self) -> bool {
        (self.active_summons.len() as u32) < self.max_slots
    }

    /// 注册一个召唤物实体到槽位。
    pub fn add_summon(&mut self, entity: Entity) {
        self.active_summons.push(entity);
    }

    /// 移除一个召唤物实体（召唤物死亡/超时时调用）。
    pub fn remove_summon(&mut self, entity: Entity) {
        self.active_summons.retain(|&e| e != entity);
    }
}

// ─── 持续时间类型 ─────────────────────────────────────────────

/// 召唤持续时间类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum SummonDurationType {
    Concentration,
    Timed { max_turns: u32 },
    Permanent,
}
