use super::events::{
    CharacterDied, DamageApplied, DotApplied, HealApplied, HotApplied, StunApplied,
};
use super::log::CombatLogPlugin;
use super::pipeline::CombatEventPlugin;
use super::record::{
    BattleEntry, BattleRecord, DamageBreakdown, EntityBattleStats, ModifierEntry,
    record_character_died, record_damage, record_dot, record_heal, record_hot, record_stun,
    record_turn_ended, record_turn_started,
};
use crate::core::effect::{
    EffectDef, EffectQueue, EffectResult, EffectResultData, PendingEffect, PendingEffectData,
};
use bevy::prelude::*;

/// 战斗插件（组合 Effect Pipeline + CombatLog + BattleRecord 子插件）
pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BattleRecord>()
            // 注册 Reflect 类型
            .register_type::<EffectDef>()
            .register_type::<PendingEffectData>()
            .register_type::<PendingEffect>()
            .register_type::<EffectResultData>()
            .register_type::<EffectResult>()
            .register_type::<EffectQueue>()
            .register_type::<BattleEntry>()
            .register_type::<DamageBreakdown>()
            .register_type::<ModifierEntry>()
            .register_type::<EntityBattleStats>()
            .register_type::<BattleRecord>()
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
