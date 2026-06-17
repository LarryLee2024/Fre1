use crate::core::capabilities::attribute::foundation::{
    AttributeCategory, AttributeDefinition, AttributeId, DerivedFormula, FormulaParameters,
    FormulaType,
};
use crate::core::capabilities::attribute::mechanism::lifecycle::{
    AttributeRegistrationError, AttributeRegistry,
};

fn make_attr(
    id: &str,
    category: AttributeCategory,
    default: f32,
    min: f32,
    max: f32,
    deps: Vec<&str>,
) -> AttributeDefinition {
    AttributeDefinition {
        id: AttributeId::new(id),
        category,
        default_base_value: default,
        min_value: min,
        max_value: max,
        derived_dependencies: deps.into_iter().map(AttributeId::new).collect(),
        hidden: false,
    }
}

#[test]
fn register_primary_attribute_succeeds() {
    let mut reg = AttributeRegistry::default();
    let def = make_attr(
        "attr_000001",
        AttributeCategory::Primary,
        10.0,
        1.0,
        30.0,
        vec![],
    );
    assert!(reg.register(def).is_ok());
    assert!(reg.contains(&AttributeId::new("attr_000001")));
}

#[test]
fn duplicate_id_rejected() {
    let mut reg = AttributeRegistry::default();
    reg.register(make_attr(
        "attr_000001",
        AttributeCategory::Primary,
        10.0,
        1.0,
        30.0,
        vec![],
    ))
    .unwrap();
    let result = reg.register(make_attr(
        "attr_000001",
        AttributeCategory::Primary,
        10.0,
        1.0,
        30.0,
        vec![],
    ));
    assert!(matches!(
        result,
        Err(AttributeRegistrationError::DuplicateId(_))
    ));
}

#[test]
fn default_value_out_of_range_rejected() {
    let mut reg = AttributeRegistry::default();
    let result = reg.register(make_attr(
        "attr_000001",
        AttributeCategory::Primary,
        50.0,
        1.0,
        30.0,
        vec![],
    ));
    assert!(matches!(
        result,
        Err(AttributeRegistrationError::DefaultValueOutOfRange { .. })
    ));
}

#[test]
fn resource_attr_min_below_zero_rejected() {
    let mut reg = AttributeRegistry::default();
    let result = reg.register(make_attr(
        "attr_000001",
        AttributeCategory::Resource,
        100.0,
        -10.0,
        100.0,
        vec![],
    ));
    assert!(matches!(
        result,
        Err(AttributeRegistrationError::ResourceMinBelowZero(_))
    ));
}

#[test]
fn derived_attr_dependency_not_found_rejected() {
    let mut reg = AttributeRegistry::default();
    let result = reg.register(make_attr(
        "attr_000010",
        AttributeCategory::Derived,
        50.0,
        0.0,
        100.0,
        vec!["attr_999999"],
    ));
    assert!(matches!(
        result,
        Err(AttributeRegistrationError::DerivedDependencyNotFound { .. })
    ));
}

#[test]
fn register_formula_succeeds() {
    let mut reg = AttributeRegistry::default();
    reg.register(make_attr(
        "attr_000001",
        AttributeCategory::Primary,
        10.0,
        1.0,
        30.0,
        vec![],
    ))
    .unwrap();
    reg.register(make_attr(
        "attr_000010",
        AttributeCategory::Derived,
        50.0,
        0.0,
        100.0,
        vec!["attr_000001"],
    ))
    .unwrap();
    let formula = DerivedFormula {
        target_attr_id: AttributeId::new("attr_000010"),
        formula_type: FormulaType::Sum,
        parameters: FormulaParameters {
            constant: None,
            source_ids: Some(vec![AttributeId::new("attr_000001")]),
            multiplier: Some(1.0),
            weights: None,
            base: None,
            formula_id: None,
        },
    };
    assert!(reg.register_formula(formula).is_ok());
}

#[test]
fn formula_target_not_found_rejected() {
    let mut reg = AttributeRegistry::default();
    let formula = DerivedFormula {
        target_attr_id: AttributeId::new("attr_999999"),
        formula_type: FormulaType::Constant,
        parameters: FormulaParameters {
            constant: Some(10.0),
            source_ids: None,
            multiplier: None,
            weights: None,
            base: None,
            formula_id: None,
        },
    };
    let result = reg.register_formula(formula);
    assert!(matches!(
        result,
        Err(AttributeRegistrationError::FormulaTargetNotFound(_))
    ));
}
