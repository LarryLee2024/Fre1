//! TabPanel 组件的类型定义
//!
//! 定义 TabPanelState（Widget Contract 的本地状态）和 TabButton（标记组件）。

use bevy::prelude::*;

/// TabPanel 本地状态（Widget Contract Local State）
///
/// 包含当前活动的标签索引和标签总数。
/// Props 字段由 spawn_tab_panel 的入参决定。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct TabPanelState {
    /// 当前激活的标签页索引
    pub active_tab: usize,
    /// 标签总数
    pub tab_count: usize,
}

/// 标签按钮标记组件
///
/// 挂载在 TabPanel 的每个标签按钮上，包含该标签的索引，
/// 用于交互系统识别和高亮当前标签。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct TabButton {
    /// 该标签在 TabPanel 中的索引
    pub index: usize,
}
