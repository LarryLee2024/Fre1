//! 死亡处理 Feature Test
//!
//! 测试角色死亡完整流程：
//! 1. Dead 标记添加后 Hook 触发：acted=true + Selected 移除
//! 2. 致命伤害触发死亡：Effect Pipeline → Dead 标记 + CharacterDied Message
//! 3. 死亡角色不再受 Buff tick 影响（来自 buff_death_feature 合并）
//! 4. 存活角色正常受 Buff tick 影响（来自 buff_death_feature 合并）

use bevy::prelude::*;
use tactical_rpg::battle::{BattleEntry, BattleRecord, execute_effects};
use tactical_rpg::buff::{ActiveBuffs, BuffData, BuffRegistry, apply_buff, resolve_status_effects};
use tactical_rpg::character::{
    Dead, Faction, GridPosition, PersistentTags, Selected, Unit, UnitName,
};
use tactical_rpg::core::attribute::{AttributeKind, AttributeModifierDef, Attributes, ModifierOp};
use tactical_rpg::core::effect::{EffectQueue, PendingEffect, PendingEffectData};
use tactical_rpg::core::registry_loader::RegistryLoader;
use tactical_rpg::core::tag::{GameplayTag, GameplayTags};
use tactical_rpg::map::TerrainRegistry;
use tactical_rpg::skill::{SkillCooldowns, SkillSlots};
use tactical_rpg::turn::NeedsResolve;

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

/// 构建死亡 Feature 测试 App：
/// full_battle_app 基础 + execute_effects + 注册表
fn death_test_app() -> App {
    let mut app = crate::common::app_builder::full_battle_app();

    // 注册表
    app.insert_resource(test_buff_registry());
    app.insert_resource(test_terrain_registry());

    // 系统
    app.add_systems(Update, execute_effects);

    app
}

/// 在 App 中生成完整战斗角色（含 GridPosition / UnitName / SkillSlots）
fn spawn_unit(app: &mut App, builder: UnitBuilder, name: &str) -> Entity {
    let entity = builder.spawn(app);
    app.world_mut().entity_mut(entity).insert((
        GridPosition::default(),
        UnitName(name.into()),
        SkillSlots::default(),
    ));
    entity
}

/// 入队伤害效果
fn enqueue_damage(app: &mut App, source: Entity, target: Entity, amount: i32) {
    let mut queue = app.world_mut().resource_mut::<EffectQueue>();
    queue.pending.push(PendingEffect {
        source,
        target,
        data: PendingEffectData::Damage {
            amount,
            is_skill: false,
            base_amount: None,
            modifiers: Vec::new(),
        },
        source_tags: vec![],
        terrain_id: "plain".into(),
    });
}

// ── 合并辅助（来自 buff_death_feature） ──

/// 构建 BuffData 通用辅助
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
        default_duration: 3,
        modifiers,
        tags,
        dot_damage,
        hot_heal,
        is_stun: false,
        is_cleanse: false,
        is_buff,
    }
}

fn make_poison() -> BuffData {
    make_buff_data(
        "poison",
        false,
        vec![],
        vec![GameplayTag::DEBUFF, GameplayTag::POISON],
        3,
        0,
    )
}

/// 构建 resolve_status_effects 所需的完整 App
fn resolve_test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
    app.init_resource::<NeedsResolve>();
    app.add_message::<tactical_rpg::battle::CharacterDied>();
    app.add_message::<tactical_rpg::battle::DotApplied>();
    app.add_message::<tactical_rpg::battle::HotApplied>();
    app.add_message::<tactical_rpg::battle::StunApplied>();
    app.add_systems(Update, resolve_status_effects);
    app
}

/// 构建战士属性
fn warrior_attrs() -> Attributes {
    UnitBuilder::warrior().attrs().clone()
}

// ══════════════════════════════════════════════════════════════
// 场景一：死亡标记添加后 Hook 触发
// ══════════════════════════════════════════════════════════════

#[test]
fn 死亡标记添加后hook触发_acted为true且selected被移除() {
    let mut world = World::new();

    // 生成一个未行动、被选中的单位
    let entity = world
        .spawn((
            Unit {
                faction: Faction::Player,
                acted: false,
            },
            Attributes::default(),
            Selected,
        ))
        .id();

    // 验证初始状态
    assert!(
        !world.get::<Unit>(entity).unwrap().acted,
        "初始 acted 应为 false"
    );
    assert!(world.get::<Selected>(entity).is_some(), "初始应有 Selected");
    assert!(world.get::<Dead>(entity).is_none(), "初始不应有 Dead");

    // ── 添加 Dead 组件 → Hook 自动触发 ──
    world.entity_mut(entity).insert(Dead);

    // ── 验证 Hook 效果 ──
    let unit = world.get::<Unit>(entity).unwrap();
    assert!(unit.acted, "Dead Hook 应将 acted 设为 true");
    assert!(
        world.get::<Selected>(entity).is_none(),
        "Dead Hook 应移除 Selected"
    );
    assert!(world.get::<Dead>(entity).is_some(), "应存在 Dead 组件");
}

// ══════════════════════════════════════════════════════════════
// 场景二：致命伤害触发死亡
// ══════════════════════════════════════════════════════════════

#[test]
fn 致命伤害触发死亡_dead标记和character_died消息() {
    let mut app = death_test_app();

    // 生成角色：HP=5 的哥布林
    let goblin = spawn_unit(&mut app, UnitBuilder::goblin().with_hp(5.0), "哥布林");
    let warrior = spawn_unit(&mut app, UnitBuilder::warrior(), "战士");

    // 验证初始状态
    let hp = app
        .world()
        .get::<Attributes>(goblin)
        .unwrap()
        .get(AttributeKind::Hp);
    assert_eq!(hp, 5.0);
    assert!(app.world().get::<Dead>(goblin).is_none());

    // ── 通过 Effect Pipeline 造成致命伤害 ──
    enqueue_damage(&mut app, warrior, goblin, 10);
    app.update();

    // ── 验证 Dead 标记 ──
    assert!(
        app.world().get::<Dead>(goblin).is_some(),
        "致命伤害后应有 Dead 标记"
    );

    // ── 验证 Dead Hook 触发 ──
    let unit = app.world().get::<Unit>(goblin).unwrap();
    assert!(unit.acted, "Dead Hook 应将 acted 设为 true");

    // ── 验证 HP 降为 0 ──
    let hp = app
        .world()
        .get::<Attributes>(goblin)
        .unwrap()
        .get(AttributeKind::Hp);
    assert_eq!(hp, 0.0, "致命伤害后 HP 应为 0");

    // ── 验证 CharacterDied Message 通过 BattleRecord 记录 ──
    // full_battle_app 包含 record_character_died 系统，
    // 该系统监听 CharacterDied Message 并写入 BattleRecord
    let record = app.world().resource::<BattleRecord>();
    let has_died_entry = record.entries.iter().any(|entry| {
        matches!(entry, BattleEntry::CharacterDied { entity, name, faction }
            if *entity == goblin && name == "哥布林" && *faction == Faction::Enemy)
    });
    assert!(has_died_entry, "BattleRecord 应记录 CharacterDied 事件");
}

// ══════════════════════════════════════════════════════════════
// 合并场景：死亡角色不再受 Buff tick 影响（来自 buff_death_feature）
// ══════════════════════════════════════════════════════════════

#[test]
fn 死亡角色_resolve_status_effects不处理() {
    let mut app = resolve_test_app();

    // 生成一个已死亡的单位（带 Poison Buff）
    let entity = app
        .world_mut()
        .spawn((
            Unit {
                faction: Faction::Player,
                acted: true, // 已死亡，acted 应为 true
            },
            UnitName("阵亡战士".into()),
            GridPosition::default(),
            {
                let mut attrs = warrior_attrs();
                attrs.set_vital(AttributeKind::Hp, 0.0); // HP=0
                attrs
            },
            {
                let mut buffs = ActiveBuffs::default();
                let poison = make_poison();
                let mut tags_tmp = GameplayTags::default();
                let mut attrs_tmp = warrior_attrs();
                apply_buff(&mut buffs, &mut attrs_tmp, &mut tags_tmp, &poison, None, 3);
                buffs
            },
            GameplayTags::default(),
            SkillCooldowns::default(),
            PersistentTags::default(),
            Dead, // 已死亡
        ))
        .id();

    // 记录死亡前的 HP
    let hp_before = app
        .world()
        .get::<Attributes>(entity)
        .unwrap()
        .get(AttributeKind::Hp);
    assert_eq!(hp_before, 0.0);

    // 确认 Dead 组件存在
    assert!(app.world().get::<Dead>(entity).is_some());

    // 确认 Poison Buff 存在
    let buffs = app.world().get::<ActiveBuffs>(entity).unwrap();
    assert_eq!(buffs.len(), 1);
    assert_eq!(buffs.dot_damage(), 3);

    // ── 触发 resolve_status_effects ──
    app.world_mut().resource_mut::<NeedsResolve>().0 = true;
    app.update();

    // ── 验证：死亡角色的 HP 不应被 DoT 进一步降低 ──
    // resolve_status_effects 的 Query 应排除 Dead 实体
    let hp_after = app
        .world()
        .get::<Attributes>(entity)
        .unwrap()
        .get(AttributeKind::Hp);
    assert_eq!(hp_after, hp_before, "死亡角色不应再受 DoT 影响");
}

#[test]
fn 存活角色_resolve_status_effects正常处理dot() {
    let mut app = resolve_test_app();

    // 生成一个存活单位（带 Poison Buff）
    let entity = app
        .world_mut()
        .spawn((
            Unit {
                faction: Faction::Player,
                acted: false,
            },
            UnitName("战士".into()),
            GridPosition::default(),
            warrior_attrs(),
            {
                let mut buffs = ActiveBuffs::default();
                let poison = make_poison();
                let mut tags_tmp = GameplayTags::default();
                let mut attrs_tmp = warrior_attrs();
                apply_buff(&mut buffs, &mut attrs_tmp, &mut tags_tmp, &poison, None, 3);
                buffs
            },
            GameplayTags::default(),
            SkillCooldowns::default(),
            PersistentTags::default(),
        ))
        .id();

    let hp_before = app
        .world()
        .get::<Attributes>(entity)
        .unwrap()
        .get(AttributeKind::Hp);

    // ── 触发 resolve_status_effects ──
    app.world_mut().resource_mut::<NeedsResolve>().0 = true;
    app.update();

    // ── 验证：存活角色受到 DoT 伤害 ──
    let hp_after = app
        .world()
        .get::<Attributes>(entity)
        .unwrap()
        .get(AttributeKind::Hp);
    assert_eq!(hp_after, hp_before - 3.0, "存活角色应受到 Poison DoT 伤害");
}
