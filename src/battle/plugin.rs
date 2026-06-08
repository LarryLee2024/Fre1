use super::event::CombatEventPlugin;
use super::log::CombatLogPlugin;
use bevy::prelude::*;

/// 战斗插件（组合 CombatEvent + CombatLog 子插件）
pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CombatEventPlugin,
            CombatLogPlugin,
        ));
    }
}
