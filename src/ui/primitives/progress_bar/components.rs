//! ProgressBar 组件的类型定义
//!
//! 定义 ProgressBarVariant 枚举和 ProgressBarState（Widget Contract 的本地状态）。
//! ProgressBarFill 和 ProgressBarLabel 是内部标记组件，供 system 更新填充条宽度和标签文本。
//!
//! 详见 `docs/06-ui/02-design-system/widget-atoms.md` §3

use bevy::prelude::*;

/// 进度条样式变体
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum ProgressBarVariant {
    /// HP 值显示（绿色）
    Hp,
    /// MP 值显示（蓝色）
    Mp,
    /// 经验值显示（金色）
    Xp,
    /// 通用进度条
    Generic,
}

/// 进度条本地状态（Widget Contract Local State）
///
/// 包含变体、当前值、最大值和标签显示开关。
/// `current`/`maximum` 由外部系统更新，progress_bar_update_system 每帧
/// 读取以重新计算填充条宽度。
///
/// 渲染比例 = clamp(current / max, 0.0, 1.0)，当 max <= 0.0 时渲染为空条。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ProgressBarState {
    /// 进度条变体
    pub variant: ProgressBarVariant,
    /// 当前值
    pub current: f32,
    /// 最大值
    pub maximum: f32,
    /// 是否显示 "HP 80/100" 格式的标签文本
    pub show_label: bool,
}

/// 填充条标记组件
///
/// 标记 ProgressBar 的子实体中充当填充条的节点，
/// 供 progress_bar_update_system 查询并更新其宽度。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ProgressBarFill;

/// 标签标记组件
///
/// 标记 ProgressBar 的子实体中充当标签文本的节点，
/// 供 progress_bar_update_system 查询并更新其文本内容。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ProgressBarLabel;
