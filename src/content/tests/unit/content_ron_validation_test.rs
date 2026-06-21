//! 全面的内容 RON 校验测试。
//!
//! 对于 `assets/config/` 中有 RON 文件的每个内容桶，此测试
//! 验证每个文件：
//! 1. 可以从磁盘读取
//! 2. 反序列化为正确的 Rust 类型
//! 3. 通过 `DefinitionType::validate()`
//!
//! 对于有 ID 的类型，还验证文件间不存在重复 ID。

use std::collections::HashSet;
use std::path::Path;

use crate::content::loading::DefinitionType;
use crate::core::capabilities::ability::foundation::AbilityDef;
use crate::core::capabilities::attribute::foundation::*;
use crate::core::capabilities::cue::foundation::*;
use crate::core::capabilities::effect::foundation::*;
use crate::core::capabilities::rule::foundation::RuleDef;
use crate::core::capabilities::tag::foundation::*;
use crate::core::capabilities::targeting::foundation::*;
use crate::core::domains::camp_rest::CampEventDef;
use crate::core::domains::crafting::*;
use crate::core::domains::economy::*;
use crate::core::domains::party::BondDef;
use crate::core::domains::quest::*;
use crate::core::domains::spell::*;
use crate::core::domains::summon::SummonTemplateDef;

// ─── helpers ────────────────────────────────────────────────────────────

/// 返回内容桶目录的路径。
fn bucket_dir(name: &str) -> std::path::PathBuf {
    Path::new("assets/config").join(name)
}

/// 返回给定目录中排序后的 `.ron` 文件，如果目录不存在则返回空 vec。
fn ron_files_in(dir: &Path) -> Vec<std::path::PathBuf> {
    if !dir.exists() {
        return vec![];
    }
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("failed to read directory {:?}: {}", dir, e))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "ron"))
        .collect();
    files.sort();
    files
}

/// 读取文件，失败时使用描述性消息 panic。
fn read_to_string(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {:?}: {}", path, e))
}

// ─── helpers: validate all RON files in a bucket ────────────────────────

/// Deserialize every `.ron` file as `T` (individual struct), validate,
/// and call `visit` on each parsed definition (e.g. to collect IDs).
fn for_each_individual_ron<T, F>(bucket: &str, mut visit: F) -> usize
where
    T: serde::de::DeserializeOwned + DefinitionType,
    F: FnMut(&T, &Path),
{
    let dir = bucket_dir(bucket);
    let files = ron_files_in(&dir);
    assert!(
        !files.is_empty(),
        "No .ron files found in assets/config/{}",
        bucket
    );

    for path in &files {
        let content = read_to_string(path);
        let def: T = ron::from_str(&content)
            .unwrap_or_else(|e| panic!("Deserialization of {:?} failed: {}", path, e));
        def.validate()
            .unwrap_or_else(|e| panic!("Validation of {:?} failed: {}", path, e));
        visit(&def, path);
    }

    eprintln!(
        "  [OK] {} files validated in assets/config/{}",
        files.len(),
        bucket
    );
    files.len()
}

/// Deserialize every `.ron` file as `Vec<T>`, validate each element,
/// and call `visit` on each parsed definition.
fn for_each_vec_ron<T, F>(bucket: &str, mut visit: F) -> usize
where
    T: serde::de::DeserializeOwned + DefinitionType,
    F: FnMut(&T, &Path, usize), // (&def, &path, index_in_vec)
{
    let dir = bucket_dir(bucket);
    let files = ron_files_in(&dir);
    assert!(
        !files.is_empty(),
        "No .ron files found in assets/config/{}",
        bucket
    );

    let mut total = 0;
    for path in &files {
        let content = read_to_string(path);
        let defs: Vec<T> = ron::from_str(&content)
            .unwrap_or_else(|e| panic!("Deserialization of {:?} failed: {}", path, e));
        for (i, def) in defs.iter().enumerate() {
            def.validate()
                .unwrap_or_else(|e| panic!("Validation of {:?}[{}] failed: {}", path, i, e));
            visit(def, path, i);
            total += 1;
        }
    }

    eprintln!(
        "  [OK] {} entries across {} files validated in assets/config/{}",
        total,
        files.len(),
        bucket
    );
    total
}

// ─── helpers: mixed single-or-vec (tags) ────────────────────────────────

/// Deserialize every `.ron` file — either as a single `T` or as `Vec<T>` —
/// validate each entry, and call `visit` on each parsed definition.
fn for_each_mixed_ron<T, F>(bucket: &str, mut visit: F) -> usize
where
    T: serde::de::DeserializeOwned + DefinitionType,
    F: FnMut(&T, &Path),
{
    let dir = bucket_dir(bucket);
    let files = ron_files_in(&dir);
    assert!(
        !files.is_empty(),
        "No .ron files found in assets/config/{}",
        bucket
    );

    let mut total = 0;
    for path in &files {
        let content = read_to_string(path);

        // Try Vec<T> first (multi-entry files).
        if let Ok(defs) = ron::from_str::<Vec<T>>(&content) {
            for def in &defs {
                def.validate().unwrap_or_else(|e| {
                    panic!("Validation of {:?} (vec entry) failed: {}", path, e)
                });
                visit(def, path);
                total += 1;
            }
        } else {
            // Fall back to single T.
            let def: T = ron::from_str(&content).unwrap_or_else(|e| {
                panic!(
                    "Deserialization of {:?} failed (tried both Vec and single): {}",
                    path, e
                )
            });
            def.validate()
                .unwrap_or_else(|e| panic!("Validation of {:?} (single) failed: {}", path, e));
            visit(&def, path);
            total += 1;
        }
    }

    eprintln!(
        "  [OK] {} entries across {} files validated in assets/config/{}",
        total,
        files.len(),
        bucket
    );
    total
}

// ─── SpellDef ───────────────────────────────────────────────────────────

#[test]
fn all_spell_ron_files_deserialize_and_validate() {
    let mut ids = HashSet::new();
    for_each_individual_ron::<SpellDef, _>("spells", |def, path| {
        let id = def.id.0.clone();
        assert!(
            ids.insert(id.clone()),
            "Duplicate SpellDef ID '{}' in {:?}",
            id,
            path
        );
    });
}

// ─── CueDef ─────────────────────────────────────────────────────────────

#[test]
fn all_cue_ron_files_deserialize_and_validate() {
    let mut ids = HashSet::new();
    for_each_individual_ron::<CueDef, _>("cues", |def, path| {
        assert!(
            ids.insert(def.id.clone()),
            "Duplicate CueDef ID '{}' in {:?}",
            def.id,
            path
        );
    });
}

// ─── EffectDef ──────────────────────────────────────────────────────────

#[test]
fn all_effect_ron_files_deserialize_and_validate() {
    let mut ids = HashSet::new();
    for_each_individual_ron::<EffectDef, _>("effects", |def, path| {
        assert!(
            ids.insert(def.id.clone()),
            "Duplicate EffectDef ID '{}' in {:?}",
            def.id,
            path
        );
    });
}

// ─── QuestDef ───────────────────────────────────────────────────────────

#[test]
fn all_quest_ron_files_deserialize_and_validate() {
    let mut ids = HashSet::new();
    for_each_individual_ron::<QuestDef, _>("quests", |def, path| {
        let id = def.id.0.clone();
        assert!(
            ids.insert(id.clone()),
            "Duplicate QuestDef ID '{}' in {:?}",
            id,
            path
        );
    });
}

// ─── RecipeDef ──────────────────────────────────────────────────────────

#[test]
fn all_recipe_ron_files_deserialize_and_validate() {
    let mut ids = HashSet::new();
    for_each_individual_ron::<RecipeDef, _>("recipes", |def, path| {
        assert!(
            ids.insert(def.id.clone()),
            "Duplicate RecipeDef ID '{}' in {:?}",
            def.id,
            path
        );
    });
}

// ─── ShopDef ────────────────────────────────────────────────────────────

#[test]
fn all_shop_ron_files_deserialize_and_validate() {
    let mut ids = HashSet::new();
    for_each_individual_ron::<ShopDef, _>("shops", |def, path| {
        assert!(
            ids.insert(def.id.clone()),
            "Duplicate ShopDef ID '{}' in {:?}",
            def.id,
            path
        );
    });
}

// ─── TargetingDef ───────────────────────────────────────────────────────

#[test]
fn all_targeting_ron_files_deserialize_and_validate() {
    // TargetingDef has no `id` field, so only validate.
    for_each_individual_ron::<TargetingDef, _>("targeting", |_, _| {});
}

// ─── TagDefinition (mixed single / Vec format) ──────────────────────────

#[test]
fn all_tag_ron_files_deserialize_and_validate() {
    let mut ids = HashSet::new();
    for_each_mixed_ron::<TagDefinition, _>("tags", |def, path| {
        let id = def.id.0.clone();
        assert!(
            ids.insert(id.clone()),
            "Duplicate TagDefinition ID '{}' in {:?}",
            id,
            path
        );
    });
}

// ─── AttributeDefinition (always Vec format) ────────────────────────────

#[test]
fn all_attribute_ron_files_deserialize_and_validate() {
    let mut ids = HashSet::new();
    for_each_vec_ron::<AttributeDefinition, _>("attributes", |def, path, idx| {
        let id = def.id.0.clone();
        assert!(
            ids.insert(id.clone()),
            "Duplicate AttributeDefinition ID '{}' in {:?}[{}]",
            id,
            path,
            idx
        );
    });
}

// ─── SummonTemplateDef ──────────────────────────────────────────────────

#[test]
fn all_summon_template_ron_files_deserialize_and_validate() {
    let mut ids = HashSet::new();
    for_each_individual_ron::<SummonTemplateDef, _>("summon_templates", |def, path| {
        assert!(
            ids.insert(def.id.clone()),
            "Duplicate SummonTemplateDef ID '{}' in {:?}",
            def.id,
            path
        );
    });
}

// ─── CampEventDef ───────────────────────────────────────────────────────

#[test]
fn all_camp_event_ron_files_deserialize_and_validate() {
    let mut ids = HashSet::new();
    for_each_individual_ron::<CampEventDef, _>("camp_events", |def, path| {
        let id = def.id.0.clone();
        assert!(
            ids.insert(id.clone()),
            "Duplicate CampEventDef ID '{}' in {:?}",
            id,
            path
        );
    });
}

// ─── BondDef ────────────────────────────────────────────────────────────

#[test]
fn all_bond_ron_files_deserialize_and_validate() {
    let mut ids = HashSet::new();
    for_each_individual_ron::<BondDef, _>("bonds", |def, path| {
        let id = def.id.0.clone();
        assert!(
            ids.insert(id.clone()),
            "Duplicate BondDef ID '{}' in {:?}",
            id,
            path
        );
    });
}

// ─── EnchantmentDef ─────────────────────────────────────────────────────

#[test]
fn all_enchantment_ron_files_deserialize_and_validate() {
    let mut ids = HashSet::new();
    for_each_individual_ron::<EnchantmentDef, _>("enchantments", |def, path| {
        assert!(
            ids.insert(def.id.clone()),
            "Duplicate EnchantmentDef ID '{}' in {:?}",
            def.id,
            path
        );
    });
}

// ─── AbilityDef (bucket exists but may have no files yet) ───────────────

#[test]
fn all_ability_ron_files_deserialize_and_validate() {
    let dir = bucket_dir("abilities");
    let files = ron_files_in(&dir);
    if files.is_empty() {
        eprintln!("  [SKIP] assets/config/abilities has no .ron files yet");
        return;
    }

    let mut ids = HashSet::new();
    let mut count = 0;
    for path in &files {
        let content = read_to_string(path);
        let def: AbilityDef = ron::from_str(&content)
            .unwrap_or_else(|e| panic!("Deserialization of {:?} failed: {}", path, e));
        def.validate()
            .unwrap_or_else(|e| panic!("Validation of {:?} failed: {}", path, e));
        assert!(
            ids.insert(def.id.clone()),
            "Duplicate AbilityDef ID '{}' in {:?}",
            def.id,
            path
        );
        count += 1;
    }
    eprintln!(
        "  [OK] {} files validated in assets/config/abilities",
        count
    );
}

// ─── RuleDef (bucket exists but may have no files yet) ──────────────────

#[test]
fn all_rule_ron_files_deserialize_and_validate() {
    let dir = bucket_dir("rules");
    let files = ron_files_in(&dir);
    if files.is_empty() {
        eprintln!("  [SKIP] assets/config/rules has no .ron files yet");
        return;
    }

    let mut ids = HashSet::new();
    let mut count = 0;
    for path in &files {
        let content = read_to_string(path);
        let def: RuleDef = ron::from_str(&content)
            .unwrap_or_else(|e| panic!("Deserialization of {:?} failed: {}", path, e));
        def.validate()
            .unwrap_or_else(|e| panic!("Validation of {:?} failed: {}", path, e));
        assert!(
            ids.insert(def.id.clone()),
            "Duplicate RuleDef ID '{}' in {:?}",
            def.id,
            path
        );
        count += 1;
    }
    eprintln!("  [OK] {} files validated in assets/config/rules", count);
}
