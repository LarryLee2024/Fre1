//! Resources — 反应领域全局资源
//!
//! 定义反应系统的全局配置与运行时队列资源。

use bevy::prelude::*;

use super::components::ReactionQueue;

/// 反应系统配置 Resource。
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct ReactionConfig {
    /// 默认每回合反应次数上限。
    pub max_reactions_per_turn: u32,
    /// 基础优先级偏移（防御型反应获得 +1000 优先级）。
    pub defense_priority_bonus: u32,
}

impl Default for ReactionConfig {
    fn default() -> Self {
        Self {
            max_reactions_per_turn: 1,
            defense_priority_bonus: 1000,
        }
    }
}

/// 全局反应队列 Resource（瞬时）。
///
/// 当前帧待处理的反应队列，一帧内创建、消费、销毁。
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct GlobalReactionQueue {
    /// 内部队列。
    pub queue: ReactionQueue,
}

impl GlobalReactionQueue {
    /// 创建空队列。
    pub fn new() -> Self {
        Self {
            queue: ReactionQueue::new(),
        }
    }

    /// 清空队列。
    pub fn clear(&mut self) {
        self.queue = ReactionQueue::new();
    }
}

impl Default for GlobalReactionQueue {
    fn default() -> Self {
        Self::new()
    }
}
