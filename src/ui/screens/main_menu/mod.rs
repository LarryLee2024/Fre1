//! Module Name: MainMenuScreen — 主菜单屏幕
//!
//! 全屏主菜单，包含游戏标题、副标题和三组菜单按钮。
//! 使用 Primitives 层工厂函数 (`spawn_panel`, `spawn_text`, `spawn_button`) 创建 UI 元素。
//! 按钮点击通过 Observer 模式处理（见 systems.rs）。

pub mod systems;

use bevy::prelude::*;

use crate::ui::primitives::button::{components::ButtonVariant, factory::spawn_button};
use crate::ui::primitives::list::{components::ListVariant, factory::spawn_list};
use crate::ui::primitives::panel::{components::PanelVariant, factory::spawn_panel};
use crate::ui::primitives::text::{components::TextVariant, factory::spawn_text};
use crate::ui::theme::Theme;

/// 菜单按钮动作标识
///
/// 作为 Component 挂载在按钮实体上，Observer 通过查询此组件识别被点击的按钮。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub enum MenuAction {
    /// 开始新游戏
    NewGame,
    /// 读取存档
    LoadGame,
    /// 游戏设置
    Settings,
}

/// 主菜单屏幕标记组件
///
/// 用于未来场景管理系统的 despawn 清理。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct MainMenuScreen;

/// Startup System: 生成主菜单屏幕
///
/// 创建全屏主菜单 UI 树。所有元素通过 Primitives 工厂函数创建，
/// 不使用直接 Node/Button/Interaction 操作。
///
/// # UI 树结构
///
/// ```text
/// Panel (Basic, full screen)
///   ├── Text ("Fre", Title, 48px)
///   ├── Text ("A Bevy SRPG", Caption)
///   ├── List (Vertical)
///   │   ├── Button ("New Game", Primary)
///   │   ├── Button ("Load Game", Secondary)
///   │   └── Button ("Settings", Secondary)
///   └── Text ("v0.1.0", Caption)
/// ```
pub fn spawn_main_menu(mut commands: Commands, theme: Res<Theme>, asset_server: Res<AssetServer>) {
    // ── 1. Root panel ──
    // 使用 Basic 变体工厂创建基础面板，再覆盖为全屏尺寸
    let root = spawn_panel(&mut commands, &theme, PanelVariant::Basic);
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
        MainMenuScreen,
    ));

    // ── 2. Title "Fre" ──
    let title = spawn_text(
        &mut commands,
        &asset_server,
        &theme,
        "Fre",
        TextVariant::Title,
    );
    // Title 默认 24px，设计规格要求大标题 48px
    commands.entity(title).insert((
        TextFont {
            font: bevy::text::FontSource::Handle(
                asset_server.load(theme.typography.font_heading.clone()),
            ),
            font_size: FontSize::Px(48.0),
            ..default()
        },
        Node {
            margin: UiRect::bottom(Val::Px(theme.spacing.sm)),
            ..default()
        },
    ));

    // ── 3. Subtitle ──
    let subtitle = spawn_text(
        &mut commands,
        &asset_server,
        &theme,
        "A Bevy SRPG",
        TextVariant::Caption,
    );
    commands.entity(subtitle).insert(Node {
        margin: UiRect::bottom(Val::Px(theme.spacing.xl)),
        ..default()
    });

    // ── 4. Menu list (vertical) ──
    let list = spawn_list(&mut commands, &theme, ListVariant::Vertical);
    commands.entity(list).insert(Node {
        align_items: AlignItems::Center,
        width: Val::Px(200.0),
        ..default()
    });

    // ── 5. Menu buttons ──
    let new_game_btn = spawn_button(&mut commands, &theme, "New Game", ButtonVariant::Primary);
    let load_game_btn = spawn_button(&mut commands, &theme, "Load Game", ButtonVariant::Secondary);
    let settings_btn = spawn_button(&mut commands, &theme, "Settings", ButtonVariant::Secondary);

    // Attach MenuAction for button identification
    commands.entity(new_game_btn).insert(MenuAction::NewGame);
    commands.entity(load_game_btn).insert(MenuAction::LoadGame);
    commands.entity(settings_btn).insert(MenuAction::Settings);

    // ── 6. Version text ──
    let version = spawn_text(
        &mut commands,
        &asset_server,
        &theme,
        "v0.1.0",
        TextVariant::Caption,
    );

    // ── 7. Build hierarchy via set_parent_in_place ──
    // Root → children
    commands.entity(title).set_parent_in_place(root);
    commands.entity(subtitle).set_parent_in_place(root);
    commands.entity(list).set_parent_in_place(root);
    commands.entity(version).set_parent_in_place(root);

    // List → buttons
    commands.entity(new_game_btn).set_parent_in_place(list);
    commands.entity(load_game_btn).set_parent_in_place(list);
    commands.entity(settings_btn).set_parent_in_place(list);
}
