//! 制作/锻造领域 — 组件定义
//!
//! 详见 docs/02-domain/domains/crafting_domain.md
//! Schema: docs/04-data/domains/crafting_schema.md

use bevy::asset::Asset;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ─── 制作台类型 ────────────────────────────────────────────────

/// 制作台类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum CraftingStation {
    Forge,
    EnchantingTable,
    AlchemyLab,
    TailoringBench,
    EngineeringBench,
}

// ─── 制作类型 ──────────────────────────────────────────────────

/// 制作类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum CraftType {
    Smithing,
    Enchanting,
    Alchemy,
    Tailoring,
    Engineering,
}

// ─── 技能要求 ──────────────────────────────────────────────────

/// 技能要求。
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct SkillRequirement {
    pub skill_id: String,
    pub dc: u32,
}

// ─── 材料消耗 ──────────────────────────────────────────────────

/// 材料消耗。
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct MaterialCost {
    pub item_id: String,
    pub quantity: u32,
}

// ─── 产出定义 ──────────────────────────────────────────────────

/// 制作产出。
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct CraftOutput {
    pub item_id: String,
    pub quantity: u32,
    pub enchantment_slots: u32,
}

// ─── 配方定义 ──────────────────────────────────────────────────

/// 配方定义。内容团队配置，运行时只读。
#[derive(Debug, Clone, Asset, Serialize, Deserialize, Reflect)]
pub struct RecipeDef {
    pub id: String,
    pub name_key: String,
    pub station: CraftingStation,
    pub skill_requirement: Option<SkillRequirement>,
    pub materials: Vec<MaterialCost>,
    pub output: CraftOutput,
    pub craft_time: u32,
    pub craft_type: CraftType,
}

// ─── 附魔槽位类型 ─────────────────────────────────────────────

/// 附魔槽位类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum EnchantmentSlotType {
    Weapon { max_slots: u32 },
    Armor { max_slots: u32 },
    Accessory { max_slots: u32 },
}

// ─── 附魔定义 ──────────────────────────────────────────────────

/// 附魔定义。
#[derive(Debug, Clone, Reflect)]
pub struct EnchantmentDef {
    pub id: String,
    pub name_key: String,
    pub modifier_id: String,
    pub exclusive_group: Option<String>,
    pub slot_type: EnchantmentSlotType,
}

// ─── 附魔槽位 ──────────────────────────────────────────────────

/// 装备的附魔槽位运行时状态。
#[derive(Debug, Clone, Component, Reflect)]
pub struct EnchantmentSlot {
    pub max_slots: u32,
    pub active_enchants: Vec<String>,
}

// ─── 升级等级 ──────────────────────────────────────────────────

/// 装备升级等级。
#[derive(Debug, Clone, Component, Reflect)]
pub struct UpgradeLevel {
    pub current: u32,
    pub max: u32,
    pub level_modifiers: Vec<(u32, Vec<String>)>,
}

impl UpgradeLevel {
    pub fn new(max: u32) -> Self {
        Self {
            current: 0,
            max,
            level_modifiers: Vec::new(),
        }
    }

    /// 检查是否可以继续升级。
    pub fn can_upgrade(&self) -> bool {
        self.current < self.max
    }
}
