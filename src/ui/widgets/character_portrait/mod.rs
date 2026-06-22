//! Module Name: CharacterPortrait Widget — 角色头像复合控件
//!
//! 组合 Panel（容器）+ Panel::Placeholder（占位色块）为一个角色头像控件。
//! 支持四种边框状态（None/Active/Inactive/Selected），分别对应无边框、
//! 当前行动单位（绿色）、非行动单位（灰色）、选中单位（蓝色高亮）。
//!
//! 契约:
//!   输入属性:    border（PortraitBorder）, color（Color）
//!   输出事件:   无（纯显示控件）
//!   本地状态:   PortraitBorder（挂载在容器上）
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md`

mod components;
mod factory;

use bevy::prelude::*;

pub use components::*;
pub use factory::*;

/// CharacterPortraitPlugin — 注册 CharacterPortrait Widget 所需的 Component 类型
///
/// 当前为纯显示控件，无需系统注册。
pub struct CharacterPortraitPlugin;

impl Plugin for CharacterPortraitPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CharacterPortrait>()
            .register_type::<PortraitBorder>();
    }
}
