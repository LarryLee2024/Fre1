//! 商店界面
//!
//! 全屏商店视图，由 ShopPanel Widget 组成。
//! 仅使用原语层和 Widget 层工厂。不直接操作 Node/Button/Interaction。
//!
//! UI 树结构：
//!
//! ```text
//! Panel (Basic, full screen, centered)
//!   └── ShopPanel
//!         ├── ShopHeader
//!         │   ├── Text ("Shop", Heading)
//!         │   └── Text ("Gold: 999", Caption)
//!         ├── TabPanel (Buy / Sell tabs)
//!         ├── ShopItemCard × 3
//!         ├── InventoryItemRow × 2
//!         └── Button ("Close", Secondary)
//! ```

use bevy::prelude::*;

use crate::ui::application::UiCommand;
use crate::ui::navigation::ScreenType;
use crate::ui::primitives::button::events::ButtonClicked;
use crate::ui::primitives::panel::{components::PanelVariant, factory::spawn_panel};
use crate::ui::theme::Theme;
use crate::ui::widgets::shop_panel::components::{ShopPanel as ShopPanelMarker, ShopPanelAction};
use crate::ui::widgets::shop_panel::factory::spawn_shop_panel;

/// 商店界面标记组件
///
/// 用于场景管理清理（离开商店界面时销毁所有携带此组件的实体）。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct ShopScreen;

/// 启动系统：生成商店界面
///
/// 创建全屏商店 UI 树。所有元素通过原语/Widget 工厂创建
/// — 不直接操作 Node/Button/Interaction。
pub fn spawn_shop_screen(
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
        ShopScreen,
    ));

    // ── 2. 商店面板 ──
    let panel = spawn_shop_panel(&mut commands, &asset_server, &theme);
    commands.entity(panel).set_parent_in_place(root);
}

/// 清除系统：离开商店时销毁所有商店屏幕实体
pub fn despawn_shop_screen(mut commands: Commands, query: Query<Entity, With<ShopScreen>>) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}

/// Observer：处理商店按钮点击，映射到 UiCommand
///
/// 当原语层的 button_interaction_system 通过 Commands::trigger 触发
/// ButtonClicked 事件时，检查按钮实体是否携带 ShopPanelAction 组件
/// 并分发对应的 UiCommand。
pub fn on_shop_button_clicked(
    trigger: On<ButtonClicked>,
    query: Query<&ShopPanelAction>,
    mut commands: Commands,
) {
    let entity = trigger.event().entity;
    let Ok(action) = query.get(entity) else {
        // 非商店按钮，忽略
        return;
    };

    let command = match action {
        ShopPanelAction::Close => UiCommand::CloseScreen,
        ShopPanelAction::BuyItem { item_id, .. } => UiCommand::BuyItem {
            item_def_id: item_id.to_string(),
            quantity: 1,
            shop_id: "default".to_string(),
        },
        ShopPanelAction::SellItem { item_id, .. } => UiCommand::SellItem {
            item_def_id: item_id.to_string(),
            quantity: 1,
            shop_id: "default".to_string(),
        },
    };

    info!(target: "ui", "[Shop] 命令映射: {:?}", command);
    commands.trigger(command);
}

/// ShopPlugin — 注册 ShopScreen 的组件类型和 Observer。
pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ShopScreen>()
            .add_observer(on_open_shop_screen)
            .add_observer(on_close_shop_screen)
            .add_observer(on_shop_button_clicked);
    }
}

/// Observer：处理 UiCommand::OpenScreen(Shop) — 生成商店界面。
pub fn on_open_shop_screen(
    on: On<UiCommand>,
    commands: Commands,
    theme: Res<Theme>,
    asset_server: Res<AssetServer>,
) {
    if let UiCommand::OpenScreen(ScreenType::Shop) = on.event() {
        spawn_shop_screen(commands, theme, asset_server);
    }
}

/// Observer：处理 UiCommand::CloseScreen — 销毁商店界面。
pub fn on_close_shop_screen(
    on: On<UiCommand>,
    mut commands: Commands,
    query: Query<Entity, With<ShopScreen>>,
) {
    if let UiCommand::CloseScreen = on.event() {
        for entity in &query {
            commands.entity(entity).despawn();
        }
    }
}
