use bevy::app::App;
use bevy::prelude::*;

use crate::core::capabilities::runtime::replay::foundation::{
    ReplayCommand, ReplayHeader,
};
use crate::core::capabilities::runtime::replay::mechanism::RecordingSession as CoreRecordingSession;
use crate::infra::replay::plugin::ReplayPlugin;
use crate::infra::replay::resources::RecordingSession;

/// 录制模式：启动录制后 RecordingSession 变为 Some，可录制命令。
#[test]
fn recording_session_can_record_commands() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ReplayPlugin);

    // 创建并启动一个录制会话
    let header = ReplayHeader::new(1, "0.1.0", "test_scene", 42);
    let mut core_session = CoreRecordingSession::new(60);
    core_session.start(header, 42);

    // 注入到 ECS Resource
    app.world_mut()
        .insert_resource(RecordingSession(Some(core_session)));

    // 录制一个命令
    {
        let mut session = app.world_mut().resource_mut::<RecordingSession>();
        if let Some(ref mut s) = session.0 {
            s.record_command(ReplayCommand::SkipTurn {
                unit: "test_unit".to_string(),
            });
        }
    }

    // 运行一帧 — frame counter + recording bookend
    app.update();

    // 验证：一帧处理后，应该仍在录制模式
    let session = app.world().resource::<RecordingSession>();
    assert!(session.0.is_some(), "session should still be active");
    if let Some(ref s) = session.0 {
        assert!(s.is_recording(), "should still be recording");
        // 至少应有一个已完成帧
        assert!(
            s.recorder.frame_count() >= 1,
            "should have completed at least one frame"
        );
    }
}

/// 录制停止后生成完整的 ReplayLog。
#[test]
fn recording_stop_produces_replay_log() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ReplayPlugin);

    let header = ReplayHeader::new(1, "0.1.0", "test_scene", 42);
    let mut core_session = CoreRecordingSession::new(60);
    core_session.start(header, 42);

    app.world_mut()
        .insert_resource(RecordingSession(Some(core_session)));

    // 录制一些命令
    {
        let mut session = app.world_mut().resource_mut::<RecordingSession>();
        if let Some(ref mut s) = session.0 {
            s.record_command(ReplayCommand::SkipTurn {
                unit: "unit_a".to_string(),
            });
        }
    }

    app.update();

    // 录制更多命令在第二帧
    {
        let mut session = app.world_mut().resource_mut::<RecordingSession>();
        if let Some(ref mut s) = session.0 {
            s.record_command(ReplayCommand::SkipTurn {
                unit: "unit_b".to_string(),
            });
        }
    }

    app.update();

    // 停止录制并获取日志
    let log = {
        let mut session = app.world_mut().resource_mut::<RecordingSession>();
        if let Some(ref mut s) = session.0 {
            s.stop(12345).ok()
        } else {
            None
        }
    };

    assert!(log.is_some(), "stop should produce a ReplayLog");
    let log = log.unwrap();
    assert_eq!(log.header.schema_version, 1);
    assert_eq!(log.header.scene_id, "test_scene");
    assert_eq!(log.header.initial_seed, 42);
    // 至少应有 2 帧（2 次 update 调用）
    assert!(log.frames.len() >= 2, "should have at least 2 frames");
}

/// 录制模式下跳过帧边界不会丢失命令。
#[test]
fn recording_frame_bookend_does_not_lose_commands() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ReplayPlugin);

    let header = ReplayHeader::new(1, "0.1.0", "test_scene", 42);
    let mut core_session = CoreRecordingSession::new(60);
    core_session.start(header, 42);
    app.world_mut()
        .insert_resource(RecordingSession(Some(core_session)));

    // 录制 3 个命令
    for i in 0..3 {
        let mut session = app.world_mut().resource_mut::<RecordingSession>();
        if let Some(ref mut s) = session.0 {
            s.record_command(ReplayCommand::SkipTurn {
                unit: format!("unit_{}", i),
            });
        }
    }

    // 运行多帧
    for _ in 0..5 {
        app.update();
    }

    // 停止并检查总命令数
    let log = {
        let mut session = app.world_mut().resource_mut::<RecordingSession>();
        if let Some(ref mut s) = session.0 {
            s.stop(999).ok()
        } else {
            None
        }
    };

    let log = log.expect("should produce ReplayLog");
    let total_cmds: usize = log.frames.iter().map(|f| f.commands.len()).sum();
    assert_eq!(
        total_cmds, 3,
        "all 3 commands should be captured across frames"
    );
}

/// 同时录制多个命令类型。
#[test]
fn recording_handles_multiple_command_types() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ReplayPlugin);

    let header = ReplayHeader::new(1, "0.1.0", "test_scene", 42);
    let mut core_session = CoreRecordingSession::new(60);
    core_session.start(header, 42);
    app.world_mut()
        .insert_resource(RecordingSession(Some(core_session)));

    // 录制不同类型的命令
    {
        let mut session = app.world_mut().resource_mut::<RecordingSession>();
        if let Some(ref mut s) = session.0 {
            s.record_command(ReplayCommand::UnitMove {
                unit: "hero".into(),
                path: vec!["A1".into(), "A2".into()],
            });
            s.record_command(ReplayCommand::UseAbility {
                caster: "hero".into(),
                ability_def_id: "fireball".into(),
                target:
                    crate::core::capabilities::runtime::replay::foundation::AbilityTarget::Single(
                        "goblin".into(),
                    ),
            });
            s.record_command(ReplayCommand::DialogueChoice {
                speaker: "narrator".into(),
                choice_id: "option_1".into(),
            });
        }
    }

    app.update();

    let log = {
        let mut session = app.world_mut().resource_mut::<RecordingSession>();
        if let Some(ref mut s) = session.0 {
            s.stop(999).ok()
        } else {
            None
        }
    };

    let log = log.expect("should produce ReplayLog");
    let total_cmds: usize = log.frames.iter().map(|f| f.commands.len()).sum();
    assert_eq!(total_cmds, 3, "all 3 command types should be captured");

    // 验证命令类型正确
    let all_cmds: Vec<_> = log.frames.iter().flat_map(|f| &f.commands).collect();
    assert_eq!(all_cmds[0].type_name(), "UnitMove");
    assert_eq!(all_cmds[1].type_name(), "UseAbility");
    assert_eq!(all_cmds[2].type_name(), "DialogueChoice");
}
