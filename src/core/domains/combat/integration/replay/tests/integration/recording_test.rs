//! Recording Systems 集成测试
//!
//! 验证 Observer 驱动的录制生命周期：
//! - record_unit_action 正确拦截 UnitActionComplete 并记录 ReplayCommand
//! - stop_recording_on_battle_end 正确清理录制会话
//!
//! Test IDs:
//! - REC-001: record_unit_action_records_command
//! - REC-002: stop_recording_cleans_up

use bevy::prelude::*;

use crate::core::capabilities::runtime::replay::foundation::ReplayHeader;
use crate::core::capabilities::runtime::replay::mechanism::RecordingSession as CoreRecordingSession;
use crate::core::domains::combat::components::{CombatParticipant, TeamId};
use crate::core::domains::combat::events::{OnBattleEnd, UnitActionComplete};
use crate::core::domains::combat::integration::replay::recording::{
    record_unit_action, stop_recording_on_battle_end,
};
use crate::infra::replay::resources::RecordingSession;
use crate::shared::ids::BattleUnitId;
use crate::shared::ids::mapping::EntityMapper;

#[test]
fn record_unit_action_records_command() {
    // Given: 一个已初始化的录制会话，包含一个已注册的 CombatParticipant 实体
    let mut app = App::new();
    app.init_resource::<RecordingSession>();

    let entity = app
        .world_mut()
        .spawn((CombatParticipant {
            team_id: TeamId::new("player"),
        },))
        .id();

    let mut mapper = EntityMapper::<BattleUnitId>::new();
    mapper.register(BattleUnitId::new("bu:0:0"), entity);
    app.world_mut().insert_resource(mapper);

    let mut core_session = CoreRecordingSession::new(60);
    core_session.start(ReplayHeader::new(1, "0.1.0", "test", 42), 0);
    app.world_mut()
        .insert_resource(RecordingSession(Some(core_session)));

    app.add_observer(record_unit_action);

    // When: 触发 UnitActionComplete
    app.world_mut().trigger(UnitActionComplete { unit: entity });
    app.update();

    // Then: 录制会话中应包含一条命令
    let recording = app.world_mut().resource::<RecordingSession>();
    let session = recording.0.as_ref().unwrap();
    let cmd_count = session
        .recorder
        .current_frame
        .as_ref()
        .map(|f| f.commands.len())
        .unwrap_or(0);
    assert_eq!(
        cmd_count, 1,
        "UnitActionComplete 应被记录为一条 ReplayCommand"
    );
}

#[test]
fn stop_recording_cleans_up() {
    // Given: 一个正在录制中的会话
    let mut app = App::new();
    app.init_resource::<RecordingSession>();
    app.insert_resource(EntityMapper::<BattleUnitId>::new());

    let mut core_session = CoreRecordingSession::new(60);
    core_session.start(ReplayHeader::new(1, "0.1.0", "test", 42), 0);
    app.world_mut()
        .insert_resource(RecordingSession(Some(core_session)));

    app.add_observer(stop_recording_on_battle_end);

    // When: 触发 OnBattleEnd
    app.world_mut().trigger(OnBattleEnd {
        result: crate::core::domains::combat::events::BattleResult::Victory,
    });
    app.update();

    // Then: 录制会话应被清理
    let recording = app.world_mut().resource::<RecordingSession>();
    assert!(
        recording.0.is_none(),
        "OnBattleEnd 后录制会话应被清理为 None"
    );
}
