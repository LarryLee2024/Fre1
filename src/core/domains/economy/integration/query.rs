//! EconomyQueryParam — Bevy SystemParam，封装所有 Economy 域组件查询。
//!
//! Systems 通过此 param 读取经济数据，完全不知道 `Wallet` /
//! `ShopInstance` 组件的存在细节。
//!
//! # 用法
//!
//! ```rust,ignore
//! fn my_system(
//!     economy_query: EconomyQueryParam,
//!     // ...
//! ) {
//!     if let Some(wallet) = economy_query.get_wallet(entity) {
//!         // 读取钱包数据
//!     }
//! }
//! ```
//!
//! # 设计决策
//!
//! - 只提供只读查询——可变操作通过 `EconomyWriteFacade` 完成
//! - 不包装 `Commands`——调用方传入以保持语义清晰

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::domains::economy::components::{CurrencyType, ShopInstance, SupplyDemand, Wallet};

/// 经济查询 SystemParam — 封装所有 Economy 域组件查询。
///
/// System 签名中使用此类型替代裸 `Query<&Wallet>` + `Query<&ShopInstance>`。
#[derive(SystemParam)]
pub struct EconomyQueryParam<'w, 's> {
    /// 钱包只读查询
    wallet_query: Query<'w, 's, &'static Wallet>,
    /// 商店实例只读查询
    shop_query: Query<'w, 's, &'static ShopInstance>,
}

impl<'w, 's> EconomyQueryParam<'w, 's> {
    /// 获取实体的钱包。
    ///
    /// # Returns
    /// - `Some(&Wallet)` — 如果实体拥有 `Wallet` 组件
    /// - `None` — 如果实体不存在或无该组件
    pub fn get_wallet(&self, entity: Entity) -> Option<&Wallet> {
        self.wallet_query.get(entity).ok()
    }

    /// 检查实体是否拥有钱包组件。
    pub fn has_wallet(&self, entity: Entity) -> bool {
        self.wallet_query.get(entity).is_ok()
    }

    /// 获取实体指定货币的持有量。
    ///
    /// # Returns
    /// - 该货币的持有量（若无该货币则返回 0）
    pub fn get_currency_amount(&self, entity: Entity, currency: &CurrencyType) -> u64 {
        self.wallet_query
            .get(entity)
            .ok()
            .and_then(|wallet| wallet.currencies.get(currency).copied())
            .unwrap_or(0)
    }

    /// 获取实体的商店实例。
    ///
    /// # Returns
    /// - `Some(&ShopInstance)` — 如果实体拥有 `ShopInstance` 组件
    /// - `None` — 如果实体不存在或无该组件
    pub fn get_shop_instance(&self, entity: Entity) -> Option<&ShopInstance> {
        self.shop_query.get(entity).ok()
    }

    /// 检查商店是否有指定物品的库存（不限数量）。
    pub fn shop_has_stock(&self, entity: Entity, item_id: &str) -> bool {
        self.shop_query
            .get(entity)
            .ok()
            .is_some_and(|shop| shop.has_stock(item_id))
    }

    /// 检查商店指定物品是否有足够库存。
    pub fn shop_has_enough_stock(&self, entity: Entity, item_id: &str, quantity: u32) -> bool {
        self.shop_query
            .get(entity)
            .ok()
            .is_some_and(|shop| shop.has_enough_stock(item_id, quantity))
    }

    /// 获取指定物品的供需系数。
    ///
    /// # Returns
    /// - `Some(SupplyDemand)` — 如果该物品有供需记录
    /// - `None` — 无供需记录或实体无商店组件
    pub fn get_supply_demand(&self, entity: Entity, item_id: &str) -> Option<SupplyDemand> {
        self.shop_query
            .get(entity)
            .ok()
            .and_then(|shop| shop.supply_demand.get(item_id).copied())
    }
}
