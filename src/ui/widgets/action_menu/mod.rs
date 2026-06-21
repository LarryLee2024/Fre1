//! Module Name: ActionMenu Widget — 战斗行动菜单复合控件
//!
//! 组合 List / Button 两个原子组件为一个垂直排列的行动菜单。
//! 包含 5 个固定行动按钮（Attack, Defend, Skill, Item, Wait），
//! 每个按钮标记对应的 ActionType 以便外部 Observer 路由交互事件。
//!
//! Contract:
//!   Props (input):    actions（通过 ActionMenuState，运行时更新）
//!   Events (output):  按钮交互由 ButtonSystem 处理，ActionType 标记供 Observer 识别
//!   Local State:      ActionMenuState（actions 列表，enabled 状态控制）
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md`

pub mod components;
pub mod factory;
pub mod systems;

use bevy::prelude::*;

use self::components::{ActionMenuState, ActionType};
use self::systems::{action_menu_sync_system, on_action_menu_button_clicked};

/// ActionMenuPlugin — 注册 ActionMenu Widget 所需的 Component/System
pub struct ActionMenuPlugin;

impl Plugin for ActionMenuPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ActionMenuState>()
            .register_type::<ActionType>()
            .add_systems(Update, action_menu_sync_system)
            .add_observer(on_action_menu_button_clicked);
    }
}
