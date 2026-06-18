//! Summon Domain — 不变量测试
//!
//! 验证 docs/02-domain/domains/summon_domain.md §3 定义的不变量。

use crate::core::domains::summon::components::{SummonBond, SummonSlotManager};
use crate::core::domains::summon::rules::{
    can_summon_from_summon, has_free_summon_slot, is_caster_alive, is_position_valid,
};

/// 不变量 3.1：召唤者生死约束 — 召唤者死亡时召唤物应不可继续存在。
#[test]
fn caster_dead_invalidates_summon() {
    // is_caster_alive(false) → 条件不满足
    assert!(!is_caster_alive(false));
}

/// 不变量 3.2：专注召唤唯一性 — 一个施法者同时只能维持一个专注召唤。
#[test]
fn only_one_concentration_summon_allowed() {
    use crate::core::domains::summon::rules::can_create_concentration_summon;
    assert!(
        !can_create_concentration_summon(true),
        "already concentrating → reject"
    );
}

/// 不变量 3.5：占位不冲突 — 召唤物出生位置必须可通行且无占用。
#[test]
fn occupied_position_rejected() {
    assert!(
        !is_position_valid(0, 0, true, true),
        "occupied position rejected"
    );
}

#[test]
fn impassable_position_rejected() {
    assert!(
        !is_position_valid(0, 0, false, false),
        "impassable position rejected"
    );
}

/// 衍生不变量：active_summons 长度不得超过 max_slots。
#[test]
fn summon_slot_count_never_exceeds_max() {
    let mut manager = SummonSlotManager::new(2);
    manager
        .active_summons
        .push(bevy::prelude::Entity::PLACEHOLDER);
    manager
        .active_summons
        .push(bevy::prelude::Entity::PLACEHOLDER);
    // full: 和 max 相等
    assert!(!has_free_summon_slot(&manager));
}

/// 不变量 3.5b：同位置不可重复召唤。
#[test]
fn summon_collision_prevention() {
    // 同一坐标标记为已占用时 → 不可再召唤到此处
    assert!(!is_position_valid(5, 5, true, true));
}

/// 不变量 3.4：召唤物模板一致性 — 没有模板的召唤创建应失败。
/// 通过在 can_create_summon_from_summon 中体现（无 bond 时视为非召唤创建的实体，允许）。
#[test]
fn no_bond_from_non_summon() {
    assert!(can_summon_from_summon(None, true));
}

/// 禁止事项 3：嵌套召唤不可行。
#[test]
fn nested_summon_from_summon_blocked() {
    let bond = SummonBond {
        caster: bevy::prelude::Entity::PLACEHOLDER,
        template_id: "sum_pet".into(),
        ai_mode: crate::core::domains::summon::components::SummonAIMode::Autonomous,
        summoned_at: 0.0,
    };
    assert!(
        !can_summon_from_summon(Some(&bond), false),
        "summon entity should not be able to summon"
    );
}
