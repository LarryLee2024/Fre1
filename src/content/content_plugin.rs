//! ContentPlugin — 内容桥接层
//!
//! 从 assets/config/ 加载配置 → 校验 → 注册到 Registry。
//! 详见 ADR-047 和 `docs/01-architecture/README.md` §3.5

use bevy::asset::AssetApp;
use bevy::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

use super::loading::{ContentFile, DefinitionType, RonAssetLoader, discover_ron_files};
use crate::core::domains::spell::SpellDef;

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

/// 已加载的法术定义集合 Resource。
#[derive(Resource, Debug, Default)]
pub struct LoadedSpellDefs {
    /// 成功加载并校验通过的法术定义。
    pub defs: Vec<SpellDef>,
    /// 加载或校验失败的文件（路径 + 错误信息）。
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
            .init_asset_loader::<RonAssetLoader<SpellDef>>();

        // ── 初始化 Resources ──
        app.init_resource::<ContentState>()
            .init_resource::<LoadedSpellDefs>();

        // ── 启动时加载所有配置内容 ──
        app.add_systems(Startup, load_all_content);
    }
}

/// 启动时加载所有配置内容的系统。
///
/// 扫描 assets/config/ 目录，发现所有 .ron 文件，
/// 按桶分类并记录到 ContentState。
/// 对已知 Definition 类型执行同步加载、反序列化和校验。
fn load_all_content(mut state: ResMut<ContentState>, mut spells: ResMut<LoadedSpellDefs>) {
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
            other => {
                info!("[Content] Unknown bucket '{}', skipping", other);
            }
        }
    }

    // 4. 日志输出
    let total = state.discovered_files.len();
    let buckets: Vec<_> = state.loaded_counts.iter().collect();

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
        "[Content] Loaded {} spell def(s), {} error(s)",
        spells.defs.len(),
        spells.errors.len()
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
