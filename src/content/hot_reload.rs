//! 内容热重载 — 定期扫描 RON 文件变更并增量更新 Loaded*Defs Resource

use bevy::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

use super::content_plugin::*;
use super::loading::{ContentFile, DefinitionType, discover_ron_files};
use crate::core::capabilities::attribute::foundation::AttributeDefinition;
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
use crate::core::domains::spell::{SpellConfig, SpellDef};
use crate::core::domains::summon::SummonTemplateDef;

const HOT_RELOAD_INTERVAL_SECS: f32 = 2.0;

/// 热重载状态 Resource。
///
/// 跟踪所有已发现 RON 文件的修改时间戳，用于检测变更。
#[derive(Resource)]
pub struct ContentHotReloadState {
    /// 文件路径 → 上次修改时间。
    pub file_mtimes: HashMap<PathBuf, SystemTime>,
    /// 热重载定时器。
    pub timer: Timer,
    /// 自上次扫描以来的变更文件数量（用于日志）。
    pub last_reload_count: usize,
}

impl Default for ContentHotReloadState {
    fn default() -> Self {
        Self {
            file_mtimes: HashMap::new(),
            timer: Timer::from_seconds(HOT_RELOAD_INTERVAL_SECS, TimerMode::Repeating),
            last_reload_count: 0,
        }
    }
}

/// 内容热重载系统。
///
/// 每隔 2 秒重新扫描 assets/config/ 目录，检测文件修改时间变化，
/// 对变更的文件重新解析并更新对应的 Loaded*Defs Resource。
pub fn hot_reload_content_system(
    time: Res<Time>,
    mut hr_state: ResMut<ContentHotReloadState>,
    mut spells: ResMut<LoadedSpellDefs>,
    mut cues: ResMut<LoadedCueDefs>,
    mut effects: ResMut<LoadedEffectDefs>,
    mut quests: ResMut<LoadedQuestDefs>,
    mut recipes: ResMut<LoadedRecipeDefs>,
    mut shops: ResMut<LoadedShopDefs>,
    mut targeting: ResMut<LoadedTargetingDefs>,
    mut tags: ResMut<LoadedTagDefs>,
    mut attributes: ResMut<LoadedAttributeDefs>,
    mut summon_templates: ResMut<LoadedSummonTemplateDefs>,
    mut camp_events: ResMut<LoadedCampEventDefs>,
    mut bonds: ResMut<LoadedBondDefs>,
    mut enchantments: ResMut<LoadedEnchantmentDefs>,
    mut spell_config: ResMut<SpellConfig>,
) {
    hr_state.timer.tick(time.delta());
    if !hr_state.timer.just_finished() {
        return;
    }

    let config_root = std::path::Path::new("assets/config");
    let files = discover_ron_files(config_root);

    // 构建当前文件的 mtime 映射
    let mut current_mtimes: HashMap<PathBuf, SystemTime> = HashMap::new();
    for file in &files {
        if let Ok(metadata) = std::fs::metadata(&file.path)
            && let Ok(mtime) = metadata.modified()
        {
            current_mtimes.insert(file.path.clone(), mtime);
        }
    }

    // 找出变更的文件（mtime 变化或新增）
    let changed: Vec<&ContentFile> = files
        .iter()
        .filter(|f| {
            match (
                hr_state.file_mtimes.get(&f.path),
                current_mtimes.get(&f.path),
            ) {
                (Some(old), Some(new)) => old != new,
                (None, Some(_)) => true, // 新文件
                _ => false,
            }
        })
        .collect();

    if changed.is_empty() {
        return;
    }

    info!(
        "[HotReload] Detected {} changed file(s), reloading...",
        changed.len()
    );

    let mut reload_count = 0usize;

    for file in &changed {
        let bucket = file.bucket_name.as_str();
        let mut reloaded = false;

        match bucket {
            "spells" => {
                if reload_single_spell(&mut spells, file) {
                    reloaded = true;
                }
            }
            "cues" => {
                if reload_single_cue(&mut cues, file) {
                    reloaded = true;
                }
            }
            "effects" => {
                if reload_single_effect(&mut effects, file) {
                    reloaded = true;
                }
            }
            "quests" => {
                if reload_single_quest(&mut quests, file) {
                    reloaded = true;
                }
            }
            "recipes" => {
                if reload_single_recipe(&mut recipes, file) {
                    reloaded = true;
                }
            }
            "shops" => {
                if reload_single_shop(&mut shops, file) {
                    reloaded = true;
                }
            }
            "targeting" => {
                if reload_single_targeting(&mut targeting, file) {
                    reloaded = true;
                }
            }
            "tags" => {
                if reload_single_tag(&mut tags, file) {
                    reloaded = true;
                }
            }
            "attributes" => {
                if reload_single_attribute(&mut attributes, file) {
                    reloaded = true;
                }
            }
            "summon_templates" => {
                if reload_single_summon_template(&mut summon_templates, file) {
                    reloaded = true;
                }
            }
            "camp_events" => {
                if reload_single_camp_event(&mut camp_events, file) {
                    reloaded = true;
                }
            }
            "bonds" => {
                if reload_single_bond(&mut bonds, file) {
                    reloaded = true;
                }
            }
            "enchantments" => {
                if reload_single_enchantment(&mut enchantments, file) {
                    reloaded = true;
                }
            }
            "spell_config" => {
                if reload_single_spell_config(&mut spell_config, file) {
                    reloaded = true;
                }
            }
            _ => {}
        }

        if reloaded {
            reload_count += 1;
        }
    }

    hr_state.file_mtimes = current_mtimes;
    hr_state.last_reload_count = reload_count;

    if reload_count > 0 {
        info!("[HotReload] Successfully reloaded {} file(s)", reload_count);
    }
}

/// 初始化热重载状态（填充初始 mtime）。
pub fn init_hot_reload_state(mut hr_state: ResMut<ContentHotReloadState>) {
    let config_root = std::path::Path::new("assets/config");
    let files = discover_ron_files(config_root);

    for file in &files {
        if let Ok(metadata) = std::fs::metadata(&file.path)
            && let Ok(mtime) = metadata.modified()
        {
            hr_state.file_mtimes.insert(file.path.clone(), mtime);
        }
    }

    info!(
        "[HotReload] Initialized with {} tracked file(s)",
        hr_state.file_mtimes.len()
    );
}

// ── 单文件重载辅助函数 ──

fn reload_single_spell(spells: &mut ResMut<LoadedSpellDefs>, file: &ContentFile) -> bool {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            warn!("[HotReload] Failed to read {}: {}", file.path.display(), e);
            return false;
        }
    };
    let def: SpellDef = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            warn!("[HotReload] Failed to parse {}: {}", file.path.display(), e);
            return false;
        }
    };
    if let Err(e) = def.validate() {
        warn!(
            "[HotReload] Validation failed for {}: {}",
            file.path.display(),
            e
        );
        return false;
    }
    // 移除同 ID 的旧定义，插入新定义
    spells.defs.retain(|d| d.id != def.id);
    info!(
        "[HotReload] Reloaded spell '{}' (id: {})",
        def.name_key, def.id
    );
    spells.defs.push(def);
    true
}

fn reload_single_cue(cues: &mut ResMut<LoadedCueDefs>, file: &ContentFile) -> bool {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            warn!("[HotReload] Failed to read {}: {}", file.path.display(), e);
            return false;
        }
    };
    let def: CueDef = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            warn!("[HotReload] Failed to parse {}: {}", file.path.display(), e);
            return false;
        }
    };
    if let Err(e) = def.validate() {
        warn!(
            "[HotReload] Validation failed for {}: {}",
            file.path.display(),
            e
        );
        return false;
    }
    cues.defs.retain(|d| d.id != def.id);
    info!("[HotReload] Reloaded cue '{}'", def.id);
    cues.defs.push(def);
    true
}

fn reload_single_effect(effects: &mut ResMut<LoadedEffectDefs>, file: &ContentFile) -> bool {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            warn!("[HotReload] Failed to read {}: {}", file.path.display(), e);
            return false;
        }
    };
    let def: EffectDef = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            warn!("[HotReload] Failed to parse {}: {}", file.path.display(), e);
            return false;
        }
    };
    if let Err(e) = def.validate() {
        warn!(
            "[HotReload] Validation failed for {}: {}",
            file.path.display(),
            e
        );
        return false;
    }
    effects.defs.retain(|d| d.id != def.id);
    info!(
        "[HotReload] Reloaded effect '{}' (id: {})",
        def.name_key, def.id
    );
    effects.defs.push(def);
    true
}

fn reload_single_quest(quests: &mut ResMut<LoadedQuestDefs>, file: &ContentFile) -> bool {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            warn!("[HotReload] Failed to read {}: {}", file.path.display(), e);
            return false;
        }
    };
    let def: QuestDef = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            warn!("[HotReload] Failed to parse {}: {}", file.path.display(), e);
            return false;
        }
    };
    if let Err(e) = def.validate() {
        warn!(
            "[HotReload] Validation failed for {}: {}",
            file.path.display(),
            e
        );
        return false;
    }
    quests.defs.retain(|d| d.id != def.id);
    info!(
        "[HotReload] Reloaded quest '{}' (id: {})",
        def.name_key, def.id
    );
    quests.defs.push(def);
    true
}

fn reload_single_recipe(recipes: &mut ResMut<LoadedRecipeDefs>, file: &ContentFile) -> bool {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            warn!("[HotReload] Failed to read {}: {}", file.path.display(), e);
            return false;
        }
    };
    let def: RecipeDef = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            warn!("[HotReload] Failed to parse {}: {}", file.path.display(), e);
            return false;
        }
    };
    if let Err(e) = def.validate() {
        warn!(
            "[HotReload] Validation failed for {}: {}",
            file.path.display(),
            e
        );
        return false;
    }
    recipes.defs.retain(|d| d.id != def.id);
    info!(
        "[HotReload] Reloaded recipe '{}' (id: {})",
        def.name_key, def.id
    );
    recipes.defs.push(def);
    true
}

fn reload_single_shop(shops: &mut ResMut<LoadedShopDefs>, file: &ContentFile) -> bool {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            warn!("[HotReload] Failed to read {}: {}", file.path.display(), e);
            return false;
        }
    };
    let def: ShopDef = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            warn!("[HotReload] Failed to parse {}: {}", file.path.display(), e);
            return false;
        }
    };
    if let Err(e) = def.validate() {
        warn!(
            "[HotReload] Validation failed for {}: {}",
            file.path.display(),
            e
        );
        return false;
    }
    shops.defs.retain(|d| d.id != def.id);
    info!(
        "[HotReload] Reloaded shop '{}' (id: {})",
        def.name_key, def.id
    );
    shops.defs.push(def);
    true
}

fn reload_single_targeting(
    targeting: &mut ResMut<LoadedTargetingDefs>,
    file: &ContentFile,
) -> bool {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            warn!("[HotReload] Failed to read {}: {}", file.path.display(), e);
            return false;
        }
    };
    let def: TargetingDef = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            warn!("[HotReload] Failed to parse {}: {}", file.path.display(), e);
            return false;
        }
    };
    if let Err(e) = def.validate() {
        warn!(
            "[HotReload] Validation failed for {}: {}",
            file.path.display(),
            e
        );
        return false;
    }
    info!(
        "[HotReload] Reloaded targeting def (type: {}, shape: {})",
        def.target_type.name(),
        def.shape.name()
    );
    targeting.defs.push(def);
    true
}

fn reload_single_tag(tags: &mut ResMut<LoadedTagDefs>, file: &ContentFile) -> bool {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            warn!("[HotReload] Failed to read {}: {}", file.path.display(), e);
            return false;
        }
    };
    let def: TagDefinition = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            warn!("[HotReload] Failed to parse {}: {}", file.path.display(), e);
            return false;
        }
    };
    if let Err(e) = def.validate() {
        warn!(
            "[HotReload] Validation failed for {}: {}",
            file.path.display(),
            e
        );
        return false;
    }
    tags.defs.retain(|d| d.id != def.id);
    info!(
        "[HotReload] Reloaded tag '{}' (path: {})",
        def.id.as_str(),
        def.path
    );
    tags.defs.push(def);
    true
}

fn reload_single_attribute(
    attributes: &mut ResMut<LoadedAttributeDefs>,
    file: &ContentFile,
) -> bool {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            warn!("[HotReload] Failed to read {}: {}", file.path.display(), e);
            return false;
        }
    };
    let def: AttributeDefinition = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            warn!("[HotReload] Failed to parse {}: {}", file.path.display(), e);
            return false;
        }
    };
    if let Err(e) = def.validate() {
        warn!(
            "[HotReload] Validation failed for {}: {}",
            file.path.display(),
            e
        );
        return false;
    }
    attributes.defs.retain(|d| d.id != def.id);
    info!(
        "[HotReload] Reloaded attribute '{}' (category: {:?})",
        def.id.as_str(),
        def.category
    );
    attributes.defs.push(def);
    true
}

fn reload_single_summon_template(
    templates: &mut ResMut<LoadedSummonTemplateDefs>,
    file: &ContentFile,
) -> bool {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            warn!("[HotReload] Failed to read {}: {}", file.path.display(), e);
            return false;
        }
    };
    let def: SummonTemplateDef = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            warn!("[HotReload] Failed to parse {}: {}", file.path.display(), e);
            return false;
        }
    };
    if let Err(e) = def.validate() {
        warn!(
            "[HotReload] Validation failed for {}: {}",
            file.path.display(),
            e
        );
        return false;
    }
    templates.defs.retain(|d| d.id != def.id);
    info!(
        "[HotReload] Reloaded summon template '{}' (id: {})",
        def.name_key, def.id
    );
    templates.defs.push(def);
    true
}

fn reload_single_camp_event(events: &mut ResMut<LoadedCampEventDefs>, file: &ContentFile) -> bool {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            warn!("[HotReload] Failed to read {}: {}", file.path.display(), e);
            return false;
        }
    };
    let def: CampEventDef = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            warn!("[HotReload] Failed to parse {}: {}", file.path.display(), e);
            return false;
        }
    };
    if let Err(e) = def.validate() {
        warn!(
            "[HotReload] Validation failed for {}: {}",
            file.path.display(),
            e
        );
        return false;
    }
    events.defs.retain(|d| d.id != def.id);
    info!(
        "[HotReload] Reloaded camp event '{}' (id: {})",
        def.title_key, def.id
    );
    events.defs.push(def);
    true
}

fn reload_single_bond(bonds: &mut ResMut<LoadedBondDefs>, file: &ContentFile) -> bool {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            warn!("[HotReload] Failed to read {}: {}", file.path.display(), e);
            return false;
        }
    };
    let def: BondDef = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            warn!("[HotReload] Failed to parse {}: {}", file.path.display(), e);
            return false;
        }
    };
    if let Err(e) = def.validate() {
        warn!(
            "[HotReload] Validation failed for {}: {}",
            file.path.display(),
            e
        );
        return false;
    }
    bonds.defs.retain(|d| d.id != def.id);
    info!(
        "[HotReload] Reloaded bond '{}' (id: {})",
        def.name_key, def.id
    );
    bonds.defs.push(def);
    true
}

fn reload_single_enchantment(
    enchantments: &mut ResMut<LoadedEnchantmentDefs>,
    file: &ContentFile,
) -> bool {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            warn!("[HotReload] Failed to read {}: {}", file.path.display(), e);
            return false;
        }
    };
    let def: EnchantmentDef = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            warn!("[HotReload] Failed to parse {}: {}", file.path.display(), e);
            return false;
        }
    };
    if let Err(e) = def.validate() {
        warn!(
            "[HotReload] Validation failed for {}: {}",
            file.path.display(),
            e
        );
        return false;
    }
    enchantments.defs.retain(|d| d.id != def.id);
    info!(
        "[HotReload] Reloaded enchantment '{}' (id: {})",
        def.name_key, def.id
    );
    enchantments.defs.push(def);
    true
}

fn reload_single_spell_config(config: &mut ResMut<SpellConfig>, file: &ContentFile) -> bool {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            warn!("[HotReload] Failed to read {}: {}", file.path.display(), e);
            return false;
        }
    };
    let cfg: SpellConfig = match ron::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            warn!("[HotReload] Failed to parse {}: {}", file.path.display(), e);
            return false;
        }
    };
    **config = cfg;
    info!(
        "[HotReload] Reloaded spell config (concentration_base_dc: {}, max_concentration: {})",
        config.concentration_base_dc, config.max_concentration
    );
    true
}
