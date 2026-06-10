// Trait 类型定义：TraitTrigger, TraitEffect, TraitDefinition, TraitData, TraitCollection

use crate::core::attribute::AttributeModifierDef;
use crate::core::tag::{GameplayTag, TagName};
use bevy::prelude::*;
use serde::Deserialize;

/// Trait 触发时机
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
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

/// Trait 效果定义（RON 反序列化用，TagName 替代 GameplayTag）
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TraitEffectDef {
    /// 授予标签
    GrantTag(TagName),
    /// 属性修饰（永久，作为基础值的一部分）
    ModifyAttribute(AttributeModifierDef),
    /// 触发时施加 Buff
    ApplyBuff { buff_id: String, duration: u32 },
}

/// Trait 效果（运行时，GameplayTag 替代 TagName）
#[derive(Clone, Debug)]
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

impl From<TraitEffectDef> for TraitEffect {
    fn from(def: TraitEffectDef) -> Self {
        match def {
            TraitEffectDef::GrantTag(tag_name) => TraitEffect::GrantTag(tag_name.to_tag()),
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

/// 单位上的 Trait 集合组件
#[derive(Component, Default, Debug, Clone)]
pub struct TraitCollection {
    pub trait_ids: Vec<String>,
}

impl TraitCollection {
    pub fn new(trait_ids: Vec<String>) -> Self {
        Self { trait_ids }
    }

    /// 是否拥有指定 trait
    pub fn has(&self, trait_id: &str) -> bool {
        self.trait_ids.iter().any(|t| t == trait_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ron::de::from_bytes;

    #[test]
    fn ron_反序列化_旧配置无version字段() {
        let ron_str = r#"
            (
                id: "old_trait",
                name: "旧配置",
                description: "没有version字段",
                trigger: Passive,
                effects: [GrantTag(WARRIOR)],
            )
        "#;
        let def: TraitDefinition = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.id, "old_trait");
        assert_eq!(def.version, 0);
    }
}
