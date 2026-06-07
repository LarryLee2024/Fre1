mod ai;
mod combat;
mod input;
mod map;
mod pathfinding;
mod turn;
mod ui;
mod unit;

use bevy::prelude::*;
use map::GameMap;
use turn::{AppState, TurnPhase};

/// 生成 2D 相机
fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

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
        .init_resource::<turn::AiTimer>()
        .init_resource::<input::AttackTarget>()
        // 状态
        .init_state::<AppState>()
        .add_sub_state::<TurnPhase>()
        // 入场系统
        .add_systems(
            OnEnter(AppState::InGame),
            (
                spawn_camera,
                map::spawn_map,
                unit::spawn_units,
                ui::spawn_ui,
            )
                .chain(),
        )
        // 回合阶段 OnEnter 系统
        .add_systems(
            OnEnter(TurnPhase::ExecuteAction),
            input::execute_action_on_enter,
        )
        .add_systems(OnEnter(TurnPhase::WaitAction), input::wait_action_on_enter)
        .add_systems(OnEnter(TurnPhase::TurnEnd), input::turn_end_on_enter)
        // 更新系统
        .add_systems(
            Update,
            (
                input::handle_click.run_if(in_state(AppState::InGame)),
                input::handle_end_turn.run_if(in_state(AppState::InGame)),
                ai::enemy_ai_system.run_if(in_state(AppState::InGame)),
                ui::update_turn_indicator.run_if(in_state(AppState::InGame)),
                ui::update_unit_info.run_if(in_state(AppState::InGame)),
                ui::update_action_menu.run_if(in_state(AppState::InGame)),
                ui::update_hp_bars.run_if(in_state(AppState::InGame)),
                ui::check_game_over.run_if(in_state(AppState::InGame)),
            ),
        )
        // 直接进入游戏（后续可加主菜单）
        .add_systems(Startup, |mut next: ResMut<NextState<AppState>>| {
            next.set(AppState::InGame);
        })
        .run();
}
