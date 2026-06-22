//! 模块名: CharacterStatusPanel Widget — 角色状态面板有机体
//!
//! 组合 CharacterPortrait / Text / ProgressBar 为一个完整的角色状态展示区域。
//! 垂直排列：顶部为肖像 + 名称 + HP 条，中部为 MP 条和 AP 条，
//! 底部为可选的状态文本。
//!
//! MVP 实现为玩家角色全尺寸模式。敌方精简模式和 BuffIcon 行
//! 在后续迭代中添加。
//!
//! 契约:
//!   输入属性: name, hp/mp/ap current/max, status_text, is_active
//!             （通过 CharacterStatusPanelState）
//!   输出事件: 无（事件委托给子 CharacterPortrait 的点击交互）
//!   本地状态: CharacterStatusPanelState（name, hp/mp/ap, status_text, is_active）
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md` §3.2

pub mod components;
pub mod factory;

use bevy::prelude::*;

use self::components::{
    CharacterStatusPanel, CharacterStatusPanelNameLabel, CharacterStatusPanelState,
    CharacterStatusPanelStatusLabel,
};

/// CharacterStatusPanelPlugin — 注册 CharacterStatusPanel Widget 所需的 Component 类型
///
/// 当前为纯显示控件，无独立系统 ——
/// 子 CharacterPortrait 和 ProgressBar 的运行由各自对应的 Plugin 负责。
pub struct CharacterStatusPanelPlugin;

impl Plugin for CharacterStatusPanelPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CharacterStatusPanel>()
            .register_type::<CharacterStatusPanelState>()
            .register_type::<CharacterStatusPanelNameLabel>()
            .register_type::<CharacterStatusPanelStatusLabel>();
    }
}
