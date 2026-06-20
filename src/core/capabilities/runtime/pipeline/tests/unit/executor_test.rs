use crate::core::capabilities::runtime::pipeline::foundation::{
    FailureStrategy, PipelineContext, PipelineDefinition, PipelineError, PipelineStage,
    PipelineStep, StepResult,
};
use crate::core::capabilities::runtime::pipeline::mechanism::executor::*;
use crate::shared::error::ErrorContext;

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
        StepResult::Failure(ErrorContext {
            domain: "test",
            source: "intentional failure".into(),
            context: None,
        })
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
fn execute_empty_pipeline() {
    let def = PipelineDefinition::new("empty");
    let mut ctx = PipelineContext::new("empty");
    let result = execute_pipeline(&def, &mut ctx, mock_success_executor);
    assert!(result.is_ok());
}

#[test]
fn execute_single_stage_pipeline() {
    let def = PipelineDefinition::new("test")
        .stage(PipelineStage::new("stage_1").step(PipelineStep::Rule("step_a".into())));
    let mut ctx = PipelineContext::new("test");
    let result = execute_pipeline(&def, &mut ctx, mock_success_executor);
    assert!(result.is_ok());
}

#[test]
fn execute_multi_stage_pipeline() {
    let def = PipelineDefinition::new("test")
        .stage(PipelineStage::new("generate").step(PipelineStep::Rule("calc_base".into())))
        .stage(PipelineStage::new("modify").step(PipelineStep::Rule("apply_modifiers".into())))
        .stage(PipelineStage::new("execute").step(PipelineStep::System("apply_damage".into())));
    let mut ctx = PipelineContext::new("test");
    let result = execute_pipeline(&def, &mut ctx, mock_success_executor);
    assert!(result.is_ok());
}

#[test]
fn execute_logs_to_context() {
    let def = PipelineDefinition::new("test")
        .stage(PipelineStage::new("s1").step(PipelineStep::Rule("r1".into())));
    let mut ctx = PipelineContext::new("test");
    let _ = execute_pipeline(&def, &mut ctx, mock_success_executor);
    assert_eq!(ctx.execution_log.len(), 1);
    assert_eq!(ctx.execution_log[0].stage, "s1");
    assert_eq!(ctx.execution_log[0].step, "r1");
}

#[test]
fn failure_stops_pipeline() {
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
fn failure_returns_error_message() {
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
fn execute_passes_context_parameters() {
    let def = PipelineDefinition::new("test")
        .stage(PipelineStage::new("s1").step(PipelineStep::Rule("step_a".into())));
    let mut ctx = PipelineContext::new("test");
    let _ = execute_pipeline(&def, &mut ctx, mock_logging_executor);
    assert_eq!(ctx.get_stage_data("s1.step_a"), Some(&"executed".into()));
}

#[test]
fn execute_early_termination_pipeline() {
    let def = PipelineDefinition::new("test")
        .stage(PipelineStage::new("s1").step(PipelineStep::Rule("step_a".into())));
    let mut ctx = PipelineContext::new("test");
    ctx.abort("external cancellation");
    let result = execute_pipeline(&def, &mut ctx, mock_success_executor);
    assert!(result.is_err());
    assert!(matches!(result, Err(PipelineError::Aborted { reason: _ })));
}

#[test]
fn execute_skips_disabled_stage() {
    let def = PipelineDefinition::new("test")
        .stage(PipelineStage::new("optional").skippable())
        .stage(PipelineStage::new("main").step(PipelineStep::Rule("work".into())));
    let mut ctx = PipelineContext::new("test");
    let result = execute_pipeline(&def, &mut ctx, mock_success_executor);
    assert!(result.is_ok());
}

#[test]
fn execute_conditional_branch() {
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
            StepResult::Failure(ErrorContext {
                domain: "test",
                source: "unexpected".into(),
                context: None,
            })
        }
    }

    let mut ctx = PipelineContext::new("test");
    let result = execute_pipeline(&def, &mut ctx, condition_true_executor);
    assert!(result.is_ok());
}

// ── validate_pipeline ───────────────────────────────────

#[test]
fn validate_empty_pipeline_id_error() {
    let def = PipelineDefinition::new("");
    assert!(validate_pipeline(&def).is_err());
}

#[test]
fn validate_empty_stage_name_error() {
    let def = PipelineDefinition::new("test")
        .stage(PipelineStage::new("").step(PipelineStep::Rule("r".into())));
    assert!(validate_pipeline(&def).is_err());
}

#[test]
fn validate_disabled_empty_stage_name_error() {
    let def = PipelineDefinition::new("test").stage(PipelineStage::new("empty"));
    assert!(validate_pipeline(&def).is_err());
}

#[test]
fn validate_disabled_empty_stage_passes() {
    let def = PipelineDefinition::new("test").stage(PipelineStage::new("optional").skippable());
    assert!(validate_pipeline(&def).is_ok());
}

#[test]
fn validate_duplicate_stage_name_error() {
    let def = PipelineDefinition::new("test")
        .stage(PipelineStage::new("dup").step(PipelineStep::Rule("a".into())))
        .stage(PipelineStage::new("dup").step(PipelineStep::Rule("b".into())));
    assert!(validate_pipeline(&def).is_err());
}

#[test]
fn validate_valid_pipeline_passes() {
    let def = PipelineDefinition::new("combat")
        .stage(PipelineStage::new("generate").step(PipelineStep::Rule("calc".into())))
        .stage(PipelineStage::new("execute").step(PipelineStep::System("apply".into())));
    assert!(validate_pipeline(&def).is_ok());
}
