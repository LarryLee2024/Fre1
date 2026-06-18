use crate::core::domains::combat::pipeline::definition::{
    COMBAT_TURN_PIPELINE_ID, build_turn_pipeline,
};

#[test]
fn pipeline_definition_id_matches_constant() {
    let def = build_turn_pipeline();
    assert_eq!(
        def.id, COMBAT_TURN_PIPELINE_ID,
        "pipeline id should match the defined constant"
    );
}

#[test]
fn pipeline_has_five_stages() {
    let def = build_turn_pipeline();
    assert_eq!(
        def.stage_count(),
        5,
        "turn pipeline should have exactly 5 stages"
    );
}

#[test]
fn pipeline_stages_in_correct_order() {
    let def = build_turn_pipeline();
    let expected = [
        "turn_start",
        "phase_check",
        "unit_action",
        "turn_settlement",
        "turn_end",
    ];
    for (i, name) in expected.iter().enumerate() {
        let stage = &def.stages[i];
        assert_eq!(stage.name, *name, "stage {} should be '{}'", i, name);
    }
}

#[test]
fn each_stage_has_exactly_one_step() {
    let def = build_turn_pipeline();
    for stage in &def.stages {
        assert_eq!(
            stage.steps.len(),
            1,
            "stage '{}' should have exactly 1 step",
            stage.name
        );
    }
}

#[test]
fn each_step_name_matches_stage_name() {
    let def = build_turn_pipeline();
    for stage in &def.stages {
        let step = &stage.steps[0];
        assert_eq!(
            step.name(),
            stage.name,
            "step name in stage '{}' should match stage name",
            stage.name
        );
    }
}

#[test]
fn all_stages_use_abort_on_failure() {
    let def = build_turn_pipeline();
    for stage in &def.stages {
        assert!(
            matches!(
                stage.on_failure,
                crate::core::capabilities::runtime::pipeline::foundation::FailureStrategy::Abort
            ),
            "stage '{}' should use Abort failure strategy",
            stage.name
        );
    }
}
