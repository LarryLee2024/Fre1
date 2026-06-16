//! 背包领域事件

use crate::shared::ids::{ItemId, UnitId};
use bevy::prelude::*;

/// 物品已使用
#[derive(Message, Debug, Clone)]
pub struct ItemUsed {
    pub user: UnitId,
    pub user_name: String,
    pub item_id: ItemId,
    pub target: Option<UnitId>,
}

/// 物品已转移
#[derive(Message, Debug, Clone)]
pub struct ItemTransferred {
    pub item_id: ItemId,
    pub amount: u32,
    pub from_container: String,
    pub to_container: String,
}
