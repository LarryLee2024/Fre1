// 修饰规则类型定义

use crate::core::tag::{GameplayTag, TagName};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 伤害/治疗修饰符条目（记录每一步修饰的 before/after 和规则名）
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub struct ModifierEntry {
    /// 修饰前值
    pub before: i32,
    /// 修饰后值
    pub after: i32,
    /// 修饰规则名称（如"火焰共鸣"）
    pub rule_name: String,
}

/// 修饰效果类型
#[derive(Clone, Debug)]
pub enum ModifierEffect {
    /// 伤害倍率修饰
    DamageMultiplier(f32),
    /// 伤害加成（固定值）
    DamageBonus(i32),
    /// 治疗倍率修饰
    HealMultiplier(f32),
    /// 治疗加成（固定值）
    HealBonus(i32),
}

impl ModifierEffect {
    /// 返回效果类型名（与 enum variant 名对应）
    pub fn type_name(&self) -> &'static str {
        match self {
            ModifierEffect::DamageMultiplier(_) => "DamageMultiplier",
            ModifierEffect::DamageBonus(_) => "DamageBonus",
            ModifierEffect::HealMultiplier(_) => "HealMultiplier",
            ModifierEffect::HealBonus(_) => "HealBonus",
        }
    }
}

/// 修饰规则（运行时）
#[derive(Clone, Debug)]
pub struct ModifierRule {
    pub name: String,
    /// 攻击方技能需要包含的标签
    pub source_tag: GameplayTag,
    /// 目标需要包含的标签
    pub target_tag: GameplayTag,
    /// 修饰效果
    pub effect: ModifierEffect,
}

/// 修饰效果类型（RON 反序列化用）
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ModifierEffectDef {
    DamageMultiplier(f32),
    DamageBonus(i32),
    HealMultiplier(f32),
    HealBonus(i32),
}

/// 修饰规则（RON 反序列化用）
#[derive(Clone, Debug, Deserialize)]
pub struct ModifierRuleDef {
    /// 配置版本号（预留，用于未来存档兼容性检查）
    #[serde(default)]
    pub version: u32,
    pub name: String,
    pub source_tag: TagName,
    pub target_tag: TagName,
    pub effect: ModifierEffectDef,
}

impl From<ModifierRuleDef> for ModifierRule {
    fn from(def: ModifierRuleDef) -> Self {
        ModifierRule {
            name: def.name,
            source_tag: def.source_tag.to_tag(),
            target_tag: def.target_tag.to_tag(),
            effect: match def.effect {
                ModifierEffectDef::DamageMultiplier(v) => ModifierEffect::DamageMultiplier(v),
                ModifierEffectDef::DamageBonus(v) => ModifierEffect::DamageBonus(v),
                ModifierEffectDef::HealMultiplier(v) => ModifierEffect::HealMultiplier(v),
                ModifierEffectDef::HealBonus(v) => ModifierEffect::HealBonus(v),
            },
        }
    }
}
