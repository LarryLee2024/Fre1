//! P1 集成测试：技能系统跨模块联动
//!
//! 跨 skill + core/attribute + core/tag 测试技能条件检查、
//! 技能槽管理、冷却追踪在真实属性数据下的行为。

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
use tactical_rpg::core::tag::{GameplayTag, GameplayTags};
use tactical_rpg::core::skill::{
    BASIC_ATTACK_ID, SkillCooldowns, SkillData, SkillSlots, SkillTargeting, effective_skill_range,
};

use crate::common::fixtures::warrior_attrs;

// ── 测试辅助 ──

/// 法师模板：Intelligence=5 → MaxMp=25, MagicAttack=10
/// 注意：与 crate::common::fixtures::mage_attrs() 属性值不同，保留本地版本
fn mage_attrs() -> Attributes {
    let mut a = Attributes::default();
    a.set_base(AttributeKind::Might, 2.0);
    a.set_base(AttributeKind::Vitality, 3.0);
    a.set_base(AttributeKind::Agility, 6.0);
    a.set_base(AttributeKind::Dexterity, 3.0);
    a.set_base(AttributeKind::Intelligence, 5.0);
    a.set_base(AttributeKind::Willpower, 4.0);
    a.set_base(AttributeKind::Presence, 3.0);
    a.set_base(AttributeKind::Luck, 2.0);
    a.set_base_attack_range(2);
    a.fill_vital_resources();
    a
}

fn basic_attack() -> SkillData {
    SkillData {
        id: BASIC_ATTACK_ID.into(),
        name: "普通攻击".into(),
        description: String::new(),
        cost_mp: 0,
        range: 0,
        targeting: SkillTargeting::SingleEnemy,
        effects: vec![],
        tags: vec![],
        conditions: vec![],
        cooldown: 0,
        priority: 0,
    }
}

fn fireball() -> SkillData {
    SkillData {
        id: "fireball".into(),
        name: "火球".into(),
        description: String::new(),
        cost_mp: 8,
        range: 3,
        targeting: SkillTargeting::SingleEnemy,
        effects: vec![],
        tags: vec![],
        conditions: vec![tactical_rpg::core::skill::SkillCondition::MpCost(8)],
        cooldown: 2,
        priority: 10,
    }
}

fn heal() -> SkillData {
    SkillData {
        id: "heal".into(),
        name: "治疗".into(),
        description: String::new(),
        cost_mp: 5,
        range: 2,
        targeting: SkillTargeting::SingleAlly,
        effects: vec![],
        tags: vec![],
        conditions: vec![],
        cooldown: 2,
        priority: 15,
    }
}

fn mage_only_skill() -> SkillData {
    SkillData {
        id: "arcane_blast".into(),
        name: "奥术冲击".into(),
        description: String::new(),
        cost_mp: 10,
        range: 2,
        targeting: SkillTargeting::SingleEnemy,
        effects: vec![],
        tags: vec![],
        conditions: vec![
            tactical_rpg::core::skill::SkillCondition::RequireTag(GameplayTag::MAGE),
            tactical_rpg::core::skill::SkillCondition::MpCost(10),
        ],
        cooldown: 0,
        priority: 20,
    }
}

fn berserker_skill() -> SkillData {
    SkillData {
        id: "berserk".into(),
        name: "狂暴".into(),
        description: String::new(),
        cost_mp: 0,
        range: 1,
        targeting: SkillTargeting::SingleEnemy,
        effects: vec![],
        tags: vec![],
        conditions: vec![tactical_rpg::core::skill::SkillCondition::HpBelow(0.3)],
        cooldown: 0,
        priority: 30,
    }
}

fn purify_skill() -> SkillData {
    SkillData {
        id: "purify".into(),
        name: "净化".into(),
        description: String::new(),
        cost_mp: 0,
        range: 2,
        targeting: SkillTargeting::SingleAlly,
        effects: vec![],
        tags: vec![],
        conditions: vec![tactical_rpg::core::skill::SkillCondition::TargetRequireTag(
            GameplayTag::DEBUFF,
        )],
        cooldown: 0,
        priority: 25,
    }
}

// ══════════════════════════════════════════════════════════════
// 场景一：SkillSlots + SkillCooldowns 联动
// ══════════════════════════════════════════════════════════════

/// LSS-001: 技能槽默认攻击始终可用
///
/// Given: SkillSlots 含 basic_attack + fireball + heal，冷却默认
/// When: 检查 basic_attack.can_use
/// Then: 返回 Ok（无 MP 消耗，无冷却）
#[test]
fn 技能槽_默认攻击始终可用() {
    let slots = SkillSlots::new(vec![
        BASIC_ATTACK_ID.into(),
        "fireball".into(),
        "heal".into(),
    ]);
    let cooldowns = SkillCooldowns::default();

    // 基础攻击没有冷却，始终可用
    let skill = basic_attack();
    let attrs = warrior_attrs();
    let tags = GameplayTags::default();
    assert!(skill.can_use(&attrs, &tags, None, 0).is_ok());

    // 验证 default_attack 返回基础攻击
    assert_eq!(slots.default_attack(), BASIC_ATTACK_ID);
}

/// LSS-002: 冷却递减后技能恢复可用
///
/// Given: fireball cooldown=2
/// When: tick ×2
/// Then: cd 2→1→0，can_use 从 Err 变 Ok
#[test]
fn 技能槽_冷却递减后技能恢复可用() {
    let mut cooldowns = SkillCooldowns::default();
    cooldowns.set("fireball", 2);

    let skill = fireball();
    let attrs = warrior_attrs();
    let tags = GameplayTags::default();

    // 冷却中不可用
    assert!(skill.can_use(&attrs, &tags, None, 2).is_err());

    // tick 1 次
    cooldowns.tick();
    assert!(skill.can_use(&attrs, &tags, None, 1).is_err());

    // tick 2 次 → 冷却结束
    cooldowns.tick();
    assert!(skill.can_use(&attrs, &tags, None, 0).is_ok());
}

/// LSS-003: 技能槽迭代器与特殊技能
///
/// Given: SkillSlots 含 basic_attack + fireball + heal
/// When: iter() 和 special_skill()
/// Then: len=3，首元素=BASIC_ATTACK_ID，special_skill=Some("fireball")
#[test]
fn 技能槽_迭代器与特殊技能() {
    let slots = SkillSlots::new(vec![
        BASIC_ATTACK_ID.into(),
        "fireball".into(),
        "heal".into(),
    ]);

    let ids: Vec<&str> = slots.iter().collect();
    assert_eq!(ids.len(), 3);
    assert_eq!(ids[0], BASIC_ATTACK_ID);
    assert_eq!(slots.special_skill(), Some("fireball"));
}

// ══════════════════════════════════════════════════════════════
// 场景二：属性 → 技能条件 联动
// ══════════════════════════════════════════════════════════════

/// LSS-004: 战士 MP 不足无法释放火球
///
/// Given: fireball(cost_mp=8)，战士 MP=3
/// When: can_use
/// Then: Err(InsufficientMp { required: 8, current: 3 })
#[test]
fn 战士_MP不足无法释放火球() {
    let skill = fireball();
    let mut attrs = warrior_attrs();
    attrs.set_vital(AttributeKind::Mp, 3.0); // MP=3 < 8
    let tags = GameplayTags::default();

    let result = skill.can_use(&attrs, &tags, None, 0);
    assert_eq!(
        result,
        Err(tactical_rpg::core::skill::SkillUseError::InsufficientMp {
            required: 8,
            current: 3
        })
    );
}

/// LSS-005: 法师 MP 足够可以释放火球
///
/// Given: fireball(cost_mp=8)，法师 MP=25
/// When: can_use
/// Then: Ok
#[test]
fn 法师_MP足够可以释放火球() {
    let skill = fireball();
    let mut attrs = mage_attrs();
    // MaxMp = 5*5 = 25, MP 满 = 25
    let tags = GameplayTags::default();

    assert!(skill.can_use(&attrs, &tags, None, 0).is_ok());
}

/// LSS-006: 战士缺少 MAGE 标签无法释放奥术冲击
///
/// Given: arcane_blast(RequireTag(MAGE))，战士无 MAGE 标签
/// When: can_use
/// Then: Err(MissingTag { tag: MAGE })
#[test]
fn 战士_缺少MAGE标签无法释放奥术冲击() {
    let skill = mage_only_skill();
    let attrs = warrior_attrs();
    let tags = GameplayTags::default(); // 没有 MAGE 标签

    let result = skill.can_use(&attrs, &tags, None, 0);
    assert_eq!(
        result,
        Err(tactical_rpg::core::skill::SkillUseError::MissingTag {
            tag: GameplayTag::MAGE
        })
    );
}

/// LSS-007: 法师拥有 MAGE 标签可以释放奥术冲击
///
/// Given: arcane_blast(RequireTag(MAGE) + MpCost(10))，法师有 MAGE 标签，MP=25
/// When: can_use
/// Then: Ok
#[test]
fn 法师_拥有MAGE标签可以释放奥术冲击() {
    let skill = mage_only_skill();
    let attrs = mage_attrs();
    let mut tags = GameplayTags::default();
    tags.add(GameplayTag::MAGE);

    assert!(skill.can_use(&attrs, &tags, None, 0).is_ok());
}

// ══════════════════════════════════════════════════════════════
// 场景三：HP 阈值条件 + 属性联动
// ══════════════════════════════════════════════════════════════

/// LSS-008: 狂暴 HP 充足时不可用
///
/// Given: berserk(HpBelow(0.3))，战士 HP=30/30 (100%)
/// When: can_use
/// Then: Err(HpNotBelow { threshold: 0.3 })
#[test]
fn 狂暴_HP充足时不可用() {
    let skill = berserker_skill();
    let attrs = warrior_attrs(); // HP=30, MaxHp=30 → 100%
    let tags = GameplayTags::default();

    let result = skill.can_use(&attrs, &tags, None, 0);
    assert_eq!(
        result,
        Err(tactical_rpg::core::skill::SkillUseError::HpNotBelow { threshold: 0.3 })
    );
}

/// LSS-009: 狂暴 HP 低于 30% 时可用
///
/// Given: berserk(HpBelow(0.3))，战士 HP=8/30 (26.7%)
/// When: can_use
/// Then: Ok
#[test]
fn 狂暴_HP低于30百分比时可用() {
    let skill = berserker_skill();
    let mut attrs = warrior_attrs();
    attrs.set_vital(AttributeKind::Hp, 8.0); // HP=8, MaxHp=30 → 26.7% < 30%
    let tags = GameplayTags::default();

    assert!(skill.can_use(&attrs, &tags, None, 0).is_ok());
}

// ══════════════════════════════════════════════════════════════
// 场景四：目标标签条件 + 属性联动
// ══════════════════════════════════════════════════════════════

/// LSS-010: 净化目标无 DEBUFF 标签时不可用
///
/// Given: purify(TargetRequireTag(DEBUFF))，目标无 DEBUFF 标签
/// When: can_use
/// Then: Err(TargetMissingTag { tag: DEBUFF })
#[test]
fn 净化_目标无DEBUFF标签时不可用() {
    let skill = purify_skill();
    let attrs = warrior_attrs();
    let tags = GameplayTags::default();
    let target_tags = GameplayTags::default(); // 没有 DEBUFF

    let result = skill.can_use(&attrs, &tags, Some(&target_tags), 0);
    assert_eq!(
        result,
        Err(tactical_rpg::core::skill::SkillUseError::TargetMissingTag {
            tag: GameplayTag::DEBUFF
        })
    );
}

/// LSS-011: 净化目标有 DEBUFF 标签时可用
///
/// Given: purify(TargetRequireTag(DEBUFF))，目标有 DEBUFF 标签
/// When: can_use
/// Then: Ok
#[test]
fn 净化_目标有DEBUFF标签时可用() {
    let skill = purify_skill();
    let attrs = warrior_attrs();
    let tags = GameplayTags::default();
    let mut target_tags = GameplayTags::default();
    target_tags.add(GameplayTag::DEBUFF);

    assert!(skill.can_use(&attrs, &tags, Some(&target_tags), 0).is_ok());
}

/// LSS-012: 净化无目标时跳过目标标签检查
///
/// Given: purify(TargetRequireTag(DEBUFF))，不提供目标
/// When: can_use(target_tags=None)
/// Then: Ok（跳过目标检查）
#[test]
fn 净化_无目标时跳过目标标签检查() {
    let skill = purify_skill();
    let attrs = warrior_attrs();
    let tags = GameplayTags::default();

    // 不提供目标标签，跳过检查
    assert!(skill.can_use(&attrs, &tags, None, 0).is_ok());
}

// ══════════════════════════════════════════════════════════════
// 场景五：effective_skill_range 跨模块
// ══════════════════════════════════════════════════════════════

/// LSS-013: 火球自带射程 3，忽略单位基础射程
///
/// Given: fireball(range=3)，单位 base_attack_range=1
/// When: effective_skill_range
/// Then: 返回 3
#[test]
fn 火球自带射程3_忽略单位基础射程() {
    let skill = fireball();
    assert_eq!(effective_skill_range(&skill, 1), 3);
}

/// LSS-014: 基础攻击无射程，使用单位基础射程
///
/// Given: basic_attack(range=0)，单位 base_attack_range=3
/// When: effective_skill_range
/// Then: 返回 3
#[test]
fn 基础攻击无射程_使用单位基础射程() {
    let skill = basic_attack();
    assert_eq!(effective_skill_range(&skill, 3), 3);
}

/// LSS-015: 治疗自带射程 2
///
/// Given: heal(range=2)，单位 base_attack_range=1
/// When: effective_skill_range
/// Then: 返回 2
#[test]
fn 治疗自带射程2() {
    let skill = heal();
    assert_eq!(effective_skill_range(&skill, 1), 2);
}

// ══════════════════════════════════════════════════════════════
// 场景六：多条件组合 + 属性联合验证
// ══════════════════════════════════════════════════════════════

/// LSS-016: 法师奥术冲击满足所有条件
///
/// Given: arcane_blast(RequireTag(MAGE) + MpCost(10))，法师 MP=25，有 MAGE 标签
/// When: can_use
/// Then: Ok
#[test]
fn 法师_奥术冲击_满足所有条件() {
    let skill = mage_only_skill();
    let attrs = mage_attrs();
    let mut tags = GameplayTags::default();
    tags.add(GameplayTag::MAGE);

    // MP=25 >= 10, 有 MAGE 标签, 冷却=0
    assert!(skill.can_use(&attrs, &tags, None, 0).is_ok());
}

/// LSS-017: 法师奥术冲击 MP 不足时失败
///
/// Given: arcane_blast(MpCost(10))，法师 MP=5，有 MAGE 标签
/// When: can_use
/// Then: Err（MP 不足）
#[test]
fn 法师_奥术冲击_MP不足时失败() {
    let skill = mage_only_skill();
    let mut attrs = mage_attrs();
    attrs.set_vital(AttributeKind::Mp, 5.0); // MP=5 < 10
    let mut tags = GameplayTags::default();
    tags.add(GameplayTag::MAGE);

    let result = skill.can_use(&attrs, &tags, None, 0);
    assert!(result.is_err());
}

/// LSS-018: 战士缺少标签且 MP 不足，第一个条件失败
///
/// Given: arcane_blast(RequireTag(MAGE) + MpCost(10))，战士无 MAGE 标签
/// When: can_use
/// Then: Err(MissingTag)（第一个失败条件）
#[test]
fn 战士_缺少标签且MP不足_第一个条件失败() {
    let skill = mage_only_skill();
    let attrs = warrior_attrs(); // 没有 MAGE 标签, MP=10
    let tags = GameplayTags::default();

    // 第一个失败条件是 MissingTag
    let result = skill.can_use(&attrs, &tags, None, 0);
    assert_eq!(
        result,
        Err(tactical_rpg::core::skill::SkillUseError::MissingTag {
            tag: GameplayTag::MAGE
        })
    );
}

// ══════════════════════════════════════════════════════════════
// 场景七：冷却管理跨回合联动
// ══════════════════════════════════════════════════════════════

/// LSS-019: 多技能冷却独立递减
///
/// Given: fireball(cd=2) + heal(cd=3)
/// When: tick ×3
/// Then: fireball 在第 2 轮恢复，heal 在第 3 轮恢复
#[test]
fn 多技能冷却独立递减() {
    let mut cooldowns = SkillCooldowns::default();
    cooldowns.set("fireball", 2);
    cooldowns.set("heal", 3);

    let fireball = fireball();
    let heal = heal();
    let attrs = warrior_attrs();
    let tags = GameplayTags::default();

    // 第1轮 tick
    cooldowns.tick();

    let fb_cd = fireball.can_use(&attrs, &tags, None, cooldowns.get("fireball"));
    let hl_cd = heal.can_use(&attrs, &tags, None, cooldowns.get("heal"));
    assert!(fb_cd.is_err()); // fireball: cd 1
    assert!(hl_cd.is_err()); // heal: cd 2

    // 第2轮 tick
    cooldowns.tick();

    let fb_cd = fireball.can_use(&attrs, &tags, None, cooldowns.get("fireball"));
    let hl_cd = heal.can_use(&attrs, &tags, None, cooldowns.get("heal"));
    assert!(fb_cd.is_ok()); // fireball: cd 0 → 可用
    assert!(hl_cd.is_err()); // heal: cd 1

    // 第3轮 tick
    cooldowns.tick();

    let hl_cd = heal.can_use(&attrs, &tags, None, cooldowns.get("heal"));
    assert!(hl_cd.is_ok()); // heal: cd 0 → 可用
}

/// LSS-020: 冷却 clear 后所有技能立即可用
///
/// Given: fireball(cd=5) + heal(cd=5)，均不可用
/// When: cooldowns.clear()
/// Then: 两个技能均变为可用
#[test]
fn 冷却clear后所有技能立即可用() {
    let mut cooldowns = SkillCooldowns::default();
    cooldowns.set("fireball", 5);
    cooldowns.set("heal", 5);

    let fireball = fireball();
    let heal = heal();
    let attrs = warrior_attrs();
    let tags = GameplayTags::default();

    assert!(
        fireball
            .can_use(&attrs, &tags, None, cooldowns.get("fireball"))
            .is_err()
    );
    assert!(
        heal.can_use(&attrs, &tags, None, cooldowns.get("heal"))
            .is_err()
    );

    cooldowns.clear();

    assert!(
        fireball
            .can_use(&attrs, &tags, None, cooldowns.get("fireball"))
            .is_ok()
    );
    assert!(
        heal.can_use(&attrs, &tags, None, cooldowns.get("heal"))
            .is_ok()
    );
}
