//! Combat Pipeline Driver — 回合管线驾驶员
//!
//! 使用 CombatPipelineDriver Resource 逐步骤驱动回合管线。
//! 在 UnitAction 步骤暂停，等待 UnitActionComplete 事件后恢复。
//!
//! # 设计说明
//!
//! 不同于 `execute_pipeline`（同步执行全部步骤），驾驶员模式适用于
//! 包含异步等待（玩家/AI 输入）的回合流程。每一帧执行一个步骤，
//! 在 UnitAction 处暂停，由 Observer 恢复推进。

use bevy::prelude::*;

use crate::core::capabilities::runtime::pipeline::foundation::{
    PipelineContext, PipelineDefinition, PipelineState,
};
use crate::core::capabilities::runtime::pipeline::registry::PipelineRegistry;

use super::definition::COMBAT_TURN_PIPELINE_ID;
use super::steps::{
    PhaseCheckResult, TurnEndResult, step_phase_check, step_turn_end, step_turn_settlement,
    step_turn_start, step_unit_action,
};
use crate::core::domains::combat::components::{
    ActionPoints, BattlePhase, CombatParticipant, Dead, TurnQueue,
};
use crate::core::domains::combat::events::UnitActionComplete;

/// 回合管线驾驶员 Resource。
#[derive(Resource)]
pub struct CombatPipelineDriver {
    /// 当前执行中的管线状态
    state: PipelineState,
    /// 是否暂停（等待 UnitAction 完成）
    paused: bool,
}

impl CombatPipelineDriver {
    /// 创建新的驾驶员。
    pub fn new() -> Self {
        Self {
            state: PipelineState {
                pipeline_id: COMBAT_TURN_PIPELINE_ID.to_string(),
                current_stage_index: 0,
                current_step_index: 0,
                context: PipelineContext::new(COMBAT_TURN_PIPELINE_ID),
                completed: false,
            },
            paused: false,
        }
    }

    /// 开始新单位的回合（重置状态，从 TurnStart 开始）。
    pub fn start_turn(&mut self) {
        self.state.current_stage_index = 0;
        self.state.current_step_index = 0;
        self.state.completed = false;
        self.state.context = PipelineContext::new("combat.turn");
        self.paused = false;
    }

    /// 检查驾驶员是否在活跃驱动中。
    pub fn is_driving(&self) -> bool {
        !self.state.completed && !self.paused
    }

    /// 检查驾驶员是否暂停（等待外部输入）。
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// 获取当前管线 ID。
    pub fn pipeline_id(&self) -> &str {
        &self.state.pipeline_id
    }

    #[cfg(test)]
    pub(crate) fn force_pause(&mut self) {
        self.paused = true;
    }
}

impl Default for CombatPipelineDriver {
    fn default() -> Self {
        Self::new()
    }
}

/// 在 PipelineDefinition 中按名称查找阶段索引。
fn find_stage_index(def: &PipelineDefinition, name: &str) -> Option<usize> {
    def.stages.iter().position(|s| s.name == name)
}

/// 驾驶员 Update 系统：每帧执行一个步骤。
///
/// 仅在驾驶员活跃且未暂停时运行。
#[tracing::instrument(skip_all, target = "combat")]
pub(crate) fn combat_pipeline_driver(
    mut driver: ResMut<CombatPipelineDriver>,
    pipeline_registry: Res<PipelineRegistry>,
    mut turn_queue: ResMut<TurnQueue>,
    mut commands: Commands,
    mut ap_query: Query<&mut ActionPoints>,
    combatant_query: Query<&CombatParticipant>,
    dead_query: Query<&CombatParticipant, With<Dead>>,
    mut battle_phase: ResMut<NextState<BattlePhase>>,
) {
    // 不活跃或暂停时跳过
    if !driver.is_driving() {
        return;
    }

    // 查找管线定义
    let def = match pipeline_registry.get(&driver.state.pipeline_id) {
        Some(d) => d.clone(),
        None => {
            tracing::warn!(target: "combat", 
                event = "combat.pipeline.not_found",
                pipeline_id = %driver.state.pipeline_id,
                "管线 '{}' 未在 Registry 中找到",
                driver.state.pipeline_id
            );
            driver.state.completed = true;
            return;
        }
    };

    // 获取当前阶段
    let stage = match def.stages.get(driver.state.current_stage_index) {
        Some(s) => s,
        None => {
            driver.state.completed = true;
            return;
        }
    };

    // 获取当前步骤
    let step = match stage.steps.get(driver.state.current_step_index) {
        Some(s) => s.clone(),
        None => {
            // 当前阶段步骤耗尽 → 进入下一阶段
            driver.state.current_stage_index += 1;
            driver.state.current_step_index = 0;
            return; // 下一帧执行下一阶段
        }
    };

    // ── 根据步骤名称分发执行 ──
    match step.name() {
        "turn_start" => {
            step_turn_start(&mut commands, &turn_queue, &mut ap_query);
            advance_step_or_stage(&mut driver, &def);
        }

        "phase_check" => {
            let result = step_phase_check(&turn_queue, &ap_query);
            match result {
                PhaseCheckResult::HasActions => {
                    // 跳转到 UnitAction 阶段
                    if let Some(idx) = find_stage_index(&def, "unit_action") {
                        driver.state.current_stage_index = idx;
                        driver.state.current_step_index = 0;
                        driver.paused = true; // 暂停等待输入
                    } else {
                        advance_step_or_stage(&mut driver, &def);
                    }
                }
                PhaseCheckResult::Idle => {
                    // 跳转到 TurnSettlement 阶段
                    if let Some(idx) = find_stage_index(&def, "turn_settlement") {
                        driver.state.current_stage_index = idx;
                        driver.state.current_step_index = 0;
                    } else {
                        advance_step_or_stage(&mut driver, &def);
                    }
                }
            }
        }

        "unit_action" => {
            step_unit_action(&mut commands, &turn_queue);
            // 不应自动推进 — PhaseCheck 已设置暂停状态
            // 如意外到达此处（无暂停），设置暂停
            driver.paused = true;
        }

        "turn_settlement" => {
            step_turn_settlement(&mut commands, &turn_queue);
            advance_step_or_stage(&mut driver, &def);
        }

        "turn_end" => {
            let result = step_turn_end(
                &mut commands,
                &mut turn_queue,
                &combatant_query,
                &dead_query,
            );
            match result {
                TurnEndResult::BattleOver => {
                    battle_phase.set(BattlePhase::Victory);
                    driver.state.completed = true;
                }
                TurnEndResult::Continue => {
                    // 重新开始管线，进入下一单位的回合
                    driver.start_turn();
                }
            }
        }

        _ => {
            advance_step_or_stage(&mut driver, &def);
        }
    }
}

/// 推进到下一步骤或下一阶段。
fn advance_step_or_stage(driver: &mut CombatPipelineDriver, def: &PipelineDefinition) {
    let stage = match def.stages.get(driver.state.current_stage_index) {
        Some(s) => s,
        None => {
            driver.state.completed = true;
            return;
        }
    };

    if driver.state.current_step_index + 1 < stage.steps.len() {
        driver.state.current_step_index += 1;
    } else if driver.state.current_stage_index + 1 < def.stages.len() {
        driver.state.current_stage_index += 1;
        driver.state.current_step_index = 0;
    } else {
        driver.state.completed = true;
    }
}

// ═══════════════════════════════════════════════════════════════════════
// UnitActionComplete Observer
// ═══════════════════════════════════════════════════════════════════════

/// Observer: 监听 `UnitActionComplete`，恢复驾驶员并跳转到 TurnSettlement。
///
/// 外部系统（Ability/Execution）在行动完成后发射此事件。
pub(crate) fn on_unit_action_complete(
    trigger: On<'_, '_, UnitActionComplete>,
    turn_queue: Res<TurnQueue>,
    mut driver: ResMut<CombatPipelineDriver>,
    pipeline_registry: Res<PipelineRegistry>,
) {
    let Some(current) = turn_queue.current() else {
        return;
    };

    let event = trigger.event();
    if event.unit != current.entity {
        return;
    }

    debug!(target: "combat", 
        "[Combat] UnitAction: 单位 {:?} 行动完成，恢复管线执行",
        event.unit
    );

    // 查找管线定义，跳转到 turn_settlement
    let def = match pipeline_registry.get(&driver.state.pipeline_id) {
        Some(d) => d.clone(),
        None => {
            tracing::warn!(target: "combat", 
                event = "combat.pipeline.missing_resume",
                pipeline_id = %driver.state.pipeline_id,
                "管线 '{}' 未找到，无法恢复执行",
                driver.state.pipeline_id
            );
            return;
        }
    };

    if let Some(idx) = find_stage_index(&def, "turn_settlement") {
        driver.state.current_stage_index = idx;
        driver.state.current_step_index = 0;
        driver.paused = false;
    } else {
        // fallback: advance one step and unpause
        advance_step_or_stage(&mut driver, &def);
        driver.paused = false;
    }
}
