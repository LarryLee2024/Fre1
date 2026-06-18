//! 经济/交易领域 — 资源配置

use bevy::prelude::*;

/// 经济系统配置。
#[derive(Debug, Clone, Resource)]
pub struct EconomyConfig {
    /// 出售物品的基准折扣系数（通常 0.5）
    pub sell_price_ratio: f32,
    /// 赃物折扣系数
    pub stolen_goods_ratio: f32,
    /// 是否启用供需系统
    pub supply_demand_enabled: bool,
    /// 是否允许透支
    pub allow_overdraft: bool,
}

impl Default for EconomyConfig {
    fn default() -> Self {
        Self {
            sell_price_ratio: 0.5,
            stolen_goods_ratio: 0.5,
            supply_demand_enabled: true,
            allow_overdraft: false,
        }
    }
}
