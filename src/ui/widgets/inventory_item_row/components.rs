//! InventoryItemRow 组件的类型定义
//!
//! 定义 InventoryItemRowState（Widget Contract 的本地状态）、
//! InventoryItemAction（按钮动作标记）和 InventoryItemRowLabel（子文本标记）。
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md`

use bevy::prelude::*;

/// 物品行本地状态（Widget Contract Local State）
///
/// 包含物品名称和数量。Props 字段由 spawn_inventory_item_row 的入参决定，
/// runtime 由外部系统更新。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct InventoryItemRowState {
    /// 物品显示名称
    pub item_name: String,
    /// 物品数量
    pub quantity: u32,
}

/// 物品行按钮动作标记
///
/// 标记物品行内的按钮为特定动作，供 Observer 或其他系统识别交互意图。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub enum InventoryItemAction {
    /// 使用物品
    Use,
    /// 查看物品详情
    Inspect,
}

/// 物品行子文本标记
///
/// 标记物品行内的文本实体类型，供更新系统区分名称文本和数量文本。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub enum InventoryItemRowLabel {
    /// 物品名称文本
    Name,
    /// 物品数量文本（如 "x5"）
    Quantity,
}
