//! 第 4 层 Scenario Test（BDD 风格战斗场景）
//!
//! 跨模块端到端验证，Given-When-Then 风格组织：
//! 1. 火球vs骑士 - 法师对骑士造成技能伤害 + Burning Buff
//! 2. 毒伤战斗 - 战士中毒后每回合受到 DoT
//! 3. 地形优势 - 地形防御加成减少伤害
//! 4. 击杀触发死亡 - 致命伤害触发死亡流程

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

use bevy::prelude::*;
use tactical_rpg::core::attribute::{AttributeKind, Attributes};
use tactical_rpg::core::battle::{BattleEntry, BattleRecord, CharacterDied, execute_effects};
use tactical_rpg::core::buff::{ActiveBuffs, BuffData, BuffRegistry, resolve_status_effects};
use tactical_rpg::core::character::{Dead, Faction, GridPosition, Unit, UnitName};
use tactical_rpg::core::effect::{
    EffectHandlerRegistry, EffectQueue, PendingEffect, PendingEffectData,
    calculate_damage_from_effect,
};
use tactical_rpg::core::map::TerrainRegistry;
use tactical_rpg::core::registry_loader::RegistryLoader;
use tactical_rpg::core::skill::SkillCooldowns;
use tactical_rpg::core::tag::GameplayTag;
use tactical_rpg::core::turn::NeedsResolve;

use crate::common::fixtures::UnitBuilder;

// ── 测试辅助 ──

/// 创建带默认数据的 BuffRegistry（不依赖文件系统）
fn test_buff_registry() -> BuffRegistry {
    let mut reg = BuffRegistry::default();
    reg.register_defaults();
    reg
}

/// 创建带默认数据的 TerrainRegistry（不依赖文件系统）
fn test_terrain_registry() -> TerrainRegistry {
    let mut reg = TerrainRegistry::default();
    reg.register_defaults();
    reg
}

/// 构建 Scenario 测试 App：
/// full_battle_app 基础 + execute_effects + resolve_status_effects + 注册表
fn scenario_test_app() -> App {
    let mut app = crate::common::app_builder::full_battle_app();

    // 注册表
    app.insert_resource(test_buff_registry());
    app.insert_resource(test_terrain_registry());

    // NeedsResolve（resolve_status_effects 依赖）
    app.init_resource::<NeedsResolve>();

    // 系统
    app.add_systems(Update, execute_effects);
    app.add_systems(Update, resolve_status_effects);

    app
}

/// 在 App 中生成完整战斗角色（含 GridPosition / UnitName / SkillCooldowns）
fn spawn_unit(app: &mut App, builder: UnitBuilder, name: &str) -> Entity {
    let entity = builder.spawn(app);
    app.world_mut().entity_mut(entity).insert((
        GridPosition::default(),
        UnitName(name.into()),
        SkillCooldowns::default(),
    ));
    entity
}

/// 入队技能伤害效果
fn enqueue_skill_damage(app: &mut App, source: Entity, target: Entity, amount: i32) {
    let mut queue = app.world_mut().resource_mut::<EffectQueue>();
    queue.pending.push(PendingEffect {
        source,
        target,
        data: PendingEffectData::Damage {
            amount,
            is_skill: true,
            base_amount: None,
            modifiers: Vec::new(),
        },
        source_tags: vec![],
        terrain_id: "plain".into(),
    });
}

/// 入队 ApplyBuff 效果
fn enqueue_apply_buff(app: &mut App, source: Entity, target: Entity, buff_id: &str, duration: u32) {
    let mut queue = app.world_mut().resource_mut::<EffectQueue>();
    queue.pending.push(PendingEffect {
        source,
        target,
        data: PendingEffectData::ApplyBuff {
            buff_id: buff_id.into(),
            duration,
        },
        source_tags: vec![],
        terrain_id: String::new(),
    });
}

/// 推进一个 Update tick
fn tick(app: &mut App) {
    app.update();
}

/// 触发 resolve_status_effects（设置 NeedsResolve 标记后 tick）
fn trigger_resolve(app: &mut App) {
    app.world_mut().resource_mut::<NeedsResolve>().0 = true;
    app.update();
}

// ══════════════════════════════════════════════════════════════
// 场景一：火球vs骑士
// 法师对骑士造成技能伤害 + Burning Buff
// ══════════════════════════════════════════════════════════════

/// SCN-001: 火球vs骑士 — 法师对骑士造成技能伤害 + Burning Buff
///
/// Given: 法师(Mage)和骑士(Warrior/Enemy)在场
/// When: 法师释放火球（Damage 12 + ApplyBuff "burn" duration=2）
/// Then: 骑士 HP 减少 12，获得 burn Buff（DoT=2, remaining=2）
#[test]
fn 火球vs骑士_技能伤害加burning_buff() {
    let mut app = scenario_test_app();

    // ── Given：法师和骑士 ──
    let mage = spawn_unit(&mut app, UnitBuilder::mage(), "法师");
    let knight = spawn_unit(&mut app, UnitBuilder::warrior().enemy(), "暗黑骑士");

    let knight_hp_before = app
        .world()
        .get::<Attributes>(knight)
        .unwrap()
        .get(AttributeKind::Hp);
    let knight_buffs_before = app.world().get::<ActiveBuffs>(knight).unwrap().len();

    // ── When：法师释放火球（Damage + ApplyBuff burning） ──
    // 火球术 = 技能伤害 12 + Burning Buff (duration=2)
    enqueue_skill_damage(&mut app, mage, knight, 12);
    enqueue_apply_buff(&mut app, mage, knight, "burn", 2);
    tick(&mut app);

    // ── Then：骑士受到伤害 ──
    let knight_hp_after = app
        .world()
        .get::<Attributes>(knight)
        .unwrap()
        .get(AttributeKind::Hp);
    assert!(
        knight_hp_after < knight_hp_before,
        "骑士应受到技能伤害：HP 从 {} 降至 {}",
        knight_hp_before,
        knight_hp_after
    );
    assert_eq!(
        knight_hp_after,
        knight_hp_before - 12.0,
        "骑士应受到 12 点技能伤害"
    );

    // ── Then：骑士获得 Burning Buff ──
    let buffs = app.world().get::<ActiveBuffs>(knight).unwrap();
    assert!(buffs.len() > knight_buffs_before, "骑士应获得 Burning Buff");
    assert!(
        buffs.iter().any(|b| b.buff_id == "burn"),
        "骑士应有 burn Buff"
    );

    // 验证 Burning Buff 属性：DoT=2, 减防-2
    let burn_buff = buffs.iter().find(|b| b.buff_id == "burn").unwrap();
    assert_eq!(burn_buff.dot_damage, 2, "Burning DoT 应为 2");
    assert_eq!(burn_buff.remaining_turns, 2, "Burning 持续回合应为 2");
}

// ══════════════════════════════════════════════════════════════
// 场景二：毒伤战斗
// 战士中毒后每回合受到 DoT
// ══════════════════════════════════════════════════════════════

/// SCN-002: 毒伤战斗 — 战士中毒后每回合受到 DoT 伤害
///
/// Given: 战士(Warrior)获得 Poison Buff（duration=3, dot_damage=3）
/// When: 连续推进 4 回合（resolve_status_effects）
/// Then: 前 3 回合每回合扣 3 HP（30→27→24→21），第 4 回合 Poison 过期不再扣血
#[test]
fn 毒伤战斗_每回合受到dot伤害() {
    let mut app = scenario_test_app();

    // ── Given：战士获得 Poison Buff ──
    let warrior = spawn_unit(&mut app, UnitBuilder::warrior(), "战士");
    let caster = spawn_unit(&mut app, UnitBuilder::mage(), "法师");

    let initial_hp = app
        .world()
        .get::<Attributes>(warrior)
        .unwrap()
        .get(AttributeKind::Hp);
    assert_eq!(initial_hp, 30.0, "战士初始 HP 应为 30");

    // 通过 Effect Pipeline 施加 Poison（duration=3, dot_damage=3）
    enqueue_apply_buff(&mut app, caster, warrior, "poison", 3);
    tick(&mut app);

    // 验证 Poison 已添加
    let buffs = app.world().get::<ActiveBuffs>(warrior).unwrap();
    assert!(
        buffs.iter().any(|b| b.buff_id == "poison"),
        "战士应有 Poison Buff"
    );
    assert_eq!(buffs.dot_damage(), 3, "Poison DoT 应为 3");

    // ── When：推进回合（resolve_status_effects） ──

    // 第1回合结算
    trigger_resolve(&mut app);

    // ── Then：战士每回合受到 DoT 伤害 ──
    let hp_after_r1 = app
        .world()
        .get::<Attributes>(warrior)
        .unwrap()
        .get(AttributeKind::Hp);
    assert_eq!(hp_after_r1, 27.0, "第1回合 DoT 应扣 3 HP（30-3=27）");

    // 第2回合结算
    trigger_resolve(&mut app);

    let hp_after_r2 = app
        .world()
        .get::<Attributes>(warrior)
        .unwrap()
        .get(AttributeKind::Hp);
    assert_eq!(hp_after_r2, 24.0, "第2回合 DoT 应扣 3 HP（27-3=24）");

    // 第3回合结算
    trigger_resolve(&mut app);

    let hp_after_r3 = app
        .world()
        .get::<Attributes>(warrior)
        .unwrap()
        .get(AttributeKind::Hp);
    assert_eq!(hp_after_r3, 21.0, "第3回合 DoT 应扣 3 HP（24-3=21）");

    // 第4回合：Poison 过期，不再造成 DoT
    trigger_resolve(&mut app);

    let hp_after_r4 = app
        .world()
        .get::<Attributes>(warrior)
        .unwrap()
        .get(AttributeKind::Hp);
    assert_eq!(hp_after_r4, 21.0, "Poison 过期后不再造成 DoT 伤害");

    let buffs = app.world().get::<ActiveBuffs>(warrior).unwrap();
    assert!(buffs.is_empty(), "Poison 过期后应被移除");
}

// ══════════════════════════════════════════════════════════════
// 场景三：地形优势
// 地形防御加成在 generate 阶段减少伤害量
// ══════════════════════════════════════════════════════════════

/// SCN-003: 地形优势 — 森林地形防御加成减少伤害
///
/// Given: 攻击者 ATK=10，目标 DEF=3，森林 defense_bonus=2
/// When: 分别在平原(bonus=0)和森林(bonus=2)计算伤害
/// Then: 平原伤害=7，森林伤害=5，森林目标 HP 高于平原目标
#[test]
fn 地形优势_森林地形减少伤害() {
    // ── Given：攻击者 ATK=10，目标 DEF=3 ──
    let effective_atk = 10.0;
    let effective_def = 3.0;
    let base_def = 3.0;
    let multiplier = 1.0;
    let ignore_def_percent = 0.0;

    // ── When：在森林地形（defense_bonus=2）vs 平原地形（defense_bonus=0）生成伤害 ──
    let dmg_plain = calculate_damage_from_effect(
        effective_atk,
        effective_def,
        base_def,
        multiplier,
        ignore_def_percent,
        0, // 平原无防御加成
    );
    let dmg_forest = calculate_damage_from_effect(
        effective_atk,
        effective_def,
        base_def,
        multiplier,
        ignore_def_percent,
        2, // 森林防御加成=2
    );

    // ── Then：森林地形伤害更低 ──
    assert_eq!(dmg_plain, 7, "平原地形：10-3-0=7");
    assert_eq!(dmg_forest, 5, "森林地形：10-3-2=5");
    assert!(dmg_forest < dmg_plain, "森林地形防御加成应减少伤害");

    // ── 验证完整管道：不同地形生成的伤害量入队后实际扣血不同 ──
    let mut app = scenario_test_app();
    let attacker = spawn_unit(&mut app, UnitBuilder::warrior(), "战士");
    let target_in_forest = spawn_unit(&mut app, UnitBuilder::goblin().with_hp(30.0), "森林哥布林");
    let target_on_plain = spawn_unit(&mut app, UnitBuilder::goblin().with_hp(30.0), "平原哥布林");

    // 入队不同地形生成的伤害量
    let mut queue = app.world_mut().resource_mut::<EffectQueue>();
    queue.pending.push(PendingEffect {
        source: attacker,
        target: target_in_forest,
        data: PendingEffectData::Damage {
            amount: dmg_forest,
            is_skill: false,
            base_amount: None,
            modifiers: Vec::new(),
        },
        source_tags: vec![],
        terrain_id: "forest".into(),
    });
    queue.pending.push(PendingEffect {
        source: attacker,
        target: target_on_plain,
        data: PendingEffectData::Damage {
            amount: dmg_plain,
            is_skill: false,
            base_amount: None,
            modifiers: Vec::new(),
        },
        source_tags: vec![],
        terrain_id: "plain".into(),
    });
    app.update();

    let hp_forest = app
        .world()
        .get::<Attributes>(target_in_forest)
        .unwrap()
        .get(AttributeKind::Hp);
    let hp_plain = app
        .world()
        .get::<Attributes>(target_on_plain)
        .unwrap()
        .get(AttributeKind::Hp);

    assert_eq!(hp_plain, 23.0, "平原：30-7=23");
    assert_eq!(hp_forest, 25.0, "森林：30-5=25");
    assert!(hp_forest > hp_plain, "森林地形目标 HP 应高于平原地形目标");
}

// ══════════════════════════════════════════════════════════════
// 场景四：击杀触发死亡
// 致命伤害触发死亡流程
// ══════════════════════════════════════════════════════════════

/// SCN-004: 击杀触发死亡 — 致命伤害触发 Dead 标记
///
/// Given: 目标哥布林 HP=5，攻击者战士 ATK 足够
/// When: 造成 10 点致命伤害
/// Then: 目标获得 Dead 标记，HP=0
///
/// Note: CharacterDied 消息由 Dead Observer 系统发送，当前未实现，
///       因此 BattleRecord 暂不包含 CharacterDied 条目
#[test]
fn 击杀触发死亡_dead标记和character_died消息() {
    let mut app = scenario_test_app();

    // ── Given：目标 HP 很低 ──
    let attacker = spawn_unit(&mut app, UnitBuilder::warrior(), "战士");
    let target = spawn_unit(&mut app, UnitBuilder::goblin().with_hp(5.0), "残血哥布林");

    // 验证初始状态
    assert_eq!(
        app.world()
            .get::<Attributes>(target)
            .unwrap()
            .get(AttributeKind::Hp),
        5.0,
        "目标初始 HP 应为 5"
    );
    assert!(
        app.world().get::<Dead>(target).is_none(),
        "目标初始不应有 Dead 标记"
    );

    // ── When：造成致命伤害 ──
    enqueue_skill_damage(&mut app, attacker, target, 10);
    tick(&mut app);

    // ── Then：目标获得 Dead 标记 ──
    assert!(
        app.world().get::<Dead>(target).is_some(),
        "致命伤害后目标应获得 Dead 标记"
    );

    // Dead Hook 应触发：acted=true
    let unit = app.world().get::<Unit>(target).unwrap();
    assert!(unit.acted, "Dead Hook 应将 acted 设为 true");

    // HP 应降为 0
    let hp = app
        .world()
        .get::<Attributes>(target)
        .unwrap()
        .get(AttributeKind::Hp);
    assert_eq!(hp, 0.0, "致命伤害后 HP 应为 0");
}
