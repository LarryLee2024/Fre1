//! Text Factory — 文本 Widget 的唯一创建入口
//!
//! 遵循 Factory 模式，禁止直接通过 commands.spawn 创建文本节点。
//! 输入 Props + Theme → 输出 Entity。
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §9

use bevy::prelude::*;
use bevy::text::FontSource;

use crate::ui::Theme;
use super::components::{TextVariant, TextWidget};

/// 根据变体计算字体大小
fn font_size_for_variant(variant: TextVariant, theme: &Theme) -> f32 {
    match variant {
        TextVariant::Body => theme.typography.size_body,
        TextVariant::Heading => theme.typography.size_heading,
        TextVariant::Title => theme.typography.size_title,
        TextVariant::Caption => theme.typography.size_small,
        TextVariant::Label => theme.typography.size_body,
        TextVariant::Mono => theme.typography.size_body,
    }
}

/// 根据变体计算字体路径
fn font_path_for_variant(variant: TextVariant, theme: &Theme) -> String {
    match variant {
        TextVariant::Mono => theme.typography.font_mono.clone(),
        TextVariant::Heading | TextVariant::Title => theme.typography.font_heading.clone(),
        TextVariant::Body | TextVariant::Caption | TextVariant::Label => {
            theme.typography.font_body.clone()
        }
    }
}

/// 根据变体计算默认文字颜色
fn color_for_variant(variant: TextVariant, theme: &Theme) -> Color {
    match variant {
        TextVariant::Caption => theme.colors.text_secondary,
        _ => theme.colors.text_primary,
    }
}

/// 工厂函数：生成一个完整配置的文本 Widget
///
/// # 参数
/// - `commands`: ECS 命令
/// - `asset_server`: 资源管理器（用于加载字体）
/// - `theme`: 主题 Resource（提供颜色/字体令牌）
/// - `content`: 文本内容
/// - `variant`: 文本样式变体
///
/// # 返回
/// 文本 Widget 实体的 Entity
///
/// # 用法
/// ```ignore
/// let txt = spawn_text(
///     &mut commands, &asset_server, &theme,
///     "Hello World", TextVariant::Body,
/// );
/// ```
pub fn spawn_text(
    commands: &mut Commands,
    asset_server: &AssetServer,
    theme: &Theme,
    content: impl Into<String>,
    variant: TextVariant,
) -> Entity {
    let content_str: String = content.into();
    let font_size = font_size_for_variant(variant, theme);
    let font_color = color_for_variant(variant, theme);
    let font_path = font_path_for_variant(variant, theme);

    commands
        .spawn((
            Text::new(content_str.clone()),
            TextFont {
                font: FontSource::Handle(asset_server.load(font_path)),
                font_size: FontSize::Px(font_size),
                ..default()
            },
            TextColor(font_color),
            TextWidget {
                variant,
                content: content_str.clone(),
                color_override: None,
            },
            Name::new(format!("Text({})", content_str)),
        ))
        .id()
}
