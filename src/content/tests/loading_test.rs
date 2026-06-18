//! Content 加载管线测试

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn discover_ron_files_finds_sample_spell() {
        let config_root = Path::new("assets/config");
        let files = crate::content::loading::discover_ron_files(config_root);

        // Should find at least the fireball.ron file
        assert!(!files.is_empty(), "should discover RON files");

        // Check that fireball.ron was found
        let fireball = files.iter().find(|f| {
            f.path
                .file_name()
                .map(|n| n == "fireball.ron")
                .unwrap_or(false)
        });
        assert!(fireball.is_some(), "should find fireball.ron");

        // Check bucket name is "spells"
        let fireball = fireball.unwrap();
        assert_eq!(fireball.bucket_name, "spells");
    }

    #[test]
    fn fireball_ron_deserializes_and_validates() {
        use crate::content::loading::DefinitionType;
        use crate::core::domains::spell::SpellDef;

        let path = Path::new("assets/config/spells/fireball.ron");
        let content = std::fs::read_to_string(path).expect("fireball.ron should exist");
        let def: SpellDef =
            ron::from_str(&content).expect("fireball.ron should deserialize to SpellDef");

        // Check key fields
        assert_eq!(def.id.as_str(), "spl_000001");
        assert_eq!(def.name_key, "spell.fireball.name");
        assert_eq!(def.desc_key, "spell.fireball.desc");

        // Validation should pass
        assert!(
            def.validate().is_ok(),
            "fireball.ron should pass validation"
        );
    }

    #[test]
    fn content_state_tracks_discovered_files() {
        let config_root = Path::new("assets/config");
        let files = crate::content::loading::discover_ron_files(config_root);

        let mut state = crate::content::ContentState::default();
        state.discovered_files = files;

        let mut counts = std::collections::HashMap::new();
        for file in &state.discovered_files {
            *counts.entry(file.bucket_name.clone()).or_insert(0) += 1;
        }
        state.loaded_counts = counts;

        assert!(state.loaded_counts.contains_key("spells"));
    }
}
