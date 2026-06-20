//! Grid System — 网格初始化与查询系统

use bevy::prelude::*;

use crate::core::domains::tactical::resources::{GridLayout, GridMap};

/// 初始化默认网格（调试/测试用）。
///
/// 在游戏启动或编辑器新建地图时调用。
pub fn initialize_default_grid(mut commands: Commands) {
    commands.insert_resource(GridMap::new(20, 15, GridLayout::Square));
    tracing::debug!(target: "tactical",
        event = "tactical.grid.initialized",
        rows = 20,
        cols = 15,
        layout = ?GridLayout::Square,
        "[Tactical] 初始化默认 20x15 方格网格",
    );
}
