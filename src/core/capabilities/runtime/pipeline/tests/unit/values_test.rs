use crate::core::capabilities::runtime::pipeline::foundation::{PipelineDefinition, PipelineStage};

#[test]
fn definition_constructed_correctly() {
    let def = PipelineDefinition::new("combat")
        .stage(PipelineStage::new("generate"))
        .stage(PipelineStage::new("modify"));
    assert_eq!(def.id, "combat");
    assert_eq!(def.stage_count(), 2);
}

#[test]
fn definition_adds_stage() {
    let def = PipelineDefinition::new("test")
        .stage(PipelineStage::new("stage_a"))
        .stage(PipelineStage::new("stage_b"));
    assert!(def.find_stage("stage_a").is_some());
    assert!(def.find_stage("nonexistent").is_none());
}

#[test]
fn state_initial_state_correct() {
    let def = PipelineDefinition::new("combat");
    let state = crate::core::capabilities::runtime::pipeline::foundation::PipelineState::new(&def);
    assert_eq!(state.current_stage_index, 0);
    assert_eq!(state.current_step_index, 0);
    assert!(!state.completed);
}

#[test]
fn state_advances_step_and_stage() {
    let def = PipelineDefinition::new("combat");
    let mut state =
        crate::core::capabilities::runtime::pipeline::foundation::PipelineState::new(&def);

    state.advance_step();
    assert_eq!(state.current_step_index, 1);

    state.advance_stage();
    assert_eq!(state.current_stage_index, 1);
    assert_eq!(state.current_step_index, 0);
}

#[test]
fn state_reset() {
    let def = PipelineDefinition::new("combat");
    let mut state =
        crate::core::capabilities::runtime::pipeline::foundation::PipelineState::new(&def);
    state.mark_completed();
    assert!(state.completed);
}
