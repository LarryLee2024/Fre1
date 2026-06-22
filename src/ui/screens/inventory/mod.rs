//! 背包管理界面
//!
//! 全屏背包视图，由 InventoryGrid Widget 组成。
//! 仅使用原语层和 Widget 层工厂。不直接操作 Node/Button/Interaction。
//!
//! UI 树结构：
//!
//! ```text
//! Panel (Basic, full screen, centered)
//!   └── InventoryGrid
//!         ├── Text ("Inventory", Heading)
//!         ├── Text ("Gold: 100", Caption)
//!         ├── InventoryItemRow × N
//!         └── Button ("Close", Secondary)
//! ```

pub mod systems;

use bevy::prelude::*;

use crate::ui::primitives::panel::{components::PanelVariant, factory::spawn_panel};
use crate::ui::theme::Theme;
use crate::ui::widgets::inventory_grid::factory::spawn_inventory_grid;

/// 背包界面标记组件
///
/// 用于场景管理清理（离开背包界面时销毁所有携带此组件的实体）。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct InventoryScreen;

/// 启动系统：生成背包界面
///
/// 创建全屏背包 UI 树。所有元素通过原语/Widget 工厂创建
/// — 不直接操作 Node/Button/Interaction。
pub fn spawn_inventory_screen(
    mut commands: Commands,
    theme: Res<Theme>,
    asset_server: Res<AssetServer>,
) {
    // ── 1. 根面板（全屏居中） ──
    let root = spawn_panel(&mut commands, &theme, PanelVariant::Basic);
    commands.entity(root).insert((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        InventoryScreen,
    ));

    // ── 2. 背包网格 ──
    let grid = spawn_inventory_grid(&mut commands, &asset_server, &theme);
    commands.entity(grid).set_parent_in_place(root);
}

/// 清除系统：离开背包时销毁所有背包屏幕实体
pub fn despawn_inventory_screen(
    mut commands: Commands,
    query: Query<Entity, With<InventoryScreen>>,
) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}
