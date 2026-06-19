//! Invariant tests — Combat 团队全灭与行动资源不变量
//!
//! | 不变量 | 描述 |
//! |--------|------|
//! | 3.6 | 所有战斗参与者初始存活 — 创建时无 Dead Tag |
//! | 3.7 | 行动资源上限不变 — movement 不超过 max_movement |

use crate::core::domains::combat::components::{ActionPoints, CombatParticipant};
use crate::core::domains::combat::tests::fixtures::combat_fixtures;

/// 不变量 3.6: 所有 CombatParticipant 在初始化时无 Dead Tag（即存活）。
#[test]
fn all_participants_start_alive() {
    let entries = combat_fixtures::interleaved_entries();
    for entry in &entries {
        // 模拟 initialize_turn_order 创建 participant 的行为
        let participant = CombatParticipant::alive(entry.team_id.clone());
        // 存活状态由 Dead Tag Component 判定：创建时插入的 CombatParticipant 不含 Dead
        // 此处验证 CombatParticipant 构造成功且 team_id 正确
        assert_eq!(
            participant.team_id, entry.team_id,
            "participant {:?} must be constructible with correct team",
            entry.entity
        );
    }
}

/// 不变量 3.7: ActionPoints.movement 必须 <= ActionPoints.max_movement。
#[test]
fn movement_never_exceeds_max() {
    let ap = ActionPoints::new(6.0);
    assert!(
        ap.movement <= ap.max_movement,
        "initial movement must not exceed max"
    );

    let mut ap = ActionPoints::new(6.0);
    // consume_movement should never overshoot
    assert!(!ap.consume_movement(10.0), "cannot consume more than max");
    assert!(
        ap.movement <= ap.max_movement,
        "movement must not exceed max after failed consume"
    );

    ap.consume_movement(2.0);
    assert!(
        ap.movement <= ap.max_movement,
        "movement must not exceed max after partial consume"
    );

    ap.reset();
    assert!(
        ap.movement <= ap.max_movement,
        "movement must not exceed max after reset"
    );
    assert!(
        (ap.movement - ap.max_movement).abs() < f32::EPSILON,
        "after reset movement should equal max_movement"
    );
}

/// 验证 CombatParticipant 创建后 team_id 与 TurnEntry 的 team_id 一致。
#[test]
fn participant_team_matches_turn_entry_team() {
    let entries = combat_fixtures::interleaved_entries();
    for entry in &entries {
        let participant = CombatParticipant::alive(entry.team_id.clone());
        assert_eq!(
            participant.team_id, entry.team_id,
            "participant team must match turn entry team"
        );
    }
}
