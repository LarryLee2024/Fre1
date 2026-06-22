//! 模块名: ShopItemCard Widget — 商品卡片复合控件
//!
//! 组合 Panel / Text / Button 三个原子组件为一个商品卡片。
//! 每个卡片显示物品名称、价格、库存和购买按钮。
//!
//! 契约:
//!   输入属性:    item_name, price, stock（通过 ShopItemCard）
//!   输出事件:  ShopItemAction::Buy 标记在按钮实体上供 Observer 路由
//!   本地状态:      ShopItemCard（item_id, price, stock）
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md`

pub mod components;
pub mod factory;

use bevy::prelude::*;

use self::components::{ShopItemAction, ShopItemCard};

/// ShopItemCardPlugin — 注册 ShopItemCard Widget 所需的 Component
pub struct ShopItemCardPlugin;

impl Plugin for ShopItemCardPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ShopItemCard>()
            .register_type::<ShopItemAction>();
    }
}
