//! CombatPlugin — 战斗领域 Plugin
//!
//! 注册回合状态机、行动资源、回合生命周期事件与系统。
//!
//! 详见 ADR-021, combat_domain.md, combat_schema.md

use bevy::prelude::*;

use super::components::{ActionPoints, BattlePhase, TurnQueue, TurnSubState};
use super::systems::turn_systems::{
    on_enter_battle, on_enter_defeat, on_enter_turn_end, on_enter_turn_settlement,
    on_enter_turn_start, on_enter_victory, on_unit_action_complete, phase_check,
};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        // ── 注册 Component 类型 ──
        app.register_type::<ActionPoints>();

        // ── 注册 State — TurnSubState 自动随 BattlePhase::Battle 激活 ──
        app.init_state::<BattlePhase>();
        app.init_state::<TurnSubState>();

        // ── 初始化 Resource ──
        app.init_resource::<TurnQueue>();

        // ── 注册 BattlePhase 生命周期 System ──
        app.add_systems(OnEnter(BattlePhase::Battle), on_enter_battle);
        app.add_systems(OnEnter(BattlePhase::Victory), on_enter_victory);
        app.add_systems(OnEnter(BattlePhase::Defeat), on_enter_defeat);

        // ── 注册 TurnSubState 生命周期 System ──
        // TurnStart (OnEnter → 重置资源, 触发 OnTurnStart → PhaseCheck)
        app.add_systems(OnEnter(TurnSubState::TurnStart), on_enter_turn_start);
        // PhaseCheck (Update → 判定可行动作)
        app.add_systems(
            Update,
            phase_check.run_if(in_state(TurnSubState::PhaseCheck)),
        );
        // TurnSettlement (OnEnter → 触发 OnTurnEnd → TurnEnd)
        app.add_systems(
            OnEnter(TurnSubState::TurnSettlement),
            on_enter_turn_settlement,
        );
        // TurnEnd (OnEnter → 切换下一个单位)
        app.add_systems(OnEnter(TurnSubState::TurnEnd), on_enter_turn_end);

        // ── 注册 Observer (Bevy 0.18 Trigger 模式) ──
        // UnitActionComplete → 外部通知行动完成, 进入结算
        app.add_observer(on_unit_action_complete);
    }
}
