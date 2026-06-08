use super::hud::HudPlugin;
use super::action_menu::ActionMenuPlugin;
use super::tile_info::TileInfoPlugin;
use super::vfx::VfxPlugin;
use bevy::prelude::*;

/// UI 插件（组合所有 UI 子插件）
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            HudPlugin,
            ActionMenuPlugin,
            TileInfoPlugin,
            VfxPlugin,
        ));
    }
}
