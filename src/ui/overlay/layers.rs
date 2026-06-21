//! 5 层 UI 根实体标记组件

use bevy::prelude::*;

/// Screen 层 — 主界面层
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct ScreenLayer;
/// Popup 层 — 弹窗层（Modal、Loading、DamageText）
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct PopupLayer;
/// Tooltip 层 — 工具提示层
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct TooltipLayer;
/// Notification 层 — 通知层
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct NotificationLayer;
/// Debug 层 — 调试层（仅 dev feature）
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct DebugLayer;

/// Startup 系统：创建 5 层 UI 根节点
pub fn create_ui_roots(mut commands: Commands) {
    commands.spawn((
        Name::new("UiRoot::Screen"),
        ScreenLayer,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
    ));
    commands.spawn((
        Name::new("UiRoot::Popup"),
        PopupLayer,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
    ));
    commands.spawn((
        Name::new("UiRoot::Tooltip"),
        TooltipLayer,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
    ));
    commands.spawn((
        Name::new("UiRoot::Notification"),
        NotificationLayer,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
    ));
    commands.spawn((
        Name::new("UiRoot::Debug"),
        DebugLayer,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
    ));
}
