//! InputPlugin — 输入系统 ECS Plugin
//!
//! 注册输入基础设施：
//! - InputMap Resource（按键绑定配置，默认值在 Default impl 中）
//! - InputState Resource（当前帧输入状态）
//! - CommandQueue Resource（统一命令入口，实例化自 core::runtime::command）
//! - PreUpdate Systems（input collection → meta command dispatch）
//!
//! 详见 ADR-043
//! 详见 docs/04-data/infrastructure/input_schema.md

use bevy::prelude::*;

use crate::core::capabilities::runtime::command::foundation::CommandQueue;

use super::action::InputMap;
use super::resources::InputState;
use super::systems::{collect_input_actions, process_meta_commands};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        // ── Resources ──
        app.init_resource::<InputMap>();
        app.init_resource::<InputState>();
        app.insert_resource(CommandQueue::new());

        // ── PreUpdate Systems ──
        // 先采集原始输入 → InputState，再处理元命令入队
        app.add_systems(PreUpdate, (collect_input_actions, process_meta_commands));

        tracing::info!("[InputPlugin] 已初始化");
    }
}
