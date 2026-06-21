//! 战斗界面系统 — 通过 UiCommand 路由处理按钮点击
//!
//! 使用 ButtonClicked 触发 Observer 和 Commands::trigger
//! 将 BattleAction 映射到领域命令（方案A）。

use bevy::ecs::observer::On;
use bevy::prelude::*;

use crate::ui::application::UiCommand;
use crate::ui::primitives::button::events::ButtonClicked;

/// 战斗按钮操作标识
///
/// 作为 Component 挂载到战斗界面的按钮上。Observer
/// 查询此组件来确定哪个按钮被点击。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub enum BattleAction {
    /// 结束当前回合
    EndTurn,
}

/// Observer：处理战斗按钮点击，映射到 UiCommand
///
/// 当原语层的 `button_interaction_system` 通过 Commands::trigger 触发
/// `ButtonClicked` 事件时，检查按钮实体是否携带 `BattleAction` 组件
/// 并分发对应的 UiCommand。
pub fn on_battle_button_clicked(
    on: On<ButtonClicked>,
    query: Query<&BattleAction>,
    mut commands: Commands,
) {
    let entity = on.event().entity;
    let Ok(action) = query.get(entity) else {
        // Not a battle button, ignore
        return;
    };

    let command = match action {
        BattleAction::EndTurn => UiCommand::EndTurn,
    };

    info!(target: "ui", "[Battle] 命令映射: {:?}", command);
    commands.trigger(command);
}
