//! Button 组件的类型定义
//!
//! 定义 ButtonVariant 枚举、ButtonState（Widget Contract 的本地状态）和
//! ButtonInteraction（系统管理的交互状态）。
//!
//! 详见 `docs/06-ui/02-design-system/widget-atoms.md` §2

use bevy::prelude::*;

/// 按钮样式变体
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum ButtonVariant {
    /// 主要操作（主色背景 + 白色文字）
    Primary,
    /// 次要操作（辅助色背景）
    Secondary,
    /// 危险/删除操作（危险色背景 + 白色文字）
    Danger,
    /// 幽灵按钮（透明背景，悬停时显示背景）
    Ghost,
}

/// 按钮本地状态（Widget Contract Local State）
///
/// 包含变体、禁用状态和标签文本。
/// Props 字段由 spawn_button 的入参决定。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ButtonState {
    /// 按钮变体
    pub variant: ButtonVariant,
    /// 是否禁用交互
    pub disabled: bool,
    /// 按钮文本
    pub label: String,
}

/// 按钮交互状态（由 system 每帧更新）
///
/// 基于 Bevy 内置的 Interaction 组件，拆分为本地状态供 Widget 独立追踪。
/// `just_clicked` 在点击释放后持续一帧，用于触发 ButtonClicked 事件。
#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct ButtonInteraction {
    /// 鼠标是否悬停在按钮上
    pub hovered: bool,
    /// 鼠标是否正在按下
    pub pressed: bool,
    /// 是否刚刚点击释放（持续一帧）
    pub just_clicked: bool,
}
