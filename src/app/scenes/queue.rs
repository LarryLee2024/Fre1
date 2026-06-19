//! 状态转移队列 — StateTransitionQueue
//!
//! 唯一的 NextState 调用入口。域系统通过队列提交请求，
//! `process_transition_queue` 在 Last 调度中执行单次转移。
//!
//! 详见 ADR-050 §3: StateTransitionQueue + TransitionRequest。

use bevy::prelude::*;

use super::components::SceneRoot;
use super::state::{GameState, TransitionRequest};

/// 状态转移请求队列 — 唯一的 NextState 调用入口。
///
/// 域系统通过 `TransitionRequest` 提交转移意图，
/// 每帧最多处理一个请求（取最后一个，忽略中间跳转）。
#[derive(Resource, Default)]
pub struct StateTransitionQueue {
    /// 待处理的转移请求列表。
    pending: Vec<TransitionRequest>,
}

impl StateTransitionQueue {
    /// 提交一个转移请求。
    pub fn push(&mut self, request: TransitionRequest) {
        self.pending.push(request);
    }

    /// 尝试取出最后一个请求（并清空队列）。
    fn pop_last(&mut self) -> Option<TransitionRequest> {
        let last = self.pending.pop()?;
        self.pending.clear();
        Some(last)
    }
}

/// 在 Last 调度中执行队列，保证每帧最多一次状态转移。
///
/// 策略：仅处理最后一个请求（忽略中间跳转），
/// 避免多系统在帧内连续提交导致状态震荡。
pub fn process_transition_queue(
    mut queue: ResMut<StateTransitionQueue>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let Some(request) = queue.pop_last() {
        match request {
            TransitionRequest::Change(state) => next_state.set(state),
            TransitionRequest::PushOverlay(_) | TransitionRequest::PopOverlay => {
                // Overlay 生命周期由 UI/输入层管理
                // 此处仅记录、不涉及 NextState
            }
        }
    }
}

/// 通用场景清理 — OnExit 时 despawn 所有 SceneRoot 实体。
///
/// 每个 GameState 的 OnExit 都应注册此系统，
/// 确保场景切换时前一场景的实体被完全卸载。
pub fn cleanup_scene(mut commands: Commands, scene_roots: Query<Entity, With<SceneRoot>>) {
    for entity in &scene_roots {
        commands.entity(entity).despawn();
    }
}
