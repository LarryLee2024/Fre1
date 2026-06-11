//! Golden Battle Test（战斗快照测试）
//!
//! 通过 insta 快照对比验证 BattleRecord 的完整战斗流程记录，
//! 确保战斗事件链路在重构后不会静默变化。

use bevy::prelude::*;
use tactical_rpg::battle::{BattleRecord, execute_effects};
use tactical_rpg::buff::BuffRegistry;
use tactical_rpg::character::{GridPosition, UnitName};
use tactical_rpg::core::effect::{EffectQueue, PendingEffect, PendingEffectData};
use tactical_rpg::core::registry_loader::RegistryLoader;
use tactical_rpg::map::TerrainRegistry;
use tactical_rpg::skill::SkillSlots;

use crate::common::fixtures::UnitBuilder;

// ── 测试辅助 ──

fn test_buff_registry() -> BuffRegistry {
    let mut reg = BuffRegistry::default();
    reg.register_defaults();
    reg
}

fn test_terrain_registry() -> TerrainRegistry {
    let mut reg = TerrainRegistry::default();
    reg.register_defaults();
    reg
}

fn golden_battle_app() -> App {
    let mut app = crate::common::app_builder::full_battle_app();
    app.insert_resource(test_buff_registry());
    app.insert_resource(test_terrain_registry());
    app.add_systems(Update, execute_effects);
    app
}

fn spawn_unit(app: &mut App, builder: UnitBuilder, name: &str) -> Entity {
    let entity = builder.spawn(app);
    app.world_mut().entity_mut(entity).insert((
        GridPosition::default(),
        UnitName(name.into()),
        SkillSlots::default(),
    ));
    entity
}

fn enqueue_damage(app: &mut App, source: Entity, target: Entity, amount: i32, is_skill: bool) {
    let mut queue = app.world_mut().resource_mut::<EffectQueue>();
    queue.pending.push(PendingEffect {
        source,
        target,
        data: PendingEffectData::Damage {
            amount,
            is_skill,
            base_amount: None,
        },
        source_tags: vec![],
        terrain_id: "plain".into(),
    });
}

fn enqueue_heal(app: &mut App, source: Entity, target: Entity, amount: i32) {
    let mut queue = app.world_mut().resource_mut::<EffectQueue>();
    queue.pending.push(PendingEffect {
        source,
        target,
        data: PendingEffectData::Heal {
            amount,
            base_amount: None,
        },
        source_tags: vec![],
        terrain_id: "plain".into(),
    });
}

fn get_record(app: &App) -> &BattleRecord {
    app.world().resource::<BattleRecord>()
}

/// 创建带 Entity redaction 的 insta Settings
fn battle_snapshot_settings() -> insta::Settings {
    let mut settings = insta::Settings::new();
    settings.add_redaction(".entries[].DamageApplied.target", "[entity]");
    settings.add_redaction(".entries[].DamageApplied.attacker", "[entity]");
    settings.add_redaction(".entries[].HealApplied.target", "[entity]");
    settings.add_redaction(".entries[].CharacterDied.entity", "[entity]");
    settings
}

// ══════════════════════════════════════════════════════════════
// 场景一：基础战斗 — 战士攻击哥布林
// ══════════════════════════════════════════════════════════════

#[test]
fn 基础战斗_战士攻击哥布林() {
    let mut app = golden_battle_app();

    let warrior = spawn_unit(&mut app, UnitBuilder::warrior(), "战士");
    let goblin = spawn_unit(&mut app, UnitBuilder::goblin(), "哥布林");

    // 战士对哥布林造成 8 点伤害
    enqueue_damage(&mut app, warrior, goblin, 8, false);
    app.update();

    let record = get_record(&app);
    let settings = battle_snapshot_settings();
    settings.bind(|| insta::assert_yaml_snapshot!(record));
}

// ══════════════════════════════════════════════════════════════
// 场景二：治疗战斗 — 角色受伤后治疗
// ══════════════════════════════════════════════════════════════

#[test]
fn 治疗战斗_角色受伤后治疗() {
    let mut app = golden_battle_app();

    let warrior = spawn_unit(&mut app, UnitBuilder::warrior().with_hp(10.0), "战士");
    let goblin = spawn_unit(&mut app, UnitBuilder::goblin(), "哥布林");
    let healer = spawn_unit(&mut app, UnitBuilder::mage().player(), "牧师");

    // 哥布林对战士造成 5 点伤害
    enqueue_damage(&mut app, goblin, warrior, 5, false);
    app.update();

    // 牧师对战士治疗 3 点
    enqueue_heal(&mut app, healer, warrior, 3);
    app.update();

    let record = get_record(&app);
    let settings = battle_snapshot_settings();
    settings.bind(|| insta::assert_yaml_snapshot!(record));
}

// ══════════════════════════════════════════════════════════════
// 场景三：致命伤害 — 角色死亡
// ══════════════════════════════════════════════════════════════

#[test]
fn 致命伤害_角色死亡() {
    let mut app = golden_battle_app();

    let warrior = spawn_unit(&mut app, UnitBuilder::warrior(), "战士");
    let goblin = spawn_unit(&mut app, UnitBuilder::goblin().with_hp(3.0), "哥布林");

    // 战士对哥布林造成 10 点伤害（致死）
    enqueue_damage(&mut app, warrior, goblin, 10, false);
    app.update();

    let record = get_record(&app);
    let settings = battle_snapshot_settings();
    settings.bind(|| insta::assert_yaml_snapshot!(record));
}
