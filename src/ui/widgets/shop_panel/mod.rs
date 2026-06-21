//! Module Name: ShopPanel Widget — 商店面板有机体
//!
//! 组合 Panel / Text / TabPanel / ShopItemCard / InventoryItemRow / Button
//! 为一个结构化的商店面板视图，包含标题、金币显示、商品卡片列表和关闭按钮。
//! 注册为 WidgetsPlugin 的子插件。

pub mod components;
pub mod factory;

use bevy::prelude::*;

use self::components::{ShopPanel, ShopPanelAction};

/// ShopPanelPlugin — 注册 ShopPanel Widget 的 Component 类型
///
/// 由 WidgetsPlugin 添加。当前为静态布局组合，无需更新系统。
pub struct ShopPanelPlugin;

impl Plugin for ShopPanelPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ShopPanel>()
            .register_type::<ShopPanelAction>();
    }
}
