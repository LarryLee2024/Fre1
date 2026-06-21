//! PipelinePlugin — 管线系统 ECS Plugin
//!
//! 注册 PipelineRegistry Resource 和管线执行相关的 ECS 集成。
//!
//! 详见 ADR-044

use bevy::prelude::*;

use crate::core::capabilities::runtime::pipeline::registry::PipelineRegistry;

/// Bevy Plugin：初始化 PipelineRegistry Resource，提供管线注册和查询基础设施。
pub struct PipelinePlugin;

impl Plugin for PipelinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PipelineRegistry>();

        tracing::info!(target: "pipeline", "[PipelinePlugin] 已初始化");
    }
}
