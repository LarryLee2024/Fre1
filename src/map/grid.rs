// 地图网格：GameMap、Tile、Terrain、坐标转换、地图生成

use crate::assets::CnFont;
use super::data::{LevelRegistry, TerrainRegistry};
use bevy::prelude::*;

/// 地形类型（Tile 组件存储，用于寻路等运行时逻辑）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Terrain {
    Plain,
    Forest,
    Mountain,
    Water,
}

impl Terrain {
    /// 从地形 ID 字符串解析
    pub fn from_id(id: &str) -> Self {
        match id {
            "forest" => Terrain::Forest,
            "mountain" => Terrain::Mountain,
            "water" => Terrain::Water,
            _ => Terrain::Plain,
        }
    }

    /// 转换为地形 ID 字符串
    pub fn to_id(&self) -> &'static str {
        match self {
            Terrain::Plain => "plain",
            Terrain::Forest => "forest",
            Terrain::Mountain => "mountain",
            Terrain::Water => "water",
        }
    }

    /// 地形中文名（用于地图标注）
    pub fn label(&self) -> &'static str {
        match self {
            Terrain::Plain => "草",
            Terrain::Forest => "林",
            Terrain::Mountain => "山",
            Terrain::Water => "水",
        }
    }

    /// 移动消耗兜底值（TerrainRegistry 未加载时使用）
    pub fn move_cost_fallback(&self) -> Option<u32> {
        match self {
            Terrain::Plain => Some(1),
            Terrain::Forest => Some(2),
            Terrain::Mountain => None,
            Terrain::Water => None,
        }
    }

    /// 地形防御加成兜底值（TerrainRegistry 未加载时使用）
    pub fn defense_bonus_fallback(&self) -> i32 {
        match self {
            Terrain::Plain => 0,
            Terrain::Forest => 2,
            Terrain::Mountain => 0,
            Terrain::Water => 0,
        }
    }

    /// 地形颜色（从 TerrainRegistry 优先获取，此为兜底）
    pub fn color(&self) -> Color {
        match self {
            Terrain::Plain => Color::srgb(0.56, 0.73, 0.35),
            Terrain::Forest => Color::srgb(0.20, 0.50, 0.18),
            Terrain::Mountain => Color::srgb(0.55, 0.50, 0.45),
            Terrain::Water => Color::srgb(0.25, 0.47, 0.85),
        }
    }

    /// 从 TerrainRegistry 获取颜色
    pub fn color_from_registry(&self, registry: &TerrainRegistry) -> Color {
        let id = match self {
            Terrain::Plain => "plain",
            Terrain::Forest => "forest",
            Terrain::Mountain => "mountain",
            Terrain::Water => "water",
        };
        registry
            .get(id)
            .map(|def| Color::srgb(def.color.0, def.color.1, def.color.2))
            .unwrap_or_else(|| self.color())
    }
}

/// 地图格子组件
#[derive(Component)]
pub struct Tile {
    /// 网格坐标
    pub coord: IVec2,
    /// 地形类型
    pub terrain: Terrain,
    /// 移动消耗（从 TerrainRegistry 加载，None 表示不可通行）
    pub move_cost: Option<u32>,
    /// 地形防御加成（从 TerrainRegistry 加载）
    pub defense_bonus: i32,
}

/// 地图资源：存储地图尺寸
#[derive(Resource)]
pub struct GameMap {
    /// 地图宽度（格子数）
    pub width: u32,
    /// 地图高度（格子数）
    pub height: u32,
    /// 格子尺寸（像素）
    pub tile_size: f32,
}

impl Default for GameMap {
    fn default() -> Self {
        Self {
            width: 10,
            height: 8,
            tile_size: 64.0,
        }
    }
}

impl GameMap {
    /// 从关卡配置创建
    pub fn from_level(level: &super::data::LevelConfig) -> Self {
        Self {
            width: level.width,
            height: level.height,
            tile_size: level.tile_size,
        }
    }

    /// 网格坐标转世界坐标
    pub fn coord_to_world(&self, coord: IVec2) -> Vec2 {
        Vec2::new(
            (coord.x as f32 - self.width as f32 / 2.0 + 0.5) * self.tile_size,
            (coord.y as f32 - self.height as f32 / 2.0 + 0.5) * self.tile_size,
        )
    }

    /// 世界坐标转网格坐标
    pub fn world_to_coord(&self, world: Vec2) -> IVec2 {
        IVec2::new(
            ((world.x / self.tile_size + self.width as f32 / 2.0).floor()) as i32,
            ((world.y / self.tile_size + self.height as f32 / 2.0).floor()) as i32,
        )
    }

    /// 坐标是否在地图范围内
    pub fn is_in_bounds(&self, coord: IVec2) -> bool {
        coord.x >= 0
            && coord.y >= 0
            && (coord.x as u32) < self.width
            && (coord.y as u32) < self.height
    }
}

/// 生成地图（从 LevelConfig 加载地形布局）
pub fn spawn_map(
    mut commands: Commands,
    mut map: ResMut<GameMap>,
    cn_font: Res<CnFont>,
    terrain_registry: Res<TerrainRegistry>,
    level_registry: Res<LevelRegistry>,
) {
    let level = level_registry.first().cloned();

    if let Some(ref level) = level {
        *map = GameMap::from_level(level);
    }

    let small_font = TextFont {
        font: cn_font.handle.clone(),
        font_size: 10.0,
        ..default()
    };

    for y in 0..map.height {
        for x in 0..map.width {
            let coord = IVec2::new(x as i32, y as i32);

            // 从关卡配置获取地形，兜底用算法生成
            let terrain = if let Some(ref level) = level {
                level
                    .terrain_map
                    .get(&(x as i32, y as i32))
                    .map(|id| Terrain::from_id(id.as_str()))
                    .unwrap_or(Terrain::Plain)
            } else {
                // 兜底：算法生成
                if x == 0 || y == 0 || x == map.width - 1 || y == map.height - 1 {
                    Terrain::Mountain
                } else if (x + y) % 7 == 0 {
                    Terrain::Water
                } else if (x + y) % 5 == 0 {
                    Terrain::Forest
                } else {
                    Terrain::Plain
                }
            };

            let world_pos = map.coord_to_world(coord);
            let tile_size = map.tile_size;
            let terrain_color = terrain.color_from_registry(&terrain_registry);

            // 从 TerrainRegistry 获取地形属性
            let terrain_id = terrain.to_id();
            let (move_cost, defense_bonus) = terrain_registry
                .get(terrain_id)
                .map(|def| (def.move_cost, def.defense_bonus))
                .unwrap_or_else(|| (terrain.move_cost_fallback(), terrain.defense_bonus_fallback()));

            commands.spawn((
                Sprite::from_color(terrain_color, Vec2::splat(tile_size - 2.0)),
                Transform::from_xyz(world_pos.x, world_pos.y, 0.0),
                Tile { coord, terrain, move_cost, defense_bonus },
                children![
                    (
                        Text2d::new(format!("{},{}", coord.x, coord.y)),
                        small_font.clone(),
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.6)),
                        TextLayout::new_with_no_wrap(),
                        Transform::from_xyz(-tile_size * 0.3, tile_size * 0.3, 0.1),
                    ),
                    (
                        Text2d::new(terrain.label().to_string()),
                        small_font.clone(),
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                        TextLayout::new_with_no_wrap(),
                        Transform::from_xyz(0.0, -tile_size * 0.25, 0.1),
                    ),
                ],
            ));
        }
    }
}

/// 地图网格插件
pub struct MapGridPlugin;

impl Plugin for MapGridPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::{AppState, GameSet};
        app.insert_resource(GameMap::default())
            .add_systems(OnEnter(AppState::InGame), spawn_map.in_set(GameSet::Map));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 地形_从id解析() {
        assert_eq!(Terrain::from_id("plain"), Terrain::Plain);
        assert_eq!(Terrain::from_id("forest"), Terrain::Forest);
        assert_eq!(Terrain::from_id("mountain"), Terrain::Mountain);
        assert_eq!(Terrain::from_id("water"), Terrain::Water);
        assert_eq!(Terrain::from_id("unknown"), Terrain::Plain);
    }

    #[test]
    fn 地形_移动消耗兜底值() {
        assert_eq!(Terrain::Plain.move_cost_fallback(), Some(1));
        assert_eq!(Terrain::Forest.move_cost_fallback(), Some(2));
        assert_eq!(Terrain::Mountain.move_cost_fallback(), None);
        assert_eq!(Terrain::Water.move_cost_fallback(), None);
    }

    #[test]
    fn 地形_防御加成兜底值() {
        assert_eq!(Terrain::Plain.defense_bonus_fallback(), 0);
        assert_eq!(Terrain::Forest.defense_bonus_fallback(), 2);
    }

    #[test]
    fn 地形_中文名() {
        assert_eq!(Terrain::Plain.label(), "草");
        assert_eq!(Terrain::Forest.label(), "林");
    }

    #[test]
    fn game_map_从关卡配置创建() {
        let level = super::super::data::LevelConfig {
            id: "test".into(),
            name: "测试".into(),
            width: 12,
            height: 10,
            tile_size: 48.0,
            terrain_map: Default::default(),
            player_units: vec![],
            enemy_units: vec![],
        };
        let map = GameMap::from_level(&level);
        assert_eq!(map.width, 12);
        assert_eq!(map.height, 10);
        assert_eq!(map.tile_size, 48.0);
    }

    fn make_map() -> GameMap {
        GameMap {
            width: 10,
            height: 8,
            tile_size: 64.0,
        }
    }

    #[test]
    fn 坐标转世界_左下角原点() {
        let map = make_map();
        let pos = map.coord_to_world(IVec2::new(0, 0));
        assert_eq!(pos.x, -288.0);
        assert_eq!(pos.y, -224.0);
    }

    #[test]
    fn 坐标转世界_地图中心() {
        let map = make_map();
        let pos = map.coord_to_world(IVec2::new(5, 4));
        assert_eq!(pos.x, 32.0);
        assert_eq!(pos.y, 32.0);
    }

    #[test]
    fn 世界转坐标_往返一致() {
        let map = make_map();
        for coord in [
            IVec2::new(0, 0),
            IVec2::new(5, 4),
            IVec2::new(9, 7),
        ] {
            let world = map.coord_to_world(coord);
            let back = map.world_to_coord(world);
            assert_eq!(coord, back);
        }
    }

    #[test]
    fn 边界_内部坐标合法() {
        let map = make_map();
        assert!(map.is_in_bounds(IVec2::new(0, 0)));
        assert!(map.is_in_bounds(IVec2::new(9, 7)));
    }

    #[test]
    fn 边界_负坐标非法() {
        let map = make_map();
        assert!(!map.is_in_bounds(IVec2::new(-1, 0)));
    }

    #[test]
    fn 边界_超出宽高非法() {
        let map = make_map();
        assert!(!map.is_in_bounds(IVec2::new(10, 0)));
        assert!(!map.is_in_bounds(IVec2::new(0, 8)));
    }
}
