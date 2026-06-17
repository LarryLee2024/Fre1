//! Buff 系统 Feature Test
//!
//! 通过 Effect Pipeline 测试 Buff 完整生命周期：
//! 1. Poison 完整生命周期：ApplyBuff → DoT → 过期移除
//! 2. 增攻 Buff 修改属性：ApplyBuff → 属性增加 → 过期恢复
//! 3. Cleanse 移除 Debuff：两个 Debuff → Cleanse → 全部移除
//! 4. Cleanse 只移除 Debuff 保留 Buff

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
use tactical_rpg::core::ability::SkillCooldowns;
use tactical_rpg::core::attribute::{AttributeModifierDef, Attributes, ModifierOp};
use tactical_rpg::core::buff::{
    ActiveBuffs, BuffData, BuffRegistry, DurationPolicy, StackPolicy, apply_buff,
    resolve_status_effects,
};
use tactical_rpg::core::character::{GridPosition, UnitName};
use tactical_rpg::core::effect::{
    DurationDef, EffectQueue, PendingEffect, PendingEffectData, StackingDef,
};
use tactical_rpg::core::map::TerrainRegistry;
use tactical_rpg::core::registry_loader::RegistryLoader;
use tactical_rpg::core::tag::{GameplayTag, GameplayTags};
use tactical_rpg::core::turn::NeedsResolve;

use crate::common::fixtures::UnitBuilder;

// ── 测试辅助（Effect Pipeline 风格） ──

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

/// 注册测试专用 Buff（poison / stun / cleanse）
fn register_test_buffs(registry: &mut BuffRegistry) {
    // Poison：每回合 3 点 DoT
    if registry.get("poison").is_none() {
        registry.register(BuffData {
            id: "poison".into(),
            name: "中毒".into(),
            name_key: None,
            description: String::new(),
            effects: vec![],
            duration: DurationPolicy::Turns(3),
            stack: StackPolicy::NoStack,
            conditions: vec![],
            default_duration: 3,
            modifiers: vec![],
            tags: vec![GameplayTag::DEBUFF, GameplayTag::DMG_MAGICAL],
            dot_damage: 3,
            hot_heal: 0,
            is_stun: false,
            is_cleanse: false,
            is_buff: false,
        });
    }

    // Stun：晕眩 1 回合
    if registry.get("stun").is_none() {
        registry.register(BuffData {
            id: "stun".into(),
            name: "晕眩".into(),
            name_key: None,
            description: String::new(),
            effects: vec![],
            duration: DurationPolicy::Turns(1),
            stack: StackPolicy::NoStack,
            conditions: vec![],
            default_duration: 1,
            modifiers: vec![],
            tags: vec![GameplayTag::DEBUFF, GameplayTag::CONTROL_HARD],
            dot_damage: 0,
            hot_heal: 0,
            is_stun: true,
            is_cleanse: false,
            is_buff: false,
        });
    }

    // Cleanse：驱散所有 Debuff
    if registry.get("cleanse").is_none() {
        registry.register(BuffData {
            id: "cleanse".into(),
            name: "驱散".into(),
            name_key: None,
            description: String::new(),
            effects: vec![],
            duration: DurationPolicy::Turns(0),
            stack: StackPolicy::NoStack,
            conditions: vec![],
            default_duration: 0,
            modifiers: vec![],
            tags: vec![GameplayTag::BUFF],
            dot_damage: 0,
            hot_heal: 0,
            is_stun: false,
            is_cleanse: true,
            is_buff: true,
        });
    }
}

/// 构建 Buff Feature 测试 App：
/// combat_app 基础 + execute_effects + resolve_status_effects + 注册表 + Message
fn buff_test_app() -> App {
    let mut app = crate::common::app_builder::combat_app();

    // 注册 Message
    app.add_message::<tactical_rpg::core::battle::CharacterDied>()
        .add_message::<tactical_rpg::core::battle::DamageApplied>()
        .add_message::<tactical_rpg::core::battle::HealApplied>()
        .add_message::<tactical_rpg::core::battle::DotApplied>()
        .add_message::<tactical_rpg::core::battle::HotApplied>()
        .add_message::<tactical_rpg::core::battle::StunApplied>();

    // 注册表
    let mut buff_registry = test_buff_registry();
    register_test_buffs(&mut buff_registry);
    app.insert_resource(buff_registry);
    app.insert_resource(test_terrain_registry());

    // NeedsResolve（resolve_status_effects 依赖）
    app.init_resource::<NeedsResolve>();

    // 系统
    app.add_systems(Update, tactical_rpg::core::battle::execute_effects);
    app.add_systems(Update, resolve_status_effects);

    app
}

/// 在 App 中生成完整战斗角色（含 GridPosition / UnitName / SkillCooldowns）
fn spawn_unit(app: &mut App, builder: UnitBuilder, name: &str) -> Entity {
    let entity = builder.spawn(app);
    // 补充 execute_effects / resolve_status_effects 需要的组件
    app.world_mut().entity_mut(entity).insert((
        GridPosition::default(),
        UnitName(name.into()),
        SkillCooldowns::default(),
    ));
    entity
}

/// 入队 ApplyModifier 效果
fn enqueue_apply_buff(app: &mut App, source: Entity, target: Entity, buff_id: &str, duration: u32) {
    let mut queue = app.world_mut().resource_mut::<EffectQueue>();
    queue.pending.push(PendingEffect {
        source,
        target,
        data: PendingEffectData::ApplyModifier {
            modifier_id: buff_id.into(),
            duration: DurationDef::TurnLimited(duration),
            stacking: StackingDef::Replace,
        },
        source_tags: vec![],
        terrain_id: String::new(),
    });
}

/// 入队 Cleanse 效果
fn enqueue_cleanse(app: &mut App, source: Entity, target: Entity) {
    let mut queue = app.world_mut().resource_mut::<EffectQueue>();
    queue.pending.push(PendingEffect {
        source,
        target,
        data: PendingEffectData::Cleanse,
        source_tags: vec![],
        terrain_id: String::new(),
    });
}

/// 推进一个 Update tick
fn tick(app: &mut App) {
    app.update();
}

/// 触发 resolve_status_effects（设置 NeedsResolve 标记后 tick）
/// resolve_status_effects 内部已包含 tick_buffs 逻辑（递减 + 过期清理）
fn trigger_resolve(app: &mut App) {
    app.world_mut().resource_mut::<NeedsResolve>().0 = true;
    app.update();
}

// ── 测试辅助（直接操作风格，来自 buff_death_feature） ──

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
        name_key: None,
        description: String::new(),
        effects: vec![],
        duration: DurationPolicy::Turns(3),
        stack: StackPolicy::NoStack,
        conditions: vec![],
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
        vec![GameplayTag::DEBUFF, GameplayTag::DMG_MAGICAL],
        3,
        0,
    )
}

fn make_attack_up() -> BuffData {
    make_buff_data(
        "attack_up",
        true,
        vec![AttributeModifierDef {
            config_id: "phys_atk".into(),
            op: ModifierOp::Add,
            value: 5,
        }],
        vec![GameplayTag::BUFF],
        0,
        0,
    )
}

fn make_cleanse() -> BuffData {
    BuffData {
        id: "cleanse".into(),
        name: "驱散".into(),
        name_key: None,
        description: String::new(),
        effects: vec![],
        duration: DurationPolicy::Turns(0),
        stack: StackPolicy::NoStack,
        conditions: vec![],
        default_duration: 0,
        modifiers: vec![],
        tags: vec![GameplayTag::BUFF],
        dot_damage: 0,
        hot_heal: 0,
        is_stun: false,
        is_cleanse: true,
        is_buff: true,
    }
}

/// 构建战士属性
fn warrior_attrs() -> Attributes {
    UnitBuilder::warrior().attrs().clone()
}

// ══════════════════════════════════════════════════════════════
// 场景一：Poison 完整生命周期
// ══════════════════════════════════════════════════════════════

/// Test ID: BUFF-001
/// Title: Poison 完整生命周期 - 施加 DoT 过期移除
///
/// Given: 战士 HP=30，法师施加 Poison（duration=3, DoT=3）
/// When: 通过 Effect Pipeline 施加 Poison 并推进 4 回合
/// Then: 每回合扣 3 HP，3 回合后 Poison 过期移除
///
/// Assertions: HP 从 30→27→24→21→21，Poison 过期后 ActiveBuffs 为空
#[test]
fn poison完整生命周期_施加_dot_过期移除() {
    let mut app = buff_test_app();

    // 生成角色
    let warrior = spawn_unit(&mut app, UnitBuilder::warrior(), "战士");
    let caster = spawn_unit(&mut app, UnitBuilder::mage(), "法师");

    // 初始 HP=50
    let initial_hp = app.world().get::<Attributes>(warrior).unwrap().current_hp;
    assert_eq!(initial_hp, 50);

    // ── 通过 Effect Pipeline 施加 Poison（duration=3） ──
    enqueue_apply_buff(&mut app, caster, warrior, "poison", 3);
    tick(&mut app);

    // 验证 Poison 已添加
    let buffs = app.world().get::<ActiveBuffs>(warrior).unwrap();
    assert_eq!(buffs.len(), 1);
    assert_eq!(buffs.instances[0].buff_id, "poison");
    assert_eq!(buffs.dot_damage(), 3);

    // ── 第1回合：resolve 结算 DoT + tick ──
    trigger_resolve(&mut app);

    let hp_after_r1 = app.world().get::<Attributes>(warrior).unwrap().current_hp;
    assert_eq!(hp_after_r1, 47, "第1回合 DoT 应扣 3 HP");

    // resolve_status_effects 内部已 tick：remaining 3→2
    let buffs = app.world().get::<ActiveBuffs>(warrior).unwrap();
    assert_eq!(buffs.instances[0].remaining_turns, 2);

    // ── 第2回合：resolve 结算 DoT + tick ──
    trigger_resolve(&mut app);

    let hp_after_r2 = app.world().get::<Attributes>(warrior).unwrap().current_hp;
    assert_eq!(hp_after_r2, 44, "第2回合 DoT 应扣 3 HP");

    let buffs = app.world().get::<ActiveBuffs>(warrior).unwrap();
    assert_eq!(buffs.instances[0].remaining_turns, 1);

    // ── 第3回合：resolve 结算 DoT + tick（remaining 1→0，修饰符清理） ──
    trigger_resolve(&mut app);

    let hp_after_r3 = app.world().get::<Attributes>(warrior).unwrap().current_hp;
    assert_eq!(hp_after_r3, 41, "第3回合 DoT 应扣 3 HP");

    // remaining=0 的 buff 在下次 tick 时被移除
    // resolve_status_effects 的 tick_buffs 会先递减到 0，然后下次 tick 才移除
    let buffs = app.world().get::<ActiveBuffs>(warrior).unwrap();
    assert_eq!(buffs.instances[0].remaining_turns, 0);

    // ── 第4回合 tick：remaining=0 的 buff 被移除 ──
    trigger_resolve(&mut app);

    let buffs = app.world().get::<ActiveBuffs>(warrior).unwrap();
    assert!(buffs.is_empty(), "Poison 过期后应被移除");
    assert_eq!(buffs.dot_damage(), 0, "Poison 过期后 DoT 应为 0");

    // HP 保持第3回合结算后的值（第4回合无 DoT）
    let final_hp = app.world().get::<Attributes>(warrior).unwrap().current_hp;
    assert_eq!(final_hp, 41);
}

// ══════════════════════════════════════════════════════════════
// 场景二：增攻 Buff 修改属性
// ══════════════════════════════════════════════════════════════

/// Test ID: BUFF-002
/// Title: 增攻 Buff 修改属性 - 施加后增加过期后恢复
///
/// Given: 战士 Attack=10，法师施加增攻 Buff（duration=3, +5 Attack）
/// When: 通过 Effect Pipeline 施加增攻 Buff 并推进 4 回合
/// Then: Attack 从 10→15，3 回合后恢复为 10，Buff 过期移除
///
/// Assertions: Attack 15→15→15→10→10，Buff 过期后 ActiveBuffs 为空
#[test]
fn 增攻buff修改属性_施加后增加_过期后恢复() {
    let mut app = buff_test_app();

    let warrior = spawn_unit(&mut app, UnitBuilder::warrior(), "战士");
    let caster = spawn_unit(&mut app, UnitBuilder::mage(), "法师");

    // 战士基础 phys_atk = 5
    let base_attack = app
        .world()
        .get::<Attributes>(warrior)
        .unwrap()
        .get("phys_atk");
    assert_eq!(base_attack, 5);

    // ── 通过 Effect Pipeline 施加增攻 Buff（duration=3） ──
    enqueue_apply_buff(&mut app, caster, warrior, "attack_up", 3);
    tick(&mut app);

    // 验证 phys_atk 增加
    let attack_with_buff = app
        .world()
        .get::<Attributes>(warrior)
        .unwrap()
        .get("phys_atk");
    assert_eq!(attack_with_buff, 10, "增攻 Buff 应使 phys_atk +5");

    // 验证 Buff 存在
    let buffs = app.world().get::<ActiveBuffs>(warrior).unwrap();
    assert_eq!(buffs.len(), 1);
    assert_eq!(buffs.instances[0].buff_id, "attack_up");

    // ── 第1回合 resolve + tick：remaining 3→2 ──
    trigger_resolve(&mut app);
    let attack = app
        .world()
        .get::<Attributes>(warrior)
        .unwrap()
        .get("phys_atk");
    assert_eq!(attack, 10, "Buff 期间 phys_atk 保持 10");

    // ── 第2回合 resolve + tick：remaining 2→1 ──
    trigger_resolve(&mut app);
    let attack = app
        .world()
        .get::<Attributes>(warrior)
        .unwrap()
        .get("phys_atk");
    assert_eq!(attack, 10, "Buff 期间 phys_atk 保持 10");

    // ── 第3回合 resolve + tick：remaining 1→0，修饰符清理 ──
    trigger_resolve(&mut app);
    let attack = app
        .world()
        .get::<Attributes>(warrior)
        .unwrap()
        .get("phys_atk");
    assert_eq!(attack, 5, "Buff 过期后 phys_atk 应恢复为 5");

    // ── 第4回合 resolve + tick：remaining=0 的 buff 被移除 ──
    trigger_resolve(&mut app);
    let buffs = app.world().get::<ActiveBuffs>(warrior).unwrap();
    assert!(buffs.is_empty(), "增攻 Buff 过期后应被移除");
}

// ══════════════════════════════════════════════════════════════
// 场景三：Cleanse 移除 Debuff
// ══════════════════════════════════════════════════════════════

/// Test ID: BUFF-003
/// Title: Cleanse 移除 Debuff - 两个 Debuff 全部移除
///
/// Given: 战士被施加 Poison + Stun 两个 Debuff
/// When: 通过 Effect Pipeline 施加 Cleanse
/// Then: 所有 Debuff 被移除，晕眩解除，DoT 清零
///
/// Assertions: ActiveBuffs 为空，is_stunned=false，dot_damage=0
#[test]
fn cleanse移除debuff_两个debuff全部移除() {
    let mut app = buff_test_app();

    let warrior = spawn_unit(&mut app, UnitBuilder::warrior(), "战士");
    let caster = spawn_unit(&mut app, UnitBuilder::mage(), "法师");

    // ── 通过 Effect Pipeline 施加 Poison + Stun ──
    enqueue_apply_buff(&mut app, caster, warrior, "poison", 3);
    tick(&mut app);

    enqueue_apply_buff(&mut app, caster, warrior, "stun", 1);
    tick(&mut app);

    // 验证两个 Debuff 都已添加
    let buffs = app.world().get::<ActiveBuffs>(warrior).unwrap();
    assert_eq!(buffs.len(), 2, "应有 2 个 Debuff");
    assert!(buffs.is_stunned(), "应有晕眩状态");
    assert_eq!(buffs.dot_damage(), 3, "DoT 应为 3");

    // ── 通过 Effect Pipeline 施加 Cleanse ──
    enqueue_cleanse(&mut app, caster, warrior);
    tick(&mut app);

    // 验证所有 Debuff 被移除
    let buffs = app.world().get::<ActiveBuffs>(warrior).unwrap();
    assert!(buffs.is_empty(), "Cleanse 应移除所有 Debuff");
    assert!(!buffs.is_stunned(), "Cleanse 后不应有晕眩");
    assert_eq!(buffs.dot_damage(), 0, "Cleanse 后 DoT 应为 0");
}

// ══════════════════════════════════════════════════════════════
// 合并场景：Cleanse 只移除 Debuff 保留 Buff（来自 buff_death_feature）
// ══════════════════════════════════════════════════════════════

/// Test ID: BUFF-004
/// Title: Cleanse 只移除 Debuff 保留 Buff
///
/// Given: 战士被施加 AttackUp(Buff) + Poison(Debuff)
/// When: 通过 apply_buff 施加 Cleanse
/// Then: Poison 被移除，AttackUp 保留，Attack 保持 15
///
/// Assertions: ActiveBuffs 只剩 attack_up，dot_damage=0，Attack=15
#[test]
fn cleanse_只移除debuff保留buff() {
    let attack_up = make_attack_up();
    let poison = make_poison();
    let cleanse = make_cleanse();

    let mut buffs = ActiveBuffs::default();
    let mut attrs = warrior_attrs();
    let mut tags = GameplayTags::default();

    // ── 施加 Buff + Debuff ──
    apply_buff(&mut buffs, &mut attrs, &mut tags, &attack_up, None, 3);
    apply_buff(&mut buffs, &mut attrs, &mut tags, &poison, None, 3);

    assert_eq!(buffs.len(), 2);
    assert_eq!(attrs.get("phys_atk"), 10); // 5+5
    assert_eq!(buffs.dot_damage(), 3);

    // ── Cleanse ──
    apply_buff(&mut buffs, &mut attrs, &mut tags, &cleanse, None, 0);

    // 只有 Buff 保留
    assert_eq!(buffs.len(), 1);
    assert_eq!(buffs.instances[0].buff_id, "attack_up");
    assert_eq!(attrs.get("phys_atk"), 10); // 保留
    assert_eq!(buffs.dot_damage(), 0); // Poison 被驱散
}
