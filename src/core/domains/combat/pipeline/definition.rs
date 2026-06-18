//! Combat Pipeline Definition — 战斗回合管线定义
//!
//! 定义 TurnPipeline 的五阶段流程。PipelineDefinition 注册到 PipelineRegistry，
//! 由 CombatPipelineDriver 按步骤驱动执行。

use crate::core::capabilities::runtime::pipeline::foundation::{
    FailureStrategy, PipelineDefinition, PipelineStage, PipelineStep,
};

/// 战斗回合管线的唯一标识。
pub const COMBAT_TURN_PIPELINE_ID: &str = "combat.turn";

/// 创建战斗回合管线定义。
///
/// 五个阶段对应战斗单位一次完整回合的生命周期。
pub fn build_turn_pipeline() -> PipelineDefinition {
    PipelineDefinition::new(COMBAT_TURN_PIPELINE_ID)
        .stage(
            PipelineStage::new("turn_start")
                .step(PipelineStep::System("turn_start".to_string()))
                .on_failure(FailureStrategy::Abort),
        )
        .stage(
            PipelineStage::new("phase_check")
                .step(PipelineStep::System("phase_check".to_string()))
                .on_failure(FailureStrategy::Abort),
        )
        .stage(
            PipelineStage::new("unit_action")
                .step(PipelineStep::System("unit_action".to_string()))
                .on_failure(FailureStrategy::Abort),
        )
        .stage(
            PipelineStage::new("turn_settlement")
                .step(PipelineStep::System("turn_settlement".to_string()))
                .on_failure(FailureStrategy::Abort),
        )
        .stage(
            PipelineStage::new("turn_end")
                .step(PipelineStep::System("turn_end".to_string()))
                .on_failure(FailureStrategy::Abort),
        )
}
