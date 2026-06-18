//! Resources — 任务领域全局资源
//!
//! 定义任务系统的全局配置。

use bevy::prelude::*;

/// 任务系统配置 Resource。
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct QuestConfig {
    /// 最大同时激活任务数（0 = 无限制）。
    pub max_active_quests: u32,
    /// 是否启用自动追踪（接受后自动开始追踪目标）。
    pub auto_tracking: bool,
}

impl Default for QuestConfig {
    fn default() -> Self {
        Self {
            max_active_quests: 50,
            auto_tracking: true,
        }
    }
}
