//! CombatPlugin — 战斗领域 Plugin
//!
//! 注册宏观状态 BattlePhase、回合队列、回合生命周期事件、Observer 响应，
//! 以及 CombatPipelineDriver 驱动的回合流程管线。
//!
//! 回合内流程（TurnStart → PhaseCheck → UnitAction → TurnSettlement → TurnEnd）
//! 由 `pipeline::CombatPipelineDriver` 驱动，替代原 TurnSubState 状态机。
//!
//! 详见 ADR-021, ADR-044, combat_domain.md, combat_schema.md

use bevy::prelude::*;

use super::components::{ActionPoints, BattlePhase, CombatParticipant, TurnQueue};
use super::integration::event::EventBus;
use super::pipeline::definition::build_turn_pipeline;
use super::pipeline::driver::{
    CombatPipelineDriver, combat_pipeline_driver, on_unit_action_complete,
};
use super::systems::effect_tick_system::on_turn_end_tick_effects;
use super::systems::input_system::{PlayerTurnState, combat_input_system};
use super::systems::turn_systems::{
    on_enter_battle, on_enter_defeat, on_enter_victory, on_turn_end_tick_ability_cooldowns,
    on_turn_start_evaluate_triggers,
};
use crate::app::scenes::GameState;
use crate::core::capabilities::runtime::pipeline::registry::PipelineRegistry;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        // ── 注册 Component 类型 ──
        app.register_type::<ActionPoints>();
        app.register_type::<CombatParticipant>();

        // ── BattlePhase 已转为 SubState，由 GameState::Combat 自动激活 ──
        // 不再需要显式 init_state，SubStates derive 自动处理注册。
        // 详见 ADR-050 §2。

        // ── 初始化 Resource ──
        app.init_resource::<TurnQueue>();
        app.init_resource::<EventBus>();
        app.init_resource::<CombatPipelineDriver>();

        // ── 注册 BattlePhase 生命周期 System ──
        app.add_systems(OnEnter(BattlePhase::Battle), on_enter_battle);
        app.add_systems(OnEnter(BattlePhase::Victory), on_enter_victory);
        app.add_systems(OnEnter(BattlePhase::Defeat), on_enter_defeat);

        // ── 注册 Pipeline 驾驶员 Update System ──
        app.add_systems(
            Update,
            combat_pipeline_driver.run_if(in_state(GameState::Combat)),
        );

        // ── Input System ──
        app.init_resource::<PlayerTurnState>();
        app.add_systems(
            Update,
            combat_input_system.run_if(in_state(GameState::Combat)),
        );

        // ── 注册 Observer (Bevy 0.19 Trigger 模式) ──
        // UnitActionComplete → 恢复驾驶员，跳转到 TurnSettlement
        app.add_observer(on_unit_action_complete);
        // OnTurnEnd → 推进 Effect 计时与周期 Tick
        app.add_observer(on_turn_end_tick_effects);
        // OnTurnEnd → 推进 Ability 冷却计时
        app.add_observer(on_turn_end_tick_ability_cooldowns);
        // OnTurnStart → 评估单位触发器
        app.add_observer(on_turn_start_evaluate_triggers);

        // ── 注册回合管线定义到 PipelineRegistry ──
        let mut registry = app.world_mut().resource_mut::<PipelineRegistry>();
        registry.register(build_turn_pipeline());
    }
}
