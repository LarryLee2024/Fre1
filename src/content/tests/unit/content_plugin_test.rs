use crate::content::content_plugin::{ContentState, LoadedSpellDefs};

#[test]
fn content_state_default_is_empty() {
    let state = ContentState::default();
    assert!(state.discovered_files.is_empty());
    assert!(state.load_errors.is_empty());
    assert!(state.loaded_counts.is_empty());
}

#[test]
fn loaded_spell_defs_default_is_empty() {
    let loaded = LoadedSpellDefs::default();
    assert!(loaded.defs.is_empty());
    assert!(loaded.errors.is_empty());
}
