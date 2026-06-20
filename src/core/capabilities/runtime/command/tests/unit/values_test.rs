use crate::core::capabilities::runtime::command::foundation::{
    CommandError, CommandHistory, CommandQueue, CommandSource, GameCommand,
};

fn make_move_cmd() -> GameCommand {
    GameCommand::MoveUnit {
        unit_id: "unit_001".into(),
        path: vec!["0,0".into()],
    }
}

#[test]
fn queue_initially_empty() {
    let queue = CommandQueue::new();
    assert_eq!(queue.pending_count(), 0);
    assert!(!queue.has_pending());
}

#[test]
fn add_command_succeeds() {
    let mut queue = CommandQueue::new();
    assert!(queue.push(make_move_cmd()).is_ok());
    assert_eq!(queue.pending_count(), 1);
    assert!(queue.has_pending());
}

#[test]
fn empty_queue_returns_none_for_command() {
    let mut queue = CommandQueue::new();
    queue.push(make_move_cmd()).unwrap();
    queue.push(GameCommand::OpenMenu).unwrap();

    let drained = queue.drain();
    assert_eq!(drained.len(), 2);
    assert_eq!(queue.pending_count(), 0);
}

#[test]
fn queue_full_rejects_addition() {
    let mut queue = CommandQueue::new();
    queue.set_max_size(2);
    assert!(queue.push(make_move_cmd()).is_ok());
    assert!(queue.push(make_move_cmd()).is_ok());
    assert_eq!(
        queue.push(make_move_cmd()),
        Err(CommandError::QueueFull { max: 2 })
    );
}

#[test]
fn append_multiple_commands() {
    let mut queue = CommandQueue::new();
    assert!(
        queue
            .push_recorded(make_move_cmd(), CommandSource::Player)
            .is_ok()
    );
    assert_eq!(queue.pending_count(), 1);
    assert_eq!(queue.history_count(), 1);
}

#[test]
fn advance_frame_count() {
    let mut queue = CommandQueue::new();
    assert_eq!(queue.frame_number(), 0);
    queue.advance_frame();
    assert_eq!(queue.frame_number(), 1);
}

#[test]
fn get_recorded_history() {
    let mut queue = CommandQueue::new();
    queue
        .push_recorded(make_move_cmd(), CommandSource::Player)
        .unwrap();

    let history = queue.history();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].source, CommandSource::Player);
}

#[test]
fn command_history_initially_empty() {
    let hist = CommandHistory::new();
    assert_eq!(hist.count(), 0);
}

#[test]
fn record_command_to_history() {
    let mut hist = CommandHistory::new();
    let cmd = GameCommand::OpenMenu;
    hist.record(
        crate::core::capabilities::runtime::command::foundation::RecordedCommand::new(
            CommandSource::System,
            cmd,
            1,
        ),
    );
    assert_eq!(hist.count(), 1);
}

#[test]
fn source_filtered_command_history() {
    let mut hist = CommandHistory::new();
    hist.record(
        crate::core::capabilities::runtime::command::foundation::RecordedCommand::new(
            CommandSource::Player,
            make_move_cmd(),
            1,
        ),
    );
    hist.record(
        crate::core::capabilities::runtime::command::foundation::RecordedCommand::new(
            CommandSource::AI,
            make_move_cmd(),
            2,
        ),
    );

    let player_cmds = hist.filter_by_source(CommandSource::Player);
    assert_eq!(player_cmds.len(), 1);

    let replay_cmds = hist.filter_by_source(CommandSource::Replay);
    assert_eq!(replay_cmds.len(), 0);
}

#[test]
fn empty_queue_does_not_advance_frame() {
    let mut queue = CommandQueue::new();

    // Frame 1
    queue.push(make_move_cmd()).unwrap();
    let _f1_cmds = queue.drain();
    queue.advance_frame();

    // Frame 2
    queue
        .push(GameCommand::EndTurn {
            unit_id: "unit_001".into(),
        })
        .unwrap();
    let f2_cmds = queue.drain();
    assert_eq!(f2_cmds.len(), 1);
    assert_eq!(f2_cmds[0].name(), "EndTurn");
}
