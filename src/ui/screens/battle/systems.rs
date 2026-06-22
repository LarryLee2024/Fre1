//! 战斗界面系统 — 通过 UiCommand 路由处理按钮点击
//!
//! 使用 ButtonClicked 触发 Observer 和 Commands::trigger
//! 将 BattleAction 映射到领域命令（方案A）。
//!
//! 数据源：UiStore（ViewModel 防火墙），禁止直接查询 Domain 组件。
//!
//! on_dirty_battle_hud — Dirty<BattleHudVm> 消费系统
//! 当 Projection 更新 UiStore.battle_hud 后，标记 Dirty<BattleHudVm>，
//! 此系统消费脏标记并同步 BattleHudData（临时数据桥接）。

use bevy::ecs::observer::On;
use bevy::prelude::*;

use crate::ui::application::UiCommand;
use crate::ui::binding::Dirty;
use crate::ui::primitives::button::events::ButtonClicked;
use crate::ui::view_models::UiStore;
use crate::ui::view_models::battle_hud::{BattleHudData, BattleHudVm};

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
///
/// 单位 ID 从 UiStore.battle_hud.current_unit_id 获取（通过投影更新），
/// 而非直接查询领域组件 TurnQueue。
pub fn on_battle_button_clicked(
    on: On<ButtonClicked>,
    query: Query<&BattleAction>,
    store: Res<UiStore>,
    mut commands: Commands,
) {
    let entity = on.event().entity;
    let Ok(action) = query.get(entity) else {
        // 非战斗按钮，忽略
        return;
    };

    let command = match action {
        BattleAction::EndTurn => {
            let unit_id = if store.battle_hud.current_unit_id != 0 {
                Entity::from_bits(store.battle_hud.current_unit_id).to_string()
            } else {
                String::new()
            };
            UiCommand::EndTurn { unit_id }
        }
    };

    info!(target: "ui", "[Battle] 命令映射: {:?}", command);
    commands.trigger(command);
}

/// Dirty<BattleHudVm> 消费系统
///
/// 当 Projection 更新 UiStore.battle_hud 后标记 Dirty<BattleHudVm>，
/// 此系统消费脏标记并将最新 ViewModel 数据同步到 BattleHudData。
///
/// 当前为骨架实现：消费脏标记但不执行实际同步，
/// 等待完整 UiBinding + Projection 管线就绪。
pub fn on_dirty_battle_hud(
    mut dirty_query: Query<&mut Dirty<BattleHudVm>>,
    _data: ResMut<BattleHudData>,
) {
    for mut dirty in &mut dirty_query {
        if dirty.consume() {
            // TODO[P2][UI][2026-07-21]: 从 UiStore.battle_hud 同步数据到 BattleHudData
            // 等 Projection 系统接入后实现：data.hp_current = store.battle_hud.hp; 等
            info!(target: "ui", "[BattleHud] Dirty 标记已消费，等待 Projection 系统接入");
        }
    }
}
