//! DefinitionType trait implementations for Core domain types.
//!
//! These impls live in the Content layer (not Core) to maintain
//! the dependency direction: Content → Core.
//!
//! See ADR-047 §1

use crate::content::loading::{DefinitionType, ValidationError, validate_id_format};
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

#[cfg(test)]
mod tests {
    use crate::content::loading::DefinitionType;
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
}
