use std::sync::Mutex;

use bevy::prelude::*;

use crate::core::capabilities::modifier::events::ModifierApplied;
use crate::core::capabilities::modifier::foundation::{
    ModifierData, ModifierInstanceId, ModifierOp, ModifierPriority, ModifierSource,
};
use crate::shared::ids::types::runtime_id::RuntimeIdAllocator;

/// 修改器 ID 生成器（Resource）。
/// 通过 `RuntimeIdAllocator` 提供带 generation 保护的唯一 ID 分配。
#[derive(Resource, Debug)]
pub struct ModifierIdGenerator {
    allocator: Mutex<RuntimeIdAllocator>,
}

impl Default for ModifierIdGenerator {
    fn default() -> Self {
        Self {
            allocator: Mutex::new(RuntimeIdAllocator::new()),
        }
    }
}

impl ModifierIdGenerator {
    /// 分配下一个唯一的 ModifierInstanceId。
    pub fn next_id(&self) -> ModifierInstanceId {
        ModifierInstanceId::from_runtime_id(self.allocator.lock().unwrap().alloc())
    }
}

/// 校验 ModifierData 的合法性。
///
/// 检查：priority ∈ [0, 100]、source_id 非空、target_attribute 非空。
/// 校验失败不修改任何状态，直接返回错误。
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
    /// priority 超出 [0, 100] 范围
    PriorityOutOfRange(ModifierPriority),
    /// source.source_id 为空，无法追溯来源
    SourceNotTraceable,
    /// target_attribute 为空，无法确定作用属性
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

/// 创建新修改器：校验 → 构建 ModifierData → 触发 ModifierApplied 事件。
///
/// 校验失败不触发事件，直接返回错误。
/// 副作用：通过 commands.trigger() 发射 ModifierApplied 事件。
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
