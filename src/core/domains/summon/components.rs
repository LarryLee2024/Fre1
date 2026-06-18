//! 召唤领域 — 组件定义
//!
//! 详见 docs/02-domain/domains/summon_domain.md
//! Schema: docs/04-data/domains/summon_schema.md

use bevy::prelude::*;

// ─── 召唤物网格大小 ────────────────────────────────────────────

/// 召唤物占用的网格大小。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum GridSize {
    Small,
    Medium,
    Large,
    Huge,
}

// ─── 召唤物 AI 模式 ───────────────────────────────────────────

/// 召唤物 AI 模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum SummonAIMode {
    Autonomous,
    Follow,
    Guard,
    Defensive,
}

// ─── 召唤消耗 ──────────────────────────────────────────────────

/// 召唤消耗。
#[derive(Debug, Clone, Reflect)]
pub struct SummonCost {
    pub ability_id: Option<String>,
    pub spell_level: Option<u32>,
    pub requires_concentration: bool,
}

// ─── 召唤物模板定义 ───────────────────────────────────────────

/// 召唤物模板定义。运行时只读。
#[derive(Debug, Clone, Reflect)]
pub struct SummonTemplateDef {
    pub id: String,
    pub name_key: String,
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
    pub fn new(max_slots: u32) -> Self {
        Self {
            active_summons: Vec::new(),
            max_slots,
        }
    }

    pub fn has_free_slot(&self) -> bool {
        (self.active_summons.len() as u32) < self.max_slots
    }

    pub fn add_summon(&mut self, entity: Entity) {
        self.active_summons.push(entity);
    }

    pub fn remove_summon(&mut self, entity: Entity) {
        self.active_summons.retain(|&e| e != entity);
    }
}

// ─── 持续时间类型 ─────────────────────────────────────────────

/// 召唤持续时间类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum SummonDurationType {
    Concentration,
    Timed { max_turns: u32 },
    Permanent,
}
