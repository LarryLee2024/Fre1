// 地图数据：数据驱动的地形定义 + 关卡配置
// 支持从 assets/terrains/*.ron 和 assets/maps/*.ron 外部配置文件加载

use crate::core::registry_loader::RegistryLoader;
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

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
    /// 地形在关卡网格中的字符代码（如 'P' → "plain"）
    pub char_code: Option<char>,
}

/// 地形定义（RON 反序列化用）
#[derive(Clone, Debug, Deserialize)]
pub struct TerrainDefRon {
    #[serde(default)]
    pub version: u32,
    pub id: String,
    pub name: String,
    /// 移动消耗，0 表示不可通行
    pub move_cost: u32,
    pub defense_bonus: i32,
    pub color: (f32, f32, f32),
    pub passable: bool,
    /// 地形在关卡网格中的字符代码
    #[serde(default)]
    pub char_code: Option<char>,
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
            char_code: def.char_code,
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

    /// 从已注册地形构建字符→地形ID映射
    pub fn char_map(&self) -> HashMap<char, String> {
        self.terrains
            .values()
            .filter_map(|def| def.char_code.map(|ch| (ch, def.id.clone())))
            .collect()
    }

    /// 兜底默认值
    pub fn register_defaults(&mut self) {
        if self.terrains.is_empty() {
            for (id, name, mc, db, color, ch) in [
                ("plain", "草", Some(1), 0, (0.56, 0.73, 0.35), Some('P')),
                ("forest", "林", Some(2), 2, (0.20, 0.50, 0.18), Some('F')),
                ("mountain", "山", None, 0, (0.55, 0.50, 0.45), Some('M')),
                ("water", "水", None, 0, (0.25, 0.47, 0.85), Some('W')),
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
                        char_code: ch,
                    },
                );
            }
        }
    }
}

impl RegistryLoader for TerrainRegistry {
    type Item = TerrainDefRon;

    fn register_item(&mut self, item: TerrainDefRon) {
        let id = item.id.clone();
        self.terrains.insert(id.clone(), item.into());
        bevy::log::info!(target: "map", id = %id, "地形已加载");
    }

    fn register_defaults(&mut self) {
        TerrainRegistry::register_defaults(self);
    }

    fn is_empty(&self) -> bool {
        self.terrains.is_empty()
    }

    fn registry_name() -> &'static str {
        "地形"
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
    #[serde(default)]
    pub version: u32,
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    #[serde(default = "default_tile_size")]
    pub tile_size: f32,
    /// 地形网格：每行一个字符串，每个字符映射到地形 ID
    pub terrain_grid: Vec<String>,
    /// 自定义字符→地形ID映射（可选，覆盖 TerrainRegistry 的 char_code）
    #[serde(default)]
    pub char_map: HashMap<char, String>,
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

impl LevelConfig {
    /// 从 LevelConfigDef 构建 LevelConfig，使用 TerrainRegistry 的 char_code 作为默认映射
    pub fn from_def(def: LevelConfigDef, terrain_registry: &TerrainRegistry) -> Self {
        // 从 TerrainRegistry 构建 char→terrain_id 映射（数据驱动）
        let mut char_map = terrain_registry.char_map();
        // 自定义 char_map 覆盖默认值
        for (ch, terrain_id) in def.char_map {
            char_map.insert(ch, terrain_id);
        }

        let mut terrain_map = HashMap::new();
        for (y, row) in def.terrain_grid.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let terrain_id = char_map
                    .get(&ch)
                    .cloned()
                    .unwrap_or_else(|| "plain".to_string());
                terrain_map.insert((x as i32, y as i32), terrain_id);
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

    /// 从目录加载关卡配置，使用 TerrainRegistry 的 char_code 作为默认字符映射
    pub fn load_from_dir_with_terrain(dir: &str, terrain_registry: &TerrainRegistry) -> Self {
        let mut registry = Self::default();
        let Ok(entries) = std::fs::read_dir(dir) else {
            bevy::log::warn!(target: "map", path = %dir, "关卡目录不存在");
            return registry;
        };
        let mut loaded = false;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "ron") {
                match std::fs::read(&path) {
                    Ok(bytes) => match ron::de::from_bytes::<LevelConfigDef>(&bytes) {
                        Ok(item) => {
                            let id = item.id.clone();
                            let config = LevelConfig::from_def(item, terrain_registry);
                            registry.levels.insert(id.clone(), config);
                            bevy::log::info!(target: "map", id = %id, "关卡已加载");
                            loaded = true;
                        }
                        Err(e) => bevy::log::error!(
                            target: "map",
                            path = %path.display(),
                            error = %e,
                            "解析关卡配置失败"
                        ),
                    },
                    Err(e) => bevy::log::error!(
                        target: "map",
                        path = %path.display(),
                        error = %e,
                        "读取关卡配置失败"
                    ),
                }
            }
        }
        if !loaded {
            bevy::log::warn!(target: "map", "关卡目录为空");
        }
        registry
    }
}

impl RegistryLoader for LevelRegistry {
    type Item = LevelConfigDef;

    fn register_item(&mut self, item: LevelConfigDef) {
        // 注意：此方法不使用 TerrainRegistry 的 char_code 映射
        // 生产环境应使用 load_from_dir_with_terrain
        let id = item.id.clone();
        let config = LevelConfig::from_def(item, &TerrainRegistry::default());
        self.levels.insert(id.clone(), config);
        bevy::log::info!(target: "map", id = %id, "关卡已加载（无 TerrainRegistry）");
    }

    /// 关卡没有硬编码兜底，空注册表即为空
    fn register_defaults(&mut self) {
        // 关卡没有默认数据
    }

    fn is_empty(&self) -> bool {
        self.levels.is_empty()
    }

    fn registry_name() -> &'static str {
        "关卡"
    }
}

/// 地图数据插件
pub struct MapDataPlugin;

impl Plugin for MapDataPlugin {
    fn build(&self, app: &mut App) {
        let mut terrain_registry = TerrainRegistry::load_from_dir("assets/terrains");
        terrain_registry.register_defaults();

        let level_registry = LevelRegistry::load_from_dir_with_terrain("assets/maps", &terrain_registry);

        app.insert_resource(terrain_registry)
            .insert_resource(level_registry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ron::de::from_bytes;

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
        let terrain_reg = {
            let mut reg = TerrainRegistry::default();
            reg.register_defaults();
            reg
        };
        let def = LevelConfigDef {
            version: 0,
            id: "test".into(),
            name: "测试".into(),
            width: 3,
            height: 2,
            tile_size: 64.0,
            terrain_grid: vec!["PPF".into(), "PMM".into()],
            char_map: HashMap::new(),
            player_units: vec![],
            enemy_units: vec![],
        };
        let config = LevelConfig::from_def(def, &terrain_reg);
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

    #[test]
    fn level_registry_查询关卡() {
        let mut registry = LevelRegistry::default();
        registry.levels.insert(
            "test".into(),
            LevelConfig {
                id: "test".into(),
                name: "测试".into(),
                width: 3,
                height: 3,
                tile_size: 64.0,
                terrain_map: HashMap::new(),
                player_units: vec![],
                enemy_units: vec![],
            },
        );
        assert!(registry.get("test").is_some());
    }

    #[test]
    fn level_registry_查询未注册返回none() {
        let registry = LevelRegistry::default();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn level_registry_first() {
        let mut registry = LevelRegistry::default();
        registry.levels.insert(
            "a".into(),
            LevelConfig {
                id: "a".into(),
                name: "A".into(),
                width: 1,
                height: 1,
                tile_size: 64.0,
                terrain_map: HashMap::new(),
                player_units: vec![],
                enemy_units: vec![],
            },
        );
        registry.levels.insert(
            "b".into(),
            LevelConfig {
                id: "b".into(),
                name: "B".into(),
                width: 1,
                height: 1,
                tile_size: 64.0,
                terrain_map: HashMap::new(),
                player_units: vec![],
                enemy_units: vec![],
            },
        );
        assert!(registry.first().is_some());
    }

    #[test]
    fn level_registry_first_空返回none() {
        let registry = LevelRegistry::default();
        assert!(registry.first().is_none());
    }

    #[test]
    fn terrain_def_ron_可通行地形move_cost() {
        let def = TerrainDefRon {
            version: 0,
            id: "forest".into(),
            name: "林".into(),
            move_cost: 2,
            defense_bonus: 2,
            color: (0.2, 0.5, 0.2),
            passable: true,
            char_code: Some('F'),
        };
        let terrain: TerrainDef = def.into();
        assert_eq!(terrain.move_cost, Some(2));
        assert!(terrain.passable);
    }

    #[test]
    fn level_config_地形网格解析() {
        let terrain_reg = {
            let mut reg = TerrainRegistry::default();
            reg.register_defaults();
            reg
        };
        let def = LevelConfigDef {
            version: 0,
            id: "test".into(),
            name: "测试".into(),
            width: 2,
            height: 2,
            tile_size: 64.0,
            terrain_grid: vec!["PF".into(), "MW".into()],
            char_map: HashMap::new(),
            player_units: vec![],
            enemy_units: vec![],
        };
        let config = LevelConfig::from_def(def, &terrain_reg);
        assert_eq!(config.terrain_map.get(&(0, 0)), Some(&"plain".to_string()));
        assert_eq!(config.terrain_map.get(&(1, 0)), Some(&"forest".to_string()));
        assert_eq!(
            config.terrain_map.get(&(0, 1)),
            Some(&"mountain".to_string())
        );
        assert_eq!(config.terrain_map.get(&(1, 1)), Some(&"water".to_string()));
    }

    #[test]
    fn level_config_自定义char_map覆盖默认() {
        let terrain_reg = {
            let mut reg = TerrainRegistry::default();
            reg.register_defaults();
            reg
        };
        let mut custom_map = HashMap::new();
        custom_map.insert('D', "desert".to_string());
        let def = LevelConfigDef {
            version: 0,
            id: "test".into(),
            name: "测试".into(),
            width: 2,
            height: 1,
            tile_size: 64.0,
            terrain_grid: vec!["PD".into()],
            char_map: custom_map,
            player_units: vec![],
            enemy_units: vec![],
        };
        let config = LevelConfig::from_def(def, &terrain_reg);
        assert_eq!(config.terrain_map.get(&(0, 0)), Some(&"plain".to_string()));
        assert_eq!(config.terrain_map.get(&(1, 0)), Some(&"desert".to_string()));
    }

    #[test]
    fn terrain_registry_char_map() {
        let mut reg = TerrainRegistry::default();
        reg.register_defaults();
        let map = reg.char_map();
        assert_eq!(map.get(&'P'), Some(&"plain".to_string()));
        assert_eq!(map.get(&'F'), Some(&"forest".to_string()));
        assert_eq!(map.get(&'M'), Some(&"mountain".to_string()));
        assert_eq!(map.get(&'W'), Some(&"water".to_string()));
    }
}
