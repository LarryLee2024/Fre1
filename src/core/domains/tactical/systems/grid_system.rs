//! Grid System — 网格初始化与查询系统

use bevy::prelude::*;

use crate::core::domains::tactical::resources::{GridLayout, GridMap};

/// 初始化默认网格（调试/测试用）。
///
/// 在游戏启动或编辑器新建地图时调用。
pub fn initialize_default_grid(mut commands: Commands) {
    commands.insert_resource(GridMap::new(20, 15, GridLayout::Square));
    tracing::info!("[Tactical] initialized default 20x15 square grid");
}
