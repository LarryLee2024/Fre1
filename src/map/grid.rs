// 地图网格：GameMap、坐标转换、地图渲染
// Terrain 枚举和 Tile 组件已删除，地形数据由 TerrainGrid 纯数据存储
// 渲染层与数据层分离：spawn_map 只负责画格子

use super::data::{LevelRegistry, TerrainRegistry};
use super::runtime::TerrainGrid;
use crate::assets::CnFont;
use bevy::prelude::*;

/// 地图资源：存储地图尺寸
#[derive(Resource, Reflect)]
#[reflect(Resource)]
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

/// 地图渲染标记组件
#[derive(Component)]
pub struct TileSprite;

/// 生成地图：从 TerrainGrid 读取地形 ID，用 TerrainRegistry 查找颜色/属性进行渲染
/// 不再生成 Tile Entity，只生成纯渲染 Sprite
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

    // 构建 TerrainGrid
    let terrain_grid = if let Some(ref level) = level {
        TerrainGrid::from_terrain_map(map.width, map.height, &level.terrain_map)
    } else {
        // LevelRegistry 为空时不创建假数据，使用全平地并输出警告
        bevy::log::warn!(
            target: "map",
            "LevelRegistry 为空，使用全平地地图"
        );
        TerrainGrid::default_plain(map.width, map.height)
    };

    // 插入 TerrainGrid 资源
    commands.insert_resource(terrain_grid.clone());

    let small_font = cn_font.text_font(10.0);

    // 渲染层：从 TerrainGrid 读取数据画格子
    for (coord, terrain_id) in terrain_grid.iter() {
        let world_pos = map.coord_to_world(coord);
        let tile_size = map.tile_size;

        // 从 TerrainRegistry 获取地形属性
        let (terrain_color, terrain_name, move_cost) = terrain_registry
            .get(terrain_id)
            .map(|def| {
                (
                    Color::srgb(def.color.0, def.color.1, def.color.2),
                    def.name.as_str(),
                    def.move_cost,
                )
            })
            .unwrap_or_else(|| {
                // TerrainRegistry 中未找到定义时使用统一默认值
                bevy::log::warn!(
                    target: "map",
                    terrain_id = %terrain_id,
                    "地形定义未找到，使用默认渲染"
                );
                (
                    Color::srgb(0.5, 0.5, 0.5),
                    "?",
                    None,
                )
            });

        let move_cost_str = match move_cost {
            Some(c) => format!("{}", c),
            None => "×".to_string(),
        };

        commands.spawn((
            Sprite::from_color(terrain_color, Vec2::splat(tile_size - 2.0)),
            Transform::from_xyz(world_pos.x, world_pos.y, 0.0),
            TileSprite,
            children![
                (
                    Text2d::new(format!("{},{}", coord.x, coord.y)),
                    small_font.clone(),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.6)),
                    TextLayout::new_with_no_wrap(),
                    Transform::from_xyz(-tile_size * 0.3, tile_size * 0.3, 0.1),
                ),
                (
                    Text2d::new(format!("{}{}", terrain_name, move_cost_str)),
                    small_font.clone(),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                    TextLayout::new_with_no_wrap(),
                    Transform::from_xyz(0.0, -tile_size * 0.25, 0.1),
                ),
            ],
        ));
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
        for coord in [IVec2::new(0, 0), IVec2::new(5, 4), IVec2::new(9, 7)] {
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
