//! Replay bridge 测试共享辅助函数
//!
//! 提供录制/回放测试中通用的 setup 函数和 fixture 构造器。

use bevy::prelude::*;

use crate::core::capabilities::runtime::replay::foundation::{
    ReplayCommand, ReplayFrame, ReplayHeader, ReplayLog,
};
use crate::infra::replay::resources::{PlaybackSession, ReplayModeGuard};
use crate::shared::ids::BattleUnitId;
use crate::shared::ids::mapping::EntityMapper;

/// 回放模式下设置最小测试环境。
///
/// 初始化 ReplayModeGuard, CombatPipelineDriver, PlaybackSession, TurnQueue,
/// 并根据 participants 构建 EntityMapper<BattleUnitId>。
pub fn setup_replay_mode(app: &mut App, participants: Vec<(&str, Entity)>) {
    use crate::core::domains::combat::components::TurnQueue;
    use crate::core::domains::combat::pipeline::driver::CombatPipelineDriver;

    app.init_resource::<ReplayModeGuard>();
    app.init_resource::<CombatPipelineDriver>();
    app.init_resource::<PlaybackSession>();
    app.init_resource::<TurnQueue>();

    // 构建 EntityMapper<BattleUnitId>
    let mut mapper = EntityMapper::<BattleUnitId>::new();
    for (id_str, entity) in &participants {
        mapper.register(BattleUnitId::new(*id_str), *entity);
    }
    app.world_mut().insert_resource(mapper);

    // 设置回放模式
    app.world_mut()
        .resource_mut::<ReplayModeGuard>()
        .0
        .is_replay = true;
}

/// 创建一个单帧的回放日志，包含指定的命令列表。
pub fn create_test_replay_log(commands: Vec<ReplayCommand>) -> ReplayLog {
    let header = ReplayHeader::new(1, "0.1.0", "test_replay", 42);
    let mut log = ReplayLog::new(header);
    let mut frame = ReplayFrame::new(0, 0);
    for cmd in commands {
        frame.add_command(cmd);
    }
    log.add_frame(frame);
    log
}
