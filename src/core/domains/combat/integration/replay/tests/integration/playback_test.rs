//! Playback Systems 集成测试
//!
//! 验证回放模式下的命令分发逻辑：
//! - 非回放模式下不进行分发（F3 修复）
//! - 匹配当前单位的命令正确触发 UnitActionComplete（F4 修复）
//!
//! Test IDs:
//! - PB-001: dispatch_skips_when_not_replay_mode
//! - PB-002: dispatch_matches_current_unit

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use bevy::prelude::*;

use crate::core::capabilities::runtime::pipeline::registry::PipelineRegistry;
use crate::core::capabilities::runtime::replay::foundation::{
    ReplayCommand, ReplayHeader, ReplayLog, ReplayMode, ReplayModeGuard as CoreReplayModeGuard,
};
use crate::core::capabilities::runtime::replay::mechanism::PlaybackSession as CorePlaybackSession;
use crate::core::domains::combat::components::{TeamId, TurnEntry, TurnQueue};
use crate::core::domains::combat::events::UnitActionComplete;
use crate::core::domains::combat::integration::replay::playback::dispatch_combat_replay_commands;
use crate::core::domains::combat::integration::replay::registry::{
    BattleUnitId, BattleUnitRegistry,
};
use crate::core::domains::combat::pipeline::driver::CombatPipelineDriver;
use crate::infra::replay::resources::{PlaybackSession, ReplayModeGuard};

/// 验证非回放模式下 dispatch_combat_replay_commands 跳过执行。
///
/// Given: 非回放模式 (is_replay = false)，管线已暂停
/// When: 运行 dispatch 系统
/// Then: 管线保持暂停（系统跳过执行）
#[test]
fn dispatch_skips_when_not_replay_mode() {
    let mut app = App::new();

    app.init_resource::<ReplayModeGuard>();
    app.init_resource::<CombatPipelineDriver>();
    app.init_resource::<PlaybackSession>();
    app.init_resource::<TurnQueue>();
    app.insert_resource(BattleUnitRegistry::default());

    // 显式设置为非回放模式（Default 已是 normal，这里冗余确保语义清晰）
    app.world_mut()
        .resource_mut::<ReplayModeGuard>()
        .0
        .is_replay = false;

    // 让管线暂停
    app.world_mut()
        .resource_mut::<CombatPipelineDriver>()
        .force_pause();

    app.add_systems(Update, dispatch_combat_replay_commands);
    app.update();

    // Then: 管线仍然暂停（dispatch 未运行）
    assert!(
        app.world_mut()
            .resource::<CombatPipelineDriver>()
            .is_paused(),
        "非回放模式下 dispatch 不应恢复管线"
    );
}

/// 验证 dispatch 匹配当前单位时正确触发 UnitActionComplete。
///
/// Given: 回放模式，管线暂停，当前单位已注册，回放日志包含 SkipTurn 命令
/// When: 运行 dispatch 系统
/// Then: 使用 AtomicBool 标志验证 UnitActionComplete 被触发
#[test]
fn dispatch_matches_current_unit() {
    let mut app = App::new();
    let entity = app.world_mut().spawn_empty().id();

    // 使用 AtomicBool 来跟踪 UnitActionComplete 是否被触发
    let action_triggered = Arc::new(AtomicBool::new(false));
    let action_triggered_clone = action_triggered.clone();

    // Setup: 回放模式 + 注册单位
    let mut registry = BattleUnitRegistry::default();
    registry.register(entity, BattleUnitId::new("bu:player:0"));
    app.insert_resource(registry);

    app.init_resource::<ReplayModeGuard>();
    app.init_resource::<CombatPipelineDriver>();
    app.init_resource::<PlaybackSession>();
    app.init_resource::<TurnQueue>();

    // 设置回放模式
    app.world_mut()
        .resource_mut::<ReplayModeGuard>()
        .0
        .is_replay = true;

    // 设置当前单位为该实体
    app.world_mut()
        .insert_resource(TurnQueue::new(vec![TurnEntry::new(
            entity,
            TeamId::new("player"),
            20,
        )]));

    // 让管线暂停
    app.world_mut()
        .resource_mut::<CombatPipelineDriver>()
        .force_pause();

    // 创建回放日志并加载到 PlaybackSession
    let log = create_test_replay_log(vec![ReplayCommand::SkipTurn {
        unit: "bu:player:0".to_string(),
    }]);

    let mut core_session = CorePlaybackSession::new(ReplayMode::Full, 42);
    core_session.load(&log).unwrap();
    core_session.start();
    app.world_mut()
        .insert_resource(PlaybackSession(Some(core_session)));

    app.add_observer(move |_trigger: On<'_, '_, UnitActionComplete>| {
        action_triggered_clone.store(true, Ordering::SeqCst);
    });

    app.add_systems(Update, dispatch_combat_replay_commands);

    app.update();

    // Then: dispatch 应触发 UnitActionComplete
    assert!(
        action_triggered.load(Ordering::SeqCst),
        "dispatch 匹配单位后应触发 UnitActionComplete"
    );
}

// ════════════════════════════════════════════
// 测试辅助函数
// ════════════════════════════════════════════

fn create_test_replay_log(commands: Vec<ReplayCommand>) -> ReplayLog {
    use crate::core::capabilities::runtime::replay::foundation::ReplayFrame;
    let header = ReplayHeader::new(1, "0.1.0", "test_replay", 42);
    let mut log = ReplayLog::new(header);
    let mut frame = ReplayFrame::new(0, 0);
    for cmd in commands {
        frame.add_command(cmd);
    }
    log.add_frame(frame);
    log
}
