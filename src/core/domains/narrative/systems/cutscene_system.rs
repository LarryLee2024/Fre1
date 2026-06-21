//! Cutscene System — 演出控制系统
//!
//! 使用 Bevy 0.19 Delayed Commands 管理演出生命周期。
//! 通过 Observer + Delayed 替代传统的逐帧 Timer tick。

use bevy::prelude::*;

use crate::core::domains::narrative::components::{CutscenePhase, CutsceneState};
use crate::core::domains::narrative::events::{CutsceneEnded, CutsceneStarted};

/// 演出开始请求事件。
#[derive(Event, Debug, Clone, Reflect)]
pub struct CutsceneStartRequest {
    /// 演出 ID
    pub cutscene_id: String,
    /// 持续时间（秒）
    pub duration: f32,
    /// 参与者列表
    pub participants: Vec<Entity>,
}

/// 响应演出开始请求：生成状态实体 + 调度延迟结束命令。
pub(crate) fn on_cutscene_start(trigger: On<CutsceneStartRequest>, mut commands: Commands) {
    let req = trigger.event();

    // CutsceneState 挂载到实体供 UI 查询演出进度（Phase/elapsed/duration）
    commands.spawn(CutsceneState {
        phase: CutscenePhase::Playing,
        cutscene_id: req.cutscene_id.clone(),
        duration: req.duration,
        elapsed: 0.0,
        participants: req.participants.clone(),
    });

    // 使用 Delayed Commands 替代逐帧 tick：duration 秒后自动触发结束
    commands
        .delayed()
        .secs(req.duration)
        .trigger(CutsceneEnded {
            cutscene_id: req.cutscene_id.clone(),
        });

    // 发布 CutsceneStarted 事件供 UI/Cue 订阅
    commands.trigger(CutsceneStarted {
        cutscene_id: req.cutscene_id.clone(),
        duration: req.duration,
        participants: req.participants.clone(),
    });
}

/// 响应演出结束：清理 CutsceneState 实体。
pub(crate) fn on_cutscene_ended(
    _trigger: On<CutsceneEnded>,
    mut commands: Commands,
    query: Query<Entity, With<CutsceneState>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
