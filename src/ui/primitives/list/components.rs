//! List 组件的类型定义
//!
//! 定义 ListVariant 枚举和 ListState（Widget Contract 的本地状态）。
//!
//! 详见 `docs/06-ui/02-design-system/widget-atoms.md` §5

use bevy::prelude::*;

/// 列表排列变体
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum ListVariant {
    /// 垂直列表（FlexDirection::Column）
    Vertical,
    /// 水平列表（FlexDirection::Row）
    Horizontal,
    /// 虚拟滚动列表（Column + overflow clip）
    Virtual,
}

/// 列表本地状态（Widget Contract Local State）
///
/// 包含变体和间距配置。
/// Props 字段由 spawn_list 的入参决定。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ListState {
    /// 列表排列变体
    pub variant: ListVariant,
    /// 列表项之间的间距（像素）
    pub spacing: f32,
}
