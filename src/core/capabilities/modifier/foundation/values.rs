use bevy::prelude::Reflect;

use crate::core::capabilities::modifier::foundation::types::*;

/// 修改器的来源信息。
#[derive(Debug, Clone, Reflect)]
pub struct ModifierSource {
    pub source_type: ModifierSourceType,
    pub source_id: String,
}

/// 修改器运行时实例。
#[derive(Debug, Clone, Reflect)]
pub struct ModifierData {
    pub id: ModifierInstanceId,
    pub op: ModifierOp,
    pub target_attribute: String,
    pub magnitude: f32,
    pub priority: ModifierPriority,
    pub source: ModifierSource,
    pub duration_frames: Option<u64>,
    pub elapsed_frames: u64,
}
