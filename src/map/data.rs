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

        let level_registry =
            LevelRegistry::load_from_dir_with_terrain("assets/maps", &terrain_registry);

        app.insert_resource(terrain_registry)
            .insert_resource(level_registry);
    }
}

#[cfg(test)]
mod tests {
    // ================================================
    // AI Self-Check (test_spec.md §13.1)
    // ================================================
    // ✅ 测试行为，不是实现
    // ✅ 符合领域规则
    // ✅ 测试是确定性的
    // ✅ 使用标准测试数据
    // ✅ 没有测试私有实现
    // ✅ 没有生成不在范围内的测试
    // ================================================

    use super::*;
    use ron::de::from_bytes;

    /// Test ID: MAP-DAT-001
    /// Title: RON 反序列化 - 可通行地形定义
    ///
    /// Given: 包含 plain 地形的 RON 字符串
    /// When: 反序列化为 TerrainDefRon
    /// Then: 解析出正确的 id, move_cost, passable
    ///
    /// Assertions: id="plain", move_cost=1, passable=true
    #[test]
    fn ron_反序列化_地形定义() {
        // Given
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

        // When
        let def: TerrainDefRon = from_bytes(ron_str.as_bytes()).unwrap();

        // Then
        assert_eq!(def.id, "plain");
        assert_eq!(def.move_cost, 1);
        assert!(def.passable);
    }

    /// Test ID: MAP-DAT-002
    /// Title: RON 反序列化 - 不可通行地形（move_cost=0）
    ///
    /// Given: 包含 mountain 的 RON 字符串（move_cost=0, passable=false）
    /// When: 反序列化为 TerrainDefRon 并转换为 TerrainDef
    /// Then: TerrainDef.move_cost = None
    ///
    /// Assertions: move_cost=None, passable=false
    #[test]
    fn ron_反序列化_不可通行地形() {
        // Given
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

        // When
        let def: TerrainDefRon = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.move_cost, 0);
        assert!(!def.passable);
        let terrain_def: TerrainDef = def.into();

        // Then
        assert_eq!(terrain_def.move_cost, None);
    }

    /// Test ID: MAP-DAT-003
    /// Title: RON 反序列化 - 关卡配置
    ///
    /// Given: 包含地形网格、玩家和敌人单位的 RON 字符串
    /// When: 反序列化为 LevelConfigDef
    /// Then: 解析出 id="tutorial", terrain_grid=4, player_units=1, enemy_units=1
    ///
    /// Assertions: 各字段长度和 ID 正确
    #[test]
    fn ron_反序列化_关卡配置() {
        // Given
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

        // When
        let def: LevelConfigDef = from_bytes(ron_str.as_bytes()).unwrap();

        // Then
        assert_eq!(def.id, "tutorial");
        assert_eq!(def.terrain_grid.len(), 4);
        assert_eq!(def.player_units.len(), 1);
        assert_eq!(def.enemy_units.len(), 1);
    }

    /// Test ID: MAP-DAT-004
    /// Title: LevelConfigDef 转换为 LevelConfig
    ///
    /// Given: 3x2 关卡，地形网格 "PPF" "PMM"
    /// When: 调用 LevelConfig::from_def()
    /// Then: terrain_map 正确解析每个格子
    ///
    /// Assertions: (0,0)="plain", (2,0)="forest", (2,1)="mountain"
    #[test]
    fn level_config_def_转换为_level_config() {
        // Given
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

        // When
        let config = LevelConfig::from_def(def, &terrain_reg);

        // Then
        assert_eq!(config.terrain_map.get(&(0, 0)), Some(&"plain".to_string()));
        assert_eq!(config.terrain_map.get(&(2, 0)), Some(&"forest".to_string()));
        assert_eq!(
            config.terrain_map.get(&(2, 1)),
            Some(&"mountain".to_string())
        );
    }

    /// Test ID: MAP-DAT-005
    /// Title: TerrainRegistry 兜底默认值
    ///
    /// Given: 空 TerrainRegistry，调用 register_defaults()
    /// When: 查询 plain/forest/mountain/water
    /// Then: 全部返回 Some，move_cost/defense_bonus 正确
    ///
    /// Assertions: plain.move_cost=Some(1), forest.defense_bonus=2
    #[test]
    fn terrain_registry_兜底默认值() {
        // Given
        let mut reg = TerrainRegistry::default();
        reg.register_defaults();

        // When & Then
        assert!(reg.get("plain").is_some());
        assert!(reg.get("forest").is_some());
        assert!(reg.get("mountain").is_some());
        assert!(reg.get("water").is_some());
        assert_eq!(reg.get("plain").unwrap().move_cost, Some(1));
        assert_eq!(reg.get("forest").unwrap().defense_bonus, 2);
    }

    /// Test ID: MAP-DAT-006
    /// Title: LevelRegistry 查询已注册关卡
    ///
    /// Given: LevelRegistry 中插入 "test" 关卡
    /// When: 调用 get("test")
    /// Then: 返回 Some
    ///
    /// Assertions: is_some()
    #[test]
    fn level_registry_查询关卡() {
        // Given
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

        // When & Then
        assert!(registry.get("test").is_some());
    }

    /// Test ID: MAP-DAT-007
    /// Title: LevelRegistry 查询未注册关卡返回 None
    ///
    /// Given: 空 LevelRegistry
    /// When: 调用 get("nonexistent")
    /// Then: 返回 None
    ///
    /// Assertions: is_none()
    #[test]
    fn level_registry_查询未注册返回none() {
        // Given
        let registry = LevelRegistry::default();

        // When & Then
        assert!(registry.get("nonexistent").is_none());
    }

    /// Test ID: MAP-DAT-008
    /// Title: LevelRegistry first() 返回第一个关卡
    ///
    /// Given: LevelRegistry 中插入 "a" 和 "b" 两个关卡
    /// When: 调用 first()
    /// Then: 返回 Some
    ///
    /// Assertions: is_some()
    #[test]
    fn level_registry_first() {
        // Given
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

        // When & Then
        assert!(registry.first().is_some());
    }

    /// Test ID: MAP-DAT-009
    /// Title: LevelRegistry first() 空注册表返回 None
    ///
    /// Given: 空 LevelRegistry
    /// When: 调用 first()
    /// Then: 返回 None
    ///
    /// Assertions: is_none()
    #[test]
    fn level_registry_first_空返回none() {
        // Given
        let registry = LevelRegistry::default();

        // When & Then
        assert!(registry.first().is_none());
    }

    /// Test ID: MAP-DAT-010
    /// Title: TerrainDefRon 转换可通行地形的 move_cost
    ///
    /// Given: TerrainDefRon 包含 forest (move_cost=2, passable=true)
    /// When: 转换为 TerrainDef
    /// Then: move_cost=Some(2), passable=true
    ///
    /// Assertions: move_cost 和 passable 正确
    #[test]
    fn terrain_def_ron_可通行地形move_cost() {
        // Given
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

        // When
        let terrain: TerrainDef = def.into();

        // Then
        assert_eq!(terrain.move_cost, Some(2));
        assert!(terrain.passable);
    }

    /// Test ID: MAP-DAT-011
    /// Title: LevelConfig 地形网格解析（2x2 多类型）
    ///
    /// Given: 2x2 关卡，地形网格 "PF" "MW"
    /// When: 调用 LevelConfig::from_def()
    /// Then: 四个格子分别解析为 plain/forest/mountain/water
    ///
    /// Assertions: (0,0)=plain, (1,0)=forest, (0,1)=mountain, (1,1)=water
    #[test]
    fn level_config_地形网格解析() {
        // Given
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

        // When
        let config = LevelConfig::from_def(def, &terrain_reg);

        // Then
        assert_eq!(config.terrain_map.get(&(0, 0)), Some(&"plain".to_string()));
        assert_eq!(config.terrain_map.get(&(1, 0)), Some(&"forest".to_string()));
        assert_eq!(
            config.terrain_map.get(&(0, 1)),
            Some(&"mountain".to_string())
        );
        assert_eq!(config.terrain_map.get(&(1, 1)), Some(&"water".to_string()));
    }

    /// Test ID: MAP-DAT-012
    /// Title: LevelConfig 自定义 char_map 覆盖默认
    ///
    /// Given: 自定义 char_map 包含 'D' -> "desert"
    /// When: 使用地形网格 "PD" 解析
    /// Then: 'P' 映射为 plain，自定义 'D' 映射为 desert
    ///
    /// Assertions: (0,0)=plain, (1,0)=desert
    #[test]
    fn level_config_自定义char_map覆盖默认() {
        // Given
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

        // When
        let config = LevelConfig::from_def(def, &terrain_reg);

        // Then
        assert_eq!(config.terrain_map.get(&(0, 0)), Some(&"plain".to_string()));
        assert_eq!(config.terrain_map.get(&(1, 0)), Some(&"desert".to_string()));
    }

    /// Test ID: MAP-DAT-013
    /// Title: TerrainRegistry char_map 返回正确映射
    ///
    /// Given: 注册了默认地形的 TerrainRegistry
    /// When: 调用 char_map()
    /// Then: P->plain, F->forest, M->mountain, W->water
    ///
    /// Assertions: 四个字符映射正确
    #[test]
    fn terrain_registry_char_map() {
        // Given
        let mut reg = TerrainRegistry::default();
        reg.register_defaults();

        // When
        let map = reg.char_map();

        // Then
        assert_eq!(map.get(&'P'), Some(&"plain".to_string()));
        assert_eq!(map.get(&'F'), Some(&"forest".to_string()));
        assert_eq!(map.get(&'M'), Some(&"mountain".to_string()));
        assert_eq!(map.get(&'W'), Some(&"water".to_string()));
    }
}
