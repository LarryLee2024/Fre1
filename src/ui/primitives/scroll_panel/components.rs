//! ScrollPanel 组件的类型定义
//!
//! 定义 ScrollPanelState（Widget Contract 的本地状态）和 ScrollContent（标记组件）。

use bevy::prelude::*;

/// ScrollPanel 本地状态（Widget Contract Local State）
///
/// 包含当前滚动偏移量、内容总高度和容器最大高度。
/// Props 字段由 spawn_scroll_panel 的入参决定。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ScrollPanelState {
    /// 当前垂直滚动偏移量（像素）
    pub scroll_offset: f32,
    /// 内容总高度（像素）
    pub content_height: f32,
    /// 容器最大可见高度（像素）
    pub max_height: f32,
}

/// 滚动内容包裹器标记组件
///
/// 挂载在 ScrollPanel 内部的子节点容器上，scroll 系统通过此组件
/// 应用 translate 偏移实现滚动效果。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ScrollContent;
