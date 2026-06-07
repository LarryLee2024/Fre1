// 地图模块：网格生成、地形数据、寻路

use crate::assets::CnFont;
use bevy::prelude::*;

/// 地形类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Terrain {
    /// 平地
    Plain,
    /// 森林（增加防御，移动消耗+1）
    Forest,
    /// 山地（不可通行）
    Mountain,
    /// 水域（不可通行）
    Water,
}

impl Terrain {
    /// 地形中文名（用于地图标注）
    pub fn label(&self) -> &'static str {
        match self {
            Terrain::Plain => "草",
            Terrain::Forest => "林",
            Terrain::Mountain => "山",
            Terrain::Water => "水",
        }
    }

    /// 移动消耗
    pub fn move_cost(&self) -> Option<u32> {
        match self {
            Terrain::Plain => Some(1),
            Terrain::Forest => Some(2),
            Terrain::Mountain => None,
            Terrain::Water => None,
        }
    }

    /// 地形防御加成
    pub fn defense_bonus(&self) -> i32 {
        match self {
            Terrain::Plain => 0,
            Terrain::Forest => 2,
            Terrain::Mountain => 0,
            Terrain::Water => 0,
        }
    }

    /// 地形颜色
    pub fn color(&self) -> Color {
        match self {
            Terrain::Plain => Color::srgb(0.56, 0.73, 0.35),
            Terrain::Forest => Color::srgb(0.20, 0.50, 0.18),
            Terrain::Mountain => Color::srgb(0.55, 0.50, 0.45),
            Terrain::Water => Color::srgb(0.25, 0.47, 0.85),
        }
    }
}

/// 地图格子组件
#[derive(Component)]
pub struct Tile {
    /// 网格坐标
    pub coord: IVec2,
    /// 地形类型
    pub terrain: Terrain,
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

/// 生成地图
pub fn spawn_map(mut commands: Commands, map: Res<GameMap>, cn_font: Res<CnFont>) {
    let small_font = TextFont {
        font: cn_font.handle.clone(),
        font_size: 10.0,
        ..default()
    };

    for y in 0..map.height {
        for x in 0..map.width {
            let coord = IVec2::new(x as i32, y as i32);
            let terrain = if x == 0 || y == 0 || x == map.width - 1 || y == map.height - 1 {
                Terrain::Mountain
            } else if (x + y) % 7 == 0 {
                Terrain::Water
            } else if (x + y) % 5 == 0 {
                Terrain::Forest
            } else {
                Terrain::Plain
            };

            let world_pos = map.coord_to_world(coord);
            let tile_size = map.tile_size;

            // 格子精灵
            commands.spawn((
                Sprite::from_color(terrain.color(), Vec2::splat(tile_size - 2.0)),
                Transform::from_xyz(world_pos.x, world_pos.y, 0.0),
                Tile { coord, terrain },
                children![
                    // 坐标标注（左上角）
                    (
                        Text2d::new(format!("{},{}", coord.x, coord.y)),
                        small_font.clone(),
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.6)),
                        TextLayout::new_with_no_wrap(),
                        Transform::from_xyz(-tile_size * 0.3, tile_size * 0.3, 0.1),
                    ),
                    // 地形类别标注（中央偏下）
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

/// 地图管理插件
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::{AppState, GameSet};
        app.init_resource::<GameMap>()
            .add_systems(OnEnter(AppState::InGame), spawn_map.in_set(GameSet::Map));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Terrain 方法 ----

    #[test]
    fn 地形_移动消耗() {
        assert_eq!(Terrain::Plain.move_cost(), Some(1));
        assert_eq!(Terrain::Forest.move_cost(), Some(2));
        assert_eq!(Terrain::Mountain.move_cost(), None);
        assert_eq!(Terrain::Water.move_cost(), None);
    }

    #[test]
    fn 地形_防御加成() {
        assert_eq!(Terrain::Plain.defense_bonus(), 0);
        assert_eq!(Terrain::Forest.defense_bonus(), 2);
        assert_eq!(Terrain::Mountain.defense_bonus(), 0);
        assert_eq!(Terrain::Water.defense_bonus(), 0);
    }

    #[test]
    fn 地形_中文名() {
        assert_eq!(Terrain::Plain.label(), "草");
        assert_eq!(Terrain::Forest.label(), "林");
        assert_eq!(Terrain::Mountain.label(), "山");
        assert_eq!(Terrain::Water.label(), "水");
    }

    // ---- GameMap 坐标转换 ----

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
        // (0,0) → (-4.5*64, -3.5*64) = (-288, -224)
        let pos = map.coord_to_world(IVec2::new(0, 0));
        assert_eq!(pos.x, -288.0);
        assert_eq!(pos.y, -224.0);
    }

    #[test]
    fn 坐标转世界_地图中心() {
        let map = make_map();
        // (5,4) → (0.5*64, 0.5*64) = (32, 32)
        let pos = map.coord_to_world(IVec2::new(5, 4));
        assert_eq!(pos.x, 32.0);
        assert_eq!(pos.y, 32.0);
    }

    #[test]
    fn 坐标转世界_右上角() {
        let map = make_map();
        // (9,7) → (4.5*64, 3.5*64) = (288, 224)
        let pos = map.coord_to_world(IVec2::new(9, 7));
        assert_eq!(pos.x, 288.0);
        assert_eq!(pos.y, 224.0);
    }

    #[test]
    fn 世界转坐标_往返一致() {
        let map = make_map();
        for coord in [
            IVec2::new(0, 0),
            IVec2::new(5, 4),
            IVec2::new(9, 7),
            IVec2::new(3, 6),
        ] {
            let world = map.coord_to_world(coord);
            let back = map.world_to_coord(world);
            assert_eq!(coord, back, "coord {:?} 往返不一致", coord);
        }
    }

    #[test]
    fn 世界转坐标_格子内任意点映射同一格() {
        let map = make_map();
        let center = map.coord_to_world(IVec2::new(3, 3));
        // 偏移不超过半格，应映射回同一格
        let offset = Vec2::new(10.0, -10.0);
        let result = map.world_to_coord(center + offset);
        assert_eq!(result, IVec2::new(3, 3));
    }

    // ---- is_in_bounds ----

    #[test]
    fn 边界_内部坐标合法() {
        let map = make_map();
        assert!(map.is_in_bounds(IVec2::new(0, 0)));
        assert!(map.is_in_bounds(IVec2::new(9, 7)));
        assert!(map.is_in_bounds(IVec2::new(5, 4)));
    }

    #[test]
    fn 边界_负坐标非法() {
        let map = make_map();
        assert!(!map.is_in_bounds(IVec2::new(-1, 0)));
        assert!(!map.is_in_bounds(IVec2::new(0, -1)));
    }

    #[test]
    fn 边界_超出宽高非法() {
        let map = make_map();
        assert!(!map.is_in_bounds(IVec2::new(10, 0)));
        assert!(!map.is_in_bounds(IVec2::new(0, 8)));
        assert!(!map.is_in_bounds(IVec2::new(10, 8)));
    }
}
