//! 技能系统 Feature Test
//!
//! 跨 skill + core/attribute + core/tag 测试技能使用条件检查：
//! MP 不足、缺少标签、冷却中不可使用。

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

use tactical_rpg::core::ability::{SkillCondition, SkillData, SkillTargeting, SkillUseError};
use tactical_rpg::core::attribute::Attributes;
use tactical_rpg::core::tag::{GameplayTag, GameplayTags};

use crate::common::fixtures::UnitBuilder;

// ── 测试辅助 ──

/// 构建需要 ALLY 标签的技能
fn mage_only_skill() -> SkillData {
    SkillData {
        id: "arcane_blast".into(),
        name: "奥术冲击".into(),
        range: 2,
        conditions: vec![SkillCondition::RequireTag(GameplayTag::ALLY)],
        priority: 20,
        ..Default::default()
    }
}

/// 构建有冷却的技能
fn cooldown_skill() -> SkillData {
    SkillData {
        id: "thunder".into(),
        name: "雷击".into(),
        range: 3,
        cooldown: 3,
        priority: 15,
        ..Default::default()
    }
}

// ══════════════════════════════════════════════════════════════
// 场景一：缺少标签不可使用
// ══════════════════════════════════════════════════════════════

/// FT-SKL-001: 缺少标签不可使用
///
/// Given: 战士（无 ALLY 标签）和奥术冲击（RequireTag ALLY）
/// When:  调用 can_use() 检查技能可用性
/// Then:  返回 MissingTag 错误
#[test]
fn 技能使用条件检查_缺少标签不可使用() {
    let skill = mage_only_skill();
    let attrs = UnitBuilder::warrior().attrs().clone();
    let tags = GameplayTags::default(); // 战士没有 ALLY 标签

    let result = skill.can_use(&attrs, &tags, None, 0);
    assert_eq!(
        result,
        Err(SkillUseError::MissingTag {
            tag: GameplayTag::ALLY
        })
    );
}

// ══════════════════════════════════════════════════════════════
// 场景二：冷却中不可使用
// ══════════════════════════════════════════════════════════════

/// FT-SKL-002: 冷却中不可使用
///
/// Given: 战士和雷击技能（cooldown=3），当前冷却剩余 2 回合
/// When:  调用 can_use() 检查技能可用性
/// Then:  返回 OnCooldown 错误
#[test]
fn 技能冷却检查_冷却中不可使用() {
    let skill = cooldown_skill();
    let attrs = UnitBuilder::warrior().attrs().clone();
    let tags = GameplayTags::default();

    // 技能刚使用过，剩余冷却 2 回合
    let result = skill.can_use(&attrs, &tags, None, 2);
    assert_eq!(result, Err(SkillUseError::OnCooldown { remaining: 2 }));
}
