//! DefinitionType trait implementations for Core domain types.
//!
//! These impls live in the Content layer (not Core) to maintain
//! the dependency direction: Content → Core.
//!
//! See ADR-047 §1

use crate::content::loading::{DefinitionType, ValidationError, validate_id_format};
use crate::core::capabilities::cue::foundation::CueDef;
use crate::core::capabilities::effect::foundation::EffectDef;
use crate::core::domains::spell::SpellDef;

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
