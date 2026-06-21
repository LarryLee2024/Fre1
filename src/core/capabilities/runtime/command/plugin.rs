//! CommandPlugin — 注册命令队列消费系统
//!
//! 在 PreUpdate 调度中注册 command_processing_system，
//! 确保每帧优先处理用户命令。
//!
//! 详见 ADR-043 §3

use bevy::prelude::*;

use super::foundation::CommandQueue;
use super::mechanism::processor::command_processing_system;

/// 命令处理插件
pub struct CommandPlugin;

impl Plugin for CommandPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CommandQueue>()
            .add_systems(PreUpdate, command_processing_system);
    }
}
