//! ShopPanel 组件 — ShopPanel 有机体的类型定义
//!
//! 定义 ShopPanel 标记组件和 ShopPanelAction 枚举，
//! 用于识别商店面板中的交互元素。

use bevy::prelude::*;

/// ShopPanel 标记组件
///
/// 标识 ShopPanel Widget 的根实体。
/// 用于清理和基于查询的定位。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ShopPanel;

/// 可从 ShopPanel 按钮触发的操作
///
/// 作为 Component 挂载到交互子实体上。
/// Observer 查询此组件来确定哪个按钮被点击。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub enum ShopPanelAction {
    /// 关闭商店界面
    Close,
    /// 购买物品
    BuyItem {
        /// 物品 ID
        item_id: u32,
        /// 物品价格（金币）
        price: u64,
    },
    /// 出售物品
    SellItem {
        /// 物品 ID
        item_id: u32,
        /// 物品价格（金币）
        price: u64,
    },
}
