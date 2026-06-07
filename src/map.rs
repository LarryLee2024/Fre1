// 地图模块：网格生成、地形数据、寻路

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
pub fn spawn_map(
    mut commands: Commands,
    map: Res<GameMap>,
) {
    // 简单测试地图：外圈山地/水域，内部随机地形
    for y in 0..map.height {
        for x in 0..map.width {
            let coord = IVec2::new(x as i32, y as i32);
            let terrain = if x == 0 || y == 0 || x == map.width - 1 || y == map.height - 1 {
                // 边界：山地
                Terrain::Mountain
            } else if (x + y) % 7 == 0 {
                // 水域
                Terrain::Water
            } else if (x + y) % 5 == 0 {
                // 森林
                Terrain::Forest
            } else {
                Terrain::Plain
            };

            let world_pos = map.coord_to_world(coord);
            let tile_size = map.tile_size;

            commands.spawn((
                Sprite::from_color(terrain.color(), Vec2::splat(tile_size - 2.0)),
                Transform::from_xyz(world_pos.x, world_pos.y, 0.0),
                Tile { coord, terrain },
            ));
        }
    }
}
