// Trait 系统：统一抽象，种族/职业/天赋/装备均可引用 traits
// 替代硬编码的 class_tag 单标签，支持多 trait 组合
// 支持从 assets/traits/*.ron 外部配置文件加载

mod handlers;
mod types;

pub use handlers::*;
pub use types::*;

use crate::core::attribute::{AttributeModifierDef, AttributeModifierInstance, BuffInstanceId};
use crate::core::registry_loader::RegistryLoader;
use crate::core::tag::{GameplayTag, GameplayTags, TagName};
use bevy::prelude::*;
use std::collections::HashMap;

// ── TraitData 方法（依赖 TraitEffectHandlerRegistry，放在 mod.rs）──

impl TraitData {
    /// 收集此 trait 授予的所有标签（通过 TraitEffectHandlerRegistry 分发）
    pub fn granted_tags(&self, handlers: &TraitEffectHandlerRegistry) -> Vec<GameplayTag> {
        self.effects
            .iter()
            .flat_map(|e| {
                handlers
                    .get(e.type_name())
                    .map(|h| h.granted_tags(e))
                    .unwrap_or_default()
            })
            .collect()
    }

    /// 收集此 trait 的所有属性修饰（通过 TraitEffectHandlerRegistry 分发）
    pub fn attribute_modifiers<'a>(
        &'a self,
        handlers: &'a TraitEffectHandlerRegistry,
    ) -> Vec<&'a AttributeModifierDef> {
        self.effects
            .iter()
            .flat_map(|e| {
                handlers
                    .get(e.type_name())
                    .map(|h| h.attribute_modifiers(e))
                    .unwrap_or_default()
            })
            .collect()
    }
}

/// 将一组 trait 的被动效果应用到单位的标签和属性上
/// 返回 (granted_tags, attribute_modifiers)
pub fn apply_passive_traits(
    trait_ids: &[String],
    registry: &TraitRegistry,
    handlers: &TraitEffectHandlerRegistry,
) -> (GameplayTags, Vec<AttributeModifierInstance>) {
    let mut tags = GameplayTags::default();
    let mut modifiers = Vec::new();
    // 每个 trait 分配独立的 source id，从 u64::MAX - 1 递减
    // 避免与 buff instance id（从 1 递增）冲突
    let mut trait_source_id = u64::MAX - 1;

    for trait_id in trait_ids {
        if let Some(trait_data) = registry.get(trait_id) {
            if trait_data.trigger != TraitTrigger::Passive {
                continue;
            }
            for tag in trait_data.granted_tags(handlers) {
                tags.add(tag);
            }
            let source = BuffInstanceId(trait_source_id);
            for mod_def in trait_data.attribute_modifiers(handlers) {
                modifiers.push(AttributeModifierInstance {
                    kind: mod_def.kind,
                    op: mod_def.op,
                    value: mod_def.value,
                    source,
                });
            }
            trait_source_id -= 1;
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

    /// 注册一个 Trait
    pub fn register(&mut self, trait_data: TraitData) {
        self.traits.insert(trait_data.id.clone(), trait_data);
    }

    /// 注册内置默认 Traits
    fn register_defaults(&mut self) {
        if !self.traits.is_empty() {
            return;
        }
        let defaults = vec![
            TraitDefinition {
                version: 0,
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
                version: 0,
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
                version: 0,
                id: "mage_mastery".into(),
                name: "法师精通".into(),
                description: "魔法职业，擅长元素攻击".into(),
                trigger: TraitTrigger::Passive,
                effects: vec![TraitEffectDef::GrantTag(TagName::Mage)],
            },
            TraitDefinition {
                version: 0,
                id: "fire_affinity".into(),
                name: "火焰亲和".into(),
                description: "拥有火焰力量".into(),
                trigger: TraitTrigger::Passive,
                effects: vec![TraitEffectDef::GrantTag(TagName::Fire)],
            },
            TraitDefinition {
                version: 0,
                id: "heavy_armor".into(),
                name: "重甲".into(),
                description: "装备重甲，防御+3".into(),
                trigger: TraitTrigger::Passive,
                effects: vec![TraitEffectDef::ModifyAttribute(AttributeModifierDef {
                    kind: crate::core::attribute::AttributeKind::Defense,
                    op: crate::core::attribute::ModifierOp::Add,
                    value: 3.0,
                })],
            },
        ];

        for def in defaults {
            let id = def.id.clone();
            self.traits.insert(id, def.into());
        }
    }
}

impl RegistryLoader for TraitRegistry {
    type Item = TraitDefinition;

    fn register_item(&mut self, item: TraitDefinition) {
        let id = item.id.clone();
        self.register(item.into());
        bevy::log::info!("加载Trait: {}", id);
    }

    fn register_defaults(&mut self) {
        TraitRegistry::register_defaults(self);
    }

    fn is_empty(&self) -> bool {
        self.traits.is_empty()
    }

    fn registry_name() -> &'static str {
        "Trait"
    }
}

/// Trait 插件
pub struct TraitPlugin;

impl Plugin for TraitPlugin {
    fn build(&self, app: &mut App) {
        let registry = TraitRegistry::load_from_dir("assets/traits");
        app.insert_resource(registry);
        app.insert_resource(TraitEffectHandlerRegistry::with_defaults());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::attribute::{AttributeKind, ModifierOp};
    use ron::de::from_bytes;

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
            version: 0,
            id: "test".into(),
            name: "测试".into(),
            description: "测试trait".into(),
            trigger: TraitTrigger::OnAttack,
            effects: vec![
                TraitEffectDef::GrantTag(TagName::Fire),
                TraitEffectDef::ModifyAttribute(AttributeModifierDef {
                    kind: AttributeKind::Attack,
                    op: ModifierOp::Add,
                    value: 5.0,
                }),
            ],
        };
        let data: TraitData = def.into();
        assert_eq!(data.id, "test");
        assert_eq!(data.trigger, TraitTrigger::OnAttack);
        assert_eq!(data.effects.len(), 2);

        let handlers = TraitEffectHandlerRegistry::with_defaults();
        let tags = data.granted_tags(&handlers);
        assert_eq!(tags, vec![GameplayTag::FIRE]);

        let mods = data.attribute_modifiers(&handlers);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].kind, AttributeKind::Attack);
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
                        kind: AttributeKind::Defense,
                        op: ModifierOp::Add,
                        value: 2.0,
                    }),
                ],
            },
        );

        let handlers = TraitEffectHandlerRegistry::with_defaults();
        let (tags, modifiers) =
            apply_passive_traits(&["warrior_mastery".into()], &registry, &handlers);

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

        let handlers = TraitEffectHandlerRegistry::with_defaults();
        let (tags, modifiers) =
            apply_passive_traits(&["on_attack_trait".into()], &registry, &handlers);

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

    #[test]
    fn apply_passive_traits_独立source_id() {
        let mut registry = TraitRegistry::default();
        registry.traits.insert(
            "trait_a".into(),
            TraitData {
                id: "trait_a".into(),
                name: "A".into(),
                description: String::new(),
                trigger: TraitTrigger::Passive,
                effects: vec![TraitEffect::ModifyAttribute(AttributeModifierDef {
                    kind: AttributeKind::Attack,
                    op: ModifierOp::Add,
                    value: 2.0,
                })],
            },
        );
        registry.traits.insert(
            "trait_b".into(),
            TraitData {
                id: "trait_b".into(),
                name: "B".into(),
                description: String::new(),
                trigger: TraitTrigger::Passive,
                effects: vec![TraitEffect::ModifyAttribute(AttributeModifierDef {
                    kind: AttributeKind::Defense,
                    op: ModifierOp::Add,
                    value: 3.0,
                })],
            },
        );
        let handlers = TraitEffectHandlerRegistry::with_defaults();
        let (_tags, modifiers) =
            apply_passive_traits(&["trait_a".into(), "trait_b".into()], &registry, &handlers);
        assert_eq!(modifiers.len(), 2);
        assert_ne!(modifiers[0].source, modifiers[1].source);
        assert_eq!(modifiers[0].source, BuffInstanceId(u64::MAX - 1));
        assert_eq!(modifiers[1].source, BuffInstanceId(u64::MAX - 2));
    }
}
