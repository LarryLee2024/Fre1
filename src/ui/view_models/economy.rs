//! EconomyVm — 经济视图模型
//!
//! 详见 `docs/06-ui/04-data-flow/projection-viewmodel.md` §3

use bevy::prelude::*;

/// 经济视图模型
#[derive(Resource, Clone, Reflect, Default)]
#[reflect(Resource)]
pub struct EconomyVm {
    /// 玩家持有金币
    pub player_gold: u32,
}
