use crate::core::capabilities::cue::foundation::error::CueError;
use crate::core::capabilities::cue::foundation::{
    AnimationParams, CueData, CueDef, CueTag, CueType, PopupDirection, PopupParams, SFXParams,
    ShakeFalloff, ShakeParams, VFXParams,
};

#[test]
fn cue_type_name_correct() {
    let vfx = CueType::VFX(VFXParams::new("explosion"));
    assert_eq!(vfx.name(), "VFX");

    let sfx = CueType::SFX(SFXParams::new("hit"));
    assert_eq!(sfx.name(), "SFX");
}

#[test]
fn vfx_params_constructed_correctly() {
    let vfx = VFXParams::new("explosion")
        .with_attach_point("target_center")
        .with_follow(true)
        .with_duration(30);
    assert_eq!(vfx.effect_key, "explosion");
    assert_eq!(vfx.attach_point, Some("target_center".into()));
    assert!(vfx.follow_target);
    assert_eq!(vfx.duration_frames, Some(30));
}

#[test]
fn sfx_params_constructed_correctly() {
    let sfx = SFXParams::new("fireball");
    assert_eq!(sfx.sound_key, "fireball");
    assert!(sfx.is_3d);
}

#[test]
fn animation_params_constructed_correctly() {
    let anim = AnimationParams::new("death");
    assert_eq!(anim.animation_name, "death");
    assert_eq!(anim.speed, 1.0);
}

#[test]
fn shake_params_constructed_correctly() {
    let shake = ShakeParams::new(0.5, 10);
    assert_eq!(shake.intensity, 0.5);
    assert_eq!(shake.duration_frames, 10);
    assert_eq!(shake.falloff, ShakeFalloff::Linear);
}

#[test]
fn popup_params_constructed_correctly() {
    let popup = PopupParams::new("damage.15", "#FF0000");
    assert_eq!(popup.text_key, "damage.15");
    assert_eq!(popup.color, "#FF0000");
    assert_eq!(popup.font_size, 16);
    assert_eq!(popup.float_direction, PopupDirection::Up);
}

#[test]
fn cue_tag_name_correct() {
    assert_eq!(CueTag::OnApply.name(), "OnApply");
    assert_eq!(CueTag::OnTick.name(), "OnTick");
    assert_eq!(CueTag::OnRemove.name(), "OnRemove");
    assert_eq!(CueTag::OnInterrupt.name(), "OnInterrupt");
    assert_eq!(CueTag::Custom("test".into()).name(), "test");
}

#[test]
fn cue_def_constructed_correctly() {
    let cue = CueDef::new(
        "cue_fireball",
        CueType::VFX(VFXParams::new("explosion")),
        CueTag::OnApply,
    );
    assert_eq!(cue.id, "cue_fireball");
}

#[test]
fn cue_def_critical_flag_correct() {
    let cue = CueDef::new(
        "cue_death",
        CueType::VFX(VFXParams::new("death")),
        CueTag::OnRemove,
    )
    .with_critical();
    assert!(cue.critical);
    assert!(!cue.interruptible);
}

#[test]
fn cue_def_delay_correct() {
    let cue = CueDef::new(
        "cue_explosion",
        CueType::VFX(VFXParams::new("boom")),
        CueTag::OnApply,
    )
    .with_delay(5);
    assert_eq!(cue.delay_frames, Some(5));
}

#[test]
fn cue_data_constructed_correctly() {
    let data = CueData::new(
        "cue_damage",
        CueType::Popup(PopupParams::new("damage.50", "#FFF")),
        CueTag::OnApply,
    )
    .with_source("attacker_001")
    .with_target("defender_001")
    .with_value(50.0);
    assert_eq!(data.source_entity, Some("attacker_001".into()));
    assert_eq!(data.target_entity, Some("defender_001".into()));
    assert_eq!(data.numeric_value, Some(50.0));
}

#[test]
fn cue_data_critical_flag_correct() {
    let data = CueData::new(
        "cue_critical",
        CueType::Popup(PopupParams::new("crit.99", "#FFD700")),
        CueTag::OnApply,
    )
    .with_value(99.0)
    .with_critical();
    assert!(data.critical);
}

#[test]
fn cue_error_message_format_correct() {
    let err = CueError::CueNotFound("cue_missing".into());
    let msg = format!("{}", err);
    assert!(msg.contains("cue_missing"));
}

#[test]
fn shake_falloff_variants_not_equal() {
    assert_ne!(ShakeFalloff::Linear, ShakeFalloff::Exponential);
    assert_ne!(ShakeFalloff::None, ShakeFalloff::Linear);
}

#[test]
fn popup_direction_variants_not_equal() {
    assert_ne!(PopupDirection::Up, PopupDirection::Down);
    assert_eq!(PopupDirection::Left, PopupDirection::Left);
}
