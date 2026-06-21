//! Module Name: Primitives — UI 原语层
//!
//! 基础 UI 组件，每个有独立的 Plugin、Factory、Component 和 System。
//! 只通过 Factory 创建，禁止直接 spawn Node。
//! 本层是唯一可以依赖基础 UI 实现的地方。
//!
//! 业务 Widget（src/ui/widgets/）只能引用本层组件，
//! 不能绕过本层直接操作 Node/Button/Interaction 等 Bevy UI 原语。
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §3

pub mod button;
pub mod list;
pub mod modal;
pub mod panel;
pub mod progress_bar;
pub mod scroll_panel;
pub mod select_list;
pub mod tab_panel;
pub mod text;
pub mod text_input;
pub mod toggle;

use bevy::prelude::*;

use self::button::ButtonPlugin;
use self::list::ListPlugin;
use self::modal::ModalPlugin;
use self::panel::PanelPlugin;
use self::progress_bar::ProgressBarPlugin;
use self::scroll_panel::ScrollPanelPlugin;
use self::select_list::SelectListPlugin;
use self::tab_panel::TabPanelPlugin;
use self::text::TextPlugin;
use self::text_input::TextInputPlugin;
use self::toggle::TogglePlugin;

/// PrimitivesPlugin — 注册所有 UI 原语 Plugin
///
/// 在 ThemePlugin 之后、Widget/Screen Plugin 之前注册。
pub struct PrimitivesPlugin;

impl Plugin for PrimitivesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ButtonPlugin,
            ListPlugin,
            ModalPlugin,
            PanelPlugin,
            ProgressBarPlugin,
            ScrollPanelPlugin,
            SelectListPlugin,
            TabPanelPlugin,
            TextPlugin,
            TextInputPlugin,
            TogglePlugin,
        ));
    }
}
