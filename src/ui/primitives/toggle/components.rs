//! Toggle 组件的类型定义
//!
//! 定义 ToggleState（Widget Contract 的本地状态）和 ToggleIndicator（标记组件）。
//!
//! 详见 `docs/06-ui/02-design-system/widget-atoms.md` §Toggle

use bevy::prelude::*;

/// Toggle 本地状态（Widget Contract Local State）
///
/// 包含选中状态、标签文本 Key 和启用状态。
/// Props 字段由 spawn_toggle 的入参决定。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ToggleState {
    /// 是否选中
    pub checked: bool,
    /// 本地化标签 Key
    pub label_key: &'static str,
    /// 是否启用交互
    pub enabled: bool,
}

/// Toggle 指示器标记组件
///
/// 挂载在 Toggle 右侧的点击区域上，用于交互系统识别。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ToggleIndicator;
