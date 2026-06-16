use std::sync::atomic::{AtomicU64, Ordering};

use bevy::prelude::*;

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
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_001_generator_allocates_unique_ids() {
        let generator = ModifierIdGenerator::default();
        let id1 = generator.next_id();
        let id2 = generator.next_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn unit_002_valid_modifier_passes() {
        let source = ModifierSource {
            source_type: crate::core::capabilities::modifier::foundation::ModifierSourceType::Buff,
            source_id: "buf_000001".to_string(),
        };
        let result = create_modifier(
            ModifierInstanceId::new(1),
            ModifierOp::Add,
            "attr_000001".to_string(),
            5.0,
            50,
            source,
            None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn unit_003_priority_out_of_range_rejected() {
        let source = ModifierSource {
            source_type: crate::core::capabilities::modifier::foundation::ModifierSourceType::Buff,
            source_id: "buf_000001".to_string(),
        };
        let result = create_modifier(
            ModifierInstanceId::new(1),
            ModifierOp::Add,
            "attr_000001".to_string(),
            5.0,
            150,
            source,
            None,
        );
        assert!(matches!(
            result,
            Err(ModifierValidationError::PriorityOutOfRange(150))
        ));
    }

    #[test]
    fn unit_004_empty_source_rejected() {
        let source = ModifierSource {
            source_type: crate::core::capabilities::modifier::foundation::ModifierSourceType::Buff,
            source_id: "".to_string(),
        };
        let result = create_modifier(
            ModifierInstanceId::new(1),
            ModifierOp::Add,
            "attr_000001".to_string(),
            5.0,
            50,
            source,
            None,
        );
        assert!(matches!(
            result,
            Err(ModifierValidationError::SourceNotTraceable)
        ));
    }

    #[test]
    fn unit_005_empty_target_rejected() {
        let source = ModifierSource {
            source_type: crate::core::capabilities::modifier::foundation::ModifierSourceType::Buff,
            source_id: "buf_000001".to_string(),
        };
        let result = create_modifier(
            ModifierInstanceId::new(1),
            ModifierOp::Add,
            "".to_string(),
            5.0,
            50,
            source,
            None,
        );
        assert!(matches!(
            result,
            Err(ModifierValidationError::EmptyTargetAttribute)
        ));
    }

    #[test]
    fn unit_006_multiply_op_created() {
        let source = ModifierSource {
            source_type:
                crate::core::capabilities::modifier::foundation::ModifierSourceType::Ability,
            source_id: "abl_000001".to_string(),
        };
        let result = create_modifier(
            ModifierInstanceId::new(1),
            ModifierOp::Multiply,
            "attr_000001".to_string(),
            1.5,
            50,
            source,
            Some(10),
        );
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.duration_frames, Some(10));
        assert_eq!(data.elapsed_frames, 0);
    }
}
