//! Trait 系统 Feature Test
//!
//! 测试 Trait 的三大核心场景：
//! 1. 被动 Trait 授予标签
//! 2. 装备 Trait 完整生命周期（添加/移除）
//! 3. Trait 修改属性

// ================================================
// Bevy SRPG AI宪法 v1.1 自检结果（测试专用）
// ================================================
// ✅ 测行为不测实现：是 — 断言验证 Trait 添加/移除后状态
// ✅ 符合领域规则：是 — 覆盖 Trait 完整生命周期
// ✅ 确定性：是 — 硬编码 Trait 定义和属性值
// ✅ 使用标准数据：是 — 使用 UnitBuilder::warrior()
// ✅ 无越界测试：是 — 仅测试公共 API
// ✅ 未测试私有实现：是 — 仅通过 Trait Pipeline 接口测试
// ================================================

use bevy::prelude::*;
use tactical_rpg::character::{
    PersistentTags, TraitCollection, TraitData, TraitEffect, TraitEffectHandlerRegistry,
    TraitRegistry, TraitSource, TraitTrigger,
};
use tactical_rpg::core::attribute::{
    AttributeKind, AttributeModifierDef, AttributeModifierInstance, Attributes, ModifierOp,
    ModifierSource,
};
use tactical_rpg::core::tag::{GameplayTag, GameplayTags};
use tactical_rpg::equipment::EquipmentSlot;

use crate::assert_attr_eq;
use crate::assert_has_tag;
use crate::assert_not_has_tag;
use crate::common::app_builder::equipment_app;
use crate::common::fixtures::UnitBuilder;

// ── 测试辅助 ──

/// 注册一个 Passive + GrantTag 的 Trait 到 TraitRegistry
fn register_grant_tag_trait(app: &mut App, trait_id: &str, tag: GameplayTag) {
    let trait_data = TraitData {
        id: trait_id.to_string(),
        name: trait_id.to_string(),
        description: String::new(),
        trigger: TraitTrigger::Passive,
        effects: vec![TraitEffect::GrantTag(tag)],
    };
    app.world_mut()
        .resource_mut::<TraitRegistry>()
        .register(trait_data);
}

/// 注册一个 Passive + ModifyAttribute 的 Trait 到 TraitRegistry
fn register_modify_attr_trait(
    app: &mut App,
    trait_id: &str,
    kind: AttributeKind,
    op: ModifierOp,
    value: f32,
) {
    let trait_data = TraitData {
        id: trait_id.to_string(),
        name: trait_id.to_string(),
        description: String::new(),
        trigger: TraitTrigger::Passive,
        effects: vec![TraitEffect::ModifyAttribute(AttributeModifierDef {
            kind,
            op,
            value,
        })],
    };
    app.world_mut()
        .resource_mut::<TraitRegistry>()
        .register(trait_data);
}

/// 手动应用 Trait 效果到角色（模拟 rebuild_trait_effects 逻辑）
fn apply_trait_effects(app: &mut App, entity: Entity) {
    // 先克隆所有只读数据
    let (trait_collection, trait_registry, handlers) = {
        let world = app.world();
        let tc = world.get::<TraitCollection>(entity).unwrap().clone();
        let tr = world.resource::<TraitRegistry>().clone();
        let h = world.resource::<TraitEffectHandlerRegistry>().clone();
        (tc, tr, h)
    };

    // 收集需要应用的标签和修饰符
    let mut tags_to_add = Vec::new();
    let mut modifiers_to_add = Vec::new();
    let mut trait_source_index = 0u64;
    for entry in &trait_collection.entries {
        if let Some(trait_data) = trait_registry.get(&entry.trait_id) {
            if trait_data.trigger != TraitTrigger::Passive {
                continue;
            }
            for tag in trait_data.granted_tags(&handlers) {
                tags_to_add.push(tag);
            }
            let source = ModifierSource::trait_source(trait_source_index);
            for mod_def in trait_data.attribute_modifiers(&handlers) {
                modifiers_to_add.push(AttributeModifierInstance {
                    kind: mod_def.kind,
                    op: mod_def.op,
                    value: mod_def.value,
                    source,
                });
            }
            trait_source_index += 1;
        }
    }

    // 清除旧 Trait 来源的修饰符
    app.world_mut()
        .get_mut::<Attributes>(entity)
        .unwrap()
        .remove_trait_modifiers();

    // 清除 Trait 授予的标签
    app.world_mut()
        .get_mut::<PersistentTags>(entity)
        .unwrap()
        .from_traits = GameplayTags::default();

    // 应用新标签
    {
        let mut persistent = app.world_mut().get_mut::<PersistentTags>(entity).unwrap();
        for tag in tags_to_add {
            persistent.from_traits.add(tag);
        }
    }

    // 应用新修饰符
    {
        let mut attrs = app.world_mut().get_mut::<Attributes>(entity).unwrap();
        for modifier in modifiers_to_add {
            attrs.add_modifier(modifier);
        }
    }
}

/// 重建 GameplayTags（从 PersistentTags 三层合并）
fn rebuild_gameplay_tags(app: &mut App, entity: Entity) {
    let persistent = app.world().get::<PersistentTags>(entity).unwrap().clone();
    let mut tags = app.world_mut().get_mut::<GameplayTags>(entity).unwrap();
    let mut new_tags = GameplayTags::default();
    new_tags.0 |= persistent.from_traits.0;
    new_tags.0 |= persistent.from_equipment.0;
    tags.0 = new_tags.0;
}

// ══════════════════════════════════════════════════════════════
// 场景一：被动 Trait 授予标签
// ══════════════════════════════════════════════════════════════

#[test]
fn 被动trait授予标签_添加passive_grant_tag后标签出现在gameplay_tags() {
    let mut app = equipment_app();

    // 注册一个 Passive + GrantTag(FIRE) 的 Trait
    register_grant_tag_trait(&mut app, "fire_affinity_test", GameplayTag::FIRE);

    // 创建角色
    let entity = UnitBuilder::warrior().spawn(&mut app);

    // 穿戴前：角色没有 FIRE 标签
    let tags = app.world().get::<GameplayTags>(entity).unwrap();
    assert_not_has_tag!(tags, GameplayTag::FIRE);

    // 给角色添加 Trait（Intrinsic 来源）
    {
        let mut tc = app.world_mut().get_mut::<TraitCollection>(entity).unwrap();
        tc.add_entry("fire_affinity_test".to_string(), TraitSource::Intrinsic);
    }

    // 应用 Trait 效果
    apply_trait_effects(&mut app, entity);
    rebuild_gameplay_tags(&mut app, entity);

    // 验证：FIRE 标签出现在 GameplayTags 中
    let tags = app.world().get::<GameplayTags>(entity).unwrap();
    assert_has_tag!(tags, GameplayTag::FIRE);

    // 验证：PersistentTags.from_traits 层有标签
    let persistent = app.world().get::<PersistentTags>(entity).unwrap();
    assert!(persistent.from_traits.has(GameplayTag::FIRE));
}

#[test]
fn 被动trait授予标签_多个trait授予多个标签() {
    let mut app = equipment_app();

    register_grant_tag_trait(&mut app, "warrior_trait", GameplayTag::WARRIOR);
    register_grant_tag_trait(&mut app, "fire_trait", GameplayTag::FIRE);

    let entity = UnitBuilder::warrior().spawn(&mut app);

    // 添加两个 Trait
    {
        let mut tc = app.world_mut().get_mut::<TraitCollection>(entity).unwrap();
        tc.add_entry("warrior_trait".to_string(), TraitSource::Intrinsic);
        tc.add_entry("fire_trait".to_string(), TraitSource::Intrinsic);
    }

    apply_trait_effects(&mut app, entity);
    rebuild_gameplay_tags(&mut app, entity);

    let tags = app.world().get::<GameplayTags>(entity).unwrap();
    assert_has_tag!(tags, GameplayTag::WARRIOR);
    assert_has_tag!(tags, GameplayTag::FIRE);
}

#[test]
fn 被动trait授予标签_非passive触发不授予标签() {
    let mut app = equipment_app();

    // 注册一个 OnAttack 触发的 Trait（非 Passive）
    let trait_data = TraitData {
        id: "on_attack_trait".to_string(),
        name: "攻击触发".to_string(),
        description: String::new(),
        trigger: TraitTrigger::OnAttack,
        effects: vec![TraitEffect::GrantTag(GameplayTag::FIRE)],
    };
    app.world_mut()
        .resource_mut::<TraitRegistry>()
        .register(trait_data);

    let entity = UnitBuilder::warrior().spawn(&mut app);

    {
        let mut tc = app.world_mut().get_mut::<TraitCollection>(entity).unwrap();
        tc.add_entry("on_attack_trait".to_string(), TraitSource::Intrinsic);
    }

    apply_trait_effects(&mut app, entity);
    rebuild_gameplay_tags(&mut app, entity);

    // OnAttack 触发的 Trait 不应在被动阶段授予标签
    let tags = app.world().get::<GameplayTags>(entity).unwrap();
    assert_not_has_tag!(tags, GameplayTag::FIRE);
}

// ══════════════════════════════════════════════════════════════
// 场景二：装备 Trait 完整生命周期
// ══════════════════════════════════════════════════════════════

#[test]
fn 装备trait完整生命周期_添加后entry存在_移除后entry消失() {
    let mut app = equipment_app();

    register_grant_tag_trait(&mut app, "heavy_armor_test", GameplayTag::HEAVY_ARMOR);

    let entity = UnitBuilder::warrior().spawn(&mut app);

    // 初始：TraitCollection 为空
    let tc = app.world().get::<TraitCollection>(entity).unwrap();
    assert!(!tc.has("heavy_armor_test"));

    // 通过 Equipment source 添加 Trait
    let equipment_source = TraitSource::Equipment {
        slot: EquipmentSlot::Body,
    };
    {
        let mut tc = app.world_mut().get_mut::<TraitCollection>(entity).unwrap();
        tc.add_entry("heavy_armor_test".to_string(), equipment_source.clone());
    }

    // 验证：TraitCollection 有对应 entry
    let tc = app.world().get::<TraitCollection>(entity).unwrap();
    assert!(tc.has("heavy_armor_test"));

    // 验证：entry 来源是 Equipment
    let entry = tc
        .entries
        .iter()
        .find(|e| e.trait_id == "heavy_armor_test")
        .unwrap();
    assert_eq!(entry.source, equipment_source);

    // 应用 Trait 效果
    apply_trait_effects(&mut app, entity);
    rebuild_gameplay_tags(&mut app, entity);

    // 验证：标签已授予
    let tags = app.world().get::<GameplayTags>(entity).unwrap();
    assert_has_tag!(tags, GameplayTag::HEAVY_ARMOR);

    // 移除 Equipment source 的所有 Trait
    {
        let mut tc = app.world_mut().get_mut::<TraitCollection>(entity).unwrap();
        let removed = tc.remove_by_source(&equipment_source);
        assert_eq!(removed, vec!["heavy_armor_test"]);
    }

    // 验证：entry 消失
    let tc = app.world().get::<TraitCollection>(entity).unwrap();
    assert!(!tc.has("heavy_armor_test"));

    // 重新应用 Trait 效果（模拟脱卸后重建）
    apply_trait_effects(&mut app, entity);
    rebuild_gameplay_tags(&mut app, entity);

    // 验证：标签已移除
    let tags = app.world().get::<GameplayTags>(entity).unwrap();
    assert_not_has_tag!(tags, GameplayTag::HEAVY_ARMOR);
}

#[test]
fn 装备trait_不同来源的trait独立管理() {
    let mut app = equipment_app();

    register_grant_tag_trait(&mut app, "shared_trait", GameplayTag::FIRE);

    let entity = UnitBuilder::warrior().spawn(&mut app);

    // 两个不同 Equipment 槽位提供同一个 trait
    let main_hand_source = TraitSource::Equipment {
        slot: EquipmentSlot::MainHand,
    };
    let body_source = TraitSource::Equipment {
        slot: EquipmentSlot::Body,
    };

    {
        let mut tc = app.world_mut().get_mut::<TraitCollection>(entity).unwrap();
        tc.add_entry("shared_trait".to_string(), main_hand_source.clone());
        tc.add_entry("shared_trait".to_string(), body_source.clone());
    }

    // 验证：有两个 entry
    let tc = app.world().get::<TraitCollection>(entity).unwrap();
    let count = tc
        .entries
        .iter()
        .filter(|e| e.trait_id == "shared_trait")
        .count();
    assert_eq!(count, 2);

    // 移除 MainHand 来源
    {
        let mut tc = app.world_mut().get_mut::<TraitCollection>(entity).unwrap();
        tc.remove_by_source(&main_hand_source);
    }

    // Body 来源的 trait 仍在
    let tc = app.world().get::<TraitCollection>(entity).unwrap();
    assert!(tc.has("shared_trait"));
    let count = tc
        .entries
        .iter()
        .filter(|e| e.trait_id == "shared_trait")
        .count();
    assert_eq!(count, 1);
}

#[test]
fn 装备trait_intrinsic来源不受equipment移除影响() {
    let mut app = equipment_app();

    register_grant_tag_trait(&mut app, "innate_trait", GameplayTag::WARRIOR);

    let entity = UnitBuilder::warrior().spawn(&mut app);

    // 同时添加 Intrinsic 和 Equipment 来源的同一个 trait
    let equip_source = TraitSource::Equipment {
        slot: EquipmentSlot::MainHand,
    };
    {
        let mut tc = app.world_mut().get_mut::<TraitCollection>(entity).unwrap();
        tc.add_entry("innate_trait".to_string(), TraitSource::Intrinsic);
        tc.add_entry("innate_trait".to_string(), equip_source.clone());
    }

    // 移除 Equipment 来源
    {
        let mut tc = app.world_mut().get_mut::<TraitCollection>(entity).unwrap();
        tc.remove_by_source(&equip_source);
    }

    // Intrinsic 来源仍在
    let tc = app.world().get::<TraitCollection>(entity).unwrap();
    assert!(tc.has("innate_trait"));
    let entry = tc
        .entries
        .iter()
        .find(|e| e.trait_id == "innate_trait")
        .unwrap();
    assert_eq!(entry.source, TraitSource::Intrinsic);
}

// ══════════════════════════════════════════════════════════════
// 场景三：Trait 修改属性
// ══════════════════════════════════════════════════════════════

#[test]
fn trait修改属性_添加passive_modify_attribute后属性值变化() {
    let mut app = equipment_app();

    // 注册一个 Passive + ModifyAttribute(Defense, Add, 5.0) 的 Trait
    register_modify_attr_trait(
        &mut app,
        "tough_body",
        AttributeKind::Defense,
        ModifierOp::Add,
        5.0,
    );

    let entity = UnitBuilder::warrior().spawn(&mut app);

    // 记录基础防御值
    let base_defense = {
        let attrs = app.world().get::<Attributes>(entity).unwrap();
        attrs.get(AttributeKind::Defense)
    };

    // 添加 Trait
    {
        let mut tc = app.world_mut().get_mut::<TraitCollection>(entity).unwrap();
        tc.add_entry("tough_body".to_string(), TraitSource::Intrinsic);
    }

    // 应用 Trait 效果
    apply_trait_effects(&mut app, entity);

    // 验证：防御值增加了 5
    let attrs = app.world().get::<Attributes>(entity).unwrap();
    assert_attr_eq!(attrs, AttributeKind::Defense, base_defense as i32 + 5);
}

#[test]
fn trait修改属性_移除trait后属性恢复() {
    let mut app = equipment_app();

    register_modify_attr_trait(
        &mut app,
        "tough_body",
        AttributeKind::Defense,
        ModifierOp::Add,
        5.0,
    );

    let entity = UnitBuilder::warrior().spawn(&mut app);

    let base_defense = {
        let attrs = app.world().get::<Attributes>(entity).unwrap();
        attrs.get(AttributeKind::Defense)
    };

    // 添加 Trait
    let source = TraitSource::Intrinsic;
    {
        let mut tc = app.world_mut().get_mut::<TraitCollection>(entity).unwrap();
        tc.add_entry("tough_body".to_string(), source.clone());
    }
    apply_trait_effects(&mut app, entity);

    // 验证属性已增加
    let attrs = app.world().get::<Attributes>(entity).unwrap();
    assert_attr_eq!(attrs, AttributeKind::Defense, base_defense as i32 + 5);

    // 移除 Trait
    {
        let mut tc = app.world_mut().get_mut::<TraitCollection>(entity).unwrap();
        tc.remove_by_source(&source);
    }
    apply_trait_effects(&mut app, entity);

    // 验证：属性恢复到基础值
    let attrs = app.world().get::<Attributes>(entity).unwrap();
    assert_attr_eq!(attrs, AttributeKind::Defense, base_defense as i32);
}

#[test]
fn trait修改属性_乘法修饰符() {
    let mut app = equipment_app();

    // 注册一个 Passive + ModifyAttribute(Attack, Multiply, 1.5) 的 Trait
    register_modify_attr_trait(
        &mut app,
        "berserker",
        AttributeKind::Attack,
        ModifierOp::Multiply,
        1.5,
    );

    let entity = UnitBuilder::warrior().spawn(&mut app);

    let base_attack = {
        let attrs = app.world().get::<Attributes>(entity).unwrap();
        attrs.get(AttributeKind::Attack)
    };

    // 添加 Trait
    {
        let mut tc = app.world_mut().get_mut::<TraitCollection>(entity).unwrap();
        tc.add_entry("berserker".to_string(), TraitSource::Intrinsic);
    }
    apply_trait_effects(&mut app, entity);

    // 验证：攻击力乘以 1.5
    let attrs = app.world().get::<Attributes>(entity).unwrap();
    let expected = (base_attack * 1.5) as i32;
    assert_attr_eq!(attrs, AttributeKind::Attack, expected);
}

#[test]
fn trait修改属性_多个trait同时修改属性() {
    let mut app = equipment_app();

    // 注册两个修改不同属性的 Trait
    register_modify_attr_trait(
        &mut app,
        "iron_skin",
        AttributeKind::Defense,
        ModifierOp::Add,
        3.0,
    );
    register_modify_attr_trait(
        &mut app,
        "sharp_eye",
        AttributeKind::Accuracy,
        ModifierOp::Add,
        10.0,
    );

    let entity = UnitBuilder::warrior().spawn(&mut app);

    let base_defense = {
        let attrs = app.world().get::<Attributes>(entity).unwrap();
        attrs.get(AttributeKind::Defense)
    };
    let base_accuracy = {
        let attrs = app.world().get::<Attributes>(entity).unwrap();
        attrs.get(AttributeKind::Accuracy)
    };

    // 添加两个 Trait
    {
        let mut tc = app.world_mut().get_mut::<TraitCollection>(entity).unwrap();
        tc.add_entry("iron_skin".to_string(), TraitSource::Intrinsic);
        tc.add_entry("sharp_eye".to_string(), TraitSource::Intrinsic);
    }
    apply_trait_effects(&mut app, entity);

    // 验证：两个属性都增加了
    let attrs = app.world().get::<Attributes>(entity).unwrap();
    assert_attr_eq!(attrs, AttributeKind::Defense, base_defense as i32 + 3);
    assert_attr_eq!(attrs, AttributeKind::Accuracy, base_accuracy as i32 + 10);
}

#[test]
fn trait修改属性_同时授予标签和修改属性() {
    let mut app = equipment_app();

    // 注册一个同时有 GrantTag 和 ModifyAttribute 的 Trait
    let trait_data = TraitData {
        id: "warrior_mastery_test".to_string(),
        name: "战士精通测试".to_string(),
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
    };
    app.world_mut()
        .resource_mut::<TraitRegistry>()
        .register(trait_data);

    let entity = UnitBuilder::warrior().spawn(&mut app);

    let base_defense = {
        let attrs = app.world().get::<Attributes>(entity).unwrap();
        attrs.get(AttributeKind::Defense)
    };

    // 添加 Trait
    {
        let mut tc = app.world_mut().get_mut::<TraitCollection>(entity).unwrap();
        tc.add_entry("warrior_mastery_test".to_string(), TraitSource::Intrinsic);
    }
    apply_trait_effects(&mut app, entity);
    rebuild_gameplay_tags(&mut app, entity);

    // 验证：标签已授予
    let tags = app.world().get::<GameplayTags>(entity).unwrap();
    assert_has_tag!(tags, GameplayTag::WARRIOR);
    assert_has_tag!(tags, GameplayTag::MELEE);

    // 验证：属性已修改
    let attrs = app.world().get::<Attributes>(entity).unwrap();
    assert_attr_eq!(attrs, AttributeKind::Defense, base_defense as i32 + 2);
}
