//! 技能系统 Feature Test
//!
//! 跨 skill + core/attribute + core/tag 测试技能使用条件检查：
//! MP 不足、缺少标签、冷却中不可使用。

use tactical_rpg::core::attribute::{AttributeKind, Attributes};
use tactical_rpg::core::tag::{GameplayTag, GameplayTags};
use tactical_rpg::skill::{SkillCondition, SkillData, SkillTargeting, SkillUseError};

use crate::common::fixtures::UnitBuilder;

// ── 测试辅助 ──

/// 构建 MP 不足的战士：MP=3
fn low_mp_warrior_attrs() -> Attributes {
    let mut attrs = UnitBuilder::warrior().attrs().clone();
    attrs.set_vital(AttributeKind::Mp, 3.0);
    attrs
}

/// 构建需要 MAGE 标签的技能
fn mage_only_skill() -> SkillData {
    SkillData {
        id: "arcane_blast".into(),
        name: "奥术冲击".into(),
        description: String::new(),
        cost_mp: 0,
        range: 2,
        targeting: SkillTargeting::SingleEnemy,
        effects: vec![],
        tags: vec![],
        conditions: vec![SkillCondition::RequireTag(GameplayTag::MAGE)],
        cooldown: 0,
        priority: 20,
    }
}

/// 构建 MP 消耗技能
fn expensive_skill() -> SkillData {
    SkillData {
        id: "fireball".into(),
        name: "火球".into(),
        description: String::new(),
        cost_mp: 10,
        range: 3,
        targeting: SkillTargeting::SingleEnemy,
        effects: vec![],
        tags: vec![],
        conditions: vec![SkillCondition::MpCost(10)],
        cooldown: 0,
        priority: 10,
    }
}

/// 构建有冷却的技能
fn cooldown_skill() -> SkillData {
    SkillData {
        id: "thunder".into(),
        name: "雷击".into(),
        description: String::new(),
        cost_mp: 0,
        range: 3,
        targeting: SkillTargeting::SingleEnemy,
        effects: vec![],
        tags: vec![],
        conditions: vec![],
        cooldown: 3,
        priority: 15,
    }
}

// ══════════════════════════════════════════════════════════════
// 场景一：MP 不足不可使用
// ══════════════════════════════════════════════════════════════

#[test]
fn 技能使用条件检查_mp不足不可使用() {
    let skill = expensive_skill();
    let attrs = low_mp_warrior_attrs(); // MP=3 < 10
    let tags = GameplayTags::default();

    let result = skill.can_use(&attrs, &tags, None, 0);
    assert_eq!(
        result,
        Err(SkillUseError::InsufficientMp {
            required: 10,
            current: 3
        })
    );
}

// ══════════════════════════════════════════════════════════════
// 场景二：缺少标签不可使用
// ══════════════════════════════════════════════════════════════

#[test]
fn 技能使用条件检查_缺少标签不可使用() {
    let skill = mage_only_skill();
    let attrs = UnitBuilder::warrior().attrs().clone();
    let tags = GameplayTags::default(); // 战士没有 MAGE 标签

    let result = skill.can_use(&attrs, &tags, None, 0);
    assert_eq!(
        result,
        Err(SkillUseError::MissingTag {
            tag: GameplayTag::MAGE
        })
    );
}

// ══════════════════════════════════════════════════════════════
// 场景三：冷却中不可使用
// ══════════════════════════════════════════════════════════════

#[test]
fn 技能冷却检查_冷却中不可使用() {
    let skill = cooldown_skill();
    let attrs = UnitBuilder::warrior().attrs().clone();
    let tags = GameplayTags::default();

    // 技能刚使用过，剩余冷却 2 回合
    let result = skill.can_use(&attrs, &tags, None, 2);
    assert_eq!(result, Err(SkillUseError::OnCooldown { remaining: 2 }));
}
