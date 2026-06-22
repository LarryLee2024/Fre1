//! InventoryVm — 库存视图模型
//!
//! 详见 `docs/06-ui/04-data-flow/projection-viewmodel.md` §3

use bevy::prelude::*;

/// 库存视图模型
#[derive(Resource, Clone, Reflect, Default)]
#[reflect(Resource)]
pub struct InventoryVm {
    /// 物品列表
    pub items: Vec<InventoryItemVm>,
    /// 持有金币数
    pub gold: u32,
}

/// 库存物品视图模型
#[derive(Clone, Reflect, Default)]
pub struct InventoryItemVm {
    /// 物品 Def ID
    pub def_id: String,
    /// 物品名称（本地化 Key）
    pub name: String,
    /// 数量
    pub quantity: u32,
}
