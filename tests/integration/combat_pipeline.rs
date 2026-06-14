//! P0 集成测试：战斗效果管道
//!
//! 测试 calculate_damage_from_effect 在各种属性组合下的行为，
//! 跨 attribute + effect/types 模块验证完整的伤害计算链路。

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

use tactical_rpg::core::attribute::{AttributeKind, Attributes};
use tactical_rpg::core::effect::{
    EffectDef, EffectHandlerRegistry, EffectPreview, EffectQueue, GenerateContext, PendingEffect,
    PendingEffectData, PreviewContext, calculate_damage_from_effect,
};
use tactical_rpg::core::tag::{GameplayTag, GameplayTags};

use crate::common::fixtures::warrior_attrs;

// ── 测试辅助 ──

/// 哥布林模板：Might=4, Vitality=3 → Attack=8, Defense=3, MaxHp=20
/// 注意：与 crate::common::fixtures::goblin_attrs() 属性值不同，保留本地版本
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

/// LCP-001: 战士攻击哥布林基础伤害
///
/// Given: ATK=10, DEF=3
/// When: calculate_damage_from_effect(10, 3, 3, 1.0, 0.0, 0)
/// Then: 伤害=7
#[test]
fn 战士攻击哥布林_基础伤害() {
    // ATK=10, DEF=3, 无地形加成 → 10-3=7
    let dmg = calculate_damage_from_effect(10.0, 3.0, 3.0, 1.0, 0.0, 0);
    assert_eq!(dmg, 7);
}

/// LCP-002: 森林地形防御加成
///
/// Given: ATK=10, DEF=3, terrain_bonus=2
/// When: calculate_damage_from_effect
/// Then: 伤害=5
#[test]
fn 森林地形_防御加成() {
    // ATK=10, DEF=3, terrain_bonus=2 → 10-3-2=5
    let dmg = calculate_damage_from_effect(10.0, 3.0, 3.0, 1.0, 0.0, 2);
    assert_eq!(dmg, 5);
}

/// LCP-003: 山地地形无防御加成
///
/// Given: ATK=10, DEF=3, terrain_bonus=0
/// When: calculate_damage_from_effect
/// Then: 伤害=7
#[test]
fn 山地地形_无防御加成() {
    let dmg = calculate_damage_from_effect(10.0, 3.0, 3.0, 1.0, 0.0, 0);
    assert_eq!(dmg, 7);
}

/// LCP-004: 伤害下限为 1
///
/// Given: ATK=5, DEF=10（攻击低于防御）
/// When: calculate_damage_from_effect
/// Then: 伤害=1（下限）
#[test]
fn 伤害下限为1() {
    let dmg = calculate_damage_from_effect(5.0, 10.0, 10.0, 1.0, 0.0, 0);
    assert_eq!(dmg, 1);
}

/// LCP-005: 技能倍率 1.5 倍
///
/// Given: ATK=10, DEF=3, multiplier=1.5
/// When: calculate_damage_from_effect
/// Then: (10-3)*1.5=10.5→10
#[test]
fn 技能倍率_1_5倍() {
    // (10-3)*1.5=10.5 → 10
    let dmg = calculate_damage_from_effect(10.0, 3.0, 3.0, 1.5, 0.0, 0);
    assert_eq!(dmg, 10);
}

/// LCP-006: 技能倍率 3 倍
///
/// Given: ATK=10, DEF=3, multiplier=3.0
/// When: calculate_damage_from_effect
/// Then: (10-3)*3=21
#[test]
fn 技能倍率_3倍() {
    let dmg = calculate_damage_from_effect(10.0, 3.0, 3.0, 3.0, 0.0, 0);
    assert_eq!(dmg, 21);
}

/// LCP-007: 无视防御 50%
///
/// Given: ATK=10, DEF=10, multiplier=1.3, ignore_def=50%
/// When: calculate_damage_from_effect
/// Then: final_def=10*0.5=5, (10-5)*1.3=6.5→6
#[test]
fn 无视防御_50百分比() {
    // final_def=10-5=5, (10-5)*1.3=6.5→6
    let dmg = calculate_damage_from_effect(10.0, 10.0, 10.0, 1.3, 50.0, 0);
    assert_eq!(dmg, 6);
}

/// LCP-008: 无视防御 100%
///
/// Given: ATK=10, DEF=10, ignore_def=100%
/// When: calculate_damage_from_effect
/// Then: final_def=0, 伤害=10
#[test]
fn 无视防御_100百分比() {
    let dmg = calculate_damage_from_effect(10.0, 10.0, 10.0, 1.0, 100.0, 0);
    assert_eq!(dmg, 10);
}

/// LCP-009: 地形加成与无视防御叠加
///
/// Given: ATK=10, DEF=10, ignore_def=50%, terrain_bonus=2
/// When: calculate_damage_from_effect
/// Then: final_def=10*0.5=5, 5-2=3
#[test]
fn 地形加成与无视防御叠加() {
    // final_def=10-5=5, base=10-5=5, 5-2=3
    let dmg = calculate_damage_from_effect(10.0, 10.0, 10.0, 1.0, 50.0, 2);
    assert_eq!(dmg, 3);
}

// ══════════════════════════════════════════════════════════════
// 场景二：EffectHandlerRegistry → generate → PendingEffect
// ══════════════════════════════════════════════════════════════

/// LCP-010: 伤害处理器 generate 基础攻击
///
/// Given: DamageHandler, 战士 ATK=10, 哥布林 DEF=3
/// When: generate(Damage { multiplier=1.0 })
/// Then: PendingEffectData::Damage { amount=7, is_skill=false }
#[test]
fn 伤害处理器_generate_基础攻击() {
    let registry = EffectHandlerRegistry::default();
    let handler = registry.find("Damage").unwrap();

    let mut target = goblin_attrs();
    target.set_vital(AttributeKind::Hp, 15.0);

    let ctx = GenerateContext {
        source_entity: bevy::prelude::Entity::from_bits(1),
        target_entity: bevy::prelude::Entity::from_bits(2),
        source_attrs: warrior_attrs(),
        target_attrs: target,
        defense_bonus: 0,
        skill_id: "basic_attack".into(),
        source_tags: vec![],
        terrain_id: "plain".to_string(),
    };

    let def = EffectDef::Damage {
        multiplier: 1.0,
        ignore_def_percent: 0.0,
    };
    let result = handler.generate(&def, &ctx).unwrap();
    match result {
        PendingEffectData::Damage {
            amount, is_skill, ..
        } => {
            assert_eq!(amount, 7);
            assert!(!is_skill);
        }
        _ => panic!("应该是 Damage"),
    }
}

/// LCP-011: 伤害处理器 generate 技能攻击
///
/// Given: DamageHandler, 战士 ATK=10, 哥布林 DEF=3, skill_id="power_strike"
/// When: generate(Damage { multiplier=1.5 })
/// Then: PendingEffectData::Damage { amount=10, is_skill=true }
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
        terrain_id: "plain".to_string(),
    };

    let def = EffectDef::Damage {
        multiplier: 1.5,
        ignore_def_percent: 0.0,
    };
    let result = handler.generate(&def, &ctx).unwrap();
    if let PendingEffectData::Damage {
        amount, is_skill, ..
    } = result
    {
        assert_eq!(amount, 10); // (10-3)*1.5=10.5→10
        assert!(is_skill);
    } else {
        panic!("应该是 Damage");
    }
}

/// LCP-012: 治疗处理器 generate
///
/// Given: HealHandler, 目标 HP=15/30
/// When: generate(Heal { amount=8 })
/// Then: PendingEffectData::Heal { amount=8 }
#[test]
fn 治疗处理器_generate() {
    let registry = EffectHandlerRegistry::default();
    let handler = registry.find("Heal").unwrap();

    let mut target = warrior_attrs();
    target.set_vital(AttributeKind::Hp, 15.0);

    let ctx = GenerateContext {
        source_entity: bevy::prelude::Entity::from_bits(1),
        target_entity: bevy::prelude::Entity::from_bits(2),
        source_attrs: warrior_attrs(),
        target_attrs: target,
        defense_bonus: 0,
        skill_id: "heal".into(),
        source_tags: vec![],
        terrain_id: "plain".to_string(),
    };

    let def = EffectDef::Heal { amount: 8 };
    let result = handler.generate(&def, &ctx).unwrap();
    if let PendingEffectData::Heal { amount, .. } = result {
        assert_eq!(amount, 8);
    } else {
        panic!("应该是 Heal");
    }
}

/// LCP-013: Buff 处理器 generate
///
/// Given: ApplyBuffHandler
/// When: generate(ApplyBuff { buff_id="burn", duration=2 })
/// Then: PendingEffectData::ApplyBuff { buff_id="burn", duration=2 }
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
        terrain_id: "plain".to_string(),
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

/// LCP-014: 净化处理器 generate
///
/// Given: CleanseHandler
/// When: generate(Cleanse)
/// Then: PendingEffectData::Cleanse
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
        terrain_id: "plain".to_string(),
    };

    let def = EffectDef::Cleanse;
    let result = handler.generate(&def, &ctx).unwrap();
    assert!(matches!(result, PendingEffectData::Cleanse));
}

// ══════════════════════════════════════════════════════════════
// 场景三：预览 → 执行一致性
// ══════════════════════════════════════════════════════════════

/// LCP-015: 伤害预览与 generate 一致
///
/// Given: DamageHandler, 战士 vs 哥布林
/// When: generate 和 preview 各调用一次
/// Then: 两者返回的伤害量一致
#[test]
fn 伤害预览与generate一致() {
    let registry = EffectHandlerRegistry::default();
    let damage_handler = registry.find("Damage").unwrap();

    let mut target = goblin_attrs();
    target.set_vital(AttributeKind::Hp, 15.0);

    let gen_ctx = GenerateContext {
        source_entity: bevy::prelude::Entity::from_bits(1),
        target_entity: bevy::prelude::Entity::from_bits(2),
        source_attrs: warrior_attrs(),
        target_attrs: target.clone(),
        defense_bonus: 0,
        skill_id: "basic_attack".into(),
        source_tags: vec![],
        terrain_id: "plain".to_string(),
    };

    let preview_ctx = PreviewContext {
        source_attrs: warrior_attrs(),
        target_attrs: target,
        terrain_defense_bonus: 0,
        buff_registry: tactical_rpg::core::buff::BuffRegistry::default(),
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

/// LCP-016: 治疗预览不超过最大 HP
///
/// Given: 目标 HP=28/30
/// When: preview(Heal { amount=8 })
/// Then: 预览治疗量=min(8, 30-28)=2
#[test]
fn 治疗预览不超过最大hp() {
    let registry = EffectHandlerRegistry::default();
    let handler = registry.find("Heal").unwrap();

    let mut target = warrior_attrs();
    target.set_vital(AttributeKind::Hp, 28.0); // MaxHp=30

    let ctx = PreviewContext {
        source_attrs: warrior_attrs(),
        target_attrs: target,
        terrain_defense_bonus: 0,
        buff_registry: tactical_rpg::core::buff::BuffRegistry::default(),
    };

    let def = EffectDef::Heal { amount: 8 };
    let preview = handler.preview(&def, &ctx).unwrap();
    if let EffectPreview::Heal { amount } = preview {
        assert_eq!(amount, 2); // min(8, 30-28)=2
    } else {
        panic!("应该是 Heal 预览");
    }
}

/// LCP-017: 伤害预览致死标记
///
/// Given: 攻击者 Might=25(ATK=50)，目标 HP=5
/// When: preview(Damage)
/// Then: lethal=true
#[test]
fn 伤害预览致死标记() {
    let registry = EffectHandlerRegistry::default();
    let handler = registry.find("Damage").unwrap();

    let mut source = warrior_attrs();
    source.set_base(AttributeKind::Might, 25.0); // Attack=50
    let mut target = goblin_attrs();
    target.set_vital(AttributeKind::Hp, 5.0);

    let ctx = PreviewContext {
        source_attrs: source,
        target_attrs: target,
        terrain_defense_bonus: 0,
        buff_registry: tactical_rpg::core::buff::BuffRegistry::default(),
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

/// LCP-018: EffectQueue push then drain
///
/// Given: 空 EffectQueue
/// When: push(Damage) + push(Heal) → drain
/// Then: 取出 2 个效果，队列变空
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
            base_amount: None,
            modifiers: Vec::new(),
        },
        source_tags: vec![],
        terrain_id: "plain".to_string(),
    });

    queue.push(PendingEffect {
        source: bevy::prelude::Entity::from_bits(3),
        target: bevy::prelude::Entity::from_bits(4),
        data: PendingEffectData::Heal {
            amount: 5,
            base_amount: None,
            modifiers: Vec::new(),
        },
        source_tags: vec![],
        terrain_id: "plain".to_string(),
    });

    assert_eq!(queue.pending.len(), 2);

    let effects: Vec<_> = queue.pending.drain(..).collect();
    assert!(matches!(
        effects[0].data,
        PendingEffectData::Damage { amount: 7, .. }
    ));
    assert!(matches!(
        effects[1].data,
        PendingEffectData::Heal { amount: 5, .. }
    ));
    assert!(queue.is_empty());
}

// ══════════════════════════════════════════════════════════════
// 场景五：伤害计算 × 属性修饰符联合
// ══════════════════════════════════════════════════════════════

/// LCP-019: 攻击力 buff 后伤害增加
///
/// Given: ATK=15（buff 后），DEF=3
/// When: calculate_damage_from_effect
/// Then: 伤害=12
#[test]
fn 攻击力buff后伤害增加() {
    let dmg = calculate_damage_from_effect(15.0, 3.0, 3.0, 1.0, 0.0, 0);
    assert_eq!(dmg, 12);
}

/// LCP-020: 防御力 buff 后伤害降低
///
/// Given: ATK=10, DEF=8（buff 后）
/// When: calculate_damage_from_effect
/// Then: 伤害=2
#[test]
fn 防御力buff后伤害降低() {
    let dmg = calculate_damage_from_effect(10.0, 8.0, 3.0, 1.0, 0.0, 0);
    assert_eq!(dmg, 2);
}

/// LCP-021: 减防 debuff 增加伤害
///
/// Given: ATK=10, DEF=0（debuff 后）
/// When: calculate_damage_from_effect
/// Then: 伤害=10
#[test]
fn 减防debuff增加伤害() {
    let dmg = calculate_damage_from_effect(10.0, 0.0, 3.0, 1.0, 0.0, 0);
    assert_eq!(dmg, 10);
}

/// LCP-022: 减攻 debuff 降低伤害
///
/// Given: ATK=5（debuff 后）, DEF=3
/// When: calculate_damage_from_effect
/// Then: 伤害=2
#[test]
fn 减攻debuff降低伤害() {
    let dmg = calculate_damage_from_effect(5.0, 3.0, 3.0, 1.0, 0.0, 0);
    assert_eq!(dmg, 2);
}

// ══════════════════════════════════════════════════════════════
// 场景六：多效果技能 generate 全流程
// ══════════════════════════════════════════════════════════════

/// LCP-023: 多效果技能 — 伤害 + buff
///
/// Given: fire_strike 含 Damage(1.2x) + ApplyBuff(burn, 2)
/// When: 遍历 effects generate → queue
/// Then: queue 有 2 个效果，Damage(amount=8, is_skill=true) + ApplyBuff(burn, 2)
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
        terrain_id: "plain".to_string(),
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
                    terrain_id: "plain".to_string(),
                });
            }
        }
    }

    assert_eq!(queue.pending.len(), 2);
    assert!(matches!(
        queue.pending[0].data,
        PendingEffectData::Damage {
            amount: 8,
            is_skill: true,
            ..
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

/// LCP-024: 标签 add/has/remove 链路
///
/// Given: 空 GameplayTags
/// When: add(FIRE) → has(FIRE) → remove(FIRE) → has(FIRE)
/// Then: true → false
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

/// LCP-025: 标签 has_any / has_all
///
/// Given: tags 含 FIRE + BUFF
/// When: has_any(FIRE|ICE) / has_all(FIRE|ICE) → add(ICE) → has_all(FIRE|ICE)
/// Then: has_any=true, has_all=false → has_all=true
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
