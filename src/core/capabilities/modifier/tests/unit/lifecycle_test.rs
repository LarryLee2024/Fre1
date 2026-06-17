use crate::core::capabilities::modifier::foundation::{
    ModifierInstanceId, ModifierOp, ModifierSource, ModifierSourceType,
};
use crate::core::capabilities::modifier::mechanism::lifecycle::{
    create_modifier, ModifierIdGenerator, ModifierValidationError,
};

#[test]
fn id_generator_produces_unique_ids() {
    let generator = ModifierIdGenerator::default();
    let id1 = generator.next_id();
    let id2 = generator.next_id();
    assert_ne!(id1, id2);
}

#[test]
fn valid_modifier_creation_succeeds() {
    let source = ModifierSource {
        source_type: ModifierSourceType::Buff,
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
fn priority_out_of_range_rejected() {
    let source = ModifierSource {
        source_type: ModifierSourceType::Buff,
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
fn missing_source_rejected() {
    let source = ModifierSource {
        source_type: ModifierSourceType::Buff,
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
fn missing_target_rejected() {
    let source = ModifierSource {
        source_type: ModifierSourceType::Buff,
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
fn create_multiply_modifier_succeeds() {
    let source = ModifierSource {
        source_type: ModifierSourceType::Ability,
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
