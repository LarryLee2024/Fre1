// Trait 系统：统一抽象，种族/职业/天赋/装备均可引用 traits
// 替代硬编码的 class_tag 单标签，支持多 trait 组合
// 支持从 assets/traits/*.ron 外部配置文件加载

use crate::core::attribute::{AttributeModifierDef, AttributeModifierInstance, BuffInstanceId};
use crate::core::tag::{GameplayTag, GameplayTags, TagName};
use bevy::prelude::*;
use ron::de::from_bytes;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{read, read_dir};

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

impl TraitData {
    /// 收集此 trait 授予的所有标签
    pub fn granted_tags(&self) -> Vec<GameplayTag> {
        self.effects
            .iter()
            .filter_map(|e| match e {
                TraitEffect::GrantTag(tag) => Some(*tag),
                _ => None,
            })
            .collect()
    }

    /// 收集此 trait 的所有属性修饰
    pub fn attribute_modifiers(&self) -> Vec<&AttributeModifierDef> {
        self.effects
            .iter()
            .filter_map(|e| match e {
                TraitEffect::ModifyAttribute(mod_def) => Some(mod_def),
                _ => None,
            })
            .collect()
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

/// 将一组 trait 的被动效果应用到单位的标签和属性上
/// 返回 (granted_tags, attribute_modifiers)
pub fn apply_passive_traits(
    trait_ids: &[String],
    registry: &TraitRegistry,
) -> (GameplayTags, Vec<AttributeModifierInstance>) {
    let mut tags = GameplayTags::default();
    let mut modifiers = Vec::new();
    // 使用固定的 source id 标识 trait 来源的修饰符
    let trait_source = BuffInstanceId(u64::MAX);

    for trait_id in trait_ids {
        if let Some(trait_data) = registry.get(trait_id) {
            if trait_data.trigger != TraitTrigger::Passive {
                continue;
            }
            for tag in trait_data.granted_tags() {
                tags.add(tag);
            }
            for mod_def in trait_data.attribute_modifiers() {
                modifiers.push(AttributeModifierInstance {
                    kind: mod_def.kind,
                    op: mod_def.op,
                    value: mod_def.value,
                    source: trait_source,
                });
            }
        }
    }

    (tags, modifiers)
}

/// Trait 注册表资源
#[derive(Resource, Default)]
pub struct TraitRegistry {
    pub traits: HashMap<String, TraitData>,
}

impl TraitRegistry {
    pub fn get(&self, id: &str) -> Option<&TraitData> {
        self.traits.get(id)
    }

    /// 从 assets/traits/ 目录加载所有 .ron 文件
    pub fn load_from_dir(dir: &str) -> Self {
        let mut registry = TraitRegistry::default();
        let Ok(entries) = read_dir(dir) else {
            bevy::log::warn!("Trait 目录不存在: {}", dir);
            registry.register_defaults();
            return registry;
        };

        let mut loaded = false;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "ron") {
                match read(&path) {
                    Ok(bytes) => match from_bytes::<TraitDefinition>(&bytes) {
                        Ok(def) => {
                            let id = def.id.clone();
                            registry.traits.insert(id.clone(), def.into());
                            bevy::log::info!("加载 Trait: {}", id);
                            loaded = true;
                        }
                        Err(e) => {
                            bevy::log::error!("解析 Trait 文件 {:?} 失败: {}", path, e);
                        }
                    },
                    Err(e) => {
                        bevy::log::error!("读取 Trait 文件 {:?} 失败: {}", path, e);
                    }
                }
            }
        }

        if !loaded {
            bevy::log::warn!("Trait 目录为空，使用默认 Traits");
            registry.register_defaults();
        }

        registry
    }

    /// 注册内置默认 Traits
    fn register_defaults(&mut self) {
        let defaults = vec![
            TraitDefinition {
                id: "warrior_mastery".into(),
                name: "战士精通".into(),
                description: "近战职业，擅长正面作战".into(),
                trigger: TraitTrigger::Passive,
                effects: vec![
                    TraitEffectDef::GrantTag(TagName::Warrior),
                    TraitEffectDef::GrantTag(TagName::Melee),
                ],
            },
            TraitDefinition {
                id: "archer_mastery".into(),
                name: "弓手精通".into(),
                description: "远程职业，擅长远距离攻击".into(),
                trigger: TraitTrigger::Passive,
                effects: vec![
                    TraitEffectDef::GrantTag(TagName::Archer),
                    TraitEffectDef::GrantTag(TagName::Ranged),
                ],
            },
            TraitDefinition {
                id: "mage_mastery".into(),
                name: "法师精通".into(),
                description: "魔法职业，擅长元素攻击".into(),
                trigger: TraitTrigger::Passive,
                effects: vec![
                    TraitEffectDef::GrantTag(TagName::Mage),
                ],
            },
            TraitDefinition {
                id: "fire_affinity".into(),
                name: "火焰亲和".into(),
                description: "拥有火焰力量".into(),
                trigger: TraitTrigger::Passive,
                effects: vec![
                    TraitEffectDef::GrantTag(TagName::Fire),
                ],
            },
            TraitDefinition {
                id: "heavy_armor".into(),
                name: "重甲".into(),
                description: "装备重甲，防御+3".into(),
                trigger: TraitTrigger::Passive,
                effects: vec![
                    TraitEffectDef::ModifyAttribute(AttributeModifierDef {
                        kind: crate::core::attribute::AttributeKind::Def,
                        op: crate::core::attribute::ModifierOp::Add,
                        value: 3.0,
                    }),
                ],
            },
        ];

        for def in defaults {
            let id = def.id.clone();
            self.traits.insert(id, def.into());
        }
    }
}

/// Trait 插件
pub struct TraitPlugin;

impl Plugin for TraitPlugin {
    fn build(&self, app: &mut App) {
        let registry = TraitRegistry::load_from_dir("assets/traits");
        app.insert_resource(registry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::attribute::{AttributeKind, ModifierOp};

    #[test]
    fn ron_反序列化_trait定义() {
        let ron_str = r#"
            (
                id: "warrior_mastery",
                name: "战士精通",
                description: "近战职业",
                trigger: Passive,
                effects: [
                    GrantTag(WARRIOR),
                    GrantTag(MELEE),
                ],
            )
        "#;
        let def: TraitDefinition = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.id, "warrior_mastery");
        assert_eq!(def.trigger, TraitTrigger::Passive);
        assert_eq!(def.effects.len(), 2);
    }

    #[test]
    fn trait_def_转换为_trait_data() {
        let def = TraitDefinition {
            id: "test".into(),
            name: "测试".into(),
            description: "测试trait".into(),
            trigger: TraitTrigger::OnAttack,
            effects: vec![
                TraitEffectDef::GrantTag(TagName::Fire),
                TraitEffectDef::ModifyAttribute(AttributeModifierDef {
                    kind: AttributeKind::Atk,
                    op: ModifierOp::Add,
                    value: 5.0,
                }),
            ],
        };
        let data: TraitData = def.into();
        assert_eq!(data.id, "test");
        assert_eq!(data.trigger, TraitTrigger::OnAttack);
        assert_eq!(data.effects.len(), 2);

        let tags = data.granted_tags();
        assert_eq!(tags, vec![GameplayTag::FIRE]);

        let mods = data.attribute_modifiers();
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].kind, AttributeKind::Atk);
        assert_eq!(mods[0].value, 5.0);
    }

    #[test]
    fn trait_collection_查询() {
        let collection = TraitCollection::new(vec!["warrior_mastery".into(), "heavy_armor".into()]);
        assert!(collection.has("warrior_mastery"));
        assert!(collection.has("heavy_armor"));
        assert!(!collection.has("mage_mastery"));
    }

    #[test]
    fn apply_passive_traits_授予标签和修饰符() {
        let mut registry = TraitRegistry::default();
        registry.traits.insert(
            "warrior_mastery".into(),
            TraitData {
                id: "warrior_mastery".into(),
                name: "战士精通".into(),
                description: String::new(),
                trigger: TraitTrigger::Passive,
                effects: vec![
                    TraitEffect::GrantTag(GameplayTag::WARRIOR),
                    TraitEffect::GrantTag(GameplayTag::MELEE),
                    TraitEffect::ModifyAttribute(AttributeModifierDef {
                        kind: AttributeKind::Def,
                        op: ModifierOp::Add,
                        value: 2.0,
                    }),
                ],
            },
        );

        let (tags, modifiers) =
            apply_passive_traits(&["warrior_mastery".into()], &registry);

        assert!(tags.has(GameplayTag::WARRIOR));
        assert!(tags.has(GameplayTag::MELEE));
        assert_eq!(modifiers.len(), 1);
        assert_eq!(modifiers[0].value, 2.0);
    }

    #[test]
    fn apply_passive_traits_跳过非被动触发() {
        let mut registry = TraitRegistry::default();
        registry.traits.insert(
            "on_attack_trait".into(),
            TraitData {
                id: "on_attack_trait".into(),
                name: "攻击触发".into(),
                description: String::new(),
                trigger: TraitTrigger::OnAttack,
                effects: vec![TraitEffect::GrantTag(GameplayTag::FIRE)],
            },
        );

        let (tags, modifiers) =
            apply_passive_traits(&["on_attack_trait".into()], &registry);

        assert!(!tags.has(GameplayTag::FIRE));
        assert!(modifiers.is_empty());
    }

    #[test]
    fn ron_反序列化_触发型trait() {
        let ron_str = r#"
            (
                id: "leader_aura",
                name: "领袖光环",
                description: "回合开始时为友军施加增益",
                trigger: OnTurnStart,
                effects: [
                    ApplyBuff(buff_id: "attack_up", duration: 1),
                ],
            )
        "#;
        let def: TraitDefinition = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.id, "leader_aura");
        assert_eq!(def.trigger, TraitTrigger::OnTurnStart);
        assert_eq!(def.effects.len(), 1);
    }
}
