// 修饰规则类型定义

use crate::core::tag::GameplayTag;
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

/// 将 Tag ID 字符串转换为 GameplayTag（临时函数，后续替换为 TagRegistry 查询）
fn tag_id_to_gameplay_tag(id: &str) -> GameplayTag {
    match id {
        "dmg_fire" => GameplayTag::DMG_FIRE,
        "dmg_ice" => GameplayTag::DMG_ICE,
        "dmg_physical" => GameplayTag::DMG_PHYSICAL,
        "dmg_magical" => GameplayTag::DMG_MAGICAL,
        "dmg_pierce" => GameplayTag::DMG_PIERCE,
        "dmg_true" => GameplayTag::DMG_TRUE,
        "buff" => GameplayTag::BUFF,
        "debuff" => GameplayTag::DEBUFF,
        "special_state" => GameplayTag::SPECIAL_STATE,
        "control_soft" => GameplayTag::CONTROL_SOFT,
        "control_hard" => GameplayTag::CONTROL_HARD,
        "control_full" => GameplayTag::CONTROL_FULL,
        "invincible" => GameplayTag::INVINCIBLE,
        "untargetable" => GameplayTag::UNTARGETABLE,
        "ally" => GameplayTag::ALLY,
        "enemy" => GameplayTag::ENEMY,
        "summon" => GameplayTag::SUMMON,
        "boss" => GameplayTag::BOSS,
        "mechanical" => GameplayTag::MECHANICAL,
        "weapon_sword" => GameplayTag::WEAPON_SWORD,
        "weapon_bow" => GameplayTag::WEAPON_BOW,
        "weapon_staff" => GameplayTag::WEAPON_STAFF,
        "heavy_armor" => GameplayTag::HEAVY_ARMOR,
        "light_armor" => GameplayTag::LIGHT_ARMOR,
        "shield" => GameplayTag::SHIELD,
        "flying" => GameplayTag::FLYING,
        "grounded" => GameplayTag::GROUNDED,
        "dispellable" => GameplayTag::DISPELLABLE,
        "undispellable" => GameplayTag::UNDISPELLABLE,
        "reflectable" => GameplayTag::REFLECTABLE,
        "untriggerable" => GameplayTag::UNTRIGGERABLE,
        _ => {
            bevy::log::warn!(target: "modifier", "Unknown tag_id in modifier: {}", id);
            GameplayTag::from_bits(0)
        }
    }
}

/// 修饰规则（RON 反序列化用）
#[derive(Clone, Debug, Deserialize)]
pub struct ModifierRuleDef {
    /// 配置版本号（预留，用于未来存档兼容性检查）
    #[serde(default)]
    pub version: u32,
    pub name: String,
    pub source_tag: String,
    pub target_tag: String,
    pub effect: ModifierEffectDef,
}

impl From<ModifierRuleDef> for ModifierRule {
    fn from(def: ModifierRuleDef) -> Self {
        ModifierRule {
            name: def.name,
            source_tag: tag_id_to_gameplay_tag(&def.source_tag),
            target_tag: tag_id_to_gameplay_tag(&def.target_tag),
            effect: match def.effect {
                ModifierEffectDef::DamageMultiplier(v) => ModifierEffect::DamageMultiplier(v),
                ModifierEffectDef::DamageBonus(v) => ModifierEffect::DamageBonus(v),
                ModifierEffectDef::HealMultiplier(v) => ModifierEffect::HealMultiplier(v),
                ModifierEffectDef::HealBonus(v) => ModifierEffect::HealBonus(v),
            },
        }
    }
}
