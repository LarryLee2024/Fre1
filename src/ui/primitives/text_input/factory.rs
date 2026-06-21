//! TextInput Factory — TextInput 的唯一创建入口
//!
//! 遵循 Factory 模式，禁止直接通过 commands.spawn 创建 TextInput。
//! 输入 Props + Theme → 输出 Entity。
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §9

use bevy::prelude::*;

use super::components::TextInputState;
use crate::ui::theme::Theme;

/// 工厂函数：生成一个完整配置的 TextInput UI 节点
///
/// # 参数
/// - `commands`: ECS 命令
/// - `theme`: 主题 Resource（提供颜色令牌）
/// - `placeholder_key`: 占位符文本的本地化 Key
/// - `max_length`: 最大可输入字符数
///
/// # 返回
/// TextInput 容器实体的 Entity。内部结构：
/// ```text
/// TextInput (Node, Panel(Basic) 样式容器，带边框)
///   └── Text (显示当前输入值或占位符)
/// ```
///
/// 通过 `text_input_system` 处理键盘输入并更新 TextInputState.value。
///
/// # 用法
/// ```ignore
/// let input = spawn_text_input(
///     &mut commands, &theme,
///     loc::ui::INPUT_PLACEHOLDER, 64,
/// );
/// ```
pub fn spawn_text_input(
    commands: &mut Commands,
    theme: &Theme,
    placeholder_key: &'static str,
    max_length: usize,
) -> Entity {
    commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(theme.spacing.md), Val::Px(theme.spacing.sm)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(theme.spacing.border_radius_sm)),
                min_height: Val::Px(theme.spacing.button_height),
                ..default()
            },
            BackgroundColor(theme.colors.surface_primary),
            BorderColor::all(theme.colors.border_default),
            TextInputState {
                value: String::new(),
                max_length,
                placeholder_key,
                is_focused: false,
            },
            Name::new("TextInput"),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(String::new()),
                TextFont {
                    font_size: FontSize::Px(theme.typography.size_body),
                    ..default()
                },
                TextColor(theme.colors.text_secondary),
                Name::new("TextInputValue"),
            ));
        })
        .id()
}
