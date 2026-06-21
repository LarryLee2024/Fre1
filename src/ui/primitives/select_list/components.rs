//! SelectList 组件的类型定义
//!
//! 定义 SelectListState（Widget Contract 的本地状态）和 SelectListItem。
//!
//! 详见 `docs/06-ui/02-design-system/widget-atoms.md` §SelectList

use bevy::prelude::*;

/// SelectList 本地状态（Widget Contract Local State）
///
/// 包含选中索引和可选条目列表。
/// Props 字段由 spawn_select_list 的入参决定。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct SelectListState {
    /// 当前选中的条目索引
    pub selected_index: usize,
    /// 所有可选条目（本地化 Key）
    pub items: Vec<&'static str>,
}

/// SelectList 条目标记组件
///
/// 挂载在每个 SelectList 的条目按钮上，用于系统识别和索引定位。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct SelectListItem {
    /// 该条目在列表中的索引
    pub index: usize,
}
