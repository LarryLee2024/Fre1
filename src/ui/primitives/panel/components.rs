//! Panel 组件的类型定义
//!
//! 定义 PanelVariant 枚举和 PanelState（Widget Contract 的本地状态）。
//!
//! 详见 `docs/06-ui/02-design-system/widget-atoms.md` §4

use bevy::prelude::*;

/// Panel 样式变体
#[derive(Debug, Clone, Copy, Reflect)]
pub enum PanelVariant {
    /// 基础容器，带背景和边框
    Basic,
    /// 信息卡片，带大圆角和阴影效果
    Card,
    /// 模态弹窗，半透明遮罩 + 居中容器
    Modal,
    /// 工具提示，浮动小面板
    Tooltip,
    /// 可滚动列表容器
    List,
    /// 可折叠分组面板
    Group,
    /// 占位符矩形（颜色/尺寸由外部控制）
    Placeholder {
        /// 宽度
        width: Val,
        /// 高度
        height: Val,
        /// 填充颜色
        color: Color,
    },
}

/// Panel 本地状态（Widget Contract Local State）
///
/// 包含变体、是否启用内边距和可选标题。
/// Props 字段由 spawn_panel 的入参决定。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct PanelState {
    /// Panel 样式变体
    pub variant: PanelVariant,
    /// 是否启用默认内边距
    pub padded: bool,
    /// Panel 标题（用于 GroupPanel 等带标题的变体）
    pub title: Option<String>,
}
