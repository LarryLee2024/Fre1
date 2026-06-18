//! Reaction Domain — 不变量测试
//!
//! 验证 docs/02-domain/domains/reaction_domain.md §3 定义的不变量。

use crate::core::domains::reaction::components::{ReactionState, ReactionType};
use crate::core::domains::reaction::rules::{
    calc_priority, can_opportunity_attack, can_react, can_trigger_on_turn, is_adjacent,
    resolve_counterspell,
};

/// 不变量 3.1：每回合反应次数上限 — 使用后不可再次反应（无额外反应能力时）。
#[test]
fn reaction_used_once_per_turn() {
    let mut state = ReactionState::new();
    assert!(can_react(&state));
    state.used = true;
    assert!(!can_react(&state), "used reaction blocks further reactions");
}

/// 不变量 3.1b：额外反应能力仍然受次数限制。
#[test]
fn extra_reactions_also_limited() {
    let mut state = ReactionState::new();
    state.used = true;
    state.extra_reactions = 1;
    assert!(can_react(&state), "extra reaction available");
    state.extra_used = 1;
    assert!(!can_react(&state), "extra reaction exhausted");
}

/// 不变量 3.2：反应默认在回合外触发。
#[test]
fn offensive_reactions_blocked_on_own_turn() {
    assert!(!can_trigger_on_turn(true, &ReactionType::OpportunityAttack));
    assert!(!can_trigger_on_turn(true, &ReactionType::Counterspell));
}

/// 不变量 3.3：机会攻击只在主动离开时触发。
#[test]
fn forced_movement_no_opportunity_attack() {
    use crate::core::domains::reaction::components::ReactionTrigger;
    let trigger = ReactionTrigger::LeaveThreatRange {
        mover: bevy::prelude::Entity::PLACEHOLDER,
        to_x: 5,
        to_y: 5,
    };
    assert!(
        !can_opportunity_attack(&trigger, true),
        "forced move should not trigger OA"
    );
}

/// 不变量 3.4：法术反制环阶匹配 — 低环反制高环需要检定。
#[test]
fn counterspell_lower_level_requires_check() {
    let result = resolve_counterspell(5, 3); // target 5, counter 3
    match result {
        crate::core::domains::reaction::components::CounterspellVerdict::CheckRequired {
            dc,
            ..
        } => {
            assert!(dc > 10, "lower level counter needs DC > 10");
        }
        _ => panic!("lower level counter must require ability check"),
    }
}

/// 不变量 3.4b：同环或高环反制自动成功。
#[test]
fn counterspell_equal_or_higher_auto_success() {
    let equal = resolve_counterspell(3, 3);
    assert_eq!(
        equal,
        crate::core::domains::reaction::components::CounterspellVerdict::AutoSuccess
    );
    let higher = resolve_counterspell(3, 5);
    assert_eq!(
        higher,
        crate::core::domains::reaction::components::CounterspellVerdict::AutoSuccess
    );
}

/// 不变量 3.5：援护距离限制 — 只在相邻格。
#[test]
fn guardian_must_be_adjacent() {
    assert!(is_adjacent(5, 5, 5, 6), "orthogonal adjacent");
    assert!(is_adjacent(5, 5, 6, 6), "diagonal adjacent");
    assert!(!is_adjacent(5, 5, 7, 5), "two steps away not adjacent");
}

/// 衍生不变量：防御型反应 > 进攻型反应优先级。
#[test]
fn defense_over_offense_priority() {
    let shield = calc_priority(&ReactionType::Shield, 0, 0);
    let oa = calc_priority(&ReactionType::OpportunityAttack, 30, 0);
    assert!(
        shield > oa,
        "shield (defense) must outprioritize OA (offense)"
    );
}
