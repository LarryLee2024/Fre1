//! ShopScreen 组件 — 商店界面 UI 组件的类型定义
//!
//! 定义 ShopScreen 标记组件、ShopAction 按钮操作枚举和 ShopTabType 标签页枚举。
//! 这些组件用于构建商店界面的交互层，通过 Observer 模式处理用户操作。

use bevy::prelude::*;

/// 商店界面根标记组件
///
/// 用于 despawn 逻辑识别界面层级根节点。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct ShopScreen;

/// 商店界面按钮的操作标识
///
/// 作为 Component 挂载到按钮实体上。Observer 匹配此组件
/// 来确定哪个按钮被点击，并执行对应操作。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub enum ShopAction {
    /// 关闭商店界面
    Close,
    /// 切换到指定标签页（购买/出售）
    SwitchTab(ShopTabType),
    /// 将物品添加到购物车（MVP：直接触发 BuyItem）
    AddToCart {
        /// 物品定义 ID
        item_def_id: String,
        /// 物品价格
        price: u32,
    },
    /// 从购物车移除物品（MVP：直接触发 SellItem）
    RemoveFromCart {
        /// 物品定义 ID
        item_def_id: String,
        /// 物品价格
        price: u32,
    },
    /// 确认购买购物车中的所有物品
    ConfirmPurchase,
    /// 重试加载商店数据
    Retry,
}

/// 商店标签页类型
///
/// 标识当前激活的标签页（购买/出售），也作为组件挂载在
/// 购买列表/出售列表容器上，用于可见性控制。
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum ShopTabType {
    /// 购买标签页
    Buy,
    /// 出售标签页
    Sell,
}
