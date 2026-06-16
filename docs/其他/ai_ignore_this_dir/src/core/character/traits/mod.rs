/// Trait 系统：统一抽象，种族/职业/天赋/装备均可引用 traits
/// 替代硬编码的 class_tag 单标签，支持多 trait 组合
/// 支持从 content/classes/*.ron 外部配置文件加载

/// TraitEffectHandler trait 与各类型处理器
mod handlers;
/// TraitDef, TraitData, TraitSource 等类型定义
mod types;

pub use handlers::*;
pub use types::*;

use crate::core::attribute::{AttributeModifierDef, AttributeModifierInstance, ModifierSource};
use crate::core::registry_loader::RegistryLoader;
use crate::core::tag::{GameplayTag, GameplayTags};
use bevy::prelude::*;
use std::collections::HashMap;

/// ── TraitData 方法（依赖 TraitEffectHandlerRegistry，放在 mod.rs）──

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

/// 将 TraitCollection 中的被动效果应用到单位的标签和属性上
/// 返回 (granted_tags, attribute_modifiers)
pub fn apply_passive_traits(
    trait_collection: &TraitCollection,
    registry: &TraitRegistry,
    handlers: &TraitEffectHandlerRegistry,
) -> (GameplayTags, Vec<AttributeModifierInstance>) {
    let mut tags = GameplayTags::default();
    let mut modifiers = Vec::new();
    // 每个 trait 分配独立的 source id，从 ModifierSource::trait_source(0) 递增
    // Trait 区间：u64::MAX ~ u64::MAX - 999，避免与 buff/equipment 区间冲突
    let mut trait_source_index = 0u64;

    for entry in &trait_collection.entries {
        if let Some(trait_data) = registry.get(&entry.trait_id) {
            if trait_data.trigger != TraitTrigger::Passive {
                continue;
            }
            for tag in trait_data.granted_tags(handlers) {
                tags.add(tag);
            }
            let source = ModifierSource::trait_source(trait_source_index);
            for mod_def in trait_data.attribute_modifiers(handlers) {
                modifiers.push(AttributeModifierInstance {
                    config_id: mod_def.config_id.clone(),
                    op: mod_def.op,
                    value: mod_def.value,
                    source,
                });
            }
            trait_source_index += 1;
        }
    }

    (tags, modifiers)
}

/// Trait 注册表资源
#[derive(Resource, Default, Clone)]
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
                    TraitEffectDef::GrantTag("warrior".into()),
                    TraitEffectDef::GrantTag("melee".into()),
                ],
            },
            TraitDefinition {
                version: 0,
                id: "archer_mastery".into(),
                name: "弓手精通".into(),
                description: "远程职业，擅长远距离攻击".into(),
                trigger: TraitTrigger::Passive,
                effects: vec![
                    TraitEffectDef::GrantTag("archer".into()),
                    TraitEffectDef::GrantTag("ranged".into()),
                ],
            },
            TraitDefinition {
                version: 0,
                id: "mage_mastery".into(),
                name: "法师精通".into(),
                description: "魔法职业，擅长元素攻击".into(),
                trigger: TraitTrigger::Passive,
                effects: vec![TraitEffectDef::GrantTag("mage".into())],
            },
            TraitDefinition {
                version: 0,
                id: "fire_affinity".into(),
                name: "火焰亲和".into(),
                description: "拥有火焰力量".into(),
                trigger: TraitTrigger::Passive,
                effects: vec![TraitEffectDef::GrantTag("dmg_fire".into())],
            },
            TraitDefinition {
                version: 0,
                id: "heavy_armor".into(),
                name: "重甲".into(),
                description: "装备重甲，防御+3".into(),
                trigger: TraitTrigger::Passive,
                effects: vec![TraitEffectDef::ModifyAttribute(AttributeModifierDef {
                    config_id: "phys_def".into(),
                    op: crate::core::attribute::ModifierOp::Add,
                    value: 3,
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
        bevy::log::info!(target: "character", event = "trait_loaded", id = %id, "Trait已加载");
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
        let registry = TraitRegistry::load_from_dir("content/classes");
        app.insert_resource(registry);
        app.insert_resource(TraitEffectHandlerRegistry::with_defaults())
            // 注册 Reflect 类型
            .register_type::<TraitTrigger>()
            .register_type::<TraitEffect>()
            .register_type::<TraitSource>()
            .register_type::<TraitEntry>()
            .register_type::<TraitCollection>();
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
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
    use crate::core::attribute::ModifierOp;
    use ron::de::from_bytes;

    /// Test ID: CHR-TRT-001
    /// Title: RON 反序列化 Trait 定义
    ///
    /// Given: 有效的 RON 字符串
    /// When: 反序列化为 TraitDefinition
    /// Then: 所有字段正确解析
    ///
    /// Assertions: id, trigger, effects 正确
    #[test]
    fn ron_反序列化_trait定义() {
        let ron_str = r#"
            (
                id: "warrior_mastery",
                name: "战士精通",
                description: "近战职业",
                trigger: Passive,
                effects: [
                    GrantTag("dmg_physical"),
                    GrantTag("weapon_sword"),
                ],
            )
        "#;
        let def: TraitDefinition = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.id, "warrior_mastery");
        assert_eq!(def.trigger, TraitTrigger::Passive);
        assert_eq!(def.effects.len(), 2);
    }

    #[test]
    fn trait定义_转换为trait数据() {
        let def = TraitDefinition {
            version: 0,
            id: "test".into(),
            name: "测试".into(),
            description: "测试trait".into(),
            trigger: TraitTrigger::OnAttack,
            effects: vec![
                TraitEffectDef::GrantTag("dmg_fire".into()),
                TraitEffectDef::ModifyAttribute(AttributeModifierDef {
                    config_id: "phys_atk".into(),
                    op: ModifierOp::Add,
                    value: 5,
                }),
            ],
        };
        let data: TraitData = def.into();
        assert_eq!(data.id, "test");
        assert_eq!(data.trigger, TraitTrigger::OnAttack);
        assert_eq!(data.effects.len(), 2);

        let handlers = TraitEffectHandlerRegistry::with_defaults();
        let tags = data.granted_tags(&handlers);
        assert_eq!(tags, vec![GameplayTag::DMG_FIRE]);

        let mods = data.attribute_modifiers(&handlers);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].config_id, "phys_atk");
        assert_eq!(mods[0].value, 5);
    }

    /// Test ID: CHR-TRT-002
    /// Title: TraitCollection 查询已注册标签
    ///
    /// Given: 包含 warrior_mastery 和 heavy_armor 的 TraitCollection
    /// When: 查询 has()
    /// Then: 正确判断存在性
    ///
    /// Assertions: has() 返回正确的 bool
    #[test]
    fn trait集合_查询() {
        // Given
        let collection = TraitCollection::new(vec!["warrior_mastery".into(), "heavy_armor".into()]);

        // When & Then
        assert!(collection.has("warrior_mastery"));
        assert!(collection.has("heavy_armor"));
        assert!(!collection.has("mage_mastery"));
    }

    /// Test ID: CHR-TRT-003
    /// Title: apply_passive_traits 授予标签和修饰符
    ///
    /// Given: 包含 Passive trait 的 Registry 和 Collection
    /// When: 调用 apply_passive_traits()
    /// Then: 正确授予 GameplayTag 和 AttributeModifier
    ///
    /// Assertions: tags 包含 WARRIOR/MELEE, modifiers[0].value == 2.0
    #[test]
    fn 施加passive_traits_授予标签和修饰符() {
        // Given
        let mut registry = TraitRegistry::default();
        registry.traits.insert(
            "warrior_mastery".into(),
            TraitData {
                id: "warrior_mastery".into(),
                name: "战士精通".into(),
                description: String::new(),
                trigger: TraitTrigger::Passive,
                effects: vec![
                    TraitEffect::GrantTag(GameplayTag::ALLY),
                    TraitEffect::GrantTag(GameplayTag::DMG_PHYSICAL),
                    TraitEffect::ModifyAttribute(AttributeModifierDef {
                        config_id: "phys_def".into(),
                        op: ModifierOp::Add,
                        value: 2,
                    }),
                ],
            },
        );

        let handlers = TraitEffectHandlerRegistry::with_defaults();
        let collection = TraitCollection::new(vec!["warrior_mastery".into()]);

        // When
        let (tags, modifiers) = apply_passive_traits(&collection, &registry, &handlers);

        // Then
        assert!(tags.has(GameplayTag::ALLY));
        assert!(tags.has(GameplayTag::DMG_PHYSICAL));
        assert_eq!(modifiers.len(), 1);
        assert_eq!(modifiers[0].value, 2);
    }

    /// Test ID: CHR-TRT-004
    /// Title: apply_passive_traits 跳过非 Passive 触发
    ///
    /// Given: 包含 OnAttack trait 的 Registry 和 Collection
    /// When: 调用 apply_passive_traits()
    /// Then: 无标签授予，无修饰符
    ///
    /// Assertions: !tags.has(FIRE), modifiers.is_empty()
    #[test]
    fn 施加passive_traits_跳过非passive触发器() {
        // Given
        let mut registry = TraitRegistry::default();
        registry.traits.insert(
            "on_attack_trait".into(),
            TraitData {
                id: "on_attack_trait".into(),
                name: "攻击触发".into(),
                description: String::new(),
                trigger: TraitTrigger::OnAttack,
                effects: vec![TraitEffect::GrantTag(GameplayTag::DMG_FIRE)],
            },
        );

        let handlers = TraitEffectHandlerRegistry::with_defaults();
        let collection = TraitCollection::new(vec!["on_attack_trait".into()]);

        // When
        let (tags, modifiers) = apply_passive_traits(&collection, &registry, &handlers);

        // Then
        assert!(!tags.has(GameplayTag::DMG_FIRE));
        assert!(modifiers.is_empty());
    }

    /// Test ID: CHR-TRT-005
    /// Title: RON 反序列化 OnTurnStart 触发型 Trait
    ///
    /// Given: 有效的 RON 字符串（OnTurnStart 触发）
    /// When: 反序列化为 TraitDefinition
    /// Then: 所有字段正确解析
    ///
    /// Assertions: id, trigger, effects.len() 正确
    #[test]
    fn ron_反序列化_trait带触发器() {
        // Given
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

        // When
        let def: TraitDefinition = from_bytes(ron_str.as_bytes()).unwrap();

        // Then
        assert_eq!(def.id, "leader_aura");
        assert_eq!(def.trigger, TraitTrigger::OnTurnStart);
        assert_eq!(def.effects.len(), 1);
    }

    /// Test ID: CHR-TRT-006
    /// Title: apply_passive_traits 独立 source_id
    ///
    /// Given: 包含多个 Passive trait 的 Registry 和 Collection
    /// When: 调用 apply_passive_traits()
    /// Then: 每个修饰符有独立的 source_id
    ///
    /// Assertions: modifiers[0].source != modifiers[1].source
    #[test]
    fn 施加passive_traits_独立source_id() {
        // Given
        let mut registry = TraitRegistry::default();
        registry.traits.insert(
            "trait_a".into(),
            TraitData {
                id: "trait_a".into(),
                name: "A".into(),
                description: String::new(),
                trigger: TraitTrigger::Passive,
                effects: vec![TraitEffect::ModifyAttribute(AttributeModifierDef {
                    config_id: "phys_atk".into(),
                    op: ModifierOp::Add,
                    value: 2,
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
                    config_id: "phys_def".into(),
                    op: ModifierOp::Add,
                    value: 3,
                })],
            },
        );

        let handlers = TraitEffectHandlerRegistry::with_defaults();
        let collection = TraitCollection::new(vec!["trait_a".into(), "trait_b".into()]);

        // When
        let (_tags, modifiers) = apply_passive_traits(&collection, &registry, &handlers);

        // Then
        assert_eq!(modifiers.len(), 2);
        assert_ne!(modifiers[0].source, modifiers[1].source);
        assert_eq!(modifiers[0].source, ModifierSource::trait_source(0));
        assert_eq!(modifiers[1].source, ModifierSource::trait_source(1));
    }
}
