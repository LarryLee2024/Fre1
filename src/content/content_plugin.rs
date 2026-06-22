//! ContentPlugin — 内容桥接层
//!
//! 从 assets/config/ 加载配置 → 校验 → 注册到 Registry。
//! 详见 ADR-047 和 `docs/01-architecture/README.md` §3.5

use bevy::asset::AssetApp;
use bevy::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

use super::hot_reload::{ContentHotReloadState, hot_reload_content_system, init_hot_reload_state};
use super::loading::{ContentFile, DefinitionType, RonAssetLoader, discover_ron_files};
use crate::content::terrain_def::TerrainDef;
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

/// 内容加载状态 Resource。
///
/// 记录已发现和加载的配置文件信息。
#[derive(Resource, Debug, Default)]
pub struct ContentState {
    /// 已发现的配置文件列表。
    pub discovered_files: Vec<ContentFile>,
    /// 加载失败的文件（路径 + 错误信息）。
    pub load_errors: Vec<(PathBuf, String)>,
    /// 各桶已加载的文件数量。
    pub loaded_counts: HashMap<String, usize>,
}

/// 内容加载摘要 Resource。
///
/// 在所有配置加载完成后生成，提供完整的加载结果概览。
#[derive(Resource, Debug, Default)]
pub struct ContentLoadSummary {
    /// 总发现文件数。
    pub total_discovered: usize,
    /// 总成功加载数。
    pub total_loaded: usize,
    /// 总失败数。
    pub total_errors: usize,
    /// 各桶加载统计。
    pub bucket_stats: HashMap<String, BucketLoadStats>,
    /// 所有错误的聚合列表。
    pub all_errors: Vec<(PathBuf, String)>,
}

/// 单个桶的加载统计。
#[derive(Debug, Clone, Default)]
pub struct BucketLoadStats {
    /// 成功加载数。
    pub loaded: usize,
    /// 失败数。
    pub errors: usize,
}

/// 已加载的 Definition 定义集合 Resource。
///
/// 通用泛型版本替代了 14 个重复的 `Loaded*Defs` 结构体。
/// 通过类型别名保持向后兼容。
#[derive(Resource, Debug)]
pub struct LoadedDefs<T: DefinitionType> {
    /// 成功加载并校验通过的定义。
    pub defs: Vec<T>,
    /// 加载或校验失败的文件（路径 + 错误信息）。
    pub errors: Vec<(PathBuf, String)>,
}

// 手动实现 Default 以避免 T: Default 约束（Def 类型不实现 Default）
impl<T: DefinitionType> Default for LoadedDefs<T> {
    fn default() -> Self {
        Self {
            defs: Vec::new(),
            errors: Vec::new(),
        }
    }
}

// ─── 类型别名（向后兼容） ────────────────────────────────────────
/// 已加载的法术定义集合。
pub type LoadedSpellDefs = LoadedDefs<SpellDef>;
/// 已加载的 Cue 信号定义集合。
pub type LoadedCueDefs = LoadedDefs<CueDef>;
/// 已加载的效果定义集合。
pub type LoadedEffectDefs = LoadedDefs<EffectDef>;
/// 已加载的任务定义集合。
pub type LoadedQuestDefs = LoadedDefs<QuestDef>;
/// 已加载的配方定义集合。
pub type LoadedRecipeDefs = LoadedDefs<RecipeDef>;
/// 已加载的商店定义集合。
pub type LoadedShopDefs = LoadedDefs<ShopDef>;
/// 已加载的目标选择定义集合。
pub type LoadedTargetingDefs = LoadedDefs<TargetingDef>;
/// 已加载的标签定义集合。
pub type LoadedTagDefs = LoadedDefs<TagDefinition>;
/// 已加载的属性定义集合。
pub type LoadedAttributeDefs = LoadedDefs<AttributeDefinition>;
/// 已加载的召唤模板定义集合。
pub type LoadedSummonTemplateDefs = LoadedDefs<SummonTemplateDef>;
/// 已加载的营地事件定义集合。
pub type LoadedCampEventDefs = LoadedDefs<CampEventDef>;
/// 已加载的羁绊定义集合。
pub type LoadedBondDefs = LoadedDefs<BondDef>;
/// 已加载的附魔定义集合。
pub type LoadedEnchantmentDefs = LoadedDefs<EnchantmentDef>;
/// 已加载的地形定义集合。
pub type LoadedTerrainDefs = LoadedDefs<TerrainDef>;

/// ContentPlugin — 内容桥接层插件。
///
/// 职责：
/// 1. 扫描 assets/config/ 目录发现 RON 配置文件
/// 2. 按桶分类加载配置
/// 3. 校验 Definition 完整性
/// 4. 注册到 Registry
///
/// 详见 ADR-047
pub struct ContentPlugin;

impl Plugin for ContentPlugin {
    fn build(&self, app: &mut App) {
        // ── 注册 Asset 类型和加载器 ──
        app.init_asset::<SpellDef>()
            .init_asset_loader::<RonAssetLoader<SpellDef>>()
            .init_asset::<CueDef>()
            .init_asset_loader::<RonAssetLoader<CueDef>>()
            .init_asset::<EffectDef>()
            .init_asset_loader::<RonAssetLoader<EffectDef>>()
            .init_asset::<QuestDef>()
            .init_asset_loader::<RonAssetLoader<QuestDef>>()
            .init_asset::<RecipeDef>()
            .init_asset_loader::<RonAssetLoader<RecipeDef>>()
            .init_asset::<ShopDef>()
            .init_asset_loader::<RonAssetLoader<ShopDef>>()
            .init_asset::<TargetingDef>()
            .init_asset_loader::<RonAssetLoader<TargetingDef>>()
            .init_asset::<TagDefinition>()
            .init_asset_loader::<RonAssetLoader<TagDefinition>>()
            .init_asset::<AttributeDefinition>()
            .init_asset_loader::<RonAssetLoader<AttributeDefinition>>()
            .init_asset::<SummonTemplateDef>()
            .init_asset_loader::<RonAssetLoader<SummonTemplateDef>>()
            .init_asset::<CampEventDef>()
            .init_asset_loader::<RonAssetLoader<CampEventDef>>()
            .init_asset::<BondDef>()
            .init_asset_loader::<RonAssetLoader<BondDef>>()
            .init_asset::<EnchantmentDef>()
            .init_asset_loader::<RonAssetLoader<EnchantmentDef>>();

        // ── 初始化 Resources ──
        app.init_resource::<ContentState>()
            .init_resource::<ContentLoadSummary>()
            .init_resource::<LoadedSpellDefs>()
            .init_resource::<LoadedCueDefs>()
            .init_resource::<LoadedEffectDefs>()
            .init_resource::<LoadedQuestDefs>()
            .init_resource::<LoadedRecipeDefs>()
            .init_resource::<LoadedShopDefs>()
            .init_resource::<LoadedTargetingDefs>()
            .init_resource::<LoadedTagDefs>()
            .init_resource::<LoadedAttributeDefs>()
            .init_resource::<LoadedSummonTemplateDefs>()
            .init_resource::<LoadedCampEventDefs>()
            .init_resource::<LoadedBondDefs>()
            .init_resource::<LoadedEnchantmentDefs>()
            .init_resource::<LoadedTerrainDefs>();

        // ── 热重载资源 ──
        app.init_resource::<ContentHotReloadState>();

        // ── 启动时加载所有配置内容 ──
        app.add_systems(Startup, load_all_content);

        // ── 地形配置加载（单独系统，避免超出 Bevy 16 参数限制）──
        app.add_systems(Startup, load_terrain_content.after(load_all_content));

        // ── 热重载系统（初始化 mtime 后启动定期扫描）──
        app.add_systems(Startup, init_hot_reload_state.after(load_all_content));
        app.add_systems(
            Update,
            hot_reload_content_system.after(init_hot_reload_state),
        );
    }
}

fn load_all_content(
    mut state: ResMut<ContentState>,
    mut summary: ResMut<ContentLoadSummary>,
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
    let config_root = std::path::Path::new("assets/config");

    // 1. 发现所有 RON 文件
    let files = discover_ron_files(config_root);
    state.discovered_files = files;

    // 2. 按桶统计
    let mut counts: HashMap<String, usize> = HashMap::new();
    for file in &state.discovered_files {
        *counts.entry(file.bucket_name.clone()).or_insert(0) += 1;
    }
    state.loaded_counts = counts;

    // 3. 同步加载已知 Definition 类型
    for file in &state.discovered_files {
        let bucket = file.bucket_name.as_str();
        match bucket {
            "spells" => load_single_def::<SpellDef>(&mut spells, file),
            "cues" => load_single_def::<CueDef>(&mut cues, file),
            "effects" => load_single_def::<EffectDef>(&mut effects, file),
            "quests" => load_single_def::<QuestDef>(&mut quests, file),
            "recipes" => load_single_def::<RecipeDef>(&mut recipes, file),
            "shops" => load_single_def::<ShopDef>(&mut shops, file),
            "targeting" => load_single_def::<TargetingDef>(&mut targeting, file),
            "tags" => load_single_def::<TagDefinition>(&mut tags, file),
            "attributes" => load_single_def::<AttributeDefinition>(&mut attributes, file),
            "summon_templates" => load_single_def::<SummonTemplateDef>(&mut summon_templates, file),
            "camp_events" => load_single_def::<CampEventDef>(&mut camp_events, file),
            "bonds" => load_single_def::<BondDef>(&mut bonds, file),
            "enchantments" => load_single_def::<EnchantmentDef>(&mut enchantments, file),
            "spell_config" => load_spell_config(&mut spell_config, file),
            other => {
                info!(target: "content", "[Content] 未知的 bucket '{}'，已跳过", other);
            }
        }
    }

    // 4. 计算加载摘要
    let total = state.discovered_files.len();
    let buckets: Vec<_> = state.loaded_counts.iter().collect();

    let total_loaded = spells.defs.len()
        + cues.defs.len()
        + effects.defs.len()
        + quests.defs.len()
        + recipes.defs.len()
        + shops.defs.len()
        + targeting.defs.len()
        + tags.defs.len()
        + attributes.defs.len()
        + summon_templates.defs.len()
        + camp_events.defs.len()
        + bonds.defs.len()
        + enchantments.defs.len();
    let total_errors = spells.errors.len()
        + cues.errors.len()
        + effects.errors.len()
        + quests.errors.len()
        + recipes.errors.len()
        + shops.errors.len()
        + targeting.errors.len()
        + tags.errors.len()
        + attributes.errors.len()
        + summon_templates.errors.len()
        + camp_events.errors.len()
        + bonds.errors.len()
        + enchantments.errors.len();

    let mut all_errors: Vec<(PathBuf, String)> = Vec::new();
    all_errors.extend(spells.errors.iter().cloned());
    all_errors.extend(cues.errors.iter().cloned());
    all_errors.extend(effects.errors.iter().cloned());
    all_errors.extend(quests.errors.iter().cloned());
    all_errors.extend(recipes.errors.iter().cloned());
    all_errors.extend(shops.errors.iter().cloned());
    all_errors.extend(targeting.errors.iter().cloned());
    all_errors.extend(tags.errors.iter().cloned());
    all_errors.extend(attributes.errors.iter().cloned());
    all_errors.extend(summon_templates.errors.iter().cloned());
    all_errors.extend(camp_events.errors.iter().cloned());
    all_errors.extend(bonds.errors.iter().cloned());
    all_errors.extend(enchantments.errors.iter().cloned());

    let bucket_stats: HashMap<String, BucketLoadStats> = state
        .loaded_counts
        .iter()
        .map(|(bucket, count)| {
            (
                bucket.clone(),
                BucketLoadStats {
                    loaded: *count,
                    errors: 0,
                },
            )
        })
        .collect();

    *summary = ContentLoadSummary {
        total_discovered: total,
        total_loaded,
        total_errors,
        bucket_stats,
        all_errors,
    };

    if total == 0 {
        info!(target: "content", "[Content] 在 assets/config/ 中未发现配置文件");
    } else {
        info!(target: "content",
            "[Content] 发现 {} 个配置文件，分布于 {} 个 bucket",
            total,
            buckets.len()
        );
        for (bucket, count) in &buckets {
            info!(target: "content", "[Content]   {}：{} 个文件", bucket, count);
        }
    }
    if total_errors > 0 {
        warn!(target: "content",
            "[Content] 加载完成: {} 个定义, {} 个错误",
            total_loaded,
            total_errors,
        );
    } else {
        info!(target: "content",
            "[Content] 加载完成: {} 个定义", total_loaded,
        );
    }
}

fn load_single_def<T: DefinitionType>(loaded: &mut LoadedDefs<T>, file: &ContentFile) {
    let path_str = file.path.display().to_string();
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("failed to read file: {}", e);
            loaded.errors.push((file.path.clone(), msg));
            warn!(target: "content", "[Content] 读取配置文件失败: {} — {}", path_str, e);
            return;
        }
    };

    let defs: Vec<T> = if T::supports_multi_def() {
        let trimmed = content.trim();
        if trimmed.starts_with('[') {
            match ron::from_str(trimmed) {
                Ok(d) => d,
                Err(e) => {
                    let msg = format!("failed to deserialize RON array: {}", e);
                    loaded.errors.push((file.path.clone(), msg));
                    warn!(target: "content", "[Content] 反序列化 RON 数组失败: {} — {}", path_str, e);
                    return;
                }
            }
        } else {
            match ron::from_str::<T>(trimmed) {
                Ok(d) => vec![d],
                Err(e) => {
                    let msg = format!("failed to deserialize RON: {}", e);
                    loaded.errors.push((file.path.clone(), msg));
                    warn!(target: "content", "[Content] 反序列化 RON 失败: {} — {}", path_str, e);
                    return;
                }
            }
        }
    } else {
        match ron::from_str(&content) {
            Ok(d) => vec![d],
            Err(e) => {
                let msg = format!("failed to deserialize RON: {}", e);
                loaded.errors.push((file.path.clone(), msg));
                warn!(target: "content", "[Content] 反序列化 RON 失败: {} — {}", path_str, e);
                return;
            }
        }
    };

    for def in &defs {
        if let Err(e) = def.validate() {
            let uid = def.def_unique_id().unwrap_or("(unknown)");
            let msg = if defs.len() > 1 {
                format!("validation failed for '{}': {}", uid, e)
            } else {
                format!("validation failed: {}", e)
            };
            loaded.errors.push((file.path.clone(), msg));
            warn!(target: "content",
                "[Content] Definition 校验失败: {} (id: {}) — {}",
                path_str, uid, e,
            );
            return;
        }
    }

    let count = defs.len();
    for def in defs {
        loaded.defs.push(def);
    }

    if count > 1 {
        info!(target: "content",
            "[Content] 从 {} 加载了 {} 个定义",
            file.path.display(),
            count,
        );
    }
}

fn load_terrain_content(mut terrains: ResMut<LoadedTerrainDefs>) {
    let terrain_dir = std::path::Path::new("assets/config/terrains");
    if !terrain_dir.exists() {
        info!(target: "content", "[Content] 地形配置目录不存在: assets/config/terrains");
        return;
    }

    let entries = match std::fs::read_dir(terrain_dir) {
        Ok(e) => e,
        Err(err) => {
            warn!(target: "content", "[Content] 读取地形配置目录失败: {}", err);
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(true, |ext| ext != "ron") {
            continue;
        }

        let path_str = path.display().to_string();
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                let msg = format!("failed to read file: {}", e);
                terrains.errors.push((path, msg));
                warn!(target: "content", "[Content] 读取地形配置文件失败: {} — {}", path_str, e);
                continue;
            }
        };

        let def: TerrainDef = match ron::from_str(&content) {
            Ok(d) => d,
            Err(e) => {
                let msg = format!("failed to deserialize RON: {}", e);
                terrains.errors.push((path, msg));
                warn!(target: "content", "[Content] 反序列化地形配置 RON 失败: {} — {}", path_str, e);
                continue;
            }
        };

        info!(target: "content",
            "[Content] 加载了地形 '{}' (id: {})",
            def.name_key,
            def.id,
        );
        terrains.defs.push(def);
    }

    info!(target: "content",
        "[Content] 地形加载完成: {} 成功, {} 失败",
        terrains.defs.len(),
        terrains.errors.len(),
    );
}

fn load_spell_config(config: &mut ResMut<SpellConfig>, file: &ContentFile) {
    let path_str = file.path.display().to_string();
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            warn!(target: "content", "[Content] 读取法术配置文件失败: {} — {}", path_str, e);
            return;
        }
    };

    let cfg: SpellConfig = match ron::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            warn!(target: "content", "[Content] 反序列化法术配置 RON 失败: {} — {}", path_str, e);
            return;
        }
    };

    **config = cfg;
    info!(target: "content",
        "[Content] 加载了法术配置 (专注基础 DC: {}, 最大专注数: {})",
        config.concentration_base_dc,
        config.max_concentration,
    );
}
