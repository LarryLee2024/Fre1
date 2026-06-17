use crate::core::capabilities::cue::foundation::{
    CueBinding, CueContainer, CueDef, CueTag, CueType, VFXParams,
};

fn make_cue(id: &str, tag: CueTag) -> CueDef {
    CueDef::new(id, CueType::VFX(VFXParams::new("test")), tag)
}

#[test]
fn container_initially_empty() {
    let container = CueContainer::new();
    assert!(container.is_empty());
}

#[test]
fn container_registers_cue() {
    let mut container = CueContainer::new();
    container.register(CueBinding::new(make_cue("cue_a", CueTag::OnApply)));
    assert_eq!(container.count(), 1);
}

#[test]
fn container_finds_by_tag() {
    let mut container = CueContainer::new();
    container.register(CueBinding::new(make_cue("cue_a", CueTag::OnApply)));
    container.register(CueBinding::new(make_cue("cue_b", CueTag::OnTick)));

    let apply = container.find_by_tag(&CueTag::OnApply);
    assert_eq!(apply.len(), 1);
}

#[test]
fn container_disables_cue() {
    let mut container = CueContainer::new();
    container.register(CueBinding::new(make_cue("cue_a", CueTag::OnApply)));
    assert!(container.disable("cue_a"));
    assert!(!container.disable("nonexistent"));

    let enabled = container.enabled();
    assert!(enabled.is_empty());
}

#[test]
fn container_enables_cue() {
    let mut container = CueContainer::new();
    container.register(CueBinding::new(make_cue("cue_a", CueTag::OnApply)));
    container.disable("cue_a");
    assert!(container.enable("cue_a"));
    assert_eq!(container.enabled().len(), 1);
}

#[test]
fn container_removes_cue() {
    let mut container = CueContainer::new();
    container.register(CueBinding::new(make_cue("cue_a", CueTag::OnApply)));
    assert!(container.remove("cue_a"));
    assert!(!container.remove("cue_a")); // already removed
}

#[test]
fn container_collects_cue_data() {
    let mut container = CueContainer::new();
    container.register(CueBinding::new(make_cue("cue_a", CueTag::OnApply)));
    container.register(CueBinding::new(make_cue("cue_b", CueTag::OnTick)));

    let apply_defs = container.collect_cue_data(&CueTag::OnApply);
    assert_eq!(apply_defs.len(), 1);
    assert_eq!(apply_defs[0].id, "cue_a");
}

#[test]
fn disabled_cue_excluded_from_collection() {
    let mut container = CueContainer::new();
    container.register(CueBinding::new(make_cue("cue_a", CueTag::OnApply)));
    container.disable("cue_a");

    let apply_defs = container.collect_cue_data(&CueTag::OnApply);
    assert!(apply_defs.is_empty());
}

#[test]
fn container_batch_construction() {
    let bindings = vec![CueBinding::new(make_cue("cue_a", CueTag::OnApply))];
    let container = CueContainer::with_bindings(bindings);
    assert_eq!(container.count(), 1);
}
