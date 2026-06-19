use bevy::prelude::*;

use crate::core::capabilities::cue::foundation::{
    CueBinding, CueContainer, CueData, CueDef, CueTag, CueType, VFXParams,
};
use crate::core::capabilities::cue::mechanism::dispatch::*;

fn make_container() -> CueContainer {
    let mut container = CueContainer::new();
    container.register(CueBinding::new(CueDef::new(
        "cue_explosion",
        CueType::VFX(VFXParams::new("explosion")),
        CueTag::OnApply,
    )));
    container.register(CueBinding::new(CueDef::new(
        "cue_tick",
        CueType::VFX(VFXParams::new("poison_tick")),
        CueTag::OnTick,
    )));
    container.register(CueBinding::new(CueDef::new(
        "cue_remove",
        CueType::VFX(VFXParams::new("dispel")),
        CueTag::OnRemove,
    )));
    container
}

#[test]
fn filter_cue_by_tag() {
    let container = make_container();
    let cues = collect_cues(&container, &CueTag::OnApply, None, None, None);
    assert_eq!(cues.len(), 1);
    assert_eq!(cues[0].cue_def_id, "cue_explosion");
}

#[test]
fn filter_on_tick_cue() {
    let container = make_container();
    let cues = collect_cues(&container, &CueTag::OnTick, None, None, None);
    assert_eq!(cues.len(), 1);
    assert_eq!(cues[0].cue_def_id, "cue_tick");
}

#[test]
fn filter_cue_by_value() {
    let container = make_container();
    let cues = collect_cues(
        &container,
        &CueTag::OnApply,
        Some("caster_001".into()),
        Some("target_001".into()),
        Some(50.0),
    );
    assert_eq!(cues.len(), 1);
    assert_eq!(cues[0].source_entity, Some("caster_001".into()));
    assert_eq!(cues[0].target_entity, Some("target_001".into()));
    assert_eq!(cues[0].numeric_value, Some(50.0));
}

#[test]
fn cue_type_maps_to_dispatch_target() {
    let vfx = CueType::VFX(VFXParams::new("test"));
    assert_eq!(DispatchTarget::from_cue_type(&vfx), DispatchTarget::VFX);

    let sfx = CueType::SFX(crate::core::capabilities::cue::foundation::SFXParams::new(
        "test",
    ));
    assert_eq!(DispatchTarget::from_cue_type(&sfx), DispatchTarget::SFX);
}

#[test]
fn dispatch_target_name_correct() {
    assert_eq!(DispatchTarget::VFX.name(), "VFX");
    assert_eq!(DispatchTarget::Popup.name(), "Popup");
}

#[test]
fn dispatch_cue_returns_target() {
    let mut world = World::new();
    let mut commands = world.commands();
    let cue_data = CueData::new(
        "test",
        CueType::VFX(VFXParams::new("boom")),
        CueTag::OnApply,
    );
    let result = dispatch_cue(&cue_data, &mut commands);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), DispatchTarget::VFX);
}

#[test]
fn active_state_can_trigger() {
    let mut world = World::new();
    let mut commands = world.commands();
    let def = CueDef::new(
        "test",
        CueType::VFX(VFXParams::new("test")),
        CueTag::OnApply,
    );
    assert!(can_trigger(&def, &[], &mut commands));
}

#[test]
fn disabled_state_cannot_trigger() {
    let mut world = World::new();
    let mut commands = world.commands();
    let def = CueDef::new(
        "test",
        CueType::VFX(VFXParams::new("test")),
        CueTag::OnApply,
    );
    assert!(!can_trigger(&def, &["test".into()], &mut commands));
}

#[test]
fn critical_always_can_trigger() {
    let mut world = World::new();
    let mut commands = world.commands();
    let def = CueDef::new(
        "test",
        CueType::VFX(VFXParams::new("test")),
        CueTag::OnApply,
    )
    .with_critical();
    assert!(can_trigger(&def, &["test".into()], &mut commands)); // critical bypasses disabled
}

#[test]
fn dispatch_result_initially_empty() {
    let result = DispatchResult::empty();
    assert_eq!(result.dispatched, 0);
    assert_eq!(result.suppressed, 0);
    assert!(result.errors.is_empty());
}

#[test]
fn disabled_cue_not_collected() {
    let mut container = make_container();
    container.disable("cue_explosion");
    let cues = collect_cues(&container, &CueTag::OnApply, None, None, None);
    assert!(cues.is_empty());
}
