//! Module Name: SaveLoadScreen — 存档/读档界面
//!
//! 提供存档/读档游戏界面。包含 SaveLoadPlugin 用于注册组件和 Observer，
//! spawn_save_load_screen 工厂函数用于构建完整 UI 层级。
//!
//! 通过 UiCommand::OpenScreen(ScreenType::SaveLoad) 触发创建，
//! 通过 UiCommand::CloseScreen 触发销毁。
//! 默认模式为存档模式 (SaveLoadMode::Save)。
//!
//! # UI 树结构
//! ```text
//! Panel (Basic, full screen)
//!   ├── Panel (Header, Group, row)
//!   │   ├── Text (title, Heading)
//!   │   ├── Button (toggle mode, Ghost)
//!   │   └── Button (close, Ghost)
//!   ├── Panel (Main content, List, row)
//!   │   ├── Panel (Slot list, List, column)
//!   │   │   ├── Panel (Card, Slot 0)
//!   │   │   │   ├── Button ("Slot 1", Primary) — action: SelectSlot(0)
//!   │   │   │   └── Text ("Empty", Caption)
//!   │   │   ├── ... (10 个存档槽位)
//!   │   │   └── Panel (Card, Slot 9)
//!   │   │       ├── Button ("Slot 10", Primary) — action: SelectSlot(9)
//!   │   │       └── Text ("Empty", Caption)
//!   │   └── Panel (Preview, Card, column)
//!   │       ├── Node (avatar placeholder, 96x96)
//!   │       ├── Text (character name — placeholder, Body)
//!   │       ├── Text (level — placeholder, Caption)
//!   │       ├── Text (location — placeholder, Caption)
//!   │       ├── Text (play time — placeholder, Caption)
//!   │       ├── Text (timestamp — placeholder, Caption)
//!   │       ├── Node (screenshot placeholder, 256x144)
//!   │       └── Text (select_hint, Caption)
//!   └── Panel (Action bar, Group, row)
//!       ├── Button (Confirm, Primary, 160x40) — action: Confirm
//!       └── Button (Delete, Danger, 120x40) — action: Delete
//! ```

mod components;
mod systems;

use bevy::ecs::observer::On;
use bevy::prelude::*;

use crate::infra::localization::generated::loc;
use crate::ui::application::UiCommand;
use crate::ui::navigation::ScreenType;
use crate::ui::primitives::button::{
    components::ButtonVariant,
    factory::{spawn_button, spawn_localized_button},
};
use crate::ui::primitives::panel::{components::PanelVariant, factory::spawn_panel};
use crate::ui::primitives::text::{
    components::TextVariant,
    factory::{spawn_localized_text, spawn_text},
};
use crate::ui::theme::Theme;

// Re-export key types for external consumers (tests, other modules)
pub use components::{SaveLoadAction, SaveLoadMode, SaveLoadScreen, SaveSlotVm, SelectedSlot};
// Re-export systems for observer registration in plugin
use systems::on_save_load_button_clicked;

/// SaveLoadPlugin — 注册 SaveLoadScreen 的组件类型、Resource 和 Observer。
///
/// 在 ScreenPlugin 中通过 `.add_plugins(save_load::SaveLoadPlugin)` 注册。
pub struct SaveLoadPlugin;

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app
            // 注册反射类型（用于序列化/编辑器支持）
            .register_type::<SaveLoadScreen>()
            .register_type::<SaveLoadMode>()
            .register_type::<SaveLoadAction>()
            // 初始化槽位选择状态 Resource
            .init_resource::<SelectedSlot>()
            // SaveLoad 界面生命周期 Observer
            .add_observer(on_open_save_load_screen)
            .add_observer(on_close_save_load_screen)
            // SaveLoad 界面按钮点击 Observer
            .add_observer(on_save_load_button_clicked);
    }
}

// ──────────────────────────────────────────────────
// 生命周期 Observer
// ──────────────────────────────────────────────────

/// Observer：处理 UiCommand::OpenScreen(SaveLoad) — 生成 SaveLoad 界面。
pub fn on_open_save_load_screen(
    on: On<UiCommand>,
    mut commands: Commands,
    theme: Res<Theme>,
    asset_server: Res<AssetServer>,
) {
    if let UiCommand::OpenScreen(ScreenType::SaveLoad) = on.event() {
        spawn_save_load_screen(&mut commands, &asset_server, &theme);
    }
}

/// Observer：处理 UiCommand::CloseScreen — 销毁 SaveLoad 界面。
///
/// 查询所有带有 SaveLoadScreen 标记的实体并递归销毁其子节点。
pub fn on_close_save_load_screen(
    on: On<UiCommand>,
    mut commands: Commands,
    query: Query<Entity, With<SaveLoadScreen>>,
) {
    if let UiCommand::CloseScreen = on.event() {
        for entity in &query {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// ──────────────────────────────────────────────────
// 工厂函数
// ──────────────────────────────────────────────────

/// 生成完整的 SaveLoad 界面 UI 层级。
///
/// 创建包含 10 个存档槽位、预览面板和操作按钮的完整界面。
/// 所有槽位当前显示为"Empty"状态，等待真实存档数据接入。
pub fn spawn_save_load_screen(commands: &mut Commands, asset_server: &AssetServer, theme: &Theme) {
    // ── 1. Root panel ──
    let root = spawn_panel(commands, theme, PanelVariant::Basic);
    commands.entity(root).insert((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(theme.spacing.xl)),
            ..default()
        },
        SaveLoadScreen,
        SaveLoadMode::Save,
        Name::new("SaveLoadScreen"),
    ));

    // ── 2. Header panel ──
    let header = spawn_panel(commands, theme, PanelVariant::Group);
    commands.entity(header).insert(Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        width: Val::Percent(100.0),
        margin: UiRect::bottom(Val::Px(theme.spacing.lg)),
        ..default()
    });
    commands.entity(header).set_parent_in_place(root);

    // Title
    let title = spawn_localized_text(
        commands,
        asset_server,
        theme,
        loc::ui::SAVE_LOAD_TITLE_SAVE,
        "Save Game",
        TextVariant::Heading,
    );
    commands.entity(title).insert(Name::new("SaveLoadTitle"));
    commands.entity(title).set_parent_in_place(header);

    // Toggle mode button
    let toggle_btn = spawn_localized_button(
        commands,
        theme,
        loc::ui::SAVE_LOAD_TOGGLE_TO_LOAD,
        "Switch to Load",
        ButtonVariant::Ghost,
    );
    commands
        .entity(toggle_btn)
        .insert((SaveLoadAction::ToggleMode, Name::new("ToggleModeBtn")));
    commands.entity(toggle_btn).set_parent_in_place(header);

    // Close button
    let close_btn = spawn_localized_button(
        commands,
        theme,
        loc::ui::CLOSE,
        "Close",
        ButtonVariant::Ghost,
    );
    commands
        .entity(close_btn)
        .insert((SaveLoadAction::Close, Name::new("CloseBtn")));
    commands.entity(close_btn).set_parent_in_place(header);

    // ── 3. Main content area (horizontal: slots + preview) ──
    let main_content = spawn_panel(commands, theme, PanelVariant::List);
    commands.entity(main_content).insert(Node {
        flex_direction: FlexDirection::Row,
        column_gap: Val::Px(theme.spacing.lg),
        width: Val::Percent(100.0),
        flex_grow: 1.0,
        ..default()
    });
    commands.entity(main_content).set_parent_in_place(root);

    // ── 4. Save slot list (10 个槽位) ──
    let slot_list = spawn_panel(commands, theme, PanelVariant::List);
    commands.entity(slot_list).insert(Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(theme.spacing.sm),
        width: Val::Percent(50.0),
        overflow: Overflow::clip(),
        ..default()
    });
    commands.entity(slot_list).insert(Name::new("SaveSlotList"));
    commands.entity(slot_list).set_parent_in_place(main_content);

    for i in 0..10 {
        let slot = spawn_panel(commands, theme, PanelVariant::Card);
        commands.entity(slot).insert(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(theme.spacing.sm),
            width: Val::Percent(100.0),
            ..default()
        });
        commands
            .entity(slot)
            .insert(Name::new(format!("SaveSlotCard({})", i + 1)));
        commands.entity(slot).set_parent_in_place(slot_list);

        // Slot select button
        let slot_btn = spawn_button(
            commands,
            theme,
            format!("Slot {}", i + 1),
            ButtonVariant::Primary,
        );
        commands.entity(slot_btn).insert((
            SaveLoadAction::SelectSlot(i),
            Name::new(format!("SelectSlotBtn({})", i + 1)),
        ));
        commands.entity(slot_btn).set_parent_in_place(slot);

        // Status text (localized "Empty" for all slots in MVP)
        let status = spawn_localized_text(
            commands,
            asset_server,
            theme,
            loc::ui::SAVE_LOAD_EMPTY,
            "Empty",
            TextVariant::Caption,
        );
        commands
            .entity(status)
            .insert(Name::new(format!("SlotStatus({})", i + 1)));
        commands.entity(status).set_parent_in_place(slot);
    }

    // ── 5. Preview panel ──
    let preview = spawn_panel(commands, theme, PanelVariant::Card);
    commands.entity(preview).insert(Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(theme.spacing.sm),
        width: Val::Px(300.0),
        align_items: AlignItems::Center,
        ..default()
    });
    commands.entity(preview).insert(Name::new("PreviewPanel"));
    commands.entity(preview).set_parent_in_place(main_content);

    // Avatar placeholder (96x96 gray square)
    let avatar = commands
        .spawn((
            Node {
                width: Val::Px(96.0),
                height: Val::Px(96.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 1.0)),
            Name::new("AvatarPlaceholder"),
        ))
        .id();
    commands.entity(avatar).set_parent_in_place(preview);

    // Character name placeholder
    let name_txt = spawn_text(commands, asset_server, theme, "--", TextVariant::Body);
    commands.entity(name_txt).insert(Name::new("PreviewName"));
    commands.entity(name_txt).set_parent_in_place(preview);

    // Level placeholder
    let level_txt = spawn_text(
        commands,
        asset_server,
        theme,
        "Lv. --",
        TextVariant::Caption,
    );
    commands.entity(level_txt).insert(Name::new("PreviewLevel"));
    commands.entity(level_txt).set_parent_in_place(preview);

    // Location placeholder
    let loc_txt = spawn_text(commands, asset_server, theme, "--", TextVariant::Caption);
    commands
        .entity(loc_txt)
        .insert(Name::new("PreviewLocation"));
    commands.entity(loc_txt).set_parent_in_place(preview);

    // Play time placeholder
    let time_txt = spawn_text(commands, asset_server, theme, "--", TextVariant::Caption);
    commands
        .entity(time_txt)
        .insert(Name::new("PreviewPlayTime"));
    commands.entity(time_txt).set_parent_in_place(preview);

    // Timestamp placeholder
    let ts_txt = spawn_text(commands, asset_server, theme, "--", TextVariant::Caption);
    commands
        .entity(ts_txt)
        .insert(Name::new("PreviewTimestamp"));
    commands.entity(ts_txt).set_parent_in_place(preview);

    // Screenshot placeholder (256x144 gray rectangle)
    let screenshot = commands
        .spawn((
            Node {
                width: Val::Px(256.0),
                height: Val::Px(144.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 1.0)),
            Name::new("ScreenshotPlaceholder"),
        ))
        .id();
    commands.entity(screenshot).set_parent_in_place(preview);

    // Select hint text (localized)
    let hint = spawn_localized_text(
        commands,
        asset_server,
        theme,
        loc::ui::SAVE_LOAD_SELECT_HINT,
        "Select a save slot",
        TextVariant::Caption,
    );
    commands.entity(hint).insert(Name::new("SelectHint"));
    commands.entity(hint).set_parent_in_place(preview);

    // ── 6. Action panel ──
    let action_bar = spawn_panel(commands, theme, PanelVariant::Group);
    commands.entity(action_bar).insert(Node {
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::Center,
        column_gap: Val::Px(theme.spacing.md),
        width: Val::Percent(100.0),
        margin: UiRect::top(Val::Px(theme.spacing.lg)),
        ..default()
    });
    commands.entity(action_bar).insert(Name::new("ActionBar"));
    commands.entity(action_bar).set_parent_in_place(root);

    // Confirm button (160x40)
    let confirm_btn = spawn_localized_button(
        commands,
        theme,
        loc::ui::SAVE_LOAD_ACTION_SAVE,
        "Save",
        ButtonVariant::Primary,
    );
    commands.entity(confirm_btn).insert((
        SaveLoadAction::Confirm,
        Node {
            width: Val::Px(160.0),
            height: Val::Px(40.0),
            ..default()
        },
        Name::new("ConfirmBtn"),
    ));
    commands.entity(confirm_btn).set_parent_in_place(action_bar);

    // Delete button (120x40, Danger variant)
    let delete_btn = spawn_localized_button(
        commands,
        theme,
        loc::ui::SAVE_LOAD_ACTION_DELETE,
        "Delete",
        ButtonVariant::Danger,
    );
    commands.entity(delete_btn).insert((
        SaveLoadAction::Delete,
        Node {
            width: Val::Px(120.0),
            height: Val::Px(40.0),
            ..default()
        },
        Name::new("DeleteBtn"),
    ));
    commands.entity(delete_btn).set_parent_in_place(action_bar);
}
