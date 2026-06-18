use crate::content::loading::DefinitionType;
use crate::core::capabilities::cue::foundation::*;
use crate::core::capabilities::effect::foundation::*;
use crate::core::domains::spell::*;

fn sample_fireball() -> SpellDef {
    SpellDef {
        id: SpellDefId("spl_000001".to_string()),
        name_key: "spell.fireball.name".to_string(),
        desc_key: "spell.fireball.desc".to_string(),
        level: SpellLevel::L3,
        casting_time: CastingTime::Action,
        components: SpellComponents {
            verbal: true,
            somatic: true,
            material: Some(MaterialComponent {
                description: "spell.fireball.material".to_string(),
                consumed: false,
                cost_gold: None,
            }),
        },
        range: SpellRange::Ranged {
            base: 150,
            max: None,
        },
        duration: SpellDuration::Instant,
        requires_concentration: false,
        saving_throw: Some(SaveType::Dexterity),
        can_upcast: true,
        effects: vec![],
    }
}

#[test]
fn valid_spell_def_passes_validation() {
    let def = sample_fireball();
    assert!(def.validate().is_ok());
}

#[test]
fn spell_def_with_empty_name_fails() {
    let mut def = sample_fireball();
    def.name_key = "".to_string();
    assert!(def.validate().is_err());
}

#[test]
fn spell_def_with_bad_id_prefix_fails() {
    let mut def = sample_fireball();
    def.id = SpellDefId("ab_000001".to_string());
    assert!(def.validate().is_err());
}

#[test]
fn spell_def_without_digit_suffix_fails() {
    let mut def = sample_fireball();
    def.id = SpellDefId("spl_abc".to_string());
    assert!(def.validate().is_err());
}

#[test]
fn spell_def_definition_type_constants() {
    assert_eq!(<SpellDef as DefinitionType>::BUCKET_NAME, "spells");
    assert_eq!(<SpellDef as DefinitionType>::EXTENSION, "ron");
}

fn sample_cue_def() -> CueDef {
    CueDef {
        id: "cue_fireball_explosion".to_string(),
        cue_type: CueType::VFX(VFXParams {
            effect_key: "vfx/fireball_explosion".to_string(),
            attach_point: None,
            follow_target: false,
            duration_frames: Some(30),
            scale: None,
            color_override: None,
        }),
        cue_tag: CueTag::OnApply,
        delay_frames: None,
        interruptible: true,
        critical: false,
    }
}

#[test]
fn valid_cue_def_passes_validation() {
    let def = sample_cue_def();
    assert!(def.validate().is_ok());
}

#[test]
fn cue_def_with_empty_id_fails() {
    let mut def = sample_cue_def();
    def.id = "".to_string();
    assert!(def.validate().is_err());
}

#[test]
fn cue_def_with_bad_id_prefix_fails() {
    let mut def = sample_cue_def();
    def.id = "vfx_explosion".to_string();
    assert!(def.validate().is_err());
}

#[test]
fn cue_def_definition_type_constants() {
    assert_eq!(<CueDef as DefinitionType>::BUCKET_NAME, "cues");
    assert_eq!(<CueDef as DefinitionType>::EXTENSION, "ron");
}

use crate::core::capabilities::effect::foundation::def::{ModifierConfig, ModifierValue};
use crate::core::capabilities::modifier::foundation::ModifierOp;
use crate::core::capabilities::stacking::foundation::StackingConfig;

fn sample_effect_def() -> EffectDef {
    EffectDef {
        id: "eff_000001".to_string(),
        name_key: "effect.eff_000001.name".to_string(),
        desc_key: "effect.eff_000001.desc".to_string(),
        icon_key: None,
        duration: EffectDuration::Instant,
        period: None,
        tick_execution: None,
        modifiers: vec![ModifierConfig {
            op: ModifierOp::Add,
            target_attribute: "attr_000030".to_string(),
            value: ModifierValue::Fixed(-25.0),
            priority: 50,
        }],
        granted_tags: vec![],
        required_tags: None,
        removed_tags: None,
        application_condition: None,
        stacking: StackingConfig::none(),
        effect_category: EffectCategory::Damage,
        execution: None,
        cues: vec![],
        visible: true,
        dispellable: false,
        display_priority: 0,
    }
}

#[test]
fn valid_effect_def_passes_validation() {
    let def = sample_effect_def();
    assert!(def.validate().is_ok());
}

#[test]
fn effect_def_with_empty_name_fails() {
    let mut def = sample_effect_def();
    def.name_key = "".to_string();
    assert!(def.validate().is_err());
}

#[test]
fn effect_def_with_empty_desc_fails() {
    let mut def = sample_effect_def();
    def.desc_key = "".to_string();
    assert!(def.validate().is_err());
}

#[test]
fn effect_def_with_bad_id_prefix_fails() {
    let mut def = sample_effect_def();
    def.id = "abc_000001".to_string();
    assert!(def.validate().is_err());
}

#[test]
fn effect_def_without_digit_suffix_fails() {
    let mut def = sample_effect_def();
    def.id = "eff_abc".to_string();
    assert!(def.validate().is_err());
}

#[test]
fn effect_def_definition_type_constants() {
    assert_eq!(<EffectDef as DefinitionType>::BUCKET_NAME, "effects");
    assert_eq!(<EffectDef as DefinitionType>::EXTENSION, "ron");
}
