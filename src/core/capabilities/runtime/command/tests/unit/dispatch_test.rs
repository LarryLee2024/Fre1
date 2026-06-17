use crate::core::capabilities::runtime::command::foundation::{
    CommandSource, DispatchResult, GameCommand,
};
use crate::core::capabilities::runtime::command::mechanism::dispatch::*;

fn mock_handler(command: &GameCommand, _source: CommandSource) -> DispatchResult {
    match command {
        GameCommand::MoveUnit { .. } => DispatchResult::Dispatched,
        GameCommand::EndTurn { .. } => DispatchResult::Dispatched,
        _ => DispatchResult::Unhandled(command.name().into()),
    }
}

#[test]
fn dispatch_move_command() {
    let cmd = GameCommand::MoveUnit {
        unit_id: "u1".into(),
        path: vec!["0,0".into()],
    };
    let result = dispatch_command(&cmd, CommandSource::Player, mock_handler);
    assert_eq!(result, DispatchResult::Dispatched);
}

#[test]
fn dispatch_unregistered_command() {
    let cmd = GameCommand::OpenMenu;
    let result = dispatch_command(&cmd, CommandSource::System, mock_handler);
    assert_eq!(result, DispatchResult::Unhandled("OpenMenu".into()));
}

#[test]
fn batch_dispatch_command() {
    let cmds = vec![
        GameCommand::MoveUnit {
            unit_id: "u1".into(),
            path: vec!["0,0".into()],
        },
        GameCommand::OpenMenu,
    ];
    let results = dispatch_batch(&cmds, CommandSource::Player, mock_handler);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0], DispatchResult::Dispatched);
    assert_eq!(results[1], DispatchResult::Unhandled("OpenMenu".into()));
}

#[test]
fn validate_move_command() {
    let cmd = GameCommand::MoveUnit {
        unit_id: "".into(),
        path: vec!["0,0".into()],
    };
    assert!(validate_command(&cmd).is_err());

    let cmd = GameCommand::MoveUnit {
        unit_id: "u1".into(),
        path: vec![],
    };
    assert!(validate_command(&cmd).is_err());

    let cmd = GameCommand::MoveUnit {
        unit_id: "u1".into(),
        path: vec!["0,0".into()],
    };
    assert!(validate_command(&cmd).is_ok());
}

#[test]
fn validate_wait_command() {
    let cmd = GameCommand::Wait { unit_id: "".into() };
    assert!(validate_command(&cmd).is_err());

    let cmd = GameCommand::Wait {
        unit_id: "u1".into(),
    };
    assert!(validate_command(&cmd).is_ok());
}

#[test]
fn validate_attack_command() {
    let cmd = GameCommand::Attack {
        attacker_id: "".into(),
        target_id: "u2".into(),
        ability_slot: None,
    };
    assert!(validate_command(&cmd).is_err());
}

#[test]
fn validate_cast_command() {
    let cmd = GameCommand::CastSpell {
        caster_id: "u1".into(),
        spell_def_id: "".into(),
        target_id: "u2".into(),
    };
    assert!(validate_command(&cmd).is_err());
}

#[test]
fn validate_use_item_command() {
    let cmd = GameCommand::UseItem {
        user_id: "u1".into(),
        item_instance_id: "".into(),
        target_id: None,
    };
    assert!(validate_command(&cmd).is_err());
}

#[test]
fn validate_meta_command() {
    assert!(validate_command(&GameCommand::OpenMenu).is_ok());
    assert!(validate_command(&GameCommand::SaveGame).is_ok());
    assert!(validate_command(&GameCommand::LoadGame).is_ok());
}

#[test]
fn validate_end_turn_command() {
    let cmd = GameCommand::EndTurn { unit_id: "".into() };
    assert!(validate_command(&cmd).is_err());

    let cmd = GameCommand::EndTurn {
        unit_id: "u1".into(),
    };
    assert!(validate_command(&cmd).is_ok());
}
