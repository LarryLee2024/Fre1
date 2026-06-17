//! TacticalPlugin — 战术空间领域 Plugin
//!
//! 注册网格系统、移动系统和相关事件。
//!
//! 详见 ADR-022

use bevy::prelude::*;

use super::components::{Facing, GridPos, MovementPoints};
use super::systems::grid_system::initialize_default_grid;

pub struct TacticalPlugin;

impl Plugin for TacticalPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GridPos>();
        app.register_type::<MovementPoints>();
        app.register_type::<Facing>();

        // GridMap 由 initialize_default_grid 初始化，或外部设置
        // 不在此处 init_resource（需要外部调用方控制尺寸和布局）

        app.add_systems(Startup, initialize_default_grid);
    }
}
