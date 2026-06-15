// Trait 类型定义：TraitTrigger, TraitEffect, TraitDefinition, TraitData, TraitSource, TraitEntry, TraitCollection

use crate::core::attribute::AttributeModifierDef;
use crate::core::equipment::EquipmentSlot;
use crate::core::tag::GameplayTag;
use bevy::prelude::*;
use serde::Deserialize;

/// Trait 触发时机
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TraitTrigger {
    /// 被动：始终生效（授予标签/属性修饰）
    Passive,
    /// 回合开始时触发
    OnTurnStart,
    /// 回合结束时触发
    OnTurnEnd,
    /// 攻击时触发
    OnAttack,
    /// 被攻击时触发
    OnHit,
    /// 击杀时触发
    OnKill,
}

/// Trait 效果定义（RON 反序列化用，tag_id 字符串）
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TraitEffectDef {
    /// 授予标签（tag_id 如 "warrior", "dmg_fire"）
    GrantTag(String),
    /// 属性修饰（永久，作为基础值的一部分）
    ModifyAttribute(AttributeModifierDef),
    /// 触发时施加 Buff
    ApplyBuff { buff_id: String, duration: u32 },
}

/// Trait 效果（运行时，GameplayTag 替代 TagName）
#[derive(Clone, Debug, Reflect)]
pub enum TraitEffect {
    GrantTag(GameplayTag),
    ModifyAttribute(AttributeModifierDef),
    ApplyBuff { buff_id: String, duration: u32 },
}

impl TraitEffect {
    /// 返回效果类型名（与 variant 名对应，用于 Handler 查找）
    pub fn type_name(&self) -> &'static str {
        match self {
            TraitEffect::GrantTag(_) => "GrantTag",
            TraitEffect::ModifyAttribute(_) => "ModifyAttribute",
            TraitEffect::ApplyBuff { .. } => "ApplyBuff",
        }
    }
}

/// 将 Tag ID 字符串转换为 GameplayTag 位掩码
///
/// 临时函数，后续会替换为 TagRegistry 运行时查询。
fn tag_id_to_gameplay_tag(id: &str) -> GameplayTag {
    match id {
        "buff" => GameplayTag::BUFF,
        "debuff" => GameplayTag::DEBUFF,
        "dmg_fire" => GameplayTag::DMG_FIRE,
        "dmg_ice" => GameplayTag::DMG_ICE,
        "dmg_physical" => GameplayTag::DMG_PHYSICAL,
        "dmg_magical" => GameplayTag::DMG_MAGICAL,
        "control_soft" => GameplayTag::CONTROL_SOFT,
        "control_hard" => GameplayTag::CONTROL_HARD,
        "control_full" => GameplayTag::CONTROL_FULL,
        "invincible" => GameplayTag::INVINCIBLE,
        "untargetable" => GameplayTag::UNTARGETABLE,
        "ally" => GameplayTag::ALLY,
        "enemy" => GameplayTag::ENEMY,
        "flying" => GameplayTag::FLYING,
        "grounded" => GameplayTag::GROUNDED,
        "dispellable" => GameplayTag::DISPELLABLE,
        "undispellable" => GameplayTag::UNDISPELLABLE,
        "weapon_sword" => GameplayTag::WEAPON_SWORD,
        "weapon_bow" => GameplayTag::WEAPON_BOW,
        "weapon_staff" => GameplayTag::WEAPON_STAFF,
        _ => {
            bevy::log::warn!(target: "traits", "Unknown tag_id in trait effect: {}", id);
            GameplayTag::from_bits(0) // 空标签
        }
    }
}

impl From<TraitEffectDef> for TraitEffect {
    fn from(def: TraitEffectDef) -> Self {
        match def {
            TraitEffectDef::GrantTag(tag_id) => {
                TraitEffect::GrantTag(tag_id_to_gameplay_tag(&tag_id))
            }
            TraitEffectDef::ModifyAttribute(mod_def) => TraitEffect::ModifyAttribute(mod_def),
            TraitEffectDef::ApplyBuff { buff_id, duration } => {
                TraitEffect::ApplyBuff { buff_id, duration }
            }
        }
    }
}

/// Trait 定义（RON 反序列化用）
#[derive(Clone, Debug, Deserialize)]
pub struct TraitDefinition {
    #[serde(default)]
    pub version: u32,
    pub id: String,
    pub name: String,
    pub description: String,
    pub trigger: TraitTrigger,
    pub effects: Vec<TraitEffectDef>,
}

/// Trait 数据（运行时）
#[derive(Clone, Debug)]
pub struct TraitData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub trigger: TraitTrigger,
    pub effects: Vec<TraitEffect>,
}

impl From<TraitDefinition> for TraitData {
    fn from(def: TraitDefinition) -> Self {
        TraitData {
            id: def.id,
            name: def.name,
            description: def.description,
            trigger: def.trigger,
            effects: def.effects.into_iter().map(Into::into).collect(),
        }
    }
}

/// Trait 来源：追踪 trait 是从哪里获得的，用于穿脱时精确增减
#[derive(Clone, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum TraitSource {
    /// 内在来源（种族/职业/天赋）
    Intrinsic,
    /// 装备来源（记录具体槽位）
    Equipment { slot: EquipmentSlot },
}

/// Trait 条目：记录 trait_id + 来源，支持按来源精确移除
#[derive(Clone, Debug, Reflect)]
pub struct TraitEntry {
    pub trait_id: String,
    pub source: TraitSource,
}

/// 单位上的 Trait 集合组件
#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct TraitCollection {
    pub entries: Vec<TraitEntry>,
}

impl TraitCollection {
    /// 从 trait_id 列表创建（全部标记为 Intrinsic 来源）
    pub fn new(trait_ids: Vec<String>) -> Self {
        Self {
            entries: trait_ids
                .into_iter()
                .map(|id| TraitEntry {
                    trait_id: id,
                    source: TraitSource::Intrinsic,
                })
                .collect(),
        }
    }

    /// 是否拥有指定 trait
    pub fn has(&self, trait_id: &str) -> bool {
        self.entries.iter().any(|e| e.trait_id == trait_id)
    }

    /// 添加一条 TraitEntry
    pub fn add_entry(&mut self, trait_id: String, source: TraitSource) {
        self.entries.push(TraitEntry { trait_id, source });
    }

    /// 移除指定来源的所有 Trait，返回被移除的 trait_id 列表
    pub fn remove_by_source(&mut self, source: &TraitSource) -> Vec<String> {
        let mut removed = Vec::new();
        self.entries.retain(|e| {
            if &e.source == source {
                removed.push(e.trait_id.clone());
                false
            } else {
                true
            }
        });
        removed
    }

    /// 获取所有 trait_id（去重）
    pub fn trait_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.entries.iter().map(|e| e.trait_id.clone()).collect();
        ids.dedup();
        ids
    }
}

#[cfg(test)]
mod tests {
    // ================================================
    // AI Self-Check (test_spec.md §13.1)
    // ================================================
    // ✅ 测试行为，不是实现
    // ✅ 符合领域规则
    // ✅ 测试是确定性的
    // ✅ 使用标准测试数据
    // ✅ 没有测试私有实现
    // ✅ 没有生成不在范围内的测试
    // ================================================

    use super::*;
    use ron::de::from_bytes;

    /// Test ID: CHR-TYP-001
    /// Title: RON 反序列化旧配置（无 version 字段）兼容
    ///
    /// Given: 不含 version 字段的 RON 字符串
    /// When: 反序列化为 TraitDefinition
    /// Then: version 默认为 0
    ///
    /// Assertions: id == "old_trait", version == 0
    #[test]
    fn ron_deserialize_trait_old_config_without_version() {
        // Given
        let ron_str = r#"
            (
                id: "old_trait",
                name: "旧配置",
                description: "没有version字段",
                trigger: Passive,
                effects: [GrantTag(WARRIOR)],
            )
        "#;

        // When
        let def: TraitDefinition = from_bytes(ron_str.as_bytes()).unwrap();

        // Then
        assert_eq!(def.id, "old_trait");
        assert_eq!(def.version, 0);
    }
}
