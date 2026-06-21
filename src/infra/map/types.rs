//! 辅助类型定义——MapObjectGuid, PropertyMap, ObjectShape 等
//!
//! 这些类型被 MapAsset 引用，单独拆分以避免 asset.rs 过于庞大。
//! 详见 ADR-065 §3 (MapAsset 结构定义) 和 map-asset-schema.md

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── MapObjectGuid ─────────────────────────────────────────────────

/// 地图对象稳定 GUID——全局唯一、跨存档稳定。
///
/// 由 Importer 使用确定性内容哈希生成，不依赖 Tiled ID。
/// GUID 生成算法: hash(map_id, layer_name, object_class, tile_x, tile_y)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct MapObjectGuid(pub u64);

impl MapObjectGuid {
    /// 创建新的 GUID。
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// 返回内部 u64 值。
    pub const fn get(&self) -> u64 {
        self.0
    }
}

impl From<u64> for MapObjectGuid {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for MapObjectGuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GUID({:#016X})", self.0)
    }
}

// ─── PropertyMap ───────────────────────────────────────────────────

/// 属性映射——泛型键值对容器。
///
/// 消费方是运行时 Domain 系统（InteractionSystem、HazardSystem 等）。
/// 不承载核心 Gameplay 数值——仅用于标记和配置覆盖。
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PropertyMap {
    /// 键值对集合
    pub entries: HashMap<String, PropertyValue>,
}

impl PropertyMap {
    /// 创建空的属性映射。
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// 获取字符串属性值。
    pub fn get_string(&self, key: &str) -> Option<&str> {
        match self.entries.get(key) {
            Some(PropertyValue::String(v)) => Some(v.as_str()),
            _ => None,
        }
    }

    /// 获取整数属性值。
    pub fn get_int(&self, key: &str) -> Option<i32> {
        match self.entries.get(key) {
            Some(PropertyValue::Int(v)) => Some(*v),
            _ => None,
        }
    }

    /// 获取浮点属性值。
    pub fn get_float(&self, key: &str) -> Option<f32> {
        match self.entries.get(key) {
            Some(PropertyValue::Float(v)) => Some(*v),
            _ => None,
        }
    }

    /// 获取布尔属性值。
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.entries.get(key) {
            Some(PropertyValue::Bool(v)) => Some(*v),
            _ => None,
        }
    }
}

/// 属性值类型——支持 Tiled 的所有原生 Property 类型。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PropertyValue {
    /// 字符串值
    String(String),
    /// 整数值
    Int(i32),
    /// 浮点值
    Float(f32),
    /// 布尔值
    Bool(bool),
    /// 颜色值（RGBA，每个分量 0.0-1.0）
    Color([f32; 4]),
    /// 文件路径（Tiled 的 File 类型）
    File(String),
}

// ─── ObjectShape ───────────────────────────────────────────────────

/// 对象形状——用于碰撞/区域判定。
///
/// Point 和 Rectangle 会被 Importer 从像素坐标→网格坐标转换，
/// 其他复杂形状保持原始像素坐标。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ObjectShape {
    /// 点（无尺寸）
    Point,
    /// 矩形
    Rectangle {
        /// 宽度（Importer 转换为格数）
        width: u32,
        /// 高度（Importer 转换为格数）
        height: u32,
    },
    /// 椭圆
    Ellipse {
        /// 宽度
        width: u32,
        /// 高度
        height: u32,
    },
    /// 多边形
    Polygon {
        /// 顶点列表（像素坐标，相对于对象原点）
        points: Vec<(f32, f32)>,
    },
    /// 折线
    Polyline {
        /// 顶点列表（像素坐标，相对于对象原点）
        points: Vec<(f32, f32)>,
    },
}

// ─── GridLayout ────────────────────────────────────────────────────

/// 网格布局类型——与 Tactical Domain 的 GridLayout 一致。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum MapGridLayout {
    /// 四向网格（简单）
    Square,
    /// 六边形，奇数列偏移
    HexRowOdd,
    /// 六边形，偶数列偏移
    HexRowEven,
    /// 六边形，奇数行偏移
    HexColOdd,
    /// 六边形，偶数列偏移
    HexColEven,
}

// ─── HexDirection ──────────────────────────────────────────────────

/// 六边形方向——用于 SpawnPoint.facing。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum MapHexDirection {
    North,
    NorthEast,
    SouthEast,
    South,
    SouthWest,
    NorthWest,
    /// 方形网格下的别名
    East,
    West,
}

// ─── TileFlags（MapAsset 层） ────────────────────────────────────

/// Tile 通行标记——MapAsset 层使用的标记结构。
///
/// 与 Tactical Domain 的 TileFlags（resources.rs）逻辑一致，
/// 但物理上是独立类型。两者通过 From/TryFrom 转换关联。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct MapTileFlags(pub u8);

impl MapTileFlags {
    pub const PASSABLE: Self = Self(0b0000_0001);
    pub const FLYABLE: Self = Self(0b0000_0010);
    pub const BUILDABLE: Self = Self(0b0000_0100);
    pub const BLOCKS_SIGHT: Self = Self(0b0000_1000);

    /// 检查是否包含指定标记位。
    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}
