//! TacticalPlugin — 战术空间领域 Plugin
//!
//! 注册网格系统、移动系统和相关事件。
//!
//! 详见 ADR-022

use bevy::prelude::*;

use super::components::{Facing, GridPos, MovementPoints};
use super::systems::grid_system::initialize_default_grid;
use super::systems::input_system::{TacticalCursor, tactical_input_system};
use super::systems::movement_system::on_compute_move;
use crate::register_domain_types;
use crate::shared::game_state::GameState;

/// 战术/网格领域 Plugin——注册网格移动、高地优势、夹击组件和战术系统。
pub struct TacticalPlugin;

impl Plugin for TacticalPlugin {
    fn build(&self, app: &mut App) {
        register_domain_types!(app, [GridPos, MovementPoints, Facing,]);

        // GridMap 由 initialize_default_grid 初始化，或外部设置
        // 不在此处 init_resource（需要外部调用方控制尺寸和布局）

        // 从 Startup 移至 OnEnter(TacticalMap)，确保每次进入战术地图时重新初始化网格
        app.add_systems(OnEnter(GameState::TacticalMap), initialize_default_grid);

        // ── Input System ──
        app.init_resource::<TacticalCursor>();
        app.add_systems(
            Update,
            tactical_input_system.run_if(in_state(GameState::TacticalMap)),
        );

        // 注册 ComputeMoveRequest Observer — Capabilities 集成验证入口
        // 通过 integration.rs 触及 Tag → Attribute → Modifier 管线
        // 详见 movement_system::on_compute_move
        app.add_observer(on_compute_move);
    }
}
