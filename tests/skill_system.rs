//! P1 集成测试：技能系统跨模块联动
//!
//! 跨 skill + core/attribute + core/tag 测试技能条件检查、
//! 技能槽管理、冷却追踪在真实属性数据下的行为。

use tactical_rpg::core::attribute::{AttributeKind, Attributes};
use tactical_rpg::core::tag::{GameplayTag, GameplayTags};
use tactical_rpg::skill::{
    BASIC_ATTACK_ID, SkillCooldowns, SkillData, SkillSlots, SkillTargeting, effective_skill_range,
};

// ── 测试辅助 ──

/// 战士模板：Might=5, Vitality=5 → Attack=10, Defense=5, MaxHp=30, MaxMp=10
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

/// 法师模板：Intelligence=5 → MaxMp=25, MagicAttack=10
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
        conditions: vec![],
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
        conditions: vec![tactical_rpg::skill::SkillCondition::RequireTag(
            GameplayTag::MAGE,
        )],
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
        conditions: vec![tactical_rpg::skill::SkillCondition::HpBelow(0.3)],
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
        conditions: vec![tactical_rpg::skill::SkillCondition::TargetRequireTag(
            GameplayTag::DEBUFF,
        )],
        cooldown: 0,
        priority: 25,
    }
}

// ══════════════════════════════════════════════════════════════
// 场景一：SkillSlots + SkillCooldowns 联动
// ══════════════════════════════════════════════════════════════

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

#[test]
fn 战士_MP不足无法释放火球() {
    let skill = fireball();
    let mut attrs = warrior_attrs();
    attrs.set_base(AttributeKind::Mp, 3.0); // MP=3 < 8
    let tags = GameplayTags::default();

    let result = skill.can_use(&attrs, &tags, None, 0);
    assert_eq!(
        result,
        Err(tactical_rpg::skill::SkillUseError::InsufficientMp {
            required: 8,
            current: 3
        })
    );
}

#[test]
fn 法师_MP足够可以释放火球() {
    let skill = fireball();
    let mut attrs = mage_attrs();
    // MaxMp = 5*5 = 25, MP 满 = 25
    let tags = GameplayTags::default();

    assert!(skill.can_use(&attrs, &tags, None, 0).is_ok());
}

#[test]
fn 战士_缺少MAGE标签无法释放奥术冲击() {
    let skill = mage_only_skill();
    let attrs = warrior_attrs();
    let tags = GameplayTags::default(); // 没有 MAGE 标签

    let result = skill.can_use(&attrs, &tags, None, 0);
    assert_eq!(
        result,
        Err(tactical_rpg::skill::SkillUseError::MissingTag {
            tag: GameplayTag::MAGE
        })
    );
}

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

#[test]
fn 狂暴_HP充足时不可用() {
    let skill = berserker_skill();
    let attrs = warrior_attrs(); // HP=30, MaxHp=30 → 100%
    let tags = GameplayTags::default();

    let result = skill.can_use(&attrs, &tags, None, 0);
    assert_eq!(
        result,
        Err(tactical_rpg::skill::SkillUseError::HpNotBelow { threshold: 0.3 })
    );
}

#[test]
fn 狂暴_HP低于30百分比时可用() {
    let skill = berserker_skill();
    let mut attrs = warrior_attrs();
    attrs.set_base(AttributeKind::Hp, 8.0); // HP=8, MaxHp=30 → 26.7% < 30%
    let tags = GameplayTags::default();

    assert!(skill.can_use(&attrs, &tags, None, 0).is_ok());
}

// ══════════════════════════════════════════════════════════════
// 场景四：目标标签条件 + 属性联动
// ══════════════════════════════════════════════════════════════

#[test]
fn 净化_目标无DEBUFF标签时不可用() {
    let skill = purify_skill();
    let attrs = warrior_attrs();
    let tags = GameplayTags::default();
    let target_tags = GameplayTags::default(); // 没有 DEBUFF

    let result = skill.can_use(&attrs, &tags, Some(&target_tags), 0);
    assert_eq!(
        result,
        Err(tactical_rpg::skill::SkillUseError::TargetMissingTag {
            tag: GameplayTag::DEBUFF
        })
    );
}

#[test]
fn 净化_目标有DEBUFF标签时可用() {
    let skill = purify_skill();
    let attrs = warrior_attrs();
    let tags = GameplayTags::default();
    let mut target_tags = GameplayTags::default();
    target_tags.add(GameplayTag::DEBUFF);

    assert!(skill.can_use(&attrs, &tags, Some(&target_tags), 0).is_ok());
}

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

#[test]
fn 火球自带射程3_忽略单位基础射程() {
    let skill = fireball();
    assert_eq!(effective_skill_range(&skill, 1), 3);
}

#[test]
fn 基础攻击无射程_使用单位基础射程() {
    let skill = basic_attack();
    assert_eq!(effective_skill_range(&skill, 3), 3);
}

#[test]
fn 治疗自带射程2() {
    let skill = heal();
    assert_eq!(effective_skill_range(&skill, 1), 2);
}

// ══════════════════════════════════════════════════════════════
// 场景六：多条件组合 + 属性联合验证
// ══════════════════════════════════════════════════════════════

#[test]
fn 法师_奥术冲击_满足所有条件() {
    let skill = mage_only_skill();
    let attrs = mage_attrs();
    let mut tags = GameplayTags::default();
    tags.add(GameplayTag::MAGE);

    // MP=25 >= 10, 有 MAGE 标签, 冷却=0
    assert!(skill.can_use(&attrs, &tags, None, 0).is_ok());
}

#[test]
fn 法师_奥术冲击_MP不足时失败() {
    let skill = mage_only_skill();
    let mut attrs = mage_attrs();
    attrs.set_base(AttributeKind::Mp, 5.0); // MP=5 < 10
    let mut tags = GameplayTags::default();
    tags.add(GameplayTag::MAGE);

    let result = skill.can_use(&attrs, &tags, None, 0);
    assert!(result.is_err());
}

#[test]
fn 战士_缺少标签且MP不足_第一个条件失败() {
    let skill = mage_only_skill();
    let attrs = warrior_attrs(); // 没有 MAGE 标签, MP=10
    let tags = GameplayTags::default();

    // 第一个失败条件是 MissingTag
    let result = skill.can_use(&attrs, &tags, None, 0);
    assert_eq!(
        result,
        Err(tactical_rpg::skill::SkillUseError::MissingTag {
            tag: GameplayTag::MAGE
        })
    );
}

// ══════════════════════════════════════════════════════════════
// 场景七：冷却管理跨回合联动
// ══════════════════════════════════════════════════════════════

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
