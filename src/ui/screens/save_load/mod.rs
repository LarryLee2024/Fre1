//! 存档/读档界面
//!
//! 提供存档/读档游戏界面。列出存档槽位，
//! 每个槽位带有存档/读档操作。MVP 实现，
//! 包含 3 个硬编码槽位和关闭按钮。
//!
//! # UI 树结构
//! ```text
//! Panel (Basic, full screen)
//!   ├── Text ("Save/Load", Heading)
//!   ├── List (Vertical)
//!   │   ├── Panel (Card) — 存档槽位 1
//!   │   │   ├── Text ("Save Slot 1", Body)
//!   │   │   └── Text ("Empty", Caption)
//!   │   ├── Panel (Card) — 存档槽位 2
//!   │   │   ├── Text ("Save Slot 2", Body)
//!   │   │   └── Text ("Empty", Caption)
//!   │   └── Panel (Card) — 存档槽位 3
//!   │       ├── Text ("Save Slot 3", Body)
//!   │       └── Text ("Empty", Caption)
//!   └── Button ("Close", Secondary)

use bevy::ecs::observer::On;
use bevy::prelude::*;

use crate::infra::localization::generated::loc;
use crate::ui::application::UiCommand;
use crate::ui::navigation::ScreenType;
use crate::ui::primitives::button::{
    components::ButtonVariant, events::ButtonClicked, factory::spawn_localized_button,
};
use crate::ui::primitives::list::{components::ListVariant, factory::spawn_list};
use crate::ui::primitives::panel::{components::PanelVariant, factory::spawn_panel};
use crate::ui::primitives::text::{components::TextVariant, factory::spawn_text};
use crate::ui::theme::Theme;

/// SaveLoad 界面根标记组件。
///
/// 用于 despawn 逻辑识别界面层级根节点。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct SaveLoadScreen;

/// SaveLoad 界面按钮的操作标识。
///
/// 作为 Component 挂载到按钮实体上。Observer 匹配此组件
/// 来确定哪个按钮被点击。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub enum SaveLoadAction {
    /// 关闭 SaveLoad 界面
    Close,
}

/// Observer：当 UiCommand::OpenScreen(SaveLoad) 触发时生成 SaveLoad 界面。
pub fn on_open_save_load_screen(
    on: On<UiCommand>,
    mut commands: Commands,
    theme: Res<Theme>,
    asset_server: Res<AssetServer>,
) {
    if let UiCommand::OpenScreen(ScreenType::SaveLoad) = on.event() {
        spawn_save_load_screen(&mut commands, &theme, &asset_server);
    }
}

/// Observer：当 UiCommand::CloseScreen 触发时销毁 SaveLoad 界面。
pub fn on_close_save_load_screen(
    on: On<UiCommand>,
    mut commands: Commands,
    query: Query<Entity, With<SaveLoadScreen>>,
) {
    if let UiCommand::CloseScreen = on.event() {
        for entity in &query {
            commands.entity(entity).despawn();
        }
    }
}

/// 生成完整的 SaveLoad 界面 UI 层级。
pub fn spawn_save_load_screen(commands: &mut Commands, theme: &Theme, asset_server: &AssetServer) {
    // ── 1. Root panel ──
    let root = spawn_panel(commands, theme, PanelVariant::Basic);
    commands.entity(root).insert((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            padding: UiRect::all(Val::Px(theme.spacing.xl)),
            ..default()
        },
        SaveLoadScreen,
    ));

    // ── 2. Title ──
    let title = spawn_text(
        commands,
        asset_server,
        theme,
        "Save/Load",
        TextVariant::Heading,
    );
    commands.entity(title).insert(Node {
        margin: UiRect::bottom(Val::Px(theme.spacing.lg)),
        ..default()
    });
    commands.entity(title).set_parent_in_place(root);

    // ── 3. Save slot list (MVP: 3 hardcoded slots) ──
    let list = spawn_list(commands, theme, ListVariant::Vertical);
    commands.entity(list).set_parent_in_place(root);

    for i in 1..=3 {
        let slot = spawn_panel(commands, theme, PanelVariant::Card);
        let label = spawn_text(
            commands,
            asset_server,
            theme,
            format!("Save Slot {}", i),
            TextVariant::Body,
        );
        commands.entity(label).set_parent_in_place(slot);

        let info = spawn_text(commands, asset_server, theme, "Empty", TextVariant::Caption);
        commands.entity(info).set_parent_in_place(slot);

        commands.entity(slot).set_parent_in_place(list);
    }

    // ── 4. Close button ──
    let close = spawn_localized_button(
        commands,
        theme,
        loc::ui::CLOSE,
        "Close",
        ButtonVariant::Secondary,
    );
    commands.entity(close).insert(SaveLoadAction::Close);
    commands.entity(close).set_parent_in_place(root);
}

/// Observer：处理 SaveLoad 界面按钮点击（关闭）。
pub fn on_save_load_button_clicked(
    on: On<ButtonClicked>,
    query: Query<&SaveLoadAction>,
    mut commands: Commands,
    screen_query: Query<Entity, With<SaveLoadScreen>>,
) {
    let entity = on.event().entity;
    if let Ok(action) = query.get(entity) {
        match action {
            SaveLoadAction::Close => {
                for entity in &screen_query {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
