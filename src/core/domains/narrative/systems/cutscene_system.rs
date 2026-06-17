//! Cutscene System — 演出控制系统
//!
//! 处理演出开始、结束、进度更新。

use bevy::prelude::*;

use crate::core::domains::narrative::components::{CutscenePhase, CutsceneState};
use crate::core::domains::narrative::events::{CutsceneEnded, CutsceneStarted};

/// 演出开始请求事件。
#[derive(Event, Debug, Clone)]
pub struct CutsceneStartRequest {
    /// 演出 ID
    pub cutscene_id: String,
    /// 持续时间（秒）
    pub duration: f32,
    /// 参与者列表
    pub participants: Vec<Entity>,
}

/// 响应演出开始请求。
pub(crate) fn on_cutscene_start(trigger: On<CutsceneStartRequest>, mut commands: Commands) {
    let req = trigger.event();

    // 发布 CutsceneStarted 事件供 UI/Cue 订阅
    commands.trigger(CutsceneStarted {
        cutscene_id: req.cutscene_id.clone(),
        duration: req.duration,
        participants: req.participants.clone(),
    });

    info!(
        "[Narrative] Cutscene started: id={}, duration={}",
        req.cutscene_id, req.duration
    );
}

/// 逐帧更新演出进度。
///
/// 在 Update schedule 中运行，推动 CutsceneState 的 elapsed 计时。
pub(crate) fn cutscene_progress_system(
    time: Res<Time>,
    mut query: Query<&mut CutsceneState>,
    mut commands: Commands,
) {
    for mut state in query.iter_mut() {
        if state.phase != CutscenePhase::Playing {
            continue;
        }

        state.tick(time.delta().as_secs_f32());

        if state.phase == CutscenePhase::Finished {
            info!("[Narrative] Cutscene finished: id={}", state.cutscene_id);
            commands.trigger(CutsceneEnded {
                cutscene_id: state.cutscene_id.clone(),
            });
        }
    }
}
