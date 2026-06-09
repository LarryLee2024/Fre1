// 通用浮窗 Widget：合并 combat_preview 和 tile_info 的重复逻辑
// 对应 4.md 推荐的 Popup/Window 封装

use crate::ui::theme::UiTheme;
use bevy::prelude::*;

/// 浮窗根节点标记
#[derive(Component)]
pub struct PopupRoot;

/// 生成浮窗节点，返回实体 ID
pub fn spawn_popup(
    commands: &mut Commands,
    theme: &UiTheme,
    screen_x: f32,
    screen_y: f32,
    bg_color: Option<Color>,
) -> Entity {
    let bg = bg_color.unwrap_or(theme.tile_info_bg);
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(screen_x + theme.popup_offset.0),
                top: Val::Px(screen_y + theme.popup_offset.1),
                padding: theme.popup_padding,
                ..default()
            },
            BackgroundColor(bg),
            PopupRoot,
        ))
        .id()
}

/// 安全销毁浮窗（接受 entity Option，返回 None）
pub fn despawn_popup(commands: &mut Commands, entity: Option<Entity>) -> Option<Entity> {
    if let Some(e) = entity {
        commands.entity(e).try_despawn();
    }
    None
}

/// 为浮窗添加文本内容
pub fn add_popup_text(
    commands: &mut Commands,
    popup_entity: Entity,
    text: &str,
    font_size: f32,
    color: Color,
) {
    commands.entity(popup_entity).with_children(|parent| {
        parent.spawn((
            Text::new(text),
            TextFont {
                font_size,
                ..default()
            },
            TextColor(color),
            TextLayout::new_with_no_wrap(),
        ));
    });
}
