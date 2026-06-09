use super::events::{CharacterDied, DamageApplied, DotApplied, HealApplied, HotApplied, StunApplied};
use super::log::CombatLogPlugin;
use super::pipeline::CombatEventPlugin;
use bevy::prelude::*;

/// 战斗插件（组合 Effect Pipeline + CombatLog 子插件）
pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CharacterDied>()
            .add_message::<DamageApplied>()
            .add_message::<HealApplied>()
            .add_message::<StunApplied>()
            .add_message::<DotApplied>()
            .add_message::<HotApplied>()
            .add_plugins((CombatEventPlugin, CombatLogPlugin));
    }
}
