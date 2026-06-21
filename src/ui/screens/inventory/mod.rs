//! InventoryScreen — 背包屏幕

pub mod systems;

use bevy::prelude::*;

use crate::ui::primitives::panel::{components::PanelVariant, factory::spawn_panel};
use crate::ui::theme::Theme;
use crate::ui::widgets::inventory_grid::factory::spawn_inventory_grid;

/// InventoryScreen 标记组件
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct InventoryScreen;

/// 生成背包屏幕
pub fn spawn_inventory_screen(mut commands: Commands, theme: Res<Theme>, asset_server: Res<AssetServer>) {
    let root = spawn_panel(&mut commands, &theme, PanelVariant::Basic);
    commands.entity(root).insert((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        InventoryScreen,
    ));

    let grid = spawn_inventory_grid(&mut commands, &asset_server, &theme);
    commands.entity(grid).set_parent_in_place(root);
}

/// 清除背包屏幕
pub fn despawn_inventory_screen(mut commands: Commands, query: Query<Entity, With<InventoryScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
