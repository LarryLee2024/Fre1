use crate::core::capabilities::runtime::replay::foundation::{
    AbilityTarget, ReplayCommand, ReplayError, ReplayFrame, ReplayHeader, RngSeeds, RngStream,
};

#[test]
fn replay_frame_construction() {
    let frame = ReplayFrame::new(0, 100);
    assert_eq!(frame.frame_number, 0);
    assert_eq!(frame.rng_seed_offset, 100);
    assert!(frame.commands.is_empty());
}

#[test]
fn replay_frame_adds_command() {
    let mut frame = ReplayFrame::new(0, 0);
    frame.add_command(ReplayCommand::SkipTurn {
        unit: "unit_001".into(),
    });
    assert_eq!(frame.command_count(), 1);
}

#[test]
fn replay_frame_sets_checksum() {
    let mut frame = ReplayFrame::new(0, 0);
    frame.set_checksum(0xDEADBEEF);
    assert_eq!(frame.checksum, Some(0xDEADBEEF));
}

#[test]
fn replay_command_type_name() {
    let cmd = ReplayCommand::UnitMove {
        unit: "u1".into(),
        path: vec![],
    };
    assert_eq!(cmd.type_name(), "UnitMove");

    let cmd = ReplayCommand::Custom {
        domain: "test".into(),
        command_type: "ping".into(),
        params: vec![],
    };
    assert_eq!(cmd.type_name(), "Custom");
}

#[test]
fn rng_stream_name() {
    assert_eq!(RngStream::Combat.name(), "Combat");
    assert_eq!(RngStream::Drop.name(), "Drop");
    assert_eq!(RngStream::AI.name(), "AI");
    assert_eq!(RngStream::World.name(), "World");
}

#[test]
fn get_all_rng_streams() {
    let all = RngStream::all();
    assert_eq!(all.len(), 4);
}

#[test]
fn rng_seeds_uniform_set() {
    let seeds = RngSeeds::uniform(42);
    assert_eq!(seeds.combat_seed, 42);
    assert_eq!(seeds.drop_seed, 42);
    assert_eq!(seeds.ai_seed, 42);
    assert_eq!(seeds.world_seed, 42);
}

#[test]
fn rng_seeds_individual_get() {
    let seeds = RngSeeds::new(1, 2, 3, 4);
    assert_eq!(seeds.get(RngStream::Combat), 1);
    assert_eq!(seeds.get(RngStream::Drop), 2);
    assert_eq!(seeds.get(RngStream::AI), 3);
    assert_eq!(seeds.get(RngStream::World), 4);
}

#[test]
fn rng_seeds_individual_set() {
    let mut seeds = RngSeeds::uniform(0);
    seeds.set(RngStream::Combat, 100);
    assert_eq!(seeds.combat_seed, 100);
}

#[test]
fn replay_header_construction() {
    let header = ReplayHeader::new(1, "0.1.0", "battle_001", 42);
    assert_eq!(header.schema_version, 1);
    assert_eq!(header.scene_id, "battle_001");
    assert_eq!(header.initial_seed, 42);
}

#[test]
fn replay_header_adds_participant() {
    let mut header = ReplayHeader::new(1, "0.1.0", "scene", 0);
    header.add_participant("unit_001");
    header.add_participant("unit_002");
    assert_eq!(header.participants.len(), 2);
}

#[test]
fn replay_header_sets_total_frames() {
    let mut header = ReplayHeader::new(1, "0.1.0", "scene", 0);
    header.set_total_frames(100);
    assert_eq!(header.total_frames, 100);
}

#[test]
fn replay_error_display() {
    let err = ReplayError::VersionMismatch {
        expected: 2,
        actual: 1,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("v2"));
    assert!(msg.contains("v1"));

    let err = ReplayError::NotRecording;
    assert_eq!(format!("{}", err), "not in recording mode");
}

#[test]
fn use_ability_command() {
    let cmd = ReplayCommand::UseAbility {
        caster: "unit_001".into(),
        ability_def_id: "abl_000001".into(),
        target: AbilityTarget::Single("enemy_001".into()),
    };
    assert_eq!(cmd.type_name(), "UseAbility");
}

#[test]
fn dialogue_choice_command() {
    let cmd = ReplayCommand::DialogueChoice {
        speaker: "npc_001".into(),
        choice_id: "accept_quest".into(),
    };
    assert_eq!(cmd.type_name(), "DialogueChoice");
}

#[test]
fn frame_max_command_count() {
    let mut frame = ReplayFrame::new(10, 555);
    frame.add_command(ReplayCommand::SkipTurn { unit: "u1".into() });
    frame.add_command(ReplayCommand::SkipTurn { unit: "u2".into() });
    frame.add_command(ReplayCommand::SkipTurn { unit: "u3".into() });
    assert_eq!(frame.command_count(), 3);
}
