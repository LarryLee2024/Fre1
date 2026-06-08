use super::action_menu::ActionMenuPlugin;
use super::command_handler::handle_ui_commands;
use super::events::UiCommand;
use super::hud::HudPlugin;
use super::theme::UiTheme;
use super::tile_info::TileInfoPlugin;
use super::vfx::VfxPlugin;
use crate::character::Faction;
use crate::turn::{AppState, TurnState};
use bevy::prelude::*;

/// UI 插件（组合所有 UI 子插件）
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiTheme>()
            .add_message::<UiCommand>()
            .add_plugins((HudPlugin, ActionMenuPlugin, TileInfoPlugin, VfxPlugin))
            .add_systems(
                Update,
                handle_ui_commands.run_if(in_state(AppState::InGame).and(player_turn)),
            );
    }
}

/// 只在玩家回合运行
fn player_turn(turn_state: Res<TurnState>) -> bool {
    turn_state.current_faction == Faction::Player
}
