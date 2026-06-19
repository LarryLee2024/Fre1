//! Spell Domain — 施法流程集成测试
//!
//! 验证 SpellPlugin 注册后，SpellCastRequest → on_spell_cast_request → SpellCastResult 的完整链路。
//! 使用 Bevy App 构建器，加载 SpellPlugin 最小环境。
//!
//! 测试范围：
//! - 施法成功：法术位消耗、专注冲突自动解除
//! - 施法失败：法术未习得、法术未准备、法术位不足
//! - 专注计时：tick_concentration_duration 推进专注回合数

use bevy::prelude::*;

use crate::core::domains::spell::components::{
    Concentration, SpellDefId, SpellLevel, SpellSlotEntry, SpellSlotPool, Spellbook,
};
use crate::core::domains::spell::events::SpellCastRequest;
use crate::core::domains::spell::plugin::SpellPlugin;

// ─── 辅助函数 ──────────────────────────────────────────────────────

/// 创建一个标准施法者实体，包含法术书和法术位池。
fn spawn_caster(
    world: &mut World,
    known: &[&str],
    prepared: &[&str],
    slots: &[(u32, u32)],
) -> Entity {
    let known_ids = known
        .iter()
        .map(|s| SpellDefId::new(*s))
        .collect::<Vec<_>>();
    let prepared_ids = prepared
        .iter()
        .map(|s| SpellDefId::new(*s))
        .collect::<Vec<_>>();

    let mut slots_by_level = Vec::with_capacity(9);
    for i in 0..9 {
        let (total, used) = slots.get(i).copied().unwrap_or((0, 0));
        slots_by_level.push(SpellSlotEntry { total, used });
    }

    world
        .spawn((
            Spellbook {
                known_spells: known_ids,
                prepared_spells: prepared_ids,
                max_prepared: 10,
            },
            SpellSlotPool { slots_by_level },
        ))
        .id()
}

/// 创建一个专注中的施法者。
fn spawn_concentrating_caster(
    world: &mut World,
    known: &[&str],
    prepared: &[&str],
    concentrating_spell: &str,
    total_duration: u32,
) -> Entity {
    let entity = spawn_caster(world, known, prepared, &[(2, 0); 9]);
    world.entity_mut(entity).insert(Concentration::new(
        SpellDefId::new(concentrating_spell),
        total_duration,
        2,
    ));
    entity
}

/// 获取法术位的剩余值。
fn remaining_slots(world: &World, entity: Entity, level: SpellLevel) -> u32 {
    world
        .get::<SpellSlotPool>(entity)
        .map(|pool| pool.remaining(level))
        .unwrap_or(0)
}

/// 检查实体是否含有 Concentration 组件。
fn has_concentration(world: &World, entity: Entity) -> bool {
    world.get::<Concentration>(entity).is_some()
}

// ─── 施法成功：法术位消耗 ────────────────────────────────────────

#[test]
fn cast_known_and_prepared_spell_consumes_slot() {
    let mut app = App::new();
    app.add_plugins(SpellPlugin);

    let caster = spawn_caster(
        app.world_mut(),
        &["spl_fireball", "spl_heal"],
        &["spl_fireball"],
        &[
            (2, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
        ],
    );
    app.world_mut().flush();

    // 执行：施放一个已知且已准备的 1 环法术
    app.world_mut().trigger(SpellCastRequest {
        caster,
        spell_id: SpellDefId::new("spl_fireball"),
        targets: vec![],
        target_position: None,
        upcast_level: Some(SpellLevel::L1),
    });
    app.world_mut().flush();

    // 验证：法术位从 2 变为 1
    assert_eq!(
        remaining_slots(app.world_mut(), caster, SpellLevel::L1),
        1,
        "消耗一个 1 环法术位后应剩余 1"
    );
}

#[test]
fn cast_cantrip_does_not_consume_slot() {
    let mut app = App::new();
    app.add_plugins(SpellPlugin);

    let caster = spawn_caster(app.world_mut(), &["spl_fire_bolt"], &[], &[(0, 0); 9]);
    app.world_mut().flush();

    // 戏法不消耗法术位（默认 upcast_level = None 时使用 Cantrip）
    app.world_mut().trigger(SpellCastRequest {
        caster,
        spell_id: SpellDefId::new("spl_fire_bolt"),
        targets: vec![],
        target_position: None,
        upcast_level: None,
    });
    app.world_mut().flush();

    // 验证：所有环级法术位均为 0（无消耗）
    for level in &[
        SpellLevel::Cantrip,
        SpellLevel::L1,
        SpellLevel::L2,
        SpellLevel::L3,
    ] {
        assert_eq!(
            remaining_slots(app.world_mut(), caster, *level),
            0,
            "戏法不应消耗任何法术位"
        );
    }
}

#[test]
fn cast_prepares_spell_at_correct_level() {
    let mut app = App::new();
    app.add_plugins(SpellPlugin);

    let caster = spawn_caster(
        app.world_mut(),
        &["spl_lightning"],
        &["spl_lightning"],
        &[
            (1, 0),
            (1, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
        ],
    );
    app.world_mut().flush();

    // 以 2 环施放
    app.world_mut().trigger(SpellCastRequest {
        caster,
        spell_id: SpellDefId::new("spl_lightning"),
        targets: vec![],
        target_position: None,
        upcast_level: Some(SpellLevel::L2),
    });
    app.world_mut().flush();

    // 验证：消耗的是 2 环法术位（1 环不变）
    assert_eq!(
        remaining_slots(app.world_mut(), caster, SpellLevel::L1),
        1,
        "1 环法术位应不变"
    );
    assert_eq!(
        remaining_slots(app.world_mut(), caster, SpellLevel::L2),
        0,
        "2 环法术位应消耗"
    );
}

// ─── 施法失败 ──────────────────────────────────────────────────────

#[test]
fn unknown_spell_rejected() {
    let mut app = App::new();
    app.add_plugins(SpellPlugin);

    let caster = spawn_caster(app.world_mut(), &["spl_heal"], &[], &[(2, 0); 9]);
    app.world_mut().flush();

    let slots_before = remaining_slots(app.world_mut(), caster, SpellLevel::L1);

    // 施放一个未习得的法术
    app.world_mut().trigger(SpellCastRequest {
        caster,
        spell_id: SpellDefId::new("spl_unknown"),
        targets: vec![],
        target_position: None,
        upcast_level: Some(SpellLevel::L1),
    });
    app.world_mut().flush();

    // 验证：法术位无变化
    assert_eq!(
        remaining_slots(app.world_mut(), caster, SpellLevel::L1),
        slots_before,
        "施法失败不应消耗法术位"
    );
}

#[test]
fn unprepared_spell_rejected() {
    let mut app = App::new();
    app.add_plugins(SpellPlugin);

    let caster = spawn_caster(
        app.world_mut(),
        &["spl_fireball"],
        &[], // 已知但未准备
        &[(2, 0); 9],
    );
    app.world_mut().flush();

    let slots_before = remaining_slots(app.world_mut(), caster, SpellLevel::L1);

    // 施放一个已知但未准备的法术
    app.world_mut().trigger(SpellCastRequest {
        caster,
        spell_id: SpellDefId::new("spl_fireball"),
        targets: vec![],
        target_position: None,
        upcast_level: Some(SpellLevel::L1),
    });
    app.world_mut().flush();

    assert_eq!(
        remaining_slots(app.world_mut(), caster, SpellLevel::L1),
        slots_before,
        "未准备的法术不应消耗法术位"
    );
}

#[test]
fn insufficient_slots_rejected() {
    let mut app = App::new();
    app.add_plugins(SpellPlugin);

    let caster = spawn_caster(
        app.world_mut(),
        &["spl_fireball"],
        &["spl_fireball"],
        &[
            (0, 0),
            (1, 1),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
        ], // 1 环已用满
    );
    app.world_mut().flush();

    let slots_before = remaining_slots(app.world_mut(), caster, SpellLevel::L1);

    app.world_mut().trigger(SpellCastRequest {
        caster,
        spell_id: SpellDefId::new("spl_fireball"),
        targets: vec![],
        target_position: None,
        upcast_level: Some(SpellLevel::L1),
    });
    app.world_mut().flush();

    assert_eq!(
        remaining_slots(app.world_mut(), caster, SpellLevel::L1),
        slots_before,
        "法术位不足时不应消耗"
    );
}

#[test]
fn caster_without_spellbook_rejected() {
    let mut app = App::new();
    app.add_plugins(SpellPlugin);

    // 一个没有 Spellbook 组件的实体
    let caster = app.world_mut().spawn_empty().id();
    app.world_mut().flush();

    // 不应 panic，应被优雅拒绝
    app.world_mut().trigger(SpellCastRequest {
        caster,
        spell_id: SpellDefId::new("spl_fireball"),
        targets: vec![],
        target_position: None,
        upcast_level: Some(SpellLevel::L1),
    });
    app.world_mut().flush();

    // 验证：不 panic 即通过
}

// ─── 专注冲突 ──────────────────────────────────────────────────────

#[test]
fn new_concentration_spell_breaks_old_concentration() {
    let mut app = App::new();
    app.add_plugins(SpellPlugin);

    // 创建已在专注中的施法者（专注 spl_old_conc）
    let caster = spawn_concentrating_caster(
        app.world_mut(),
        &["spl_old_conc", "spl_new_conc"],
        &["spl_new_conc"],
        "spl_old_conc",
        10,
    );
    app.world_mut().flush();

    assert!(
        has_concentration(app.world_mut(), caster),
        "初始应有专注状态"
    );

    // 施放另一个专注法术
    app.world_mut().trigger(SpellCastRequest {
        caster,
        spell_id: SpellDefId::new("spl_new_conc"),
        targets: vec![],
        target_position: None,
        upcast_level: Some(SpellLevel::L1),
    });
    app.world_mut().flush();

    // 验证：旧专注被移除
    assert!(
        !has_concentration(app.world_mut(), caster),
        "新专注法术会移除旧专注（当前简化实现中 Concentration 被整体移除）"
    );
}

// ─── 专注计时 ──────────────────────────────────────────────────────

#[test]
fn concentration_ticks_each_turn() {
    let mut app = App::new();
    app.add_plugins(SpellPlugin);

    let caster = spawn_concentrating_caster(
        app.world_mut(),
        &[],
        &[],
        "spl_conc",
        5, // 持续 5 回合
    );
    app.world_mut().flush();

    // 执行 1 次 Update → tick 一次
    app.update();
    app.world_mut().flush();

    let conc = app.world_mut().get::<Concentration>(caster).unwrap();
    assert_eq!(conc.elapsed_rounds, 1, "一次 Update 后应推进 1 回合");

    // 再执行 3 次 Update
    for _ in 0..3 {
        app.update();
    }
    app.world_mut().flush();

    let conc = app.world_mut().get::<Concentration>(caster).unwrap();
    assert_eq!(conc.elapsed_rounds, 4, "总计 4 次 Update 后应推进 4 回合");
}

#[test]
fn concentration_expires_when_duration_reached() {
    let mut app = App::new();
    app.add_plugins(SpellPlugin);

    let caster = spawn_concentrating_caster(
        app.world_mut(),
        &[],
        &[],
        "spl_short",
        2, // 持续 2 回合
    );
    app.world_mut().flush();

    // 第 1 回合
    app.update();
    app.world_mut().flush();
    assert!(
        has_concentration(app.world_mut(), caster),
        "第 1 回合仍应专注"
    );

    // 第 2 回合（到期）
    app.update();
    app.world_mut().flush();

    // 第 3 回合（expired_rounds >= total_duration → 移除）
    app.update();
    app.world_mut().flush();

    assert!(
        !has_concentration(app.world_mut(), caster),
        "专注时长耗尽后应被移除"
    );
}
