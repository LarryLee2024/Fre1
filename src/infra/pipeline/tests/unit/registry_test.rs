use crate::core::capabilities::runtime::pipeline::foundation::{PipelineDefinition, PipelineStage};
use crate::core::capabilities::runtime::pipeline::registry::PipelineRegistry;

#[test]
fn test_empty_registry() {
    let registry = PipelineRegistry::new();
    assert_eq!(registry.count(), 0);
    assert!(registry.get("nonexistent").is_none());
}

#[test]
fn test_register_and_get() {
    let mut registry = PipelineRegistry::new();
    let def = PipelineDefinition::new("test_pipeline").stage(PipelineStage::new("stage_1"));

    registry.register(def);

    assert_eq!(registry.count(), 1);
    assert!(registry.get("test_pipeline").is_some());
}

#[test]
#[should_panic(expected = "duplicate pipeline registration")]
fn test_duplicate_registration_panics() {
    let mut registry = PipelineRegistry::new();
    let def1 = PipelineDefinition::new("dup_pipeline");
    let def2 = PipelineDefinition::new("dup_pipeline");

    registry.register(def1);
    registry.register(def2); // should panic
}
