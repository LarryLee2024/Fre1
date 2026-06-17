use crate::core::capabilities::runtime::pipeline::foundation::{
    ExecutionLogEntry, FailureStrategy, PipelineContext, PipelineError, PipelineStage,
    PipelineStep, StepResult,
};

#[test]
fn stage_constructed_correctly() {
    let stage = PipelineStage::new("damage_calculation")
        .step(PipelineStep::Rule("calc_base_damage".into()))
        .step(PipelineStep::Rule("apply_modifiers".into()));
    assert_eq!(stage.name, "damage_calculation");
    assert_eq!(stage.steps.len(), 2);
}

#[test]
fn stage_default_type_correct() {
    let stage = PipelineStage::new("optional").skippable();
    assert!(stage.skippable);
}

#[test]
fn step_constructed_correctly() {
    assert_eq!(PipelineStep::System("physics".into()).name(), "physics");
    assert_eq!(
        PipelineStep::Rule("calc_damage".into()).name(),
        "calc_damage"
    );
    assert_eq!(PipelineStep::SubPipeline("combat".into()).name(), "combat");
}

#[test]
fn configured_step_constructed_correctly() {
    let step = PipelineStep::Conditional {
        condition: "has_shield".into(),
        if_true: Box::new(PipelineStep::Rule("apply_shield".into())),
        if_false: Box::new(PipelineStep::Rule("apply_damage".into())),
    };
    assert_eq!(step.name(), "has_shield");
}

#[test]
fn failure_handler_constructed_correctly() {
    assert_eq!(FailureStrategy::Abort.name(), "Abort");
    assert_eq!(FailureStrategy::SkipAndContinue.name(), "SkipAndContinue");
    assert_eq!(FailureStrategy::Retry { max_retries: 3 }.name(), "Retry");
}

#[test]
fn context_constructed_correctly() {
    let ctx = PipelineContext::new("combat_pipeline");
    assert_eq!(ctx.pipeline_id, "combat_pipeline");
    assert!(!ctx.aborted);
}

#[test]
fn context_stage_data_read_write() {
    let mut ctx = PipelineContext::new("test");
    ctx.set_stage_data("generate", "damage=50,type=fire");
    assert_eq!(
        ctx.get_stage_data("generate"),
        Some(&"damage=50,type=fire".into())
    );
    assert_eq!(ctx.get_stage_data("nonexistent"), None);
}

#[test]
fn context_halt_state() {
    let mut ctx = PipelineContext::new("test");
    ctx.abort("critical failure");
    assert!(ctx.aborted);
    assert_eq!(ctx.abort_reason, Some("critical failure".into()));
}

#[test]
fn context_log_recording() {
    let mut ctx = PipelineContext::new("test");
    ctx.log(ExecutionLogEntry::new(
        "damage",
        "calc_base",
        StepResult::Success,
    ));
    assert_eq!(ctx.execution_log.len(), 1);
}

#[test]
fn error_message_format_correct() {
    let err = PipelineError::StageNotFound("combat".into());
    let msg = format!("{}", err);
    assert!(msg.contains("combat"));
}

#[test]
fn step_failure_error_message() {
    let err = PipelineError::StepFailed {
        stage: "damage".into(),
        step: "calc".into(),
        detail: "division by zero".into(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("calc"));
    assert!(msg.contains("division by zero"));
}

#[test]
fn execution_log_count_correct() {
    let entry = ExecutionLogEntry::new("stage_1", "step_a", StepResult::Success);
    assert_eq!(entry.stage, "stage_1");
    assert_eq!(entry.step, "step_a");
}
