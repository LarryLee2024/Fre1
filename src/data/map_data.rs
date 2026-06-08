// 地图数据：数据驱动的地形定义 + 关卡配置
// 支持从 assets/terrains/*.ron 和 assets/maps/*.ron 外部配置文件加载

use bevy::prelude::*;
use ron::de::from_bytes;
use ron::extensions::Extensions;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{read, read_dir};

// ── 地形定义 ──

/// 地形定义（运行时）
#[derive(Clone, Debug)]
pub struct TerrainDef {
    pub id: String,
    pub name: String,
    pub move_cost: Option<u32>,
    pub defense_bonus: i32,
    pub color: (f32, f32, f32),
    pub passable: bool,
}

/// 地形定义（RON 反序列化用）
#[derive(Clone, Debug, Deserialize)]
pub struct TerrainDefRon {
    pub id: String,
    pub name: String,
    /// 移动消耗，0 表示不可通行
    pub move_cost: u32,
    pub defense_bonus: i32,
    pub color: (f32, f32, f32),
    pub passable: bool,
}

fn default_true() -> bool {
    true
}

impl From<TerrainDefRon> for TerrainDef {
    fn from(def: TerrainDefRon) -> Self {
        let move_cost = if def.passable && def.move_cost > 0 {
            Some(def.move_cost)
        } else {
            None
        };
        TerrainDef {
            id: def.id,
            name: def.name,
            move_cost,
            defense_bonus: def.defense_bonus,
            color: def.color,
            passable: def.passable,
        }
    }
}

/// 地形注册表资源
#[derive(Resource, Default)]
pub struct TerrainRegistry {
    pub terrains: HashMap<String, TerrainDef>,
}

impl TerrainRegistry {
    pub fn get(&self, id: &str) -> Option<&TerrainDef> {
        self.terrains.get(id)
    }

    /// 从 assets/terrains/ 目录加载所有 .ron 文件
    pub fn load_from_dir(dir: &str) -> Self {
        let mut registry = TerrainRegistry::default();
        let Ok(entries) = read_dir(dir) else {
            bevy::log::warn!("地形目录不存在: {}", dir);
            return registry;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "ron") {
                match read(&path) {
                    Ok(bytes) => match from_bytes::<TerrainDefRon>(&bytes) {
                        Ok(def) => {
                            let id = def.id.clone();
                            registry.terrains.insert(id.clone(), def.into());
                            bevy::log::info!("加载地形: {}", id);
                        }
                        Err(e) => {
                            bevy::log::error!("解析地形文件 {:?} 失败: {}", path, e);
                        }
                    },
                    Err(e) => {
                        bevy::log::error!("读取地形文件 {:?} 失败: {}", path, e);
                    }
                }
            }
        }
        registry
    }

    /// 兜底默认值
    pub fn register_defaults(&mut self) {
        if self.terrains.is_empty() {
            for (id, name, mc, db, color) in [
                ("plain", "草", Some(1), 0, (0.56, 0.73, 0.35)),
                ("forest", "林", Some(2), 2, (0.20, 0.50, 0.18)),
                ("mountain", "山", None, 0, (0.55, 0.50, 0.45)),
                ("water", "水", None, 0, (0.25, 0.47, 0.85)),
            ] {
                self.terrains.insert(
                    id.to_string(),
                    TerrainDef {
                        id: id.to_string(),
                        name: name.to_string(),
                        move_cost: mc,
                        defense_bonus: db,
                        color,
                        passable: mc.is_some(),
                    },
                );
            }
        }
    }
}

// ── 关卡配置 ──

/// 单位部署条目（RON 反序列化用）
#[derive(Clone, Debug, Deserialize)]
pub struct UnitDeployDef {
    pub template: String,
    pub coord: (i32, i32),
}

/// 关卡配置（RON 反序列化用）
#[derive(Clone, Debug, Deserialize)]
pub struct LevelConfigDef {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    #[serde(default = "default_tile_size")]
    pub tile_size: f32,
    /// 地形网格：每行一个字符串，每个字符映射到地形 ID
    pub terrain_grid: Vec<String>,
    /// 玩家单位部署
    #[serde(default)]
    pub player_units: Vec<UnitDeployDef>,
    /// 敌方单位部署
    #[serde(default)]
    pub enemy_units: Vec<UnitDeployDef>,
}

fn default_tile_size() -> f32 {
    64.0
}

/// 关卡配置（运行时）
#[derive(Clone, Debug)]
pub struct LevelConfig {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub tile_size: f32,
    /// (x, y) → terrain_id
    pub terrain_map: HashMap<(i32, i32), String>,
    pub player_units: Vec<UnitDeployDef>,
    pub enemy_units: Vec<UnitDeployDef>,
}

impl From<LevelConfigDef> for LevelConfig {
    fn from(def: LevelConfigDef) -> Self {
        let mut terrain_map = HashMap::new();
        for (y, row) in def.terrain_grid.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let terrain_id = match ch {
                    'P' => "plain",
                    'F' => "forest",
                    'M' => "mountain",
                    'W' => "water",
                    _ => "plain",
                };
                terrain_map.insert((x as i32, y as i32), terrain_id.to_string());
            }
        }

        LevelConfig {
            id: def.id,
            name: def.name,
            width: def.width,
            height: def.height,
            tile_size: def.tile_size,
            terrain_map,
            player_units: def.player_units,
            enemy_units: def.enemy_units,
        }
    }
}

/// 关卡注册表资源
#[derive(Resource, Default)]
pub struct LevelRegistry {
    pub levels: HashMap<String, LevelConfig>,
}

impl LevelRegistry {
    pub fn get(&self, id: &str) -> Option<&LevelConfig> {
        self.levels.get(id)
    }

    /// 获取第一个关卡（默认关卡）
    pub fn first(&self) -> Option<&LevelConfig> {
        self.levels.values().next()
    }

    /// 从 assets/maps/ 目录加载所有 .ron 文件
    pub fn load_from_dir(dir: &str) -> Self {
        let mut registry = LevelRegistry::default();
        let Ok(entries) = read_dir(dir) else {
            bevy::log::warn!("关卡目录不存在: {}", dir);
            return registry;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "ron") {
                match read(&path) {
                    Ok(bytes) => match from_bytes::<LevelConfigDef>(&bytes) {
                        Ok(def) => {
                            let id = def.id.clone();
                            registry.levels.insert(id.clone(), def.into());
                            bevy::log::info!("加载关卡: {}", id);
                        }
                        Err(e) => {
                            bevy::log::error!("解析关卡文件 {:?} 失败: {}", path, e);
                        }
                    },
                    Err(e) => {
                        bevy::log::error!("读取关卡文件 {:?} 失败: {}", path, e);
                    }
                }
            }
        }
        registry
    }
}

/// 地图数据插件
pub struct MapDataPlugin;

impl Plugin for MapDataPlugin {
    fn build(&self, app: &mut App) {
        let mut terrain_registry = TerrainRegistry::load_from_dir("assets/terrains");
        terrain_registry.register_defaults();

        let level_registry = LevelRegistry::load_from_dir("assets/maps");

        app.insert_resource(terrain_registry)
            .insert_resource(level_registry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ron_反序列化_地形定义() {
        let ron_str = r#"
            (
                id: "plain",
                name: "草",
                move_cost: 1,
                defense_bonus: 0,
                color: (0.56, 0.73, 0.35),
                passable: true,
            )
        "#;
        let def: TerrainDefRon = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.id, "plain");
        assert_eq!(def.move_cost, 1);
        assert!(def.passable);
    }

    #[test]
    fn ron_反序列化_不可通行地形() {
        let ron_str = r#"
            (
                id: "mountain",
                name: "山",
                move_cost: 0,
                defense_bonus: 0,
                color: (0.55, 0.50, 0.45),
                passable: false,
            )
        "#;
        let def: TerrainDefRon = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.move_cost, 0);
        assert!(!def.passable);
        let terrain_def: TerrainDef = def.into();
        assert_eq!(terrain_def.move_cost, None);
    }

    #[test]
    fn ron_反序列化_关卡配置() {
        let ron_str = r#"
            (
                id: "tutorial",
                name: "教学关",
                width: 5,
                height: 4,
                terrain_grid: [
                    "MMMMM",
                    "MPPPM",
                    "MPFPM",
                    "MMMMM",
                ],
                player_units: [
                    (template: "player_warrior", coord: (2, 2)),
                ],
                enemy_units: [
                    (template: "enemy_goblin", coord: (3, 2)),
                ],
            )
        "#;
        let def: LevelConfigDef = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.id, "tutorial");
        assert_eq!(def.terrain_grid.len(), 4);
        assert_eq!(def.player_units.len(), 1);
        assert_eq!(def.enemy_units.len(), 1);
    }

    #[test]
    fn level_config_def_转换为_level_config() {
        let def = LevelConfigDef {
            id: "test".into(),
            name: "测试".into(),
            width: 3,
            height: 2,
            tile_size: 64.0,
            terrain_grid: vec!["PPF".into(), "PMM".into()],
            player_units: vec![],
            enemy_units: vec![],
        };
        let config: LevelConfig = def.into();
        assert_eq!(config.terrain_map.get(&(0, 0)), Some(&"plain".to_string()));
        assert_eq!(config.terrain_map.get(&(2, 0)), Some(&"forest".to_string()));
        assert_eq!(
            config.terrain_map.get(&(2, 1)),
            Some(&"mountain".to_string())
        );
    }

    #[test]
    fn terrain_registry_兜底默认值() {
        let mut reg = TerrainRegistry::default();
        reg.register_defaults();
        assert!(reg.get("plain").is_some());
        assert!(reg.get("forest").is_some());
        assert!(reg.get("mountain").is_some());
        assert!(reg.get("water").is_some());
        assert_eq!(reg.get("plain").unwrap().move_cost, Some(1));
        assert_eq!(reg.get("forest").unwrap().defense_bonus, 2);
    }
}
