//! Unit tests — CombatParticipant 与 mark_unit_dead API
//!
//! 验证 CombatParticipant 组件的生命周期：
//! - 创建时存活
//! - mark_unit_dead 正确标记
//! - 不可逆（死后不能复活）

use crate::core::domains::combat::integration::turn::mark_unit_dead;
use crate::core::domains::combat::components::{CombatParticipant, TeamId};
use crate::core::domains::combat::tests::fixtures::combat_fixtures;

#[test]
fn participant_starts_alive() {
    let p = CombatParticipant::alive(combat_fixtures::player_team());
    assert!(p.is_alive, "participant must start alive");
    assert_eq!(p.team_id, combat_fixtures::player_team());
}

#[test]
fn participant_team_is_preserved() {
    let p = combat_fixtures::alive_participant(combat_fixtures::enemy_team());
    assert_eq!(p.team_id.as_str(), "enemy");
}

#[test]
fn mark_unit_dead_changes_is_alive() {
    let mut p = combat_fixtures::alive_participant(combat_fixtures::player_team());
    mark_unit_dead(&mut p);
    assert!(!p.is_alive, "participant must be dead after mark_unit_dead");
}

#[test]
fn mark_unit_dead_is_idempotent() {
    let mut p = combat_fixtures::alive_participant(combat_fixtures::player_team());
    mark_unit_dead(&mut p);
    mark_unit_dead(&mut p); // second call should not panic
    assert!(!p.is_alive, "still dead after second call");
}

#[test]
fn dead_participant_start_dead() {
    let p = combat_fixtures::dead_participant(combat_fixtures::enemy_team());
    assert!(!p.is_alive, "dead_participant should start dead");
}

#[test]
fn participant_team_id_matches_expected() {
    let player = combat_fixtures::alive_participant(combat_fixtures::player_team());
    let enemy = combat_fixtures::alive_participant(combat_fixtures::enemy_team());
    assert_ne!(player.team_id, enemy.team_id, "teams must differ");
}
