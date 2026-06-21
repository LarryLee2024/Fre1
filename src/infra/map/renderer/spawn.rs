//! Spawn — Tile Entity 生成（V1: Entity-per-Tile）
//!
//! 将 GridMap 数据转换为视觉 Tile Entity。
//! 每个 Tile 一个 Entity，带 SpriteSheetBundle + TextureAtlas。
//!
//! 详见 docs/06-ui/04-data-flow/map-rendering.md

use bevy::prelude::*;

// ─── 渲染标记组件 ────────────────────────────────────────────────

/// 标记当前激活的地图根 Entity。只存在一个 MapRoot。
///
/// 所有 Tile Entity 以 MapRoot 为父节点，清理时只需 despawn MapRoot。
#[derive(Component, Debug, Clone)]
pub struct MapRoot;

/// 标记 Tile Entity 的 Marker 组件。
///
/// 不含渲染数据——Sprite/Transform 由 SpriteSheetBundle 提供。
/// 此组件仅用于查询和过滤。
#[derive(Component, Debug, Clone)]
pub struct MapTileMarker;

/// 每个 Tile 的渲染标识数据。
///
/// 存储在 Tile Entity 上，供调试和系统查询使用。
#[derive(Component, Debug, Clone)]
pub struct TileVisual {
    /// 地形索引（对应 TextureAtlas 中的帧索引）
    pub terrain_index: u16,
    /// 网格坐标 X
    pub grid_x: u32,
    /// 网格坐标 Y
    pub grid_y: u32,
}

// ─── 渲染配置 ────────────────────────────────────────────────────

/// 地图渲染配置 Resource。
///
/// 存储渲染所需的资产句柄和显示参数。
#[derive(Resource, Debug, Clone)]
pub struct MapRenderConfig {
    /// 地图瓦片图集纹理句柄（可选，未设置时跳过精灵渲染）
    pub tileset_texture: Option<Handle<Image>>,
    /// 地图瓦片图集布局句柄（可选，未设置时跳过精灵渲染）
    pub tileset_layout: Option<Handle<TextureAtlasLayout>>,
    /// 是否可见（用于性能优化）
    pub visible: bool,
    /// 每个 Tile 的像素大小
    pub tile_pixel_size: f32,
}

impl Default for MapRenderConfig {
    fn default() -> Self {
        Self {
            tileset_texture: None,
            tileset_layout: None,
            visible: true,
            tile_pixel_size: 64.0,
        }
    }
}

// ─── 生成系统 ────────────────────────────────────────────────────

/// 生成所有 Tile 的视觉实体。
///
/// 参数:
///   commands: Commands（ECS 命令）
///   grid: GridMap（转换后的网格数据）
///   config: MapRenderConfig（渲染配置）
///   terrain_index: TerrainIndex（地形索引，用于查询 terrain_index）
///
/// 返回: Entity — 新创建的 MapRoot Entity
///
/// 在每个 Tile 上生成:
/// - SpatialBundle（Transform）
/// - MapTileMarker
/// - TileVisual
/// - (可选) SpriteSheetBundle（如果 tileset_texture 已设置）
pub fn spawn_tile_entities(
    commands: &mut Commands,
    width: u32,
    height: u32,
    tiles: &[crate::core::domains::tactical::resources::TileData],
    config: &MapRenderConfig,
) -> Entity {
    let map_root = commands.spawn((Name::new("MapRoot"), MapRoot)).id();

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let tile = &tiles[idx];
            let terrain_id = tile.terrain_def_id();
            let world_x = x as f32 * config.tile_pixel_size;
            let world_y = y as f32 * config.tile_pixel_size;

            let mut entity_cmd = commands.spawn((
                Name::new(format!("Tile({}, {})", x, y)),
                MapTileMarker,
                TileVisual {
                    terrain_index: terrain_id,
                    grid_x: x,
                    grid_y: y,
                },
                Transform::from_xyz(world_x, world_y, 0.0),
            ));

            // TODO[P3][Map][2026-07-01]: 接入 TextureAtlas 精灵渲染
            // 当 tileset_texture 和 tileset_layout 就绪后，添加 Sprite 组件:
            // entity_cmd.insert(SpriteBundle {
            //     texture: tex.clone(),
            //     ..Default::default()
            // });

            entity_cmd.set_parent_in_place(map_root);
        }
    }

    map_root
}
