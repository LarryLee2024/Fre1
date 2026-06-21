//! ShopItemCard 组件类型定义
//!
//! 定义 ShopItemCard（Widget Contract 的本地状态）和 ShopItemAction（操作标记）。

use bevy::prelude::*;

/// ShopItemCard 卡片本地状态
///
/// 包含物品 ID、价格和库存数量。
/// Props 字段由 spawn_shop_item_card 的入参决定。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ShopItemCard {
    /// 物品 ID
    pub item_id: u32,
    /// 物品价格（金币）
    pub price: u32,
    /// 剩余库存数量
    pub stock: u32,
}

/// ShopItemCard 按钮动作标记
///
/// 标记卡片内的交互元素以区分不同操作。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub enum ShopItemAction {
    /// 购买物品
    Buy,
}
