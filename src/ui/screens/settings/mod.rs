//! Module Name: SettingsScreen — 游戏设置屏幕
//!
//! 提供游戏设置界面，包含主题切换、伤害数字显示开关等功能。
//! 设置变更即时生效（主题切换立即应用），持久化通过"保存"按钮触发。
//!
//! # UI 树结构
//! ```text
//! Panel (Basic, full screen)
//!   ├── Text ("Settings", Heading) — localized
//!   ├── Toggle ("Show Damage Numbers", checked)
//!   ├── Toggle ("Dark Theme", checked) — 绑定 ThemeVariant
//!   ├── Button ("Close", Secondary) — 关闭不保存
//!   └── Button ("Save", Primary) — 保存设置到磁盘
//! ```

use bevy::ecs::observer::On;
use bevy::prelude::*;

use crate::infra::localization::generated::loc;
use crate::ui::application::UiCommand;
use crate::ui::navigation::ScreenType;
use crate::ui::primitives::button::{
    components::ButtonVariant, events::ButtonClicked, factory::spawn_localized_button,
};
use crate::ui::primitives::panel::{components::PanelVariant, factory::spawn_panel};
use crate::ui::primitives::text::{components::TextVariant, factory::spawn_localized_text};
use crate::ui::primitives::toggle::{components::ToggleState, factory::spawn_toggle};
use crate::ui::settings::{UiSettings, save_settings};
use crate::ui::theme::Theme;
use crate::ui::theme::switch::{ThemeVariant, switch_theme};

/// 设置屏幕标记组件
///
/// 用于 despawn 清理时识别设置屏幕实体。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct SettingsScreen;

/// 设置按钮动作标识
///
/// 作为 Component 挂载在按钮实体上，Observer 通过查询此组件识别被点击的按钮。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub enum SettingsAction {
    /// 关闭设置（不保存）
    Close,
    /// 保存设置
    Save,
}

/// 设置开关标识
///
/// 作为 Component 挂载在 Toggle 父实体上，用于识别每个 Toggle 对应的设置项。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub enum SettingsToggle {
    /// 显示伤害数字
    ShowDamageNumbers,
    /// 深色主题
    DarkTheme,
}

/// Observer: 处理 UiCommand::OpenScreen(Settings) — 生成设置屏幕
pub fn on_open_settings_screen(
    on: On<UiCommand>,
    mut commands: Commands,
    theme: Res<Theme>,
    asset_server: Res<AssetServer>,
    settings: Res<UiSettings>,
) {
    if let UiCommand::OpenScreen(ScreenType::Settings) = on.event() {
        spawn_settings_screen(&mut commands, &theme, &asset_server, &settings);
    }
}

/// Observer: 处理 UiCommand::CloseScreen — 销毁设置屏幕
pub fn on_close_settings_screen(
    on: On<UiCommand>,
    mut commands: Commands,
    query: Query<Entity, With<SettingsScreen>>,
) {
    if let UiCommand::CloseScreen = on.event() {
        for entity in &query {
            commands.entity(entity).despawn();
        }
    }
}

/// 生成设置屏幕 UI 树
///
/// 创建完整的设置屏幕 UI 层次结构。所有元素通过 Primitives 工厂函数创建。
pub fn spawn_settings_screen(
    commands: &mut Commands,
    theme: &Theme,
    asset_server: &AssetServer,
    settings: &UiSettings,
) {
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
        SettingsScreen,
    ));

    // ── 2. Title "Settings" — localized ──
    let title = spawn_localized_text(
        commands,
        asset_server,
        theme,
        loc::ui::SETTINGS,
        "Settings",
        TextVariant::Heading,
    );
    commands.entity(title).insert(Node {
        margin: UiRect::bottom(Val::Px(theme.spacing.lg)),
        ..default()
    });

    // ── 3. Toggle: Show Damage Numbers ──
    let damage_toggle = spawn_toggle(
        commands,
        theme,
        "ui.settings.show_damage",
        "Show Damage Numbers",
        settings.show_damage_numbers,
    );
    commands.entity(damage_toggle).insert((
        SettingsToggle::ShowDamageNumbers,
        Node {
            width: Val::Px(280.0),
            margin: UiRect::bottom(Val::Px(theme.spacing.sm)),
            ..default()
        },
    ));

    // ── 4. Toggle: Dark Theme ──
    let is_dark = settings.theme == ThemeVariant::Dark;
    let theme_toggle = spawn_toggle(
        commands,
        theme,
        "ui.settings.dark_theme",
        "Dark Theme",
        is_dark,
    );
    commands.entity(theme_toggle).insert((
        SettingsToggle::DarkTheme,
        Node {
            width: Val::Px(280.0),
            margin: UiRect::bottom(Val::Px(theme.spacing.lg)),
            ..default()
        },
    ));

    // ── 5. Action buttons ──
    let close_btn = spawn_localized_button(
        commands,
        theme,
        loc::ui::CLOSE,
        "Close",
        ButtonVariant::Secondary,
    );
    commands.entity(close_btn).insert((
        SettingsAction::Close,
        Node {
            width: Val::Px(200.0),
            margin: UiRect::bottom(Val::Px(theme.spacing.sm)),
            ..default()
        },
    ));

    let save_btn = spawn_localized_button(
        commands,
        theme,
        loc::ui::SAVE,
        "Save",
        ButtonVariant::Primary,
    );
    commands.entity(save_btn).insert((
        SettingsAction::Save,
        Node {
            width: Val::Px(200.0),
            ..default()
        },
    ));

    // ── 6. Build hierarchy via set_parent_in_place ──
    commands.entity(title).set_parent_in_place(root);
    commands.entity(damage_toggle).set_parent_in_place(root);
    commands.entity(theme_toggle).set_parent_in_place(root);
    commands.entity(close_btn).set_parent_in_place(root);
    commands.entity(save_btn).set_parent_in_place(root);
}

/// Observer: 处理设置按钮点击（Close / Save）
pub fn on_settings_button_clicked(
    on: On<ButtonClicked>,
    query: Query<&SettingsAction>,
    mut commands: Commands,
    settings: Res<UiSettings>,
    settings_query: Query<Entity, With<SettingsScreen>>,
) {
    let entity = on.event().entity;
    if let Ok(action) = query.get(entity) {
        match action {
            SettingsAction::Close => {
                for entity in &settings_query {
                    commands.entity(entity).despawn();
                }
            }
            SettingsAction::Save => {
                save_settings(&settings);
                for entity in &settings_query {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

/// Update 系统: 检测设置 Toggle 状态变更并应用
///
/// 当 Toggle 值变化时即时应用设置（主题切换立即生效）。
/// 使用 Changed<ToggleState> 确保只在值变化时触发。
pub fn settings_toggle_system(
    mut toggles: Query<(&SettingsToggle, &ToggleState), Changed<ToggleState>>,
    mut ui_settings: ResMut<UiSettings>,
    mut theme: ResMut<Theme>,
) {
    for (toggle_type, state) in &mut toggles {
        match toggle_type {
            SettingsToggle::ShowDamageNumbers => {
                ui_settings.show_damage_numbers = state.checked;
            }
            SettingsToggle::DarkTheme => {
                let new_variant = if state.checked {
                    ThemeVariant::Dark
                } else {
                    ThemeVariant::Light
                };
                ui_settings.theme = new_variant;
                switch_theme(&mut theme, new_variant);
            }
        }
    }
}
