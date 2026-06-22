//! SettingsScreen — 游戏设置屏幕
//!
//! 提供游戏设置界面，包含 Gameplay / Display 分组面板，支持主题切换、伤害数字显示、网格、
//! 小地图、自动战斗等开关。设置变更即时生效（主题切换立即应用），持久化通过"保存"按钮触发。
//!
//! # UI 树结构
//! ```text
//! Panel (Basic, full screen)
//!   ├── Text ("Settings", Heading) — localized
//!   ├── Panel (Group, "Gameplay")
//!   │   ├── Text ("Gameplay", Label) — section header
//!   │   ├── Toggle ("Show Damage Numbers", checked)
//!   │   ├── Toggle ("Show Minimap", checked)
//!   │   ├── Toggle ("Show Grid", checked)
//!   │   └── Toggle ("Auto Battle", unchecked)
//!   ├── Panel (Group, "Display")
//!   │   ├── Text ("Display", Label) — section header
//!   │   └── Toggle ("Dark Theme", checked) — 绑定 ThemeVariant
//!   ├── Button ("Save", Primary) — 保存设置到磁盘
//!   └── Button ("Close", Secondary) — 关闭不保存
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

// ── 本地化 Key（暂无 FTL 条目时的兜底文本） ──

/// 设置屏幕 Gameplay 分节标题 Key
const KEY_SECTION_GAMEPLAY: &str = "ui.settings.section.gameplay";
/// 设置屏幕 Display 分节标题 Key
const KEY_SECTION_DISPLAY: &str = "ui.settings.section.display";
/// 小地图开关 Key
const KEY_SHOW_MINIMAP: &str = "ui.settings.show_minimap";
/// 网格开关 Key
const KEY_SHOW_GRID: &str = "ui.settings.show_grid";
/// 自动战斗开关 Key
const KEY_AUTO_BATTLE: &str = "ui.settings.auto_battle";

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
    /// 显示小地图
    ShowMinimap,
    /// 显示网格
    ShowGrid,
    /// 自动战斗
    AutoBattle,
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
    // ── 1. 根面板 ──
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

    // ── 2. 标题 "Settings" — 已本地化 ──
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

    // ══════════════════════════════════════════════
    //  3. Gameplay 分组面板
    // ══════════════════════════════════════════════
    let gameplay_panel = spawn_panel(commands, theme, PanelVariant::Group);
    commands.entity(gameplay_panel).insert((
        Node {
            width: Val::Px(320.0),
            margin: UiRect::bottom(Val::Px(theme.spacing.md)),
            ..default()
        },
        Name::new("SettingsGameplayPanel"),
    ));

    // 3a. Section header
    let gameplay_header = spawn_localized_text(
        commands,
        asset_server,
        theme,
        KEY_SECTION_GAMEPLAY,
        "Gameplay",
        TextVariant::Label,
    );
    commands.entity(gameplay_header).insert(Node {
        margin: UiRect::bottom(Val::Px(theme.spacing.sm)),
        ..default()
    });

    // 3b. 开关：显示伤害数字
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

    // 3c. 开关：显示小地图
    let minimap_toggle = spawn_toggle(
        commands,
        theme,
        KEY_SHOW_MINIMAP,
        "Show Minimap",
        settings.show_minimap,
    );
    commands.entity(minimap_toggle).insert((
        SettingsToggle::ShowMinimap,
        Node {
            width: Val::Px(280.0),
            margin: UiRect::bottom(Val::Px(theme.spacing.sm)),
            ..default()
        },
    ));

    // 3d. 开关：显示网格
    let grid_toggle = spawn_toggle(
        commands,
        theme,
        KEY_SHOW_GRID,
        "Show Grid",
        settings.show_grid,
    );
    commands.entity(grid_toggle).insert((
        SettingsToggle::ShowGrid,
        Node {
            width: Val::Px(280.0),
            margin: UiRect::bottom(Val::Px(theme.spacing.sm)),
            ..default()
        },
    ));

    // 3e. 开关：自动战斗
    let auto_battle_toggle = spawn_toggle(
        commands,
        theme,
        KEY_AUTO_BATTLE,
        "Auto Battle",
        settings.auto_battle,
    );
    commands.entity(auto_battle_toggle).insert((
        SettingsToggle::AutoBattle,
        Node {
            width: Val::Px(280.0),
            // 组内最后一个 Toggle：不添加底部边距，由 Group Panel 的 padding 提供间距
            ..default()
        },
    ));

    // ══════════════════════════════════════════════
    //  4. Display 分组面板（主题）
    // ══════════════════════════════════════════════
    let display_panel = spawn_panel(commands, theme, PanelVariant::Group);
    commands.entity(display_panel).insert((
        Node {
            width: Val::Px(320.0),
            margin: UiRect::bottom(Val::Px(theme.spacing.lg)),
            ..default()
        },
        Name::new("SettingsDisplayPanel"),
    ));

    // 4a. Section header
    let display_header = spawn_localized_text(
        commands,
        asset_server,
        theme,
        KEY_SECTION_DISPLAY,
        "Display",
        TextVariant::Label,
    );
    commands.entity(display_header).insert(Node {
        margin: UiRect::bottom(Val::Px(theme.spacing.sm)),
        ..default()
    });

    // 4b. 开关：深色主题
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
            // Display 组内唯一 Toggle，无需底部边距
            ..default()
        },
    ));

    // ══════════════════════════════════════════════
    //  5. 操作按钮
    // ══════════════════════════════════════════════
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
            margin: UiRect::bottom(Val::Px(theme.spacing.sm)),
            ..default()
        },
    ));

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
            ..default()
        },
    ));

    // ══════════════════════════════════════════════
    //  6. 通过 set_parent_in_place 构建层级
    // ══════════════════════════════════════════════
    commands.entity(title).set_parent_in_place(root);
    // Gameplay 分组
    commands.entity(gameplay_panel).set_parent_in_place(root);
    commands
        .entity(gameplay_header)
        .set_parent_in_place(gameplay_panel);
    commands
        .entity(damage_toggle)
        .set_parent_in_place(gameplay_panel);
    commands
        .entity(minimap_toggle)
        .set_parent_in_place(gameplay_panel);
    commands
        .entity(grid_toggle)
        .set_parent_in_place(gameplay_panel);
    commands
        .entity(auto_battle_toggle)
        .set_parent_in_place(gameplay_panel);
    // Display 分组
    commands.entity(display_panel).set_parent_in_place(root);
    commands
        .entity(display_header)
        .set_parent_in_place(display_panel);
    commands
        .entity(theme_toggle)
        .set_parent_in_place(display_panel);
    // 按钮
    commands.entity(save_btn).set_parent_in_place(root);
    commands.entity(close_btn).set_parent_in_place(root);
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
            SettingsToggle::ShowMinimap => {
                ui_settings.show_minimap = state.checked;
            }
            SettingsToggle::ShowGrid => {
                ui_settings.show_grid = state.checked;
            }
            SettingsToggle::AutoBattle => {
                ui_settings.auto_battle = state.checked;
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
