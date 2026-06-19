use std::sync::atomic::{AtomicU64, Ordering};

use bevy::prelude::*;

use crate::core::capabilities::modifier::events::ModifierApplied;
use crate::core::capabilities::modifier::foundation::{
    ModifierData, ModifierInstanceId, ModifierOp, ModifierPriority, ModifierSource,
};

/// 修改器 ID 生成器（Resource）。
/// 提供线程安全的唯一 ID 分配。
#[derive(Resource, Debug)]
pub struct ModifierIdGenerator {
    next_id: AtomicU64,
}

impl Default for ModifierIdGenerator {
    fn default() -> Self {
        Self {
            next_id: AtomicU64::new(1),
        }
    }
}

impl ModifierIdGenerator {
    pub fn next_id(&self) -> ModifierInstanceId {
        ModifierInstanceId::new(self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

/// 修改器校验函数。
pub fn validate_modifier_data(data: &ModifierData) -> Result<(), ModifierValidationError> {
    if (data.priority) > 100 {
        return Err(ModifierValidationError::PriorityOutOfRange(data.priority));
    }
    if data.source.source_id.is_empty() {
        return Err(ModifierValidationError::SourceNotTraceable);
    }
    if data.target_attribute.is_empty() {
        return Err(ModifierValidationError::EmptyTargetAttribute);
    }
    Ok(())
}

/// 修改器校验错误。
#[derive(Debug)]
pub enum ModifierValidationError {
    PriorityOutOfRange(ModifierPriority),
    SourceNotTraceable,
    EmptyTargetAttribute,
}

impl std::fmt::Display for ModifierValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PriorityOutOfRange(p) => {
                write!(f, "modifier priority {} out of range [0, 100]", p)
            }
            Self::SourceNotTraceable => write!(f, "modifier source must be traceable"),
            Self::EmptyTargetAttribute => write!(f, "modifier target attribute must not be empty"),
        }
    }
}

impl std::error::Error for ModifierValidationError {}

/// 创建新修改器，执行校验并返回 ModifierData。
pub fn create_modifier(
    id: ModifierInstanceId,
    op: ModifierOp,
    target_attribute: String,
    magnitude: f32,
    priority: ModifierPriority,
    source: ModifierSource,
    duration_frames: Option<u64>,
    entity: Entity,
    commands: &mut Commands,
) -> Result<ModifierData, ModifierValidationError> {
    let data = ModifierData {
        id,
        op,
        target_attribute,
        magnitude,
        priority,
        source,
        duration_frames,
        elapsed_frames: 0,
    };
    validate_modifier_data(&data)?;
    let cloned = data.clone();
    commands.trigger(ModifierApplied {
        entity,
        modifier_data: cloned,
    });
    Ok(data)
}
