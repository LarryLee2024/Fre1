//! P0 集成测试：战斗效果管道
//!
//! 测试 calculate_damage_from_effect 在各种属性组合下的行为，
//! 跨 attribute + effect/types 模块验证完整的伤害计算链路。

use tactical_rpg::gameplay::attribute::{AttributeKind, Attributes};
use tactical_rpg::gameplay::effect::{
    EffectDef, EffectHandlerRegistry, EffectPreview, EffectQueue, GenerateContext, PendingEffect,
    PendingEffectData, PreviewContext, calculate_damage_from_effect,
};
use tactical_rpg::gameplay::tag::{GameplayTag, GameplayTags};
use tactical_rpg::map::Terrain;

// ── 测试辅助 ──

/// 战士模板：Might=5, Vitality=5 → Attack=10, Defense=5, MaxHp=30
fn warrior_attrs() -> Attributes {
    let mut a = Attributes::default();
    a.set_base(AttributeKind::Might, 5.0);
    a.set_base(AttributeKind::Vitality, 5.0);
    a.set_base(AttributeKind::Agility, 6.0);
    a.set_base(AttributeKind::Dexterity, 3.0);
    a.set_base(AttributeKind::Intelligence, 2.0);
    a.set_base(AttributeKind::Willpower, 3.0);
    a.set_base(AttributeKind::Presence, 2.0);
    a.set_base(AttributeKind::Luck, 2.0);
    a.set_base_attack_range(1);
    a.fill_vital_resources();
    a
}

/// 哥布林模板：Might=4, Vitality=3 → Attack=8, Defense=3, MaxHp=20
fn goblin_attrs() -> Attributes {
    let mut a = Attributes::default();
    a.set_base(AttributeKind::Might, 4.0);
    a.set_base(AttributeKind::Vitality, 3.0);
    a.set_base(AttributeKind::Agility, 4.0);
    a.set_base(AttributeKind::Dexterity, 2.0);
    a.set_base(AttributeKind::Intelligence, 1.0);
    a.set_base(AttributeKind::Willpower, 2.0);
    a.set_base(AttributeKind::Presence, 1.0);
    a.set_base(AttributeKind::Luck, 2.0);
    a.set_base_attack_range(1);
    a.fill_vital_resources();
    a
}

// ══════════════════════════════════════════════════════════════
// 场景一：基础伤害计算（calculate_damage_from_effect）
// ══════════════════════════════════════════════════════════════

#[test]
fn 战士攻击哥布林_基础伤害() {
    // ATK=10, DEF=3, 无地形加成 → 10-3=7
    let dmg = calculate_damage_from_effect(10.0, 3.0, 3.0, 1.0, 0.0, 0);
    assert_eq!(dmg, 7);
}

#[test]
fn 森林地形_防御加成() {
    // ATK=10, DEF=3, terrain_bonus=2 → 10-3-2=5
    let dmg = calculate_damage_from_effect(10.0, 3.0, 3.0, 1.0, 0.0, 2);
    assert_eq!(dmg, 5);
}

#[test]
fn 山地地形_无防御加成() {
    let dmg = calculate_damage_from_effect(10.0, 3.0, 3.0, 1.0, 0.0, 0);
    assert_eq!(dmg, 7);
}

#[test]
fn 伤害下限为1() {
    let dmg = calculate_damage_from_effect(5.0, 10.0, 10.0, 1.0, 0.0, 0);
    assert_eq!(dmg, 1);
}

#[test]
fn 技能倍率_1_5倍() {
    // (10-3)*1.5=10.5 → 10
    let dmg = calculate_damage_from_effect(10.0, 3.0, 3.0, 1.5, 0.0, 0);
    assert_eq!(dmg, 10);
}

#[test]
fn 技能倍率_3倍() {
    let dmg = calculate_damage_from_effect(10.0, 3.0, 3.0, 3.0, 0.0, 0);
    assert_eq!(dmg, 21);
}

#[test]
fn 无视防御_50百分比() {
    // final_def=10-5=5, (10-5)*1.3=6.5→6
    let dmg = calculate_damage_from_effect(10.0, 10.0, 10.0, 1.3, 50.0, 0);
    assert_eq!(dmg, 6);
}

#[test]
fn 无视防御_100百分比() {
    let dmg = calculate_damage_from_effect(10.0, 10.0, 10.0, 1.0, 100.0, 0);
    assert_eq!(dmg, 10);
}

#[test]
fn 地形加成与无视防御叠加() {
    // final_def=10-5=5, base=10-5=5, 5-2=3
    let dmg = calculate_damage_from_effect(10.0, 10.0, 10.0, 1.0, 50.0, 2);
    assert_eq!(dmg, 3);
}

// ══════════════════════════════════════════════════════════════
// 场景二：EffectHandlerRegistry → generate → PendingEffect
// ══════════════════════════════════════════════════════════════

#[test]
fn 伤害处理器_generate_基础攻击() {
    let registry = EffectHandlerRegistry::default();
    let handler = registry.find("Damage").unwrap();

    let mut target = goblin_attrs();
    target.set_base(AttributeKind::Hp, 15.0);

    let ctx = GenerateContext {
        source_entity: bevy::prelude::Entity::from_bits(1),
        target_entity: bevy::prelude::Entity::from_bits(2),
        source_attrs: warrior_attrs(),
        target_attrs: target,
        defense_bonus: 0,
        skill_id: "basic_attack".into(),
        source_tags: vec![],
        terrain: Terrain::Plain,
    };

    let def = EffectDef::Damage {
        multiplier: 1.0,
        ignore_def_percent: 0.0,
    };
    let result = handler.generate(&def, &ctx).unwrap();
    match result {
        PendingEffectData::Damage { amount, is_skill } => {
            assert_eq!(amount, 7);
            assert!(!is_skill);
        }
        _ => panic!("应该是 Damage"),
    }
}

#[test]
fn 伤害处理器_generate_技能攻击() {
    let registry = EffectHandlerRegistry::default();
    let handler = registry.find("Damage").unwrap();

    let ctx = GenerateContext {
        source_entity: bevy::prelude::Entity::from_bits(1),
        target_entity: bevy::prelude::Entity::from_bits(2),
        source_attrs: warrior_attrs(),
        target_attrs: goblin_attrs(),
        defense_bonus: 0,
        skill_id: "power_strike".into(),
        source_tags: vec![],
        terrain: Terrain::Plain,
    };

    let def = EffectDef::Damage {
        multiplier: 1.5,
        ignore_def_percent: 0.0,
    };
    let result = handler.generate(&def, &ctx).unwrap();
    if let PendingEffectData::Damage { amount, is_skill } = result {
        assert_eq!(amount, 10); // (10-3)*1.5=10.5→10
        assert!(is_skill);
    } else {
        panic!("应该是 Damage");
    }
}

#[test]
fn 治疗处理器_generate() {
    let registry = EffectHandlerRegistry::default();
    let handler = registry.find("Heal").unwrap();

    let mut target = warrior_attrs();
    target.set_base(AttributeKind::Hp, 15.0);

    let ctx = GenerateContext {
        source_entity: bevy::prelude::Entity::from_bits(1),
        target_entity: bevy::prelude::Entity::from_bits(2),
        source_attrs: warrior_attrs(),
        target_attrs: target,
        defense_bonus: 0,
        skill_id: "heal".into(),
        source_tags: vec![],
        terrain: Terrain::Plain,
    };

    let def = EffectDef::Heal { amount: 8 };
    let result = handler.generate(&def, &ctx).unwrap();
    if let PendingEffectData::Heal { amount } = result {
        assert_eq!(amount, 8);
    } else {
        panic!("应该是 Heal");
    }
}

#[test]
fn buff处理器_generate() {
    let registry = EffectHandlerRegistry::default();
    let handler = registry.find("ApplyBuff").unwrap();

    let ctx = GenerateContext {
        source_entity: bevy::prelude::Entity::from_bits(1),
        target_entity: bevy::prelude::Entity::from_bits(2),
        source_attrs: warrior_attrs(),
        target_attrs: goblin_attrs(),
        defense_bonus: 0,
        skill_id: "fireball".into(),
        source_tags: vec![],
        terrain: Terrain::Plain,
    };

    let def = EffectDef::ApplyBuff {
        buff_id: "burn".into(),
        duration: 2,
    };
    let result = handler.generate(&def, &ctx).unwrap();
    if let PendingEffectData::ApplyBuff { buff_id, duration } = result {
        assert_eq!(buff_id, "burn");
        assert_eq!(duration, 2);
    } else {
        panic!("应该是 ApplyBuff");
    }
}

#[test]
fn 净化处理器_generate() {
    let registry = EffectHandlerRegistry::default();
    let handler = registry.find("Cleanse").unwrap();

    let ctx = GenerateContext {
        source_entity: bevy::prelude::Entity::from_bits(1),
        target_entity: bevy::prelude::Entity::from_bits(2),
        source_attrs: warrior_attrs(),
        target_attrs: goblin_attrs(),
        defense_bonus: 0,
        skill_id: "cleanse".into(),
        source_tags: vec![],
        terrain: Terrain::Plain,
    };

    let def = EffectDef::Cleanse;
    let result = handler.generate(&def, &ctx).unwrap();
    assert!(matches!(result, PendingEffectData::Cleanse));
}

// ══════════════════════════════════════════════════════════════
// 场景三：预览 → 执行一致性
// ══════════════════════════════════════════════════════════════

#[test]
fn 伤害预览与generate一致() {
    let registry = EffectHandlerRegistry::default();
    let damage_handler = registry.find("Damage").unwrap();

    let mut target = goblin_attrs();
    target.set_base(AttributeKind::Hp, 15.0);

    let gen_ctx = GenerateContext {
        source_entity: bevy::prelude::Entity::from_bits(1),
        target_entity: bevy::prelude::Entity::from_bits(2),
        source_attrs: warrior_attrs(),
        target_attrs: target.clone(),
        defense_bonus: 0,
        skill_id: "basic_attack".into(),
        source_tags: vec![],
        terrain: Terrain::Plain,
    };

    let preview_ctx = PreviewContext {
        source_attrs: warrior_attrs(),
        target_attrs: target,
        terrain_defense_bonus: 0,
        buff_registry: tactical_rpg::buff::BuffRegistry::default(),
    };

    let def = EffectDef::Damage {
        multiplier: 1.0,
        ignore_def_percent: 0.0,
    };

    let generated = damage_handler.generate(&def, &gen_ctx).unwrap();
    let gen_amount = match generated {
        PendingEffectData::Damage { amount, .. } => amount,
        _ => panic!(),
    };

    let preview = damage_handler.preview(&def, &preview_ctx).unwrap();
    let preview_amount = match preview {
        EffectPreview::Damage { amount, .. } => amount,
        _ => panic!(),
    };

    assert_eq!(gen_amount, preview_amount);
}

#[test]
fn 治疗预览不超过最大hp() {
    let registry = EffectHandlerRegistry::default();
    let handler = registry.find("Heal").unwrap();

    let mut target = warrior_attrs();
    target.set_base(AttributeKind::Hp, 28.0); // MaxHp=30

    let ctx = PreviewContext {
        source_attrs: warrior_attrs(),
        target_attrs: target,
        terrain_defense_bonus: 0,
        buff_registry: tactical_rpg::buff::BuffRegistry::default(),
    };

    let def = EffectDef::Heal { amount: 8 };
    let preview = handler.preview(&def, &ctx).unwrap();
    if let EffectPreview::Heal { amount } = preview {
        assert_eq!(amount, 2); // min(8, 30-28)=2
    } else {
        panic!("应该是 Heal 预览");
    }
}

#[test]
fn 伤害预览致死标记() {
    let registry = EffectHandlerRegistry::default();
    let handler = registry.find("Damage").unwrap();

    let mut source = warrior_attrs();
    source.set_base(AttributeKind::Might, 25.0); // Attack=50
    let mut target = goblin_attrs();
    target.set_base(AttributeKind::Hp, 5.0);

    let ctx = PreviewContext {
        source_attrs: source,
        target_attrs: target,
        terrain_defense_bonus: 0,
        buff_registry: tactical_rpg::buff::BuffRegistry::default(),
    };

    let def = EffectDef::Damage {
        multiplier: 1.0,
        ignore_def_percent: 0.0,
    };
    let preview = handler.preview(&def, &ctx).unwrap();
    if let EffectPreview::Damage { lethal, .. } = preview {
        assert!(lethal);
    } else {
        panic!("应该是 Damage 预览");
    }
}

// ══════════════════════════════════════════════════════════════
// 场景四：EffectQueue 管道串联
// ══════════════════════════════════════════════════════════════

#[test]
fn effect_queue_push_then_drain() {
    let mut queue = EffectQueue::default();
    assert!(queue.is_empty());

    queue.push(PendingEffect {
        source: bevy::prelude::Entity::from_bits(1),
        target: bevy::prelude::Entity::from_bits(2),
        data: PendingEffectData::Damage {
            amount: 7,
            is_skill: false,
        },
        source_tags: vec![],
        terrain: Terrain::Plain,
    });

    queue.push(PendingEffect {
        source: bevy::prelude::Entity::from_bits(3),
        target: bevy::prelude::Entity::from_bits(4),
        data: PendingEffectData::Heal { amount: 5 },
        source_tags: vec![],
        terrain: Terrain::Plain,
    });

    assert_eq!(queue.pending.len(), 2);

    let effects: Vec<_> = queue.pending.drain(..).collect();
    assert!(matches!(
        effects[0].data,
        PendingEffectData::Damage { amount: 7, .. }
    ));
    assert!(matches!(
        effects[1].data,
        PendingEffectData::Heal { amount: 5 }
    ));
    assert!(queue.is_empty());
}

// ══════════════════════════════════════════════════════════════
// 场景五：伤害计算 × 属性修饰符联合
// ══════════════════════════════════════════════════════════════

#[test]
fn 攻击力buff后伤害增加() {
    let dmg = calculate_damage_from_effect(15.0, 3.0, 3.0, 1.0, 0.0, 0);
    assert_eq!(dmg, 12);
}

#[test]
fn 防御力buff后伤害降低() {
    let dmg = calculate_damage_from_effect(10.0, 8.0, 3.0, 1.0, 0.0, 0);
    assert_eq!(dmg, 2);
}

#[test]
fn 减防debuff增加伤害() {
    let dmg = calculate_damage_from_effect(10.0, 0.0, 3.0, 1.0, 0.0, 0);
    assert_eq!(dmg, 10);
}

#[test]
fn 减攻debuff降低伤害() {
    let dmg = calculate_damage_from_effect(5.0, 3.0, 3.0, 1.0, 0.0, 0);
    assert_eq!(dmg, 2);
}

// ══════════════════════════════════════════════════════════════
// 场景六：多效果技能 generate 全流程
// ══════════════════════════════════════════════════════════════

#[test]
fn 多效果技能_伤害加buff() {
    let registry = EffectHandlerRegistry::default();

    let gen_ctx = GenerateContext {
        source_entity: bevy::prelude::Entity::from_bits(1),
        target_entity: bevy::prelude::Entity::from_bits(2),
        source_attrs: warrior_attrs(),
        target_attrs: goblin_attrs(),
        defense_bonus: 0,
        skill_id: "fire_strike".into(),
        source_tags: vec![],
        terrain: Terrain::Plain,
    };

    let effects = vec![
        EffectDef::Damage {
            multiplier: 1.2,
            ignore_def_percent: 0.0,
        },
        EffectDef::ApplyBuff {
            buff_id: "burn".into(),
            duration: 2,
        },
    ];

    let mut queue = EffectQueue::default();

    for effect_def in &effects {
        if let Some(handler) = registry.find(effect_def.type_name()) {
            if let Some(data) = handler.generate(effect_def, &gen_ctx) {
                queue.push(PendingEffect {
                    source: gen_ctx.source_entity,
                    target: gen_ctx.target_entity,
                    data,
                    source_tags: vec![],
                    terrain: Terrain::Plain,
                });
            }
        }
    }

    assert_eq!(queue.pending.len(), 2);
    assert!(matches!(
        queue.pending[0].data,
        PendingEffectData::Damage {
            amount: 8,
            is_skill: true
        }
    ));
    assert!(matches!(
        queue.pending[1].data,
        PendingEffectData::ApplyBuff { ref buff_id, duration: 2 } if buff_id == "burn"
    ));
}

// ══════════════════════════════════════════════════════════════
// 场景七：标签系统基础
// ══════════════════════════════════════════════════════════════

#[test]
fn 标签_add_has_remove_链路() {
    let mut tags = GameplayTags::default();
    assert!(!tags.has(GameplayTag::FIRE));

    tags.add(GameplayTag::FIRE);
    assert!(tags.has(GameplayTag::FIRE));
    assert!(!tags.has(GameplayTag::ICE));

    tags.remove(GameplayTag::FIRE);
    assert!(!tags.has(GameplayTag::FIRE));
}

#[test]
fn 标签_has_any_has_all() {
    let mut tags = GameplayTags::default();
    tags.add(GameplayTag::FIRE);
    tags.add(GameplayTag::BUFF);

    let check = GameplayTags::from_tags(&[GameplayTag::FIRE, GameplayTag::ICE]);
    assert!(tags.has_any(&check));
    assert!(!tags.has_all(&check));

    tags.add(GameplayTag::ICE);
    assert!(tags.has_all(&check));
}
