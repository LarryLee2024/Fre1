//! Focusable、FocusGroup 和焦点导航类型定义
//!
//! Focusable 是我们的业务层包装，附加 UI 上下文信息和组归属。
//! FocusGroup 定义组内导航规则（Grid/Linear/Custom）。
//! TabIndex 控制同组内元素的 Tab 导航顺序。
//!
//! 参见 `docs/06-ui/02-design-system/focus-binding.md` §2

use bevy::prelude::*;

/// Tab 导航顺序组件
///
/// 控制 FocusGroup 内元素的 Tab 导航顺序。
/// 数值越小越优先，同组内按 TabIndex 升序导航。
/// 独立于 Bevy 内置 TabIndex，用于我们自己的焦点导航系统。
///
/// 注意：UI 元素可以同时拥有此 TabIndex 和 Bevy 内置的 TabIndex。
/// 我们使用此组件进行方向键导航排序，
/// Bevy 内置 TabIndex 处理 Tab/Shift+Tab 导航。
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Reflect)]
#[reflect(Component)]
pub struct TabIndex(pub u16);

/// 可聚焦元素标记组件
///
/// 标记 UI 元素可被键盘/手柄选中，并提供组归属和视觉效果信息。
/// 与 TabIndex 组件配合使用，TabIndex 控制同组内的排序。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Focusable {
    /// 焦点组 ID，用于将焦点限制在特定导航组内
    pub group_id: u32,
    /// 聚焦时的视觉效果
    pub focus_style: FocusStyle,
}

/// 焦点视觉效果
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum FocusStyle {
    /// 轮廓高亮（默认，白色边框）
    Outline,
    /// 背景色变化
    Highlight,
    /// 无视觉效果（仅逻辑聚焦，不改变外观）
    None,
}

impl Default for FocusStyle {
    fn default() -> Self {
        Self::Outline
    }
}

/// 焦点组标识组件
///
/// 附加在父级 UI 节点上，声明该子树内的焦点导航规则。
/// 同一组内通过 Tab 键或方向键循环导航。
/// 当某个组被激活时，其内的元素才能接收键盘导航。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct FocusGroup {
    /// 组 ID（与 Focusable.group_id 匹配）
    pub group_id: u32,
    /// 组名称（调试/日志用途）
    pub name: &'static str,
    /// 导航模式
    pub navigation: FocusNavigation,
    /// 是否循环导航（越界时回绕到另一端）
    pub wrap: bool,
}

/// 焦点组内导航模式
#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum FocusNavigation {
    /// 网格导航 — 支持方向键上下左右
    Grid {
        /// 网格列数
        cols: u32,
    },
    /// 线性导航 — 仅支持上下（或左右）移动
    Linear,
    /// 自定义导航规则（使用映射表）
    Custom,
}

impl Default for FocusNavigation {
    fn default() -> Self {
        Self::Grid { cols: 1 }
    }
}
