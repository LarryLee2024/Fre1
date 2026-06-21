//! Pipeline Executor — 管线执行引擎
//!
//! 提供管线执行的核心编排逻辑：按顺序遍历阶段和步骤，
//! 处理失败策略（Abort/Skip/Retry）和条件分支。
//!
//! 详见 docs/04-data/infrastructure/pipeline_schema.md §2。

use crate::core::capabilities::runtime::pipeline::foundation::{
    ExecutionLogEntry, FailureStrategy, PipelineContext, PipelineDefinition, PipelineError,
    PipelineStage, PipelineStep, StepResult,
};

/// 步骤执行函数签名。
///
/// 调用方提供此函数来实际执行一个步骤的逻辑。
/// - `step_name`: 步骤名称（System/Rule 的 ID）
/// - `context`: 当前管线上下文（可读写）
/// - `stage_name`: 当前阶段名称
pub type StepExecutor =
    fn(step_name: &str, context: &mut PipelineContext, stage_name: &str) -> StepResult;

/// 执行整条管线。
///
/// 遍历所有阶段和步骤，按失败策略处理执行错误。
///
/// # 流程
/// 1. 遍历每个阶段
/// 2. 阶段内按顺序遍历每个步骤
/// 3. 对每个步骤调用 `executor` 函数
/// 4. 根据阶段的 `on_failure` 策略处理步骤失败
/// 5. 累积执行日志到上下文
///
/// # Errors
/// - Abort 策略下任何步骤失败 → PipelineError::StepFailed → 终止
/// - Retry 策略全部重试失败 → PipelineError::StepFailed
pub fn execute_pipeline(
    definition: &PipelineDefinition,
    context: &mut PipelineContext,
    executor: StepExecutor,
) -> Result<(), PipelineError> {
    for stage in &definition.stages {
        // 管线中止检查：aborted 由外部触发（如战斗结束），提前终止所有阶段
        if context.aborted {
            return Err(PipelineError::Aborted {
                reason: context
                    .abort_reason
                    .clone()
                    .unwrap_or_else(|| "unknown reason".into()),
            });
        }

        let result = execute_stage(stage, context, executor)?;

        // Skipped 阶段记录到上下文日志，用于回放时还原执行路径
        if matches!(result, ExecutionStageResult::Skipped) {
            context.log(ExecutionLogEntry::new(
                &stage.name,
                "__stage__",
                StepResult::Skipped,
            ));
        }
    }

    Ok(())
}

enum ExecutionStageResult {
    Completed,
    Skipped,
}

/// 执行单个阶段的所有步骤。
fn execute_stage(
    stage: &PipelineStage,
    context: &mut PipelineContext,
    executor: StepExecutor,
) -> Result<ExecutionStageResult, PipelineError> {
    if stage.steps.is_empty() {
        return Ok(ExecutionStageResult::Skipped);
    }

    for step in &stage.steps {
        let result = execute_step(stage, step, context, executor)?;

        context.log(ExecutionLogEntry::new(&stage.name, step.name(), result));
    }

    Ok(ExecutionStageResult::Completed)
}

/// 执行单个步骤（含条件分支和子管线展开）。
fn execute_step(
    stage: &PipelineStage,
    step: &PipelineStep,
    context: &mut PipelineContext,
    executor: StepExecutor,
) -> Result<StepResult, PipelineError> {
    match step {
        PipelineStep::System(step_name)
        | PipelineStep::Rule(step_name)
        | PipelineStep::SubPipeline(step_name) => {
            execute_atomic_step(stage, step_name, context, executor)
        }
        PipelineStep::Conditional {
            condition: _,
            if_true,
            if_false,
        } => {
            // 条件分支：评估条件，选择分支执行
            // 当前简化实现——由 executor 函数评估条件
            // executor 返回 Success 表示条件为真，Failure 表示条件为假
            let condition_result = executor("__condition__", context, &stage.name);
            match condition_result {
                StepResult::Success => {
                    // 条件为真 → 执行 if_true 分支
                    execute_single_step(stage, if_true.as_ref(), context, executor)
                }
                _ => {
                    // 条件为假 → 执行 if_false 分支
                    execute_single_step(stage, if_false.as_ref(), context, executor)
                }
            }
        }
    }
}

/// 执行单个步骤（展开可能的子步骤）。
fn execute_single_step(
    stage: &PipelineStage,
    step: &PipelineStep,
    context: &mut PipelineContext,
    executor: StepExecutor,
) -> Result<StepResult, PipelineError> {
    match step {
        PipelineStep::SubPipeline(sub_id) => {
            // 子管线在当前步骤中展开，记录为一次执行
            let result = executor(sub_id, context, &stage.name);
            Ok(result)
        }
        other => execute_step(stage, other, context, executor),
    }
}

/// 执行原子步骤（System/Rule）并处理失败策略。
fn execute_atomic_step(
    stage: &PipelineStage,
    step_name: &str,
    context: &mut PipelineContext,
    executor: StepExecutor,
) -> Result<StepResult, PipelineError> {
    let max_attempts = match stage.on_failure {
        FailureStrategy::Retry { max_retries } => max_retries as u32 + 1,
        _ => 1,
    };

    for attempt in 0..max_attempts {
        let result = executor(step_name, context, &stage.name);

        match result {
            StepResult::Success => return Ok(StepResult::Success),
            StepResult::Failure(ref err_ctx) => {
                if attempt + 1 < max_attempts {
                    // Retry: 继续下一次尝试
                    continue;
                }
                // 所有重试耗尽或非 Retry 策略
                match stage.on_failure {
                    FailureStrategy::Abort => {
                        return Err(PipelineError::StepFailed {
                            stage: stage.name.clone(),
                            step: step_name.into(),
                            detail: err_ctx.source.clone(),
                        });
                    }
                    FailureStrategy::SkipAndContinue => {
                        return Ok(StepResult::Failure(err_ctx.clone()));
                    }
                    FailureStrategy::Retry { .. } => {
                        return Err(PipelineError::StepFailed {
                            stage: stage.name.clone(),
                            step: step_name.into(),
                            detail: format!("all retries exhausted: {}", err_ctx.source),
                        });
                    }
                }
            }
            StepResult::Skipped => return Ok(StepResult::Skipped),
        }
    }

    Ok(StepResult::Success)
}

/// 验证管线定义是否合法。
///
/// 检查：
/// - 阶段名称不能为空
/// - 每个阶段至少有一个步骤（除非 skippable）
/// - 没有重复的阶段名称
pub fn validate_pipeline(definition: &PipelineDefinition) -> Result<(), PipelineError> {
    if definition.id.is_empty() {
        return Err(PipelineError::MissingContext {
            key: "pipeline id is empty".into(),
        });
    }

    let mut seen_names = Vec::new();
    for stage in &definition.stages {
        if stage.name.is_empty() {
            return Err(PipelineError::MissingContext {
                key: "stage name must not be empty".into(),
            });
        }

        if seen_names.contains(&stage.name) {
            return Err(PipelineError::MissingContext {
                key: format!("duplicate stage name: {}", stage.name),
            });
        }
        seen_names.push(stage.name.clone());

        if stage.steps.is_empty() && !stage.skippable {
            return Err(PipelineError::MissingContext {
                key: format!("stage '{}' has no steps and is not skippable", stage.name),
            });
        }
    }

    Ok(())
}
