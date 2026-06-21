//! Importer — 构建时转换逻辑（TMX → MapAsset）
//!
//! 本模块包含 Importer 使用的纯函数转换逻辑：
//! - 稳定 GUID 生成（内容哈希）
//! - 像素坐标 → GridPos 转换
//! - NavigationMask 预计算
//! - 地图数据验证
//!
//! 本模块不包含 TMX 解析代码——TMX 解析位于 `tools/map_importer/` 独立工具 crate。
//! 🟥 游戏运行时不需要此模块中的任何代码——此模块提供给 Importer 工具共享。
//!
//! 详见 ADR-065 §2 (Importer 管线设计) 和 map-importer-schema.md

use std::collections::HashMap;

use super::asset::{MapAsset, NavigationMask, TerrainGrid, TileEntry};

// ─── 稳定 GUID 生成 ─────────────────────────────────────────────

/// 使用确定性内容哈希生成稳定 GUID。
///
/// GUID = SipHash-2-4(map_id, layer_name, object_class, pos_x, pos_y)
///
/// 相同输入永远产生相同输出，不依赖 Tiled 内部 ID。
pub fn generate_guid(
    map_id: &str,
    layer_name: &str,
    object_class: &str,
    pos_x: i32,
    pos_y: i32,
) -> u64 {
    use std::hash::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    map_id.hash(&mut hasher);
    layer_name.hash(&mut hasher);
    object_class.hash(&mut hasher);
    pos_x.hash(&mut hasher);
    pos_y.hash(&mut hasher);
    hasher.finish()
}

// ─── 像素坐标 → GridPos 转换 ───────────────────────────────────

/// 将 TMX 像素坐标转换为网格坐标。
///
/// TMX 坐标系统：左上角原点，X 向右，Y 向下。
/// GridPos 坐标系统：左下角或左上角原点（取决于布局）。
pub fn pixel_to_gridpos(
    pixel_x: f32,
    pixel_y: f32,
    tile_width: u32,
    tile_height: u32,
    _layout: super::types::MapGridLayout,
) -> (i32, i32) {
    let gx = (pixel_x / tile_width as f32).floor() as i32;
    let gy = (pixel_y / tile_height as f32).floor() as i32;
    (gx, gy)
}

// ─── NavigationMask 预计算 ────────────────────────────────────

/// 导航掩码位定义。
pub mod nav_bits {
    /// 地面单位可通行 (0x01)
    pub const WALK: u8 = 0x01;
    /// 飞行单位可通行 (0x02)
    pub const FLY: u8 = 0x02;
    /// 游泳单位可通行 (0x04)
    pub const SWIM: u8 = 0x04;
}

/// 从 TerrainGrid 预计算 NavigationMask。
///
/// 参数:
///   grid: TerrainGrid（MapAsset 中的地形网格）
///   terrain_passability: terrain_id → (walkable, flyable) 的映射表
///
/// 返回: NavigationMask
pub fn build_navigation_mask(
    grid: &TerrainGrid,
    terrain_passability: &HashMap<String, (bool, bool)>,
) -> NavigationMask {
    let size = (grid.width * grid.height) as usize;
    let mut data = vec![0u8; size];

    for (i, tile) in grid.tiles.iter().enumerate() {
        if let Some((walkable, flyable)) = terrain_passability.get(&tile.terrain_id) {
            let mut byte = 0u8;
            if *walkable {
                byte |= nav_bits::WALK;
            }
            if *flyable {
                byte |= nav_bits::FLY;
            }
            data[i] = byte;
        }
    }

    NavigationMask {
        width: grid.width,
        height: grid.height,
        data,
    }
}

// ─── 验证 ─────────────────────────────────────────────────────

/// 验证结果。
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// 错误列表（必须修复）。
    pub errors: Vec<String>,
    /// 警告列表（建议修复）。
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// 创建空的验证结果。
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// 是否无错误。
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// 合并另一个验证结果。
    pub fn merge(&mut self, other: Self) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }

    /// 添加错误。
    pub fn add_error(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
    }

    /// 添加警告。
    pub fn add_warning(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// 验证 MapAsset 的尺寸一致性。
///
/// 检查:
/// - width * height == tiles.len()
/// - NavigationMask 尺寸与 terrain_grid 一致
pub fn validate_map_asset_dimensions(map: &MapAsset) -> ValidationResult {
    let mut result = ValidationResult::new();

    let expected_tiles = map.metadata.width * map.metadata.height;
    let actual_tiles = map.terrain_grid.tiles.len() as u32;

    if expected_tiles != actual_tiles {
        result.add_error(format!(
            "Tile 数量不一致: 期望 {} ({}x{}), 实际 {}",
            expected_tiles, map.metadata.width, map.metadata.height, actual_tiles
        ));
    }

    if map.navigation_mask.width != map.metadata.width {
        result.add_error(format!(
            "NavigationMask 宽度不一致: 期望 {}, 实际 {}",
            map.metadata.width, map.navigation_mask.width
        ));
    }

    if map.navigation_mask.height != map.metadata.height {
        result.add_error(format!(
            "NavigationMask 高度不一致: 期望 {}, 实际 {}",
            map.metadata.height, map.navigation_mask.height
        ));
    }

    let nav_expected = (map.navigation_mask.width * map.navigation_mask.height) as usize;
    if map.navigation_mask.data.len() != nav_expected {
        result.add_error(format!(
            "NavigationMask 数据长度不一致: 期望 {}, 实际 {}",
            nav_expected,
            map.navigation_mask.data.len()
        ));
    }

    result
}

/// 验证 GUID 唯一性（所有 ObjectLayer + SpawnPoints 中无重复 GUID）。
pub fn validate_guid_uniqueness(map: &MapAsset) -> ValidationResult {
    let mut result = ValidationResult::new();
    let mut seen = std::collections::HashSet::new();

    for layer in &map.object_layers {
        for obj in &layer.objects {
            if !seen.insert(obj.guid) {
                result.add_error(format!(
                    "重复 GUID {} 在对象 '{}/{}' 中",
                    obj.guid, layer.name, obj.name
                ));
            }
        }
    }

    for spawn in &map.spawn_points {
        if !seen.insert(spawn.guid) {
            result.add_error(format!(
                "重复 GUID {} 在出生点 '{}' 中",
                spawn.guid, spawn.spawn_group_id
            ));
        }
    }

    result
}

/// 验证 NavigationMask 与 TileEntry flags 的一致性。
pub fn validate_navigation_consistency(
    grid: &TerrainGrid,
    nav: &NavigationMask,
) -> ValidationResult {
    let mut result = ValidationResult::new();

    for (i, tile) in grid.tiles.iter().enumerate() {
        let nav_walk = (nav.data[i] & nav_bits::WALK) != 0;
        let tile_walk = tile.flags.contains(super::types::MapTileFlags::PASSABLE);

        if nav_walk != tile_walk {
            let x = i as u32 % grid.width;
            let y = i as u32 / grid.width;
            result.add_warning(format!(
                "POS({},{}) Walk 不一致: nav={}, tile={}",
                x, y, nav_walk, tile_walk
            ));
        }
    }

    result
}
