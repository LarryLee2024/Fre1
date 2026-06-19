use crate::content::loading::DefinitionType;
use crate::core::capabilities::ability::foundation::AbilityDef;
use crate::core::capabilities::attribute::foundation::*;
use crate::core::capabilities::cue::foundation::*;
use crate::core::capabilities::effect::foundation::*;
use crate::core::capabilities::tag::foundation::*;
use crate::core::capabilities::targeting::foundation::*;
use crate::core::domains::crafting::*;
use crate::core::domains::economy::*;
use crate::core::domains::quest::*;
use crate::core::domains::spell::*;
use crate::shared::localization_key::LocalizationKey;

fn sample_fireball() -> SpellDef {
    SpellDef {
        id: SpellDefId("spl_000001".to_string()),
        name_key: LocalizationKey::new("spell.fireball.name"),
        desc_key: LocalizationKey::new("spell.fireball.desc"),
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
    def.name_key = "".into();
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
        name_key: LocalizationKey::new("effect.eff_000001.name"),
        desc_key: LocalizationKey::new("effect.eff_000001.desc"),
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
        ignored_tags: None,
        removed_tags: None,
        remove_effects_with_tags: None,
        application_condition: None,
        stacking: StackingConfig::none(),
        effect_tags: vec!["Status.Damage".to_string()],
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
    def.name_key = "".into();
    assert!(def.validate().is_err());
}

#[test]
fn effect_def_with_empty_desc_fails() {
    let mut def = sample_effect_def();
    def.desc_key = "".into();
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

fn sample_ability_def() -> AbilityDef {
    AbilityDef {
        id: "abl_000001".to_string(),
        name_key: LocalizationKey::new("ability.abl_000001.name"),
        desc_key: LocalizationKey::new("ability.abl_000001.desc"),
        icon_key: None,
        ability_tags: vec!["Ability.Type.Active".to_string()],
        cancel_by_tags: vec![],
        block_by_tags: vec![],
        activation_owned_tags: vec![],
        effect_ids: vec![],
        cooldown_turns: 0,
        shared_cooldown_group: None,
        costs: vec![],
        max_level: 1,
        passive: false,
        interruptible: true,
        cast_time_frames: 0,
        visible: true,
    }
}

#[test]
fn valid_ability_def_passes_validation() {
    let def = sample_ability_def();
    assert!(def.validate().is_ok());
}

#[test]
fn ability_def_with_empty_name_fails() {
    let mut def = sample_ability_def();
    def.name_key = "".into();
    assert!(def.validate().is_err());
}

#[test]
fn ability_def_with_empty_desc_fails() {
    let mut def = sample_ability_def();
    def.desc_key = "".into();
    assert!(def.validate().is_err());
}

#[test]
fn ability_def_wrong_prefix_fails() {
    let mut def = sample_ability_def();
    def.id = "eff_000001".to_string();
    assert!(def.validate().is_err());
}

#[test]
fn ability_def_definition_type_constants() {
    assert_eq!(<AbilityDef as DefinitionType>::BUCKET_NAME, "abilities");
    assert_eq!(<AbilityDef as DefinitionType>::EXTENSION, "ron");
}

fn sample_quest() -> QuestDef {
    QuestDef {
        id: QuestDefId("qst_000001".to_string()),
        name_key: LocalizationKey::new("quest.main_quest.name"),
        desc_key: LocalizationKey::new("quest.main_quest.desc"),
        quest_type: QuestType::Main,
        prerequisites: vec![],
        objectives: vec![ObjectiveDef {
            id: ObjectiveId("obj_001".to_string()),
            description_key: "quest.main_quest.obj_001".to_string(),
            objective_type: ObjectiveType::Kill {
                enemy_tags: vec!["goblin".to_string()],
            },
            target_value: 5,
            associated_id: None,
        }],
        rewards: QuestRewardDef {
            xp_reward: 100,
            gold_reward: 50,
            item_rewards: vec![],
            reputation_rewards: vec![],
            unlocks: vec![],
        },
        is_critical: false,
        exclusive_with: vec![],
    }
}

#[test]
fn valid_quest_def_passes_validation() {
    let def = sample_quest();
    assert!(def.validate().is_ok());
}

#[test]
fn quest_def_with_empty_name_fails() {
    let mut def = sample_quest();
    def.name_key = "".into();
    assert!(def.validate().is_err());
}

#[test]
fn quest_def_with_empty_desc_fails() {
    let mut def = sample_quest();
    def.desc_key = "".into();
    assert!(def.validate().is_err());
}

#[test]
fn quest_def_with_bad_id_prefix_fails() {
    let mut def = sample_quest();
    def.id = QuestDefId("abc_000001".to_string());
    assert!(def.validate().is_err());
}

#[test]
fn quest_def_without_objectives_fails() {
    let mut def = sample_quest();
    def.objectives = vec![];
    assert!(def.validate().is_err());
}

#[test]
fn quest_def_definition_type_constants() {
    assert_eq!(<QuestDef as DefinitionType>::BUCKET_NAME, "quests");
    assert_eq!(<QuestDef as DefinitionType>::EXTENSION, "ron");
}

#[test]
fn quest_ron_deserializes_and_validates() {
    let path = std::path::Path::new("assets/config/quests/main_quest_001.ron");
    let content = std::fs::read_to_string(path).expect("main_quest_001.ron should exist");
    let def: QuestDef =
        ron::from_str(&content).expect("main_quest_001.ron should deserialize to QuestDef");

    assert_eq!(def.id.as_str(), "qst_000001");
    assert_eq!(def.name_key, "quest.main_quest_001.name");
    assert!(def.validate().is_ok());
    assert_eq!(def.objectives.len(), 2);
    assert!(def.is_critical);
}

fn sample_recipe() -> RecipeDef {
    RecipeDef {
        id: "rcp_000001".to_string(),
        name_key: LocalizationKey::new("recipe.iron_sword.name"),
        station: CraftingStation::Forge,
        skill_requirement: Some(SkillRequirement {
            skill_id: "skill_smithing".to_string(),
            dc: 10,
        }),
        materials: vec![
            MaterialCost {
                item_id: "itm_iron_ingot".to_string(),
                quantity: 3,
            },
            MaterialCost {
                item_id: "itm_wood_handle".to_string(),
                quantity: 1,
            },
        ],
        output: CraftOutput {
            item_id: "itm_iron_sword".to_string(),
            quantity: 1,
            enchantment_slots: 1,
        },
        craft_time: 60,
        craft_type: CraftType::Smithing,
    }
}

#[test]
fn valid_recipe_def_passes_validation() {
    let def = sample_recipe();
    assert!(def.validate().is_ok());
}

#[test]
fn recipe_def_with_empty_name_fails() {
    let mut def = sample_recipe();
    def.name_key = "".into();
    assert!(def.validate().is_err());
}

#[test]
fn recipe_def_with_bad_id_prefix_fails() {
    let mut def = sample_recipe();
    def.id = "abc_000001".to_string();
    assert!(def.validate().is_err());
}

#[test]
fn recipe_def_without_materials_fails() {
    let mut def = sample_recipe();
    def.materials = vec![];
    assert!(def.validate().is_err());
}

#[test]
fn recipe_def_with_empty_output_item_fails() {
    let mut def = sample_recipe();
    def.output.item_id = "".to_string();
    assert!(def.validate().is_err());
}

#[test]
fn recipe_def_definition_type_constants() {
    assert_eq!(<RecipeDef as DefinitionType>::BUCKET_NAME, "recipes");
    assert_eq!(<RecipeDef as DefinitionType>::EXTENSION, "ron");
}

#[test]
fn recipe_ron_deserializes_and_validates() {
    let path = std::path::Path::new("assets/config/recipes/iron_sword.ron");
    let content = std::fs::read_to_string(path).expect("iron_sword.ron should exist");
    let def: RecipeDef =
        ron::from_str(&content).expect("iron_sword.ron should deserialize to RecipeDef");

    assert_eq!(def.id, "rcp_000001");
    assert_eq!(def.name_key, "recipe.iron_sword.name");
    assert!(def.validate().is_ok());
    assert_eq!(def.materials.len(), 2);
    assert_eq!(def.output.item_id, "itm_iron_sword");
}

fn sample_shop() -> ShopDef {
    ShopDef {
        id: "shp_000001".to_string(),
        name_key: LocalizationKey::new("shop.general_store.name"),
        faction_id: "fac_merchants_guild".to_string(),
        inventory: vec![
            ShopEntryDef {
                item_id: "itm_health_potion".to_string(),
                base_price: Some(25),
                initial_stock: 10,
                restock_amount: 5,
                buys_stolen: false,
            },
            ShopEntryDef {
                item_id: "itm_mana_potion".to_string(),
                base_price: Some(30),
                initial_stock: 8,
                restock_amount: 3,
                buys_stolen: false,
            },
        ],
        restock_policy: RestockPolicy::Timed { interval_hours: 24 },
    }
}

#[test]
fn valid_shop_def_passes_validation() {
    let def = sample_shop();
    assert!(def.validate().is_ok());
}

#[test]
fn shop_def_with_empty_name_fails() {
    let mut def = sample_shop();
    def.name_key = "".into();
    assert!(def.validate().is_err());
}

#[test]
fn shop_def_with_empty_faction_fails() {
    let mut def = sample_shop();
    def.faction_id = "".to_string();
    assert!(def.validate().is_err());
}

#[test]
fn shop_def_with_bad_id_prefix_fails() {
    let mut def = sample_shop();
    def.id = "abc_000001".to_string();
    assert!(def.validate().is_err());
}

#[test]
fn shop_def_without_inventory_fails() {
    let mut def = sample_shop();
    def.inventory = vec![];
    assert!(def.validate().is_err());
}

#[test]
fn shop_def_definition_type_constants() {
    assert_eq!(<ShopDef as DefinitionType>::BUCKET_NAME, "shops");
    assert_eq!(<ShopDef as DefinitionType>::EXTENSION, "ron");
}

#[test]
fn shop_ron_deserializes_and_validates() {
    let path = std::path::Path::new("assets/config/shops/general_store.ron");
    let content = std::fs::read_to_string(path).expect("general_store.ron should exist");
    let def: ShopDef =
        ron::from_str(&content).expect("general_store.ron should deserialize to ShopDef");

    assert_eq!(def.id, "shp_000001");
    assert_eq!(def.name_key, "shop.general_store.name");
    assert!(def.validate().is_ok());
    assert_eq!(def.inventory.len(), 3);
    assert_eq!(def.faction_id, "fac_merchants_guild");
}

#[test]
fn shop_def_ron_roundtrip() {
    let original = sample_shop();
    let ron_str = ron::to_string(&original).expect("ShopDef should serialize to RON");
    let restored: ShopDef =
        ron::from_str(&ron_str).expect("RON should deserialize back to ShopDef");

    assert_eq!(original.id, restored.id);
    assert_eq!(original.name_key, restored.name_key);
    assert_eq!(original.faction_id, restored.faction_id);
    assert_eq!(original.inventory.len(), restored.inventory.len());
}

// ─── TargetingDef ───────────────────────────────────────────────────

fn sample_targeting() -> TargetingDef {
    TargetingDef::new(TargetType::Enemy, TargetShape::Single, Some(30.0), 1).unwrap()
}

#[test]
fn valid_targeting_def_passes_validation() {
    let def = sample_targeting();
    assert!(def.validate().is_ok());
}

#[test]
fn targeting_def_with_zero_max_targets_fails() {
    let def = TargetingDef::new(TargetType::Enemy, TargetShape::Single, Some(30.0), 0);
    assert!(def.is_err());
}

#[test]
fn targeting_def_definition_type_constants() {
    assert_eq!(<TargetingDef as DefinitionType>::BUCKET_NAME, "targeting");
    assert_eq!(<TargetingDef as DefinitionType>::EXTENSION, "ron");
}

#[test]
fn targeting_ron_deserializes_and_validates() {
    let path = std::path::Path::new("assets/config/targeting/single_enemy.ron");
    let content = std::fs::read_to_string(path).expect("single_enemy.ron should exist");
    let def: TargetingDef =
        ron::from_str(&content).expect("single_enemy.ron should deserialize to TargetingDef");

    assert_eq!(def.target_type, TargetType::Enemy);
    assert_eq!(def.shape, TargetShape::Single);
    assert_eq!(def.max_targets, 1);
    assert!(def.require_los);
    assert!(def.validate().is_ok());
}

// ─── TagDefinition ──────────────────────────────────────────────────

fn sample_tag() -> TagDefinition {
    TagDefinition {
        id: TagId::new("tag:fire"),
        path: "DamageType.Elemental.Fire".to_string(),
        parent_id: None,
        bit_index: 0,
        is_abstract: false,
        namespace: TagNamespace::Damage,
    }
}

#[test]
fn valid_tag_def_passes_validation() {
    let def = sample_tag();
    assert!(def.validate().is_ok());
}

#[test]
fn tag_def_with_empty_path_fails() {
    let mut def = sample_tag();
    def.path = "".to_string();
    assert!(def.validate().is_err());
}

#[test]
fn tag_def_definition_type_constants() {
    assert_eq!(<TagDefinition as DefinitionType>::BUCKET_NAME, "tags");
    assert_eq!(<TagDefinition as DefinitionType>::EXTENSION, "ron");
}

#[test]
fn tag_ron_deserializes_and_validates() {
    let path = std::path::Path::new("assets/config/tags/fire.ron");
    let content = std::fs::read_to_string(path).expect("fire.ron should exist");
    let def: TagDefinition =
        ron::from_str(&content).expect("fire.ron should deserialize to TagDefinition");

    assert_eq!(def.id.as_str(), "fire");
    assert_eq!(def.path, "DamageType.Elemental.Fire");
    assert_eq!(def.namespace, TagNamespace::Damage);
    assert!(!def.is_abstract);
    assert!(def.validate().is_ok());
}

// ─── AttributeDefinition ────────────────────────────────────────────

fn sample_attribute() -> AttributeDefinition {
    AttributeDefinition {
        id: AttributeId::new("attr:hp"),
        category: AttributeCategory::Primary,
        default_base_value: 100.0,
        min_value: 0.0,
        max_value: 999.0,
        derived_dependencies: vec![],
        hidden: false,
    }
}

#[test]
fn valid_attribute_def_passes_validation() {
    let def = sample_attribute();
    assert!(def.validate().is_ok());
}

#[test]
fn attribute_def_with_inverted_min_max_fails() {
    let mut def = sample_attribute();
    def.min_value = 100.0;
    def.max_value = 0.0;
    assert!(def.validate().is_err());
}

#[test]
fn attribute_def_definition_type_constants() {
    assert_eq!(
        <AttributeDefinition as DefinitionType>::BUCKET_NAME,
        "attributes"
    );
    assert_eq!(<AttributeDefinition as DefinitionType>::EXTENSION, "ron");
}

#[test]
fn attribute_ron_deserializes_and_validates() {
    let path = std::path::Path::new("assets/config/attributes/hp.ron");
    let content = std::fs::read_to_string(path).expect("hp.ron should exist");
    let def: AttributeDefinition =
        ron::from_str(&content).expect("hp.ron should deserialize to AttributeDefinition");

    assert_eq!(def.id.as_str(), "hp");
    assert_eq!(def.category, AttributeCategory::Primary);
    assert_eq!(def.default_base_value, 100.0);
    assert_eq!(def.min_value, 0.0);
    assert_eq!(def.max_value, 999.0);
    assert!(def.validate().is_ok());
}
