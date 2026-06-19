use crate::core::domains::narrative::components::{
    ChoiceOption, CutscenePhase, CutsceneState, DialogueHistory, DialogueNodeDef, DialoguePhase,
    DialogueState, DialogueTreeRegistry, StoryFlags,
};

#[test]
fn story_flags_set_once() {
    let mut flags = StoryFlags::new();
    assert!(flags.set_flag("saved_villager", "true"));
    assert!(flags.check("saved_villager", "true"));

    assert!(flags.set_flag("saved_villager", "true"));

    assert!(!flags.set_flag("saved_villager", "false"));
}

#[test]
fn dialogue_state_advance() {
    let choices = vec![ChoiceOption {
        choice_id: "c1".into(),
        text: "Tell me more".into(),
        visible: true,
    }];
    let mut state = DialogueState::new("tree_001", "node_001", choices, 0.0);
    assert_eq!(state.phase, DialoguePhase::Speaking);
    assert_eq!(state.current_node_id, "node_001");

    state.advance("node_002", vec![]);
    assert_eq!(state.current_node_id, "node_002");

    state.end();
    assert_eq!(state.phase, DialoguePhase::End);
}

#[test]
fn cutscene_initial_state() {
    let cs = CutsceneState::new("cs_intro", 5.0, vec![]);
    assert_eq!(cs.phase, CutscenePhase::Playing);
    assert_eq!(cs.duration, 5.0);
    assert_eq!(cs.elapsed, 0.0);
}

#[test]
fn cutscene_pause_resume() {
    let mut cs = CutsceneState::new("cs_intro", 10.0, vec![]);
    cs.pause();
    assert_eq!(cs.phase, CutscenePhase::Paused);
    cs.resume();
    assert_eq!(cs.phase, CutscenePhase::Playing);
}

#[test]
fn dialogue_history_tracking() {
    let mut history = DialogueHistory::new();
    assert!(!history.can_skip("tree_001"));

    history.record_choice("tree_001", "c1");
    assert!(history.can_skip("tree_001"));

    history.visit_node("tree_001", "node_001");
    assert!(history.has_visited("tree_001", "node_001"));
    assert!(!history.has_visited("tree_001", "node_002"));
}

#[test]
fn dialogue_tree_registry() {
    let mut reg = DialogueTreeRegistry::new();
    reg.register_node(DialogueNodeDef {
        id: "node_start".into(),
        npc_text: "Hello!".into(),
        choices: vec![],
        is_important: false,
        condition_ref: None,
    });
    reg.register_tree("tree_greeting", "node_start");

    let entry = reg.entry_node("tree_greeting");
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().npc_text, "Hello!");
}
