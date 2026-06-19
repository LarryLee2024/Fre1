//! DefinitionType trait implementations for Core domain types.
//!
//! These impls live in the Content layer (not Core) to maintain
//! the dependency direction: Content → Core.
//!
//! See ADR-047 §1

use crate::content::loading::{DefinitionType, ValidationError, validate_id_format};
use crate::core::capabilities::attribute::foundation::AttributeDefinition;
use crate::core::capabilities::ability::foundation::AbilityDef;
use crate::core::capabilities::cue::foundation::CueDef;
use crate::core::capabilities::effect::foundation::EffectDef;
use crate::core::capabilities::tag::foundation::TagDefinition;
use crate::core::capabilities::targeting::foundation::TargetingDef;
use crate::core::domains::camp_rest::CampEventDef;
use crate::core::domains::crafting::EnchantmentDef;
use crate::core::domains::crafting::RecipeDef;
use crate::core::domains::economy::ShopDef;
use crate::core::domains::party::BondDef;
use crate::core::domains::quest::QuestDef;
use crate::core::domains::spell::SpellDef;
use crate::core::domains::summon::SummonTemplateDef;

impl DefinitionType for SpellDef {
    const BUCKET_NAME: &'static str = "spells";
    const EXTENSION: &'static str = "ron";

    fn validate(&self) -> Result<(), ValidationError> {
        // ID must start with "spl_" followed by digits
        validate_id_format(&self.id.0, "spl_")?;

        // Name and description keys must be non-empty
        if self.name_key.is_empty() {
            return Err(ValidationError::MissingField {
                field: "name_key".to_string(),
            });
        }
        if self.desc_key.is_empty() {
            return Err(ValidationError::MissingField {
                field: "desc_key".to_string(),
            });
        }

        Ok(())
    }
}

impl DefinitionType for CueDef {
    const BUCKET_NAME: &'static str = "cues";
    const EXTENSION: &'static str = "ron";

    fn validate(&self) -> Result<(), ValidationError> {
        if self.id.is_empty() {
            return Err(ValidationError::EmptyId);
        }
        if !self.id.starts_with("cue_") {
            return Err(ValidationError::InvalidIdPrefix {
                id: self.id.clone(),
                expected_prefix: "cue_".to_string(),
            });
        }
        Ok(())
    }
}

impl DefinitionType for EffectDef {
    const BUCKET_NAME: &'static str = "effects";
    const EXTENSION: &'static str = "ron";

    fn validate(&self) -> Result<(), ValidationError> {
        validate_id_format(&self.id, "eff_")?;

        if self.name_key.is_empty() {
            return Err(ValidationError::MissingField {
                field: "name_key".to_string(),
            });
        }
        if self.desc_key.is_empty() {
            return Err(ValidationError::MissingField {
                field: "desc_key".to_string(),
            });
        }

        Ok(())
    }
}

impl DefinitionType for AbilityDef {
    const BUCKET_NAME: &'static str = "abilities";
    const EXTENSION: &'static str = "ron";

    fn validate(&self) -> Result<(), ValidationError> {
        validate_id_format(&self.id, "abl_")?;

        if self.name_key.is_empty() {
            return Err(ValidationError::MissingField {
                field: "name_key".to_string(),
            });
        }
        if self.desc_key.is_empty() {
            return Err(ValidationError::MissingField {
                field: "desc_key".to_string(),
            });
        }

        Ok(())
    }
}

impl DefinitionType for QuestDef {
    const BUCKET_NAME: &'static str = "quests";
    const EXTENSION: &'static str = "ron";

    fn validate(&self) -> Result<(), ValidationError> {
        validate_id_format(self.id.as_str(), "qst_")?;

        if self.name_key.is_empty() {
            return Err(ValidationError::MissingField {
                field: "name_key".to_string(),
            });
        }
        if self.desc_key.is_empty() {
            return Err(ValidationError::MissingField {
                field: "desc_key".to_string(),
            });
        }
        if self.objectives.is_empty() {
            return Err(ValidationError::MissingField {
                field: "objectives".to_string(),
            });
        }

        Ok(())
    }
}

impl DefinitionType for RecipeDef {
    const BUCKET_NAME: &'static str = "recipes";
    const EXTENSION: &'static str = "ron";

    fn validate(&self) -> Result<(), ValidationError> {
        validate_id_format(&self.id, "rcp_")?;

        if self.name_key.is_empty() {
            return Err(ValidationError::MissingField {
                field: "name_key".to_string(),
            });
        }
        if self.materials.is_empty() {
            return Err(ValidationError::MissingField {
                field: "materials".to_string(),
            });
        }
        if self.output.item_id.is_empty() {
            return Err(ValidationError::MissingField {
                field: "output.item_id".to_string(),
            });
        }

        Ok(())
    }
}

impl DefinitionType for ShopDef {
    const BUCKET_NAME: &'static str = "shops";
    const EXTENSION: &'static str = "ron";

    fn validate(&self) -> Result<(), ValidationError> {
        validate_id_format(&self.id, "shp_")?;

        if self.name_key.is_empty() {
            return Err(ValidationError::MissingField {
                field: "name_key".to_string(),
            });
        }
        if self.faction_id.is_empty() {
            return Err(ValidationError::MissingField {
                field: "faction_id".to_string(),
            });
        }
        if self.inventory.is_empty() {
            return Err(ValidationError::MissingField {
                field: "inventory".to_string(),
            });
        }

        Ok(())
    }
}

impl DefinitionType for TargetingDef {
    const BUCKET_NAME: &'static str = "targeting";
    const EXTENSION: &'static str = "ron";

    fn validate(&self) -> Result<(), ValidationError> {
        if self.max_targets == 0 {
            return Err(ValidationError::MissingField {
                field: "max_targets".to_string(),
            });
        }
        Ok(())
    }
}

impl DefinitionType for TagDefinition {
    const BUCKET_NAME: &'static str = "tags";
    const EXTENSION: &'static str = "ron";

    fn validate(&self) -> Result<(), ValidationError> {
        if self.id.as_str().is_empty() {
            return Err(ValidationError::EmptyId);
        }
        if self.path.is_empty() {
            return Err(ValidationError::MissingField {
                field: "path".to_string(),
            });
        }
        Ok(())
    }
}

impl DefinitionType for AttributeDefinition {
    const BUCKET_NAME: &'static str = "attributes";
    const EXTENSION: &'static str = "ron";

    fn validate(&self) -> Result<(), ValidationError> {
        if self.id.as_str().is_empty() {
            return Err(ValidationError::EmptyId);
        }
        if self.min_value > self.max_value {
            return Err(ValidationError::MissingField {
                field: "min/max_value".to_string(),
            });
        }
        Ok(())
    }
}

impl DefinitionType for SummonTemplateDef {
    const BUCKET_NAME: &'static str = "summon_templates";
    const EXTENSION: &'static str = "ron";

    fn validate(&self) -> Result<(), ValidationError> {
        if self.id.is_empty() {
            return Err(ValidationError::EmptyId);
        }
        if self.name_key.is_empty() {
            return Err(ValidationError::MissingField {
                field: "name_key".to_string(),
            });
        }
        if self.base_attributes.is_empty() {
            return Err(ValidationError::MissingField {
                field: "base_attributes".to_string(),
            });
        }
        Ok(())
    }
}

impl DefinitionType for CampEventDef {
    const BUCKET_NAME: &'static str = "camp_events";
    const EXTENSION: &'static str = "ron";

    fn validate(&self) -> Result<(), ValidationError> {
        if self.id.as_str().is_empty() {
            return Err(ValidationError::EmptyId);
        }
        if self.title_key.is_empty() {
            return Err(ValidationError::MissingField {
                field: "title_key".to_string(),
            });
        }
        if self.desc_key.is_empty() {
            return Err(ValidationError::MissingField {
                field: "desc_key".to_string(),
            });
        }
        Ok(())
    }
}

impl DefinitionType for BondDef {
    const BUCKET_NAME: &'static str = "bonds";
    const EXTENSION: &'static str = "ron";

    fn validate(&self) -> Result<(), ValidationError> {
        if self.id.as_str().is_empty() {
            return Err(ValidationError::EmptyId);
        }
        if self.name_key.is_empty() {
            return Err(ValidationError::MissingField {
                field: "name_key".to_string(),
            });
        }
        if self.desc_key.is_empty() {
            return Err(ValidationError::MissingField {
                field: "desc_key".to_string(),
            });
        }
        if self.required_members.is_empty() {
            return Err(ValidationError::MissingField {
                field: "required_members".to_string(),
            });
        }
        if self.max_level == 0 {
            return Err(ValidationError::MissingField {
                field: "max_level".to_string(),
            });
        }
        Ok(())
    }
}

impl DefinitionType for EnchantmentDef {
    const BUCKET_NAME: &'static str = "enchantments";
    const EXTENSION: &'static str = "ron";

    fn validate(&self) -> Result<(), ValidationError> {
        if self.id.is_empty() {
            return Err(ValidationError::EmptyId);
        }
        if self.name_key.is_empty() {
            return Err(ValidationError::MissingField {
                field: "name_key".to_string(),
            });
        }
        if self.modifier_id.is_empty() {
            return Err(ValidationError::MissingField {
                field: "modifier_id".to_string(),
            });
        }
        Ok(())
    }
}
