mod map;
mod unit;
mod turn;
mod combat;
mod pathfinding;
mod input;
mod ui;

use bevy::prelude::*;
use map::GameMap;
use turn::{AppState, TurnPhase};

fn main() {
    App::new()
        // 基础插件
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "回合制战棋".to_string(),
                resolution: (1024u32, 768u32).into(),
                ..default()
            }),
            ..default()
        }))
        // 资源
        .init_resource::<GameMap>()
        .init_resource::<turn::TurnState>()
        // 状态
        .init_state::<AppState>()
        // TurnPhase 作为 SubState，在 InGame 时自动激活
        .add_sub_state::<TurnPhase>()
        // 入场系统
        .add_systems(OnEnter(AppState::InGame), (
            map::spawn_map,
            unit::spawn_units,
            ui::spawn_ui,
        ).chain())
        // 更新系统
        .add_systems(Update, (
            input::handle_click.run_if(in_state(AppState::InGame)),
            input::execute_action.run_if(in_state(TurnPhase::ExecuteAction)),
            input::handle_turn_end.run_if(in_state(TurnPhase::TurnEnd)),
            ui::update_turn_indicator.run_if(in_state(AppState::InGame)),
            ui::update_unit_info.run_if(in_state(AppState::InGame)),
        ))
        // 直接进入游戏（后续可加主菜单）
        .add_systems(Startup, |mut next: ResMut<NextState<AppState>>| {
            next.set(AppState::InGame);
        })
        .run();
}
