//! P1 集成测试：Buff 完整生命周期
//!
//! 跨 buff/apply + buff/instance + buff/resolve + core/attribute + core/tag
//! 测试 Buff 从施加 → tick → 过期的完整生命周期。

use tactical_rpg::buff::{
    ActiveBuffs, BuffData, BuffInstance, apply_buff, remove_all_debuffs, remove_buff,
};
use tactical_rpg::core::attribute::{
    AttributeKind, AttributeModifierDef, AttributeModifierInstance, Attributes, BuffInstanceId,
    ModifierOp, ModifierSource,
};
use tactical_rpg::core::tag::{GameplayTag, GameplayTags};

use crate::common::fixtures::*;

// ── 测试辅助 ──

fn make_buff_data(
    id: &str,
    is_buff: bool,
    modifiers: Vec<AttributeModifierDef>,
    tags: Vec<GameplayTag>,
    dot_damage: i32,
    hot_heal: i32,
) -> BuffData {
    BuffData {
        id: id.into(),
        name: id.into(),
        default_duration: 2,
        modifiers,
        tags,
        dot_damage,
        hot_heal,
        is_stun: false,
        is_cleanse: false,
        is_buff,
    }
}

fn make_stun_buff() -> BuffData {
    BuffData {
        id: "stun".into(),
        name: "晕眩".into(),
        default_duration: 1,
        modifiers: vec![],
        tags: vec![GameplayTag::DEBUFF, GameplayTag::STUN],
        dot_damage: 0,
        hot_heal: 0,
        is_stun: true,
        is_cleanse: false,
        is_buff: false,
    }
}

// ══════════════════════════════════════════════════════════════
// 场景一：攻击 Buff 完整生命周期
// ══════════════════════════════════════════════════════════════

#[test]
fn 攻击buff_施加_递减_过期_修饰符清理() {
    let buff_data = make_buff_data(
        "attack_up",
        true,
        vec![AttributeModifierDef {
            kind: AttributeKind::Attack,
            op: ModifierOp::Add,
            value: 5.0,
        }],
        vec![GameplayTag::BUFF],
        0,
        0,
    );

    let mut buffs = ActiveBuffs::default();
    let mut attrs = warrior_attrs();
    let mut tags = GameplayTags::default();

    // ── 施加 ──
    let instance_id = apply_buff(&mut buffs, &mut attrs, &mut tags, &buff_data, None, 3);
    assert_eq!(buffs.len(), 1);
    assert_eq!(attrs.get(AttributeKind::Attack), 15.0); // 10+5=15
    assert!(tags.has(GameplayTag::BUFF));

    // ── tick 第1轮：remaining 3→2 ──
    // 手动模拟 tick_buffs 逻辑
    let expired: Vec<_> = buffs
        .instances
        .iter()
        .filter(|i| i.remaining_turns <= 1)
        .map(|i| i.instance_id.to_modifier_source())
        .collect();
    buffs.tick();
    for id in &expired {
        attrs.remove_modifiers_from(*id);
    }
    assert_eq!(buffs.len(), 1);
    assert_eq!(buffs.instances[0].remaining_turns, 2);
    assert_eq!(attrs.get(AttributeKind::Attack), 15.0); // 仍在

    // ── tick 第2轮：remaining 2→1 ──
    let expired: Vec<_> = buffs
        .instances
        .iter()
        .filter(|i| i.remaining_turns <= 1)
        .map(|i| i.instance_id.to_modifier_source())
        .collect();
    buffs.tick();
    for id in &expired {
        attrs.remove_modifiers_from(*id);
    }
    assert_eq!(buffs.instances[0].remaining_turns, 1);
    assert_eq!(attrs.get(AttributeKind::Attack), 15.0);

    // ── tick 第3轮：remaining 1→0 → 过期 → 修饰符清理 ──
    let expired: Vec<_> = buffs
        .instances
        .iter()
        .filter(|i| i.remaining_turns <= 1)
        .map(|i| i.instance_id.to_modifier_source())
        .collect();
    buffs.tick();
    for id in &expired {
        attrs.remove_modifiers_from(*id);
    }
    assert!(attrs.modifiers.is_empty());
    assert_eq!(attrs.get(AttributeKind::Attack), 10.0); // 恢复

    // ── tick 第4轮：remaining=0 的被移除 ──
    buffs.tick();
    assert!(buffs.is_empty());
}

// ══════════════════════════════════════════════════════════════
// 场景二：晕眩 Buff 生命周期
// ══════════════════════════════════════════════════════════════

#[test]
fn 晕眩buff_施加后被消耗() {
    let stun = make_stun_buff();
    let mut buffs = ActiveBuffs::default();
    let mut attrs = warrior_attrs();
    let mut tags = GameplayTags::default();

    apply_buff(&mut buffs, &mut attrs, &mut tags, &stun, None, 1);
    assert!(buffs.is_stunned());

    let was_stunned = buffs.consume_stun();
    assert!(was_stunned);
    assert!(!buffs.is_stunned());
    assert!(tags.has(GameplayTag::DEBUFF)); // 标签由 rebuild 管理
}

// ══════════════════════════════════════════════════════════════
// 场景三：DoT Buff 生命周期
// ══════════════════════════════════════════════════════════════

#[test]
fn dot_buff_每轮扣血() {
    let poison = make_buff_data(
        "poison",
        false,
        vec![],
        vec![GameplayTag::DEBUFF, GameplayTag::POISON],
        3, // dot_damage=3
        0,
    );

    let mut buffs = ActiveBuffs::default();
    let mut attrs = warrior_attrs();
    let mut tags = GameplayTags::default();

    attrs.set_base(AttributeKind::Hp, 20.0);
    apply_buff(&mut buffs, &mut attrs, &mut tags, &poison, None, 3);

    // 模拟 resolve_status_effects 中的 DoT 结算
    let dot = buffs.dot_damage();
    assert_eq!(dot, 3);
    let hp = attrs.get(AttributeKind::Hp);
    let new_hp = (hp - dot as f32).max(0.0);
    attrs.set_base(AttributeKind::Hp, new_hp);
    assert_eq!(attrs.get(AttributeKind::Hp), 17.0);

    // tick
    let expired: Vec<_> = buffs
        .instances
        .iter()
        .filter(|i| i.remaining_turns <= 1)
        .map(|i| i.instance_id.to_modifier_source())
        .collect();
    buffs.tick();
    for id in &expired {
        attrs.remove_modifiers_from(*id);
    }

    // 第2轮 dot
    let dot = buffs.dot_damage();
    assert_eq!(dot, 3);
    let hp = attrs.get(AttributeKind::Hp);
    attrs.set_base(AttributeKind::Hp, (hp - dot as f32).max(0.0));
    assert_eq!(attrs.get(AttributeKind::Hp), 14.0);
}

// ══════════════════════════════════════════════════════════════
// 场景四：HoT Buff 生命周期
// ══════════════════════════════════════════════════════════════

#[test]
fn hot_buff_每轮回血_不超过最大hp() {
    let regen = make_buff_data(
        "regen",
        true,
        vec![],
        vec![GameplayTag::BUFF],
        0,
        4, // hot_heal=4
    );

    let mut buffs = ActiveBuffs::default();
    let mut attrs = warrior_attrs();
    let mut tags = GameplayTags::default();

    attrs.set_base(AttributeKind::Hp, 18.0); // MaxHp=30
    apply_buff(&mut buffs, &mut attrs, &mut tags, &regen, None, 3);

    // 第1轮 HoT
    let hot = buffs.hot_heal();
    assert_eq!(hot, 4);
    let hp = attrs.get(AttributeKind::Hp);
    let max_hp = attrs.get(AttributeKind::MaxHp);
    attrs.set_base(AttributeKind::Hp, (hp + hot as f32).min(max_hp));
    assert_eq!(attrs.get(AttributeKind::Hp), 22.0);

    // 第2轮 HoT
    let hot = buffs.hot_heal();
    let hp = attrs.get(AttributeKind::Hp);
    attrs.set_base(AttributeKind::Hp, (hp + hot as f32).min(max_hp));
    assert_eq!(attrs.get(AttributeKind::Hp), 26.0);

    // 第3轮 HoT
    let hot = buffs.hot_heal();
    let hp = attrs.get(AttributeKind::Hp);
    attrs.set_base(AttributeKind::Hp, (hp + hot as f32).min(max_hp));
    assert_eq!(attrs.get(AttributeKind::Hp), 30.0); // cap at MaxHp
}

// ══════════════════════════════════════════════════════════════
// 场景五：驱散 Cleanse
// ══════════════════════════════════════════════════════════════

#[test]
fn cleanse_移除所有debuff保留buff() {
    let atk_up = make_buff_data(
        "attack_up",
        true,
        vec![AttributeModifierDef {
            kind: AttributeKind::Attack,
            op: ModifierOp::Add,
            value: 5.0,
        }],
        vec![GameplayTag::BUFF],
        0,
        0,
    );
    let def_down = make_buff_data(
        "defense_down",
        false,
        vec![AttributeModifierDef {
            kind: AttributeKind::Defense,
            op: ModifierOp::Add,
            value: -3.0,
        }],
        vec![GameplayTag::DEBUFF],
        0,
        0,
    );
    let poison = make_buff_data(
        "poison",
        false,
        vec![],
        vec![GameplayTag::DEBUFF, GameplayTag::POISON],
        3,
        0,
    );

    let mut buffs = ActiveBuffs::default();
    let mut attrs = warrior_attrs();
    let mut tags = GameplayTags::default();

    apply_buff(&mut buffs, &mut attrs, &mut tags, &atk_up, None, 3);
    apply_buff(&mut buffs, &mut attrs, &mut tags, &def_down, None, 3);
    apply_buff(&mut buffs, &mut attrs, &mut tags, &poison, None, 3);

    assert_eq!(buffs.len(), 3);
    assert_eq!(attrs.get(AttributeKind::Attack), 15.0); // 10+5
    assert_eq!(attrs.get(AttributeKind::Defense), 2.0); // 5-3
    assert_eq!(buffs.dot_damage(), 3);

    // 驱散
    remove_all_debuffs(&mut buffs, &mut attrs, &mut tags);

    assert_eq!(buffs.len(), 1);
    assert_eq!(buffs.instances[0].buff_id, "attack_up");
    assert_eq!(attrs.get(AttributeKind::Attack), 15.0); // 保留
    assert_eq!(attrs.get(AttributeKind::Defense), 5.0); // 恢复
    assert_eq!(buffs.dot_damage(), 0);
}

// ══════════════════════════════════════════════════════════════
// 场景六：共享标签引用计数
// ══════════════════════════════════════════════════════════════

#[test]
fn 共享标签_两个buff共享FIRE_移除一个_标签保留() {
    let fire_a = make_buff_data(
        "fire_a",
        true,
        vec![AttributeModifierDef {
            kind: AttributeKind::Attack,
            op: ModifierOp::Add,
            value: 3.0,
        }],
        vec![GameplayTag::BUFF, GameplayTag::FIRE],
        0,
        0,
    );
    let fire_b = make_buff_data(
        "fire_b",
        true,
        vec![AttributeModifierDef {
            kind: AttributeKind::Defense,
            op: ModifierOp::Add,
            value: 2.0,
        }],
        vec![GameplayTag::BUFF, GameplayTag::FIRE],
        0,
        0,
    );

    let mut buffs = ActiveBuffs::default();
    let mut attrs = warrior_attrs();
    let mut tags = GameplayTags::default();

    let id_a = apply_buff(&mut buffs, &mut attrs, &mut tags, &fire_a, None, 3);
    let id_b = apply_buff(&mut buffs, &mut attrs, &mut tags, &fire_b, None, 3);

    assert!(tags.has(GameplayTag::FIRE));
    assert_eq!(buffs.len(), 2);

    // 移除 fire_a
    remove_buff(&mut buffs, &mut attrs, &mut tags, id_a);

    // FIRE 仍由 fire_b 提供
    assert!(tags.has(GameplayTag::FIRE));
    assert!(tags.has(GameplayTag::BUFF));
    assert_eq!(buffs.len(), 1);
    // Attack 恢复，Defense 保留
    assert_eq!(attrs.get(AttributeKind::Attack), 10.0);
    assert_eq!(attrs.get(AttributeKind::Defense), 7.0);
}

// ══════════════════════════════════════════════════════════════
// 场景七：同源 Buff 刷新
// ══════════════════════════════════════════════════════════════

#[test]
fn 同源buff_刷新持续时间_不重复添加修饰符() {
    let poison = make_buff_data("poison", false, vec![], vec![GameplayTag::DEBUFF], 3, 0);

    let mut buffs = ActiveBuffs::default();
    let mut attrs = warrior_attrs();
    let mut tags = GameplayTags::default();
    let source = bevy::prelude::Entity::from_bits(42);

    // 施加1回合
    apply_buff(&mut buffs, &mut attrs, &mut tags, &poison, Some(source), 1);
    assert_eq!(buffs.len(), 1);
    assert_eq!(buffs.instances[0].remaining_turns, 1);

    // 同源再次施加3回合 → 刷新为3
    apply_buff(&mut buffs, &mut attrs, &mut tags, &poison, Some(source), 3);
    assert_eq!(buffs.len(), 1); // 不重复
    assert_eq!(buffs.instances[0].remaining_turns, 3); // 刷新
}

// ══════════════════════════════════════════════════════════════
// 场景八：多 Buff 叠加
// ══════════════════════════════════════════════════════════════

#[test]
fn 多buff_属性修饰符正确叠加() {
    let atk_up = make_buff_data(
        "attack_up",
        true,
        vec![AttributeModifierDef {
            kind: AttributeKind::Attack,
            op: ModifierOp::Add,
            value: 5.0,
        }],
        vec![GameplayTag::BUFF],
        0,
        0,
    );
    let atk_up2 = make_buff_data(
        "attack_up2",
        true,
        vec![AttributeModifierDef {
            kind: AttributeKind::Attack,
            op: ModifierOp::Add,
            value: 3.0,
        }],
        vec![GameplayTag::BUFF],
        0,
        0,
    );

    let mut buffs = ActiveBuffs::default();
    let mut attrs = warrior_attrs();
    let mut tags = GameplayTags::default();

    apply_buff(&mut buffs, &mut attrs, &mut tags, &atk_up, None, 3);
    assert_eq!(attrs.get(AttributeKind::Attack), 15.0); // 10+5

    apply_buff(&mut buffs, &mut attrs, &mut tags, &atk_up2, None, 3);
    assert_eq!(attrs.get(AttributeKind::Attack), 18.0); // 10+5+3

    // 移除第一个
    remove_buff(&mut buffs, &mut attrs, &mut tags, BuffInstanceId(1));
    assert_eq!(attrs.get(AttributeKind::Attack), 13.0); // 10+3
}

// ══════════════════════════════════════════════════════════════
// 场景九：Buff + 标签联合
// ══════════════════════════════════════════════════════════════

#[test]
fn buff施加_标签同步更新() {
    let burn = make_buff_data(
        "burn",
        false,
        vec![AttributeModifierDef {
            kind: AttributeKind::Defense,
            op: ModifierOp::Add,
            value: -2.0,
        }],
        vec![GameplayTag::DEBUFF, GameplayTag::BURN, GameplayTag::FIRE],
        2,
        0,
    );

    let mut buffs = ActiveBuffs::default();
    let mut attrs = warrior_attrs();
    let mut tags = GameplayTags::default();

    apply_buff(&mut buffs, &mut attrs, &mut tags, &burn, None, 2);

    assert!(tags.has(GameplayTag::DEBUFF));
    assert!(tags.has(GameplayTag::BURN));
    assert!(tags.has(GameplayTag::FIRE));
    assert_eq!(attrs.get(AttributeKind::Defense), 3.0); // 5-2

    // 移除后标签清除
    remove_buff(&mut buffs, &mut attrs, &mut tags, BuffInstanceId(1));
    assert!(!tags.has(GameplayTag::BURN));
    assert!(!tags.has(GameplayTag::FIRE));
    assert_eq!(attrs.get(AttributeKind::Defense), 5.0);
}
