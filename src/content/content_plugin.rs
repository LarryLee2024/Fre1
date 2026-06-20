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
use crate::core::capabilities::ability::foundation::AbilityDef;
use crate::core::capabilities::attribute::foundation::AttributeDefinition;
use crate::core::capabilities::cue::foundation::CueDef;
use crate::core::capabilities::effect::foundation::EffectDef;
use crate::core::capabilities::rule::foundation::RuleDef;
use crate::core::capabilities::tag::foundation::TagDefinition;
use crate::core::capabilities::targeting::foundation::TargetingDef;
use crate::core::domains::camp_rest::CampEventDef;
use crate::core::domains::crafting::EnchantmentDef;
use crate::core::domains::crafting::RecipeDef;
use crate::core::domains::economy::ShopDef;
use crate::core::domains::party::BondDef;
use crate::core::domains::progression::LevelProgressionTable;
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

/// 已加载的法术定义集合 Resource。
#[derive(Resource, Debug, Default)]
pub struct LoadedSpellDefs {
    /// 成功加载并校验通过的法术定义。
    pub defs: Vec<SpellDef>,
    /// 加载或校验失败的文件（路径 + 错误信息）。
    pub errors: Vec<(PathBuf, String)>,
}

/// 已加载的 Cue 信号定义集合 Resource。
#[derive(Resource, Debug, Default)]
pub struct LoadedCueDefs {
    /// 成功加载并校验通过的 Cue 定义。
    pub defs: Vec<CueDef>,
    /// 加载或校验失败的文件（路径 + 错误信息）。
    pub errors: Vec<(PathBuf, String)>,
}

/// 已加载的效果定义集合 Resource。
#[derive(Resource, Debug, Default)]
pub struct LoadedEffectDefs {
    /// 成功加载并校验通过的效果定义。
    pub defs: Vec<EffectDef>,
    /// 加载或校验失败的文件（路径 + 错误信息）。
    pub errors: Vec<(PathBuf, String)>,
}

/// 已加载的技能定义集合 Resource。
#[derive(Resource, Debug, Default)]
pub struct LoadedAbilityDefs {
    /// 成功加载并校验通过的技能定义。
    pub defs: Vec<AbilityDef>,
    /// 加载或校验失败的文件（路径 + 错误信息）。
    pub errors: Vec<(PathBuf, String)>,
}

/// 已加载的规则定义集合 Resource。
#[derive(Resource, Debug, Default)]
pub struct LoadedRuleDefs {
    /// 成功加载并校验通过的规则定义。
    pub defs: Vec<RuleDef>,
    /// 加载或校验失败的文件（路径 + 错误信息）。
    pub errors: Vec<(PathBuf, String)>,
}

/// 已加载的任务定义集合 Resource。
#[derive(Resource, Debug, Default)]
pub struct LoadedQuestDefs {
    /// 成功加载并校验通过的任务定义。
    pub defs: Vec<QuestDef>,
    /// 加载或校验失败的文件（路径 + 错误信息）。
    pub errors: Vec<(PathBuf, String)>,
}

/// 已加载的配方定义集合 Resource。
#[derive(Resource, Debug, Default)]
pub struct LoadedRecipeDefs {
    /// 成功加载并校验通过的配方定义。
    pub defs: Vec<RecipeDef>,
    /// 加载或校验失败的文件（路径 + 错误信息）。
    pub errors: Vec<(PathBuf, String)>,
}

/// 已加载的商店定义集合 Resource。
#[derive(Resource, Debug, Default)]
pub struct LoadedShopDefs {
    /// 成功加载并校验通过的商店定义。
    pub defs: Vec<ShopDef>,
    /// 加载或校验失败的文件（路径 + 错误信息）。
    pub errors: Vec<(PathBuf, String)>,
}

/// 已加载的目标选择定义集合 Resource。
#[derive(Resource, Debug, Default)]
pub struct LoadedTargetingDefs {
    pub defs: Vec<TargetingDef>,
    pub errors: Vec<(PathBuf, String)>,
}

/// 已加载的标签定义集合 Resource。
#[derive(Resource, Debug, Default)]
pub struct LoadedTagDefs {
    pub defs: Vec<TagDefinition>,
    pub errors: Vec<(PathBuf, String)>,
}

/// 已加载的属性定义集合 Resource。
#[derive(Resource, Debug, Default)]
pub struct LoadedAttributeDefs {
    pub defs: Vec<AttributeDefinition>,
    pub errors: Vec<(PathBuf, String)>,
}

/// 已加载的召唤模板定义集合 Resource。
#[derive(Resource, Debug, Default)]
pub struct LoadedSummonTemplateDefs {
    pub defs: Vec<SummonTemplateDef>,
    pub errors: Vec<(PathBuf, String)>,
}

/// 已加载的营地事件定义集合 Resource。
#[derive(Resource, Debug, Default)]
pub struct LoadedCampEventDefs {
    pub defs: Vec<CampEventDef>,
    pub errors: Vec<(PathBuf, String)>,
}

/// 已加载的羁绊定义集合 Resource。
#[derive(Resource, Debug, Default)]
pub struct LoadedBondDefs {
    pub defs: Vec<BondDef>,
    pub errors: Vec<(PathBuf, String)>,
}

/// 已加载的附魔定义集合 Resource。
#[derive(Resource, Debug, Default)]
pub struct LoadedEnchantmentDefs {
    pub defs: Vec<EnchantmentDef>,
    pub errors: Vec<(PathBuf, String)>,
}

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
            .init_asset_loader::<RonAssetLoader<EnchantmentDef>>()
            .init_asset::<AbilityDef>()
            .init_asset_loader::<RonAssetLoader<AbilityDef>>();

        // ── 初始化 Resources ──
        app.init_resource::<ContentState>()
            .init_resource::<ContentLoadSummary>()
            .init_resource::<LoadedSpellDefs>()
            .init_resource::<LoadedCueDefs>()
            .init_resource::<LoadedEffectDefs>()
            .init_resource::<LoadedAbilityDefs>()
            .init_resource::<LoadedQuestDefs>()
            .init_resource::<LoadedRecipeDefs>()
            .init_resource::<LoadedShopDefs>()
            .init_resource::<LoadedTargetingDefs>()
            .init_resource::<LoadedTagDefs>()
            .init_resource::<LoadedAttributeDefs>()
            .init_resource::<LoadedSummonTemplateDefs>()
            .init_resource::<LoadedCampEventDefs>()
            .init_resource::<LoadedBondDefs>()
            .init_resource::<LoadedEnchantmentDefs>();

        // ── 热重载资源 ──
        app.init_resource::<ContentHotReloadState>();

        // ── 启动时加载所有配置内容 ──
        app.add_systems(Startup, load_all_content);

        // ── 热重载系统（初始化 mtime 后启动定期扫描）──
        app.add_systems(Startup, init_hot_reload_state.after(load_all_content));
        app.add_systems(
            Update,
            hot_reload_content_system.after(init_hot_reload_state),
        );
    }
}

/// 启动时加载所有配置内容的系统。
///
/// 扫描 assets/config/ 目录，发现所有 .ron 文件，
/// 按桶分类并记录到 ContentState。
/// 对已知 Definition 类型执行同步加载、反序列化和校验。
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
        match file.bucket_name.as_str() {
            "spells" => load_spell_def(&mut spells, file),
            "cues" => load_cue_def(&mut cues, file),
            "effects" => load_effect_def(&mut effects, file),
            "quests" => load_quest_def(&mut quests, file),
            "recipes" => load_recipe_def(&mut recipes, file),
            "shops" => load_shop_def(&mut shops, file),
            "targeting" => load_targeting_def(&mut targeting, file),
            "tags" => load_tag_def(&mut tags, file),
            "attributes" => load_attribute_def(&mut attributes, file),
            "summon_templates" => load_summon_template_def(&mut summon_templates, file),
            "camp_events" => load_camp_event_def(&mut camp_events, file),
            "bonds" => load_bond_def(&mut bonds, file),
            "enchantments" => load_enchantment_def(&mut enchantments, file),
            "spell_config" => load_spell_config(&mut spell_config, file),
            other => {
                info!("[Content] Unknown bucket '{}', skipping", other);
            }
        }
    }

    // 4. 日志输出
    let total = state.discovered_files.len();
    let buckets: Vec<_> = state.loaded_counts.iter().collect();

    // 5. 计算加载摘要
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

    let mut bucket_stats: HashMap<String, BucketLoadStats> = HashMap::new();
    for (bucket, count) in &buckets {
        bucket_stats.insert(
            bucket.to_string(),
            BucketLoadStats {
                loaded: **count,
                errors: 0,
            },
        );
    }

    *summary = ContentLoadSummary {
        total_discovered: total,
        total_loaded,
        total_errors,
        bucket_stats,
        all_errors,
    };

    if total == 0 {
        info!("[Content] No config files found in assets/config/");
    } else {
        info!(
            "[Content] Discovered {} config file(s) across {} bucket(s)",
            total,
            buckets.len()
        );
        for (bucket, count) in &buckets {
            info!("[Content]   {}: {} file(s)", bucket, count);
        }
    }
    info!(
        "[Content] Loaded {} spell(s), {} cue(s), {} effect(s), {} quest(s), {} recipe(s), {} shop(s), {} targeting(s), {} tag(s), {} attribute(s), {} summon(s), {} camp_event(s), {} bond(s), {} enchantment(s), {} error(s)",
        spells.defs.len(),
        cues.defs.len(),
        effects.defs.len(),
        quests.defs.len(),
        recipes.defs.len(),
        shops.defs.len(),
        targeting.defs.len(),
        tags.defs.len(),
        attributes.defs.len(),
        summon_templates.defs.len(),
        camp_events.defs.len(),
        bonds.defs.len(),
        enchantments.defs.len(),
        spells.errors.len()
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
            + enchantments.errors.len(),
    );
}

/// 从 RON 文件同步加载一个 SpellDef。
fn load_spell_def(spells: &mut ResMut<LoadedSpellDefs>, file: &ContentFile) {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("failed to read file: {}", e);
            spells.errors.push((file.path.clone(), msg));
            return;
        }
    };

    let def: SpellDef = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            let msg = format!("failed to deserialize RON: {}", e);
            spells.errors.push((file.path.clone(), msg));
            return;
        }
    };

    // 校验（通过 DefinitionType trait）
    if let Err(e) = def.validate() {
        let msg = format!("validation failed: {}", e);
        spells.errors.push((file.path.clone(), msg));
        return;
    }

    info!("[Content] Loaded spell '{}' (id: {})", def.name_key, def.id);
    spells.defs.push(def);
}

/// 从 RON 文件同步加载一个 CueDef。
fn load_cue_def(cues: &mut ResMut<LoadedCueDefs>, file: &ContentFile) {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("failed to read file: {}", e);
            cues.errors.push((file.path.clone(), msg));
            return;
        }
    };

    let def: CueDef = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            let msg = format!("failed to deserialize RON: {}", e);
            cues.errors.push((file.path.clone(), msg));
            return;
        }
    };

    // 校验（通过 DefinitionType trait）
    if let Err(e) = def.validate() {
        let msg = format!("validation failed: {}", e);
        cues.errors.push((file.path.clone(), msg));
        return;
    }

    info!("[Content] Loaded cue '{}'", def.id);
    cues.defs.push(def);
}

/// 从 RON 文件同步加载一个 EffectDef。
fn load_effect_def(effects: &mut ResMut<LoadedEffectDefs>, file: &ContentFile) {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("failed to read file: {}", e);
            effects.errors.push((file.path.clone(), msg));
            return;
        }
    };

    let def: EffectDef = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            let msg = format!("failed to deserialize RON: {}", e);
            effects.errors.push((file.path.clone(), msg));
            return;
        }
    };

    if let Err(e) = def.validate() {
        let msg = format!("validation failed: {}", e);
        effects.errors.push((file.path.clone(), msg));
        return;
    }

    info!(
        "[Content] Loaded effect '{}' (id: {})",
        def.name_key, def.id
    );
    effects.defs.push(def);
}

/// 从 RON 文件同步加载一个 AbilityDef。
fn load_ability_def(abilities: &mut ResMut<LoadedAbilityDefs>, file: &ContentFile) {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("failed to read file: {}", e);
            abilities.errors.push((file.path.clone(), msg));
            return;
        }
    };

    let def: AbilityDef = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            let msg = format!("failed to deserialize RON: {}", e);
            abilities.errors.push((file.path.clone(), msg));
            return;
        }
    };

    if let Err(e) = def.validate() {
        let msg = format!("validation failed: {}", e);
        abilities.errors.push((file.path.clone(), msg));
        return;
    }

    info!(
        "[Content] Loaded ability '{}' (id: {})",
        def.name_key, def.id
    );
    abilities.defs.push(def);
}

/// 从 RON 文件同步加载一个 RuleDef。
fn load_rule_def(rules: &mut ResMut<LoadedRuleDefs>, file: &ContentFile) {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("failed to read file: {}", e);
            rules.errors.push((file.path.clone(), msg));
            return;
        }
    };

    let def: RuleDef = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            let msg = format!("failed to deserialize RON: {}", e);
            rules.errors.push((file.path.clone(), msg));
            return;
        }
    };

    if let Err(e) = def.validate() {
        let msg = format!("validation failed: {}", e);
        rules.errors.push((file.path.clone(), msg));
        return;
    }

    info!("[Content] Loaded rule '{}' (id: {})", def.name_key, def.id);
    rules.defs.push(def);
}

/// 从 RON 文件同步加载一个 QuestDef。
fn load_quest_def(quests: &mut ResMut<LoadedQuestDefs>, file: &ContentFile) {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("failed to read file: {}", e);
            quests.errors.push((file.path.clone(), msg));
            return;
        }
    };

    let def: QuestDef = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            let msg = format!("failed to deserialize RON: {}", e);
            quests.errors.push((file.path.clone(), msg));
            return;
        }
    };

    if let Err(e) = def.validate() {
        let msg = format!("validation failed: {}", e);
        quests.errors.push((file.path.clone(), msg));
        return;
    }

    info!("[Content] Loaded quest '{}' (id: {})", def.name_key, def.id);
    quests.defs.push(def);
}

/// 从 RON 文件同步加载一个 RecipeDef。
fn load_recipe_def(recipes: &mut ResMut<LoadedRecipeDefs>, file: &ContentFile) {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("failed to read file: {}", e);
            recipes.errors.push((file.path.clone(), msg));
            return;
        }
    };

    let def: RecipeDef = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            let msg = format!("failed to deserialize RON: {}", e);
            recipes.errors.push((file.path.clone(), msg));
            return;
        }
    };

    if let Err(e) = def.validate() {
        let msg = format!("validation failed: {}", e);
        recipes.errors.push((file.path.clone(), msg));
        return;
    }

    info!(
        "[Content] Loaded recipe '{}' (id: {})",
        def.name_key, def.id
    );
    recipes.defs.push(def);
}

/// 从 RON 文件同步加载一个 ShopDef。
fn load_shop_def(shops: &mut ResMut<LoadedShopDefs>, file: &ContentFile) {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("failed to read file: {}", e);
            shops.errors.push((file.path.clone(), msg));
            return;
        }
    };

    let def: ShopDef = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            let msg = format!("failed to deserialize RON: {}", e);
            shops.errors.push((file.path.clone(), msg));
            return;
        }
    };

    if let Err(e) = def.validate() {
        let msg = format!("validation failed: {}", e);
        shops.errors.push((file.path.clone(), msg));
        return;
    }

    info!("[Content] Loaded shop '{}' (id: {})", def.name_key, def.id);
    shops.defs.push(def);
}

/// 从 RON 文件同步加载一个 TargetingDef。
fn load_targeting_def(targeting: &mut ResMut<LoadedTargetingDefs>, file: &ContentFile) {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("failed to read file: {}", e);
            targeting.errors.push((file.path.clone(), msg));
            return;
        }
    };

    let def: TargetingDef = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            let msg = format!("failed to deserialize RON: {}", e);
            targeting.errors.push((file.path.clone(), msg));
            return;
        }
    };

    if let Err(e) = def.validate() {
        let msg = format!("validation failed: {}", e);
        targeting.errors.push((file.path.clone(), msg));
        return;
    }

    info!(
        "[Content] Loaded targeting def (type: {}, shape: {})",
        def.target_type.name(),
        def.shape.name()
    );
    targeting.defs.push(def);
}

/// 从 RON 文件同步加载标签定义（支持单条和数组格式）。
fn load_tag_def(tags: &mut ResMut<LoadedTagDefs>, file: &ContentFile) {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("failed to read file: {}", e);
            tags.errors.push((file.path.clone(), msg));
            return;
        }
    };

    let trimmed = content.trim();
    let defs: Vec<TagDefinition> = if trimmed.starts_with('[') {
        // 数组格式: [item1, item2, ...]
        match ron::from_str(trimmed) {
            Ok(d) => d,
            Err(e) => {
                let msg = format!("failed to deserialize RON array: {}", e);
                tags.errors.push((file.path.clone(), msg));
                return;
            }
        }
    } else {
        // 单条格式: (id: ..., path: ..., ...)
        match ron::from_str::<TagDefinition>(trimmed) {
            Ok(d) => vec![d],
            Err(e) => {
                let msg = format!("failed to deserialize RON: {}", e);
                tags.errors.push((file.path.clone(), msg));
                return;
            }
        }
    };

    for def in &defs {
        if let Err(e) = def.validate() {
            let msg = format!("validation failed for '{}': {}", def.id.as_str(), e);
            tags.errors.push((file.path.clone(), msg));
            return;
        }
    }

    let count = defs.len();
    for def in defs {
        info!(
            "[Content] Loaded tag '{}' (path: {})",
            def.id.as_str(),
            def.path
        );
        tags.defs.push(def);
    }

    if count > 1 {
        info!(
            "[Content] Loaded {} tags from {}",
            count,
            file.path.display()
        );
    }
}

/// 从 RON 文件同步加载 AttributeDefinition（支持单条和数组格式）。
fn load_attribute_def(attributes: &mut ResMut<LoadedAttributeDefs>, file: &ContentFile) {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("failed to read file: {e}");
            attributes.errors.push((file.path.clone(), msg));
            return;
        }
    };

    let trimmed = content.trim();
    let defs: Vec<AttributeDefinition> = if trimmed.starts_with('[') {
        // 数组格式: [item1, item2, ...]
        match ron::from_str(trimmed) {
            Ok(d) => d,
            Err(e) => {
                let msg = format!("failed to deserialize RON array: {e}");
                attributes.errors.push((file.path.clone(), msg));
                return;
            }
        }
    } else {
        // 单条格式: (id: "attr:...", category: ..., ...)
        match ron::from_str::<AttributeDefinition>(trimmed) {
            Ok(d) => vec![d],
            Err(e) => {
                let msg = format!("failed to deserialize RON: {e}");
                attributes.errors.push((file.path.clone(), msg));
                return;
            }
        }
    };

    for def in &defs {
        if let Err(e) = def.validate() {
            let msg = format!("validation failed for '{}': {e}", def.id.as_str());
            attributes.errors.push((file.path.clone(), msg));
            return;
        }
    }

    let count = defs.len();
    for def in defs {
        info!(
            "[Content] Loaded attribute '{}' (category: {:?})",
            def.id.as_str(),
            def.category
        );
        attributes.defs.push(def);
    }

    if count > 1 {
        info!(
            "[Content] Loaded {count} attributes from {}",
            file.path.display()
        );
    }
}

/// 从 RON 文件同步加载一个 SummonTemplateDef。
fn load_summon_template_def(templates: &mut ResMut<LoadedSummonTemplateDefs>, file: &ContentFile) {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("failed to read file: {}", e);
            templates.errors.push((file.path.clone(), msg));
            return;
        }
    };

    let def: SummonTemplateDef = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            let msg = format!("failed to deserialize RON: {}", e);
            templates.errors.push((file.path.clone(), msg));
            return;
        }
    };

    if let Err(e) = def.validate() {
        let msg = format!("validation failed: {}", e);
        templates.errors.push((file.path.clone(), msg));
        return;
    }

    info!(
        "[Content] Loaded summon template '{}' (id: {})",
        def.name_key, def.id
    );
    templates.defs.push(def);
}

/// 从 RON 文件同步加载一个 CampEventDef。
fn load_camp_event_def(events: &mut ResMut<LoadedCampEventDefs>, file: &ContentFile) {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("failed to read file: {}", e);
            events.errors.push((file.path.clone(), msg));
            return;
        }
    };

    let def: CampEventDef = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            let msg = format!("failed to deserialize RON: {}", e);
            events.errors.push((file.path.clone(), msg));
            return;
        }
    };

    if let Err(e) = def.validate() {
        let msg = format!("validation failed: {}", e);
        events.errors.push((file.path.clone(), msg));
        return;
    }

    info!(
        "[Content] Loaded camp event '{}' (id: {})",
        def.title_key, def.id
    );
    events.defs.push(def);
}

/// 从 RON 文件同步加载一个 BondDef。
fn load_bond_def(bonds: &mut ResMut<LoadedBondDefs>, file: &ContentFile) {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("failed to read file: {}", e);
            bonds.errors.push((file.path.clone(), msg));
            return;
        }
    };

    let def: BondDef = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            let msg = format!("failed to deserialize RON: {}", e);
            bonds.errors.push((file.path.clone(), msg));
            return;
        }
    };

    if let Err(e) = def.validate() {
        let msg = format!("validation failed: {}", e);
        bonds.errors.push((file.path.clone(), msg));
        return;
    }

    info!("[Content] Loaded bond '{}' (id: {})", def.name_key, def.id);
    bonds.defs.push(def);
}

/// 从 RON 文件同步加载一个 EnchantmentDef。
fn load_enchantment_def(enchantments: &mut ResMut<LoadedEnchantmentDefs>, file: &ContentFile) {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("failed to read file: {}", e);
            enchantments.errors.push((file.path.clone(), msg));
            return;
        }
    };

    let def: EnchantmentDef = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            let msg = format!("failed to deserialize RON: {}", e);
            enchantments.errors.push((file.path.clone(), msg));
            return;
        }
    };

    if let Err(e) = def.validate() {
        let msg = format!("validation failed: {}", e);
        enchantments.errors.push((file.path.clone(), msg));
        return;
    }

    info!(
        "[Content] Loaded enchantment '{}' (id: {})",
        def.name_key, def.id
    );
    enchantments.defs.push(def);
}

/// 从 RON 文件同步加载 LevelProgressionTable。
fn load_progression_balance(balance: &mut ResMut<LevelProgressionTable>, file: &ContentFile) {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            warn!("[Content] Failed to read progression balance file: {}", e);
            return;
        }
    };

    let table: LevelProgressionTable = match ron::from_str(&content) {
        Ok(t) => t,
        Err(e) => {
            warn!(
                "[Content] Failed to deserialize progression balance RON: {}",
                e
            );
            return;
        }
    };

    **balance = table;
    info!(
        "[Content] Loaded progression balance table (max_level: {})",
        balance.max_level
    );
}

/// 从 RON 文件同步加载 SpellConfig。
fn load_spell_config(config: &mut ResMut<SpellConfig>, file: &ContentFile) {
    let content = match std::fs::read_to_string(&file.path) {
        Ok(c) => c,
        Err(e) => {
            warn!("[Content] Failed to read spell config file: {}", e);
            return;
        }
    };

    let cfg: SpellConfig = match ron::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            warn!("[Content] Failed to deserialize spell config RON: {}", e);
            return;
        }
    };

    **config = cfg;
    info!(
        "[Content] Loaded spell config (concentration_base_dc: {}, max_concentration: {})",
        config.concentration_base_dc, config.max_concentration
    );
}
