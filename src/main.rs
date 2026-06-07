mod ai;
mod assets;
mod camera;
mod combat;
mod combat_event;
mod combat_log;
mod input;
mod map;
mod pathfinding;
mod turn;
mod ui;
mod unit;
mod vfx;

use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
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
        // 调试工具
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        // 资源
        .init_resource::<GameMap>()
        .init_resource::<turn::TurnState>()
        .init_resource::<turn::AiTimer>()
        .init_resource::<input::AttackTarget>()
        .init_resource::<input::PrevPosition>()
        .init_resource::<input::ActionMenuEntity>()
        .init_resource::<combat_log::CombatLog>()
        // 状态
        .init_state::<AppState>()
        .add_sub_state::<TurnPhase>()
        // 入场系统
        .add_systems(
            OnEnter(AppState::InGame),
            (camera::spawn_camera, map::spawn_map, unit::spawn_units, ui::spawn_ui).chain(),
        )
        // 回合阶段 OnEnter 系统
        .add_systems(OnEnter(TurnPhase::ExecuteAction), input::execute_action_on_enter)
        .add_systems(OnEnter(TurnPhase::WaitAction), input::wait_action_on_enter)
        .add_systems(OnEnter(TurnPhase::TurnEnd), input::turn_end_on_enter)
        // 更新系统
        .add_systems(
            Update,
            (
                camera::camera_control,
                input::handle_click.run_if(in_state(AppState::InGame)),
                input::handle_right_cancel.run_if(in_state(AppState::InGame)),
                input::handle_end_turn.run_if(in_state(AppState::InGame)),
                input::handle_action_menu_interaction.run_if(in_state(AppState::InGame)),
                ai::enemy_ai_system.run_if(in_state(AppState::InGame)),
                ui::setup_ui_font.run_if(in_state(AppState::InGame)),
                ui::update_turn_indicator.run_if(in_state(AppState::InGame)),
                ui::update_unit_info.run_if(in_state(AppState::InGame)),
            ),
        )
        .add_systems(
            Update,
            (
                ui::update_action_menu.run_if(in_state(AppState::InGame)),
                ui::update_hp_bars.run_if(in_state(AppState::InGame)),
                ui::check_game_over.run_if(in_state(AppState::InGame)),
                combat_log::update_combat_log.run_if(in_state(AppState::InGame)),
                vfx::update_damage_popups,
            ),
        )
        // 直接进入游戏（后续可加主菜单）
        .add_systems(
            Startup,
            (
                |mut next: ResMut<NextState<AppState>>| {
                    next.set(AppState::InGame);
                },
                assets::init_cn_font,
            ),
        )
        .run();
}
