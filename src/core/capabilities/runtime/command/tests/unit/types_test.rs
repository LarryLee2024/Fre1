use crate::core::capabilities::runtime::command::foundation::{
    CommandError, CommandSource, GameCommand,
};

#[test]
fn command_source_constructed_correctly() {
    assert_eq!(CommandSource::Player.name(), "Player");
    assert_eq!(CommandSource::AI.name(), "AI");
    assert_eq!(CommandSource::Replay.name(), "Replay");
    assert_eq!(CommandSource::System.name(), "System");
}

#[test]
fn move_command_constructed_correctly() {
    let cmd = GameCommand::MoveUnit {
        unit_id: "unit_001".into(),
        path: vec!["0,0".into(), "0,1".into()],
    };
    assert_eq!(cmd.name(), "MoveUnit");
}

#[test]
fn wait_command_constructed_correctly() {
    let cmd = GameCommand::Wait {
        unit_id: "unit_001".into(),
    };
    assert_eq!(cmd.name(), "Wait");
}

#[test]
fn attack_command_constructed_correctly() {
    let cmd = GameCommand::Attack {
        attacker_id: "unit_001".into(),
        target_id: "unit_002".into(),
        ability_slot: None,
    };
    assert_eq!(cmd.name(), "Attack");
}

#[test]
fn cast_command_constructed_correctly() {
    let cmd = GameCommand::CastSpell {
        caster_id: "unit_001".into(),
        spell_def_id: "spl_000001".into(),
        target_id: "unit_002".into(),
    };
    assert_eq!(cmd.name(), "CastSpell");
}

#[test]
fn use_item_command_constructed_correctly() {
    let cmd = GameCommand::UseItem {
        user_id: "unit_001".into(),
        item_instance_id: "itm_000001".into(),
        target_id: None,
    };
    assert_eq!(cmd.name(), "UseItem");
}

#[test]
fn end_turn_command_constructed_correctly() {
    let cmd = GameCommand::EndTurn {
        unit_id: "unit_001".into(),
    };
    assert_eq!(cmd.name(), "EndTurn");
}

#[test]
fn meta_command_constructed_correctly() {
    assert_eq!(GameCommand::OpenMenu.name(), "OpenMenu");
    assert_eq!(GameCommand::SaveGame.name(), "SaveGame");
    assert_eq!(GameCommand::LoadGame.name(), "LoadGame");
}

#[test]
fn record_command_correct() {
    let cmd = GameCommand::Wait {
        unit_id: "unit_001".into(),
    };
    let recorded = crate::core::capabilities::runtime::command::foundation::RecordedCommand::new(
        CommandSource::Player,
        cmd,
        42,
    );
    assert_eq!(recorded.source, CommandSource::Player);
    assert_eq!(recorded.frame_number, 42);
    assert_eq!(recorded.command.name(), "Wait");
}

#[test]
fn error_message_format_correct() {
    let err = CommandError::QueueFull { max: 128 };
    let msg = format!("{}", err);
    assert!(msg.contains("128"));

    let err = CommandError::InvalidCommand { reason: "unknown action".into() };
    let msg = format!("{}", err);
    assert!(msg.contains("unknown action"));
}
