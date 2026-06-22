//! 内容热重载 — 定期扫描 RON 文件变更并增量更新 LoadedDefs Resource

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
/// 对变更的文件重新解析并更新对应的 LoadedDefs Resource。
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

    info!(target: "content",
        "[HotReload] 检测到 {} 个变更文件，正在重载...",
        changed.len()
    );

    let mut reload_count = 0usize;

    for file in &changed {
        let bucket = file.bucket_name.as_str();
        let mut reloaded = false;

        match bucket {
            "spells" => {
                if reload_single_def::<SpellDef>(&mut spells, file) {
                    reloaded = true;
                }
            }
            "cues" => {
                if reload_single_def::<CueDef>(&mut cues, file) {
                    reloaded = true;
                }
            }
            "effects" => {
                if reload_single_def::<EffectDef>(&mut effects, file) {
                    reloaded = true;
                }
            }
            "quests" => {
                if reload_single_def::<QuestDef>(&mut quests, file) {
                    reloaded = true;
                }
            }
            "recipes" => {
                if reload_single_def::<RecipeDef>(&mut recipes, file) {
                    reloaded = true;
                }
            }
            "shops" => {
                if reload_single_def::<ShopDef>(&mut shops, file) {
                    reloaded = true;
                }
            }
            "targeting" => {
                if reload_single_def::<TargetingDef>(&mut targeting, file) {
                    reloaded = true;
                }
            }
            "tags" => {
                if reload_single_def::<TagDefinition>(&mut tags, file) {
                    reloaded = true;
                }
            }
            "attributes" => {
                if reload_single_def::<AttributeDefinition>(&mut attributes, file) {
                    reloaded = true;
                }
            }
            "summon_templates" => {
                if reload_single_def::<SummonTemplateDef>(&mut summon_templates, file) {
                    reloaded = true;
                }
            }
            "camp_events" => {
                if reload_single_def::<CampEventDef>(&mut camp_events, file) {
                    reloaded = true;
                }
            }
            "bonds" => {
                if reload_single_def::<BondDef>(&mut bonds, file) {
                    reloaded = true;
                }
            }
            "enchantments" => {
                if reload_single_def::<EnchantmentDef>(&mut enchantments, file) {
                    reloaded = true;
                }
            }
            "spell_config" if reload_single_spell_config(&mut spell_config, file) => {
                reloaded = true;
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
        info!(target: "content", "[HotReload] 成功重载了 {} 个文件", reload_count);
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

    info!(target: "content",
        "[HotReload] 已初始化，跟踪 {} 个文件",
        hr_state.file_mtimes.len()
    );
}

// ── 单文件重载辅助函数 ──

/// 从 RON 文件重载单个 Definition（通用版本）。
///
/// 处理单条和数组格式（通过 `supports_multi_def()` 开关）。
/// 通过 `def_unique_id()` 去重：移除同 ID 的旧定义后再插入新定义。
fn reload_single_def<T: DefinitionType>(loaded: &mut LoadedDefs<T>, file: &ContentFile) -> bool {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            warn!(target: "content", "[HotReload] 读取失败 {}: {}", file.path.display(), e);
            return false;
        }
    };

    let defs: Vec<T> = if T::supports_multi_def() {
        let trimmed = content.trim();
        if trimmed.starts_with('[') {
            match ron::from_str(trimmed) {
                Ok(d) => d,
                Err(e) => {
                    warn!(target: "content",
                        "[HotReload] 解析数组失败 {}: {}",
                        file.path.display(),
                        e
                    );
                    return false;
                }
            }
        } else {
            match ron::from_str::<T>(trimmed) {
                Ok(d) => vec![d],
                Err(e) => {
                    warn!(target: "content",
                        "[HotReload] 解析失败 {}: {}",
                        file.path.display(),
                        e
                    );
                    return false;
                }
            }
        }
    } else {
        match ron::from_str::<T>(&content) {
            Ok(d) => vec![d],
            Err(e) => {
                warn!(target: "content",
                    "[HotReload] 解析失败 {}: {}",
                    file.path.display(),
                    e
                );
                return false;
            }
        }
    };

    // 校验所有定义
    for def in &defs {
        if let Err(e) = def.validate() {
            warn!(target: "content",
                "[HotReload] 验证失败 {}: {}",
                file.path.display(),
                e
            );
            return false;
        }
    }

    // 去重：对每个新定义，移除旧版本（若有唯一 ID）
    for def in &defs {
        if let Some(uid) = def.def_unique_id() {
            loaded.defs.retain(|d| d.def_unique_id() != Some(uid));
        }
    }

    let count = defs.len();
    for def in defs {
        loaded.defs.push(def);
    }

    if count == 1 {
        info!(target: "content",
            "[HotReload] 重载了定义（{}）",
            file.path.display()
        );
    } else {
        info!(target: "content",
            "[HotReload] 从 {} 重载了 {} 个定义",
            file.path.display(),
            count
        );
    }
    true
}

/// 从 RON 文件重载 SpellConfig。
fn reload_single_spell_config(config: &mut ResMut<SpellConfig>, file: &ContentFile) -> bool {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            warn!(target: "content", "[HotReload] 读取失败 {}: {}", file.path.display(), e);
            return false;
        }
    };
    let cfg: SpellConfig = match ron::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            warn!(target: "content", "[HotReload] 解析失败 {}: {}", file.path.display(), e);
            return false;
        }
    };
    **config = cfg;
    info!(target: "content",
        "[HotReload] 重载了法术配置（专注基础 DC: {}, 最大专注数: {}）",
        config.concentration_base_dc, config.max_concentration
    );
    true
}
