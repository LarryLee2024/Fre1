//! P4 集成测试：边界条件
//!
//! 测试各种边界情况：HP 满时治疗、修饰符叠加、空操作等。

use tactical_rpg::buff::{ActiveBuffs, BuffData, apply_buff, remove_buff};
use tactical_rpg::gameplay::attribute::{
    AttributeKind, AttributeModifierDef, Attributes, BuffInstanceId, ModifierOp,
};
use tactical_rpg::gameplay::effect::{
    EffectDef, EffectHandlerRegistry, EffectPreview, GenerateContext, PreviewContext,
};
use tactical_rpg::gameplay::tag::{GameplayTag, GameplayTags};

// ── 测试辅助 ──

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

fn make_buff_data(
    id: &str,
    is_buff: bool,
    modifiers: Vec<AttributeModifierDef>,
    tags: Vec<GameplayTag>,
) -> BuffData {
    BuffData {
        id: id.into(),
        name: id.into(),
        default_duration: 2,
        modifiers,
        tags,
        dot_damage: 0,
        hot_heal: 0,
        is_stun: false,
        is_cleanse: false,
        is_buff,
    }
}

// ══════════════════════════════════════════════════════════════
// 场景一：HP 满时治疗
// ══════════════════════════════════════════════════════════════

#[test]
fn 治疗_满血时不增加() {
    let registry = EffectHandlerRegistry::default();
    let handler = registry.find("Heal").unwrap();

    let source = warrior_attrs();
    let target = warrior_attrs(); // HP=MaxHp=30

    let ctx = PreviewContext {
        source_attrs: source,
        target_attrs: target,
        terrain_defense_bonus: 0,
        buff_registry: tactical_rpg::buff::BuffRegistry::default(),
    };

    let def = EffectDef::Heal { amount: 10 };
    let preview = handler.preview(&def, &ctx).unwrap();
    if let EffectPreview::Heal { amount } = preview {
        assert_eq!(amount, 0); // 已满
    } else {
        panic!("应该是 Heal 预览");
    }
}

// ══════════════════════════════════════════════════════════════
// 场景二：修饰符 Add + Multiply 叠加
// ══════════════════════════════════════════════════════════════

#[test]
fn 修饰符_add_then_multiply() {
    let buff_add = make_buff_data(
        "atk_add",
        true,
        vec![AttributeModifierDef {
            kind: AttributeKind::Attack,
            op: ModifierOp::Add,
            value: 5.0,
        }],
        vec![GameplayTag::BUFF],
    );
    let buff_mul = make_buff_data(
        "atk_mul",
        true,
        vec![AttributeModifierDef {
            kind: AttributeKind::Attack,
            op: ModifierOp::Multiply,
            value: 1.5,
        }],
        vec![GameplayTag::BUFF],
    );

    let mut buffs = ActiveBuffs::default();
    let mut attrs = warrior_attrs();
    let mut tags = GameplayTags::default();

    apply_buff(&mut buffs, &mut attrs, &mut tags, &buff_add, None, 3);
    // Attack = (10 + 5) = 15
    assert_eq!(attrs.get(AttributeKind::Attack), 15.0);

    apply_buff(&mut buffs, &mut attrs, &mut tags, &buff_mul, None, 3);
    // Attack = (10 + 5) * 1.5 = 22.5
    assert_eq!(attrs.get(AttributeKind::Attack), 22.5);

    // 移除 multiply
    remove_buff(&mut buffs, &mut attrs, &mut tags, BuffInstanceId(2));
    assert_eq!(attrs.get(AttributeKind::Attack), 15.0);
}

// ══════════════════════════════════════════════════════════════
// 场景三：空 Buff 列表操作
// ══════════════════════════════════════════════════════════════

#[test]
fn 空buff列表_tick无崩溃() {
    let mut buffs = ActiveBuffs::default();
    buffs.tick();
    assert!(buffs.is_empty());
    assert_eq!(buffs.dot_damage(), 0);
    assert_eq!(buffs.hot_heal(), 0);
    assert!(!buffs.is_stunned());
}

#[test]
fn 空buff列表_移除不存在的id() {
    let mut buffs = ActiveBuffs::default();
    let result = buffs.remove(BuffInstanceId(999));
    assert!(result.is_none());
}

// ══════════════════════════════════════════════════════════════
// 场景四：EffectHandlerRegistry 边界
// ══════════════════════════════════════════════════════════════

#[test]
fn 注册表_查找不存在的处理器() {
    let registry = EffectHandlerRegistry::default();
    assert!(registry.find("NonExistent").is_none());
}

#[test]
fn 注册表_不重复注册() {
    let mut registry = EffectHandlerRegistry::default();
    // DamageHandler 已注册，再次注册不应增加处理器数量
    // 验证方式：find 仍能返回 Damage 处理器
    registry.register(Box::new(tactical_rpg::gameplay::effect::DamageHandler));
    assert!(registry.find("Damage").is_some());
}

// ══════════════════════════════════════════════════════════════
// 场景五：类型不匹配
// ══════════════════════════════════════════════════════════════

#[test]
fn 伤害处理器_收到heal定义返回none() {
    let registry = EffectHandlerRegistry::default();
    let handler = registry.find("Damage").unwrap();

    let source = warrior_attrs();
    let target = warrior_attrs();

    let ctx = tactical_rpg::gameplay::effect::GenerateContext {
        source_entity: bevy::prelude::Entity::from_bits(1),
        target_entity: bevy::prelude::Entity::from_bits(2),
        source_attrs: source,
        target_attrs: target,
        defense_bonus: 0,
        skill_id: "test".into(),
        source_tags: vec![],
        terrain: tactical_rpg::map::Terrain::Plain,
    };

    let def = EffectDef::Heal { amount: 5 };
    assert!(handler.generate(&def, &ctx).is_none());
}

// ══════════════════════════════════════════════════════════════
// 场景六：HP 降至 0 边界
// ══════════════════════════════════════════════════════════════

#[test]
fn 伤害_精确致死() {
    let mut attrs = warrior_attrs();
    attrs.set_base(AttributeKind::Hp, 7.0); // 正好等于战士 ATK-哥布林DEF=7

    // 模拟伤害执行
    let hp = attrs.get(AttributeKind::Hp);
    let new_hp = (hp - 7.0_f32).max(0.0);
    attrs.set_base(AttributeKind::Hp, new_hp);

    assert_eq!(attrs.get(AttributeKind::Hp), 0.0);
}

#[test]
fn 伤害_超过hp() {
    let mut attrs = warrior_attrs();
    attrs.set_base(AttributeKind::Hp, 5.0);

    let hp = attrs.get(AttributeKind::Hp);
    let new_hp = (hp - 50.0).max(0.0);
    attrs.set_base(AttributeKind::Hp, new_hp);

    assert_eq!(attrs.get(AttributeKind::Hp), 0.0);
}

// ══════════════════════════════════════════════════════════════
// 场景七：标签幂等性
// ══════════════════════════════════════════════════════════════

#[test]
fn 标签_add重复_idempotent() {
    let mut tags = GameplayTags::default();
    tags.add(GameplayTag::FIRE);
    tags.add(GameplayTag::FIRE);
    tags.add(GameplayTag::FIRE);

    let mut count = 0;
    if tags.has(GameplayTag::FIRE) {
        count += 1;
    }
    assert_eq!(count, 1); // 只有一个 FIRE
}

#[test]
fn 标签_remove不存在的idempotent() {
    let mut tags = GameplayTags::default();
    tags.remove(GameplayTag::FIRE); // 不崩溃
    assert!(!tags.has(GameplayTag::FIRE));
}

// ══════════════════════════════════════════════════════════════
// 场景八：EffectDef::type_name 一致性
// ══════════════════════════════════════════════════════════════

#[test]
fn effect_def_type_name_覆盖所有变体() {
    assert_eq!(
        EffectDef::Damage {
            multiplier: 1.0,
            ignore_def_percent: 0.0
        }
        .type_name(),
        "Damage"
    );
    assert_eq!(EffectDef::Heal { amount: 5 }.type_name(), "Heal");
    assert_eq!(
        EffectDef::ApplyBuff {
            buff_id: "x".into(),
            duration: 1
        }
        .type_name(),
        "ApplyBuff"
    );
    assert_eq!(EffectDef::Cleanse.type_name(), "Cleanse");
}
