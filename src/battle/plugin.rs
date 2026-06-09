use super::events::{
    CharacterDied, DamageApplied, DotApplied, HealApplied, HotApplied, StunApplied,
};
use super::log::CombatLogPlugin;
use super::pipeline::CombatEventPlugin;
use super::record::{
    record_character_died, record_damage, record_dot, record_heal, record_hot, record_stun,
    record_turn_ended, record_turn_started, BattleRecord,
};
use bevy::prelude::*;

/// 战斗插件（组合 Effect Pipeline + CombatLog + BattleRecord 子插件）
pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BattleRecord>()
            .add_message::<CharacterDied>()
            .add_message::<DamageApplied>()
            .add_message::<HealApplied>()
            .add_message::<StunApplied>()
            .add_message::<DotApplied>()
            .add_message::<HotApplied>()
            .add_plugins((CombatEventPlugin, CombatLogPlugin))
            .add_systems(
                Update,
                (
                    record_turn_started,
                    record_turn_ended,
                    record_damage,
                    record_heal,
                    record_dot,
                    record_hot,
                    record_stun,
                    record_character_died,
                ),
            );
    }
}
