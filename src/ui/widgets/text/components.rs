//! Text Widget 组件的类型定义
//!
//! 定义 TextVariant 枚举和 TextWidget（Widget Contract 的本地状态）。
//!
//! 详见 `docs/06-ui/02-design-system/widget-atoms.md` §4

use bevy::prelude::*;

/// 文本样式变体
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum TextVariant {
    /// 正文文本
    Body,
    /// 章节标题
    Heading,
    /// 页面标题
    Title,
    /// 辅助说明文字
    Caption,
    /// 表单标签
    Label,
    /// 代码/等宽文本
    Mono,
}

/// 文本 Widget 状态
///
/// 包含变体、文本内容和可选的文字颜色覆盖。
/// `content` 未来将被 `LocalizationKey` 替换。
///
/// Props 字段由 spawn_text 的入参决定。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct TextWidget {
    /// 文本样式变体
    pub variant: TextVariant,
    /// 文本内容（未来将被 LocalizationKey 替换）
    pub content: String,
    /// 可选的文字颜色覆盖（Some 时覆盖变体默认色）
    pub color_override: Option<Color>,
}
