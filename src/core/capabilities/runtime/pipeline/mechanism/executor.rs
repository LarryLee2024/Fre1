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
        // 检查管线是否已被中止
        if context.aborted {
            return Err(PipelineError::Aborted(
                context
                    .abort_reason
                    .clone()
                    .unwrap_or_else(|| "unknown reason".into()),
            ));
        }

        let result = execute_stage(stage, context, executor)?;

        // 如果阶段被跳过，记录到上下文
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
            StepResult::Failure(ref msg) => {
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
                            detail: msg.clone(),
                        });
                    }
                    FailureStrategy::SkipAndContinue => {
                        return Ok(StepResult::Failure(msg.clone()));
                    }
                    FailureStrategy::Retry { .. } => {
                        return Err(PipelineError::StepFailed {
                            stage: stage.name.clone(),
                            step: step_name.into(),
                            detail: format!("all retries exhausted: {}", msg),
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
        return Err(PipelineError::MissingContext("pipeline id is empty".into()));
    }

    let mut seen_names = Vec::new();
    for stage in &definition.stages {
        if stage.name.is_empty() {
            return Err(PipelineError::MissingContext(
                "stage name must not be empty".into(),
            ));
        }

        if seen_names.contains(&stage.name) {
            return Err(PipelineError::MissingContext(format!(
                "duplicate stage name: {}",
                stage.name
            )));
        }
        seen_names.push(stage.name.clone());

        if stage.steps.is_empty() && !stage.skippable {
            return Err(PipelineError::MissingContext(format!(
                "stage '{}' has no steps and is not skippable",
                stage.name
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::capabilities::runtime::pipeline::foundation::{
        PipelineDefinition, PipelineStage, PipelineStep,
    };

    /// 模拟执行器：检查步骤名称并返回成功。
    fn mock_success_executor(
        _step_name: &str,
        _context: &mut PipelineContext,
        _stage_name: &str,
    ) -> StepResult {
        StepResult::Success
    }

    /// 模拟执行器：根据步骤名称返回失败或成功。
    fn mock_selective_executor(
        step_name: &str,
        _context: &mut PipelineContext,
        _stage_name: &str,
    ) -> StepResult {
        if step_name == "fail_step" {
            StepResult::Failure("intentional failure".into())
        } else {
            StepResult::Success
        }
    }

    /// 模拟执行器：记录步骤执行到上下文。
    fn mock_logging_executor(
        step_name: &str,
        context: &mut PipelineContext,
        stage_name: &str,
    ) -> StepResult {
        context.set_stage_data(
            format!("{}.{}", stage_name, step_name),
            "executed".to_string(),
        );
        StepResult::Success
    }

    #[test]
    fn unit_030_execute_empty_pipeline() {
        let def = PipelineDefinition::new("empty");
        let mut ctx = PipelineContext::new("empty");
        let result = execute_pipeline(&def, &mut ctx, mock_success_executor);
        assert!(result.is_ok());
    }

    #[test]
    fn unit_031_execute_single_stage() {
        let def = PipelineDefinition::new("test")
            .stage(PipelineStage::new("stage_1").step(PipelineStep::Rule("step_a".into())));
        let mut ctx = PipelineContext::new("test");
        let result = execute_pipeline(&def, &mut ctx, mock_success_executor);
        assert!(result.is_ok());
    }

    #[test]
    fn unit_032_execute_multi_stage() {
        let def = PipelineDefinition::new("test")
            .stage(PipelineStage::new("generate").step(PipelineStep::Rule("calc_base".into())))
            .stage(PipelineStage::new("modify").step(PipelineStep::Rule("apply_modifiers".into())))
            .stage(PipelineStage::new("execute").step(PipelineStep::System("apply_damage".into())));
        let mut ctx = PipelineContext::new("test");
        let result = execute_pipeline(&def, &mut ctx, mock_success_executor);
        assert!(result.is_ok());
    }

    #[test]
    fn unit_033_execute_logs_steps() {
        let def = PipelineDefinition::new("test")
            .stage(PipelineStage::new("s1").step(PipelineStep::Rule("r1".into())));
        let mut ctx = PipelineContext::new("test");
        let _ = execute_pipeline(&def, &mut ctx, mock_success_executor);
        assert_eq!(ctx.execution_log.len(), 1);
        assert_eq!(ctx.execution_log[0].stage, "s1");
        assert_eq!(ctx.execution_log[0].step, "r1");
    }

    #[test]
    fn unit_034_execute_abort_on_failure() {
        let def = PipelineDefinition::new("test")
            .stage(
                PipelineStage::new("s1")
                    .step(PipelineStep::Rule("ok_step".into()))
                    .step(PipelineStep::Rule("fail_step".into())),
            )
            .stage(PipelineStage::new("s2").step(PipelineStep::Rule("should_not_reach".into())));
        let mut ctx = PipelineContext::new("test");
        let result = execute_pipeline(&def, &mut ctx, mock_selective_executor);
        assert!(result.is_err());
        match result {
            Err(PipelineError::StepFailed { stage, step, .. }) => {
                assert_eq!(stage, "s1");
                assert_eq!(step, "fail_step");
            }
            _ => panic!("expected StepFailed"),
        }
        // s2 should not be reached
        assert!(!ctx.stage_data.contains_key("s2.should_not_reach"));
    }

    #[test]
    fn unit_035_execute_skip_and_continue() {
        let def = PipelineDefinition::new("test").stage(
            PipelineStage::new("s1")
                .step(PipelineStep::Rule("ok_step".into()))
                .step(PipelineStep::Rule("fail_step".into()))
                .on_failure(FailureStrategy::SkipAndContinue),
        );
        let mut ctx = PipelineContext::new("test");
        let result = execute_pipeline(&def, &mut ctx, mock_selective_executor);
        assert!(result.is_ok()); // SkipAndContinue should not fail
    }

    #[test]
    fn unit_036_execute_with_context_passing() {
        let def = PipelineDefinition::new("test")
            .stage(PipelineStage::new("s1").step(PipelineStep::Rule("step_a".into())));
        let mut ctx = PipelineContext::new("test");
        let _ = execute_pipeline(&def, &mut ctx, mock_logging_executor);
        assert_eq!(ctx.get_stage_data("s1.step_a"), Some(&"executed".into()));
    }

    #[test]
    fn unit_037_execute_aborted_pipeline() {
        let def = PipelineDefinition::new("test")
            .stage(PipelineStage::new("s1").step(PipelineStep::Rule("step_a".into())));
        let mut ctx = PipelineContext::new("test");
        ctx.abort("external cancellation");
        let result = execute_pipeline(&def, &mut ctx, mock_success_executor);
        assert!(result.is_err());
        assert!(matches!(result, Err(PipelineError::Aborted(_))));
    }

    #[test]
    fn unit_038_execute_skippable_empty_stage() {
        let def = PipelineDefinition::new("test")
            .stage(PipelineStage::new("optional").skippable())
            .stage(PipelineStage::new("main").step(PipelineStep::Rule("work".into())));
        let mut ctx = PipelineContext::new("test");
        let result = execute_pipeline(&def, &mut ctx, mock_success_executor);
        assert!(result.is_ok());
    }

    #[test]
    fn unit_039_execute_conditional_branch() {
        let def = PipelineDefinition::new("test").stage(PipelineStage::new("decision").step(
            PipelineStep::Conditional {
                condition: "has_shield".into(),
                if_true: Box::new(PipelineStep::Rule("apply_shield".into())),
                if_false: Box::new(PipelineStep::Rule("apply_damage".into())),
            },
        ));

        // Condition true: executor returns Success for __condition__
        fn condition_true_executor(
            step_name: &str,
            _ctx: &mut PipelineContext,
            _stage: &str,
        ) -> StepResult {
            if step_name == "__condition__" {
                StepResult::Success
            } else if step_name == "apply_shield" {
                StepResult::Success
            } else {
                StepResult::Failure("unexpected".into())
            }
        }

        let mut ctx = PipelineContext::new("test");
        let result = execute_pipeline(&def, &mut ctx, condition_true_executor);
        assert!(result.is_ok());
    }

    // ── validate_pipeline ───────────────────────────────────

    #[test]
    fn unit_040_validate_empty_id() {
        let def = PipelineDefinition::new("");
        assert!(validate_pipeline(&def).is_err());
    }

    #[test]
    fn unit_041_validate_empty_stage_name() {
        let def = PipelineDefinition::new("test")
            .stage(PipelineStage::new("").step(PipelineStep::Rule("r".into())));
        assert!(validate_pipeline(&def).is_err());
    }

    #[test]
    fn unit_042_validate_non_skippable_empty_stage() {
        let def = PipelineDefinition::new("test").stage(PipelineStage::new("empty"));
        assert!(validate_pipeline(&def).is_err());
    }

    #[test]
    fn unit_043_validate_skippable_empty_stage_ok() {
        let def = PipelineDefinition::new("test").stage(PipelineStage::new("optional").skippable());
        assert!(validate_pipeline(&def).is_ok());
    }

    #[test]
    fn unit_044_validate_duplicate_stage_names() {
        let def = PipelineDefinition::new("test")
            .stage(PipelineStage::new("dup").step(PipelineStep::Rule("a".into())))
            .stage(PipelineStage::new("dup").step(PipelineStep::Rule("b".into())));
        assert!(validate_pipeline(&def).is_err());
    }

    #[test]
    fn unit_045_validate_valid_pipeline() {
        let def = PipelineDefinition::new("combat")
            .stage(PipelineStage::new("generate").step(PipelineStep::Rule("calc".into())))
            .stage(PipelineStage::new("execute").step(PipelineStep::System("apply".into())));
        assert!(validate_pipeline(&def).is_ok());
    }
}
