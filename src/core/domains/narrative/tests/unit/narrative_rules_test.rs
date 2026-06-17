use crate::core::domains::narrative::components::{ChoiceDef, DialogueNodeDef, StoryFlags};
use crate::core::domains::narrative::rules::{filter_visible_choices, validate_no_cycles};

#[test]
fn unconditional_choice_always_visible() {
    let choice = ChoiceDef {
        id: "c1".into(),
        text: "Hello".into(),
        next_node_id: None,
        set_flags: vec![],
        condition_ref: None,
    };
    let node = DialogueNodeDef {
        id: "n1".into(),
        npc_text: "Hi".into(),
        choices: vec![choice],
        is_important: false,
        condition_ref: None,
    };
    let flags = StoryFlags::new();
    let result = filter_visible_choices(&node, &flags);
    assert_eq!(result.len(), 1);
    assert!(result[0].visible);
}

#[test]
fn conditional_choice_hidden_when_flag_missing() {
    let choice = ChoiceDef {
        id: "c_quest".into(),
        text: "I finished the quest".into(),
        next_node_id: Some("n_reward".into()),
        set_flags: vec![],
        condition_ref: Some("quest_completed=true".into()),
    };
    let node = DialogueNodeDef {
        id: "n1".into(),
        npc_text: "How's the quest?".into(),
        choices: vec![choice],
        is_important: false,
        condition_ref: None,
    };

    let mut flags = StoryFlags::new();
    let result = filter_visible_choices(&node, &flags);
    assert!(!result[0].visible);

    flags.set_flag("quest_completed", "true");
    let result = filter_visible_choices(&node, &flags);
    assert!(result[0].visible);
}

#[test]
fn cycle_detection_clean() {
    let edges = vec![("n1", Some("n2")), ("n2", Some("n3")), ("n3", None)];
    assert!(validate_no_cycles("n1", &edges).is_ok());
}

#[test]
fn cycle_detection_with_cycle() {
    let edges = vec![("n1", Some("n2")), ("n2", Some("n3")), ("n3", Some("n1"))];
    assert!(validate_no_cycles("n1", &edges).is_err());
}
