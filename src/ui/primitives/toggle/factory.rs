//! Toggle Factory — Toggle 的唯一创建入口
//!
//! 遵循 Factory 模式，禁止直接通过 commands.spawn 创建 Toggle。
//! 输入 Props + Theme → 输出 Entity。
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §9

use bevy::prelude::*;

use super::components::{ToggleIndicator, ToggleState};
use crate::infra::localization::LocalizedText;
use crate::ui::theme::Theme;

/// 工厂函数：生成一个完整配置的 Toggle Widget
///
/// # 参数
/// - `commands`: ECS 命令
/// - `theme`: 主题 Resource（提供颜色令牌）
/// - `label_key`: 本地化标签 Key
/// - `default_label`: FTL 未加载时的兜底文本
/// - `checked`: 初始选中状态
///
/// # 返回
/// Toggle Widget 实体的 Entity。内部结构：
/// ```text
/// Toggle (Node, flex: row)
///   ├── Text (label_key / default_label)
///   └── Node (Button, ToggleIndicator, 24x24 box)
/// ```
pub fn spawn_toggle(
    commands: &mut Commands,
    theme: &Theme,
    label_key: &'static str,
    default_label: &str,
    checked: bool,
) -> Entity {
    let indicator_color = if checked {
        theme.colors.accent_primary
    } else {
        theme.colors.surface_secondary
    };

    commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::axes(Val::Px(theme.spacing.sm), Val::Px(theme.spacing.xs)),
                column_gap: Val::Px(theme.spacing.md),
                ..default()
            },
            ToggleState {
                checked,
                label_key,
                enabled: true,
            },
            Name::new(format!("Toggle({})", default_label)),
        ))
        .with_children(|parent| {
            // 左侧：标签文本（本地化）
            parent.spawn((
                Text::new(default_label.to_string()),
                TextFont {
                    font_size: FontSize::Px(theme.typography.size_body),
                    ..default()
                },
                TextColor(theme.colors.text_primary),
                LocalizedText::static_text(label_key),
                Name::new(format!("ToggleLabel({})", default_label)),
            ));
            // 右侧：可点击的指示器框
            parent.spawn((
                Node {
                    width: Val::Px(24.0),
                    height: Val::Px(24.0),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                Button,
                BackgroundColor(indicator_color),
                BorderColor::all(theme.colors.border_default),
                ToggleIndicator,
                Name::new(format!("ToggleIndicator({})", default_label)),
            ));
        })
        .id()
}
