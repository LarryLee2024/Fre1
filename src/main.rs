mod action_menu;
mod ai;
mod assets;
mod camera;
mod combat;
mod combat_event;
mod combat_log;
mod input;
mod map;
mod pathfinding;
mod status;
mod tile_info;
mod turn;
mod ui;
mod unit;
mod vfx;

use action_menu::ActionMenuPlugin;
use ai::AiPlugin;
use assets::AssetsPlugin;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use camera::CameraPlugin;
use combat_event::CombatEventPlugin;
use combat_log::CombatLogPlugin;
use input::InputPlugin;
use map::MapPlugin;
use status::StatusPlugin;
use tile_info::TileInfoPlugin;
use turn::{AppState, TurnPlugin};
use ui::UiPlugin;
use unit::UnitPlugin;
use vfx::VfxPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "回合制战棋".to_string(),
                resolution: (1024u32, 768u32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            EguiPlugin::default(),
            WorldInspectorPlugin::new(),
        ))
        .add_plugins((
            // 游戏插件
            AssetsPlugin,
            TurnPlugin,
            CameraPlugin,
            MapPlugin,
            UnitPlugin,
            UiPlugin,
            CombatEventPlugin,
            CombatLogPlugin,
            InputPlugin,
            ActionMenuPlugin,
            TileInfoPlugin,
            AiPlugin,
            VfxPlugin,
            StatusPlugin,
        ))
        // 直接进入游戏（后续可加主菜单）
        .add_systems(Startup, |mut next: ResMut<NextState<AppState>>| {
            next.set(AppState::InGame);
        })
        .run();
}
