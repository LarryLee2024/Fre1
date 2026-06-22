//! ShopPanelVm — 商店面板视图模型
//!
//! 详见 `docs/06-ui/04-data-flow/projection-viewmodel.md` §3

use bevy::prelude::*;

/// 商店面板视图模型
#[derive(Resource, Clone, Reflect, Default)]
#[reflect(Resource)]
pub struct ShopPanelVm {
    /// 玩家持有金币
    pub gold: u32,
    /// 当前激活标签页
    pub active_tab: ShopTab,
}

/// 商店标签页枚举
#[derive(Clone, Reflect, Default, PartialEq, Eq)]
pub enum ShopTab {
    #[default]
    /// 购买标签页
    Buy,
    /// 出售标签页
    Sell,
}
