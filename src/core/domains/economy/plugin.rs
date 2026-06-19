//! EconomyPlugin — 经济/交易领域 Plugin
//!
//! 注册钱包、商店组件和交易系统。
//! 详见 docs/02-domain/domains/economy_domain.md

use bevy::prelude::*;

use super::components::{ShopInstance, Wallet};
use super::resources::EconomyConfig;
use super::systems::{on_purchase_request, on_sell_request};

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Wallet>();
        app.register_type::<ShopInstance>();

        app.init_resource::<EconomyConfig>();

        app.add_observer(on_purchase_request);
        app.add_observer(on_sell_request);
    }
}
