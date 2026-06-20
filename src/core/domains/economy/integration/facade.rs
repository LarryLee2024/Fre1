//! EconomyReadFacade + EconomyWriteFacade — Economy 域组件读写入口。
//!
//! # ReadFacade — 只读查询 API
//!
//! 通过 `&World` 提供对 Economy 域 ECS 组件的不可变访问。
//! 所有方法为静态函数，可在任何能访问 `&World` 的地方使用：
//! - Bevy Systems 中通过 `system_param` 获取 `&World`
//! - 测试代码中直接使用
//!
//! # WriteFacade — 可变操作 API
//!
//! 提供对 Economy 域组件的修改操作，使用两种方式：
//! - `&mut World` 方法：立即执行，适合独占 System / 测试
//! - `Commands` 方法：延迟执行，适合常规 System
//!
//! # 设计
//!
//! - 所有方法不发射事件（Event）——事件发射由调用方（System）负责
//! - WriteFacade 仅执行原始数据变更，不含业务校验逻辑
//! - 校验应在调用 WriteFacade 之前通过 domain rules 完成

use bevy::prelude::*;

use crate::core::domains::economy::components::{CurrencyType, ShopInstance, SupplyDemand, Wallet};
use crate::core::domains::economy::resources::EconomyConfig;

// ─── EconomyReadFacade ─────────────────────────────────────────────

/// ReadFacade — 只读查询 API
///
/// 提供对 Economy 域 ECS 组件的只读访问。
/// 所有方法通过 `&World` 查询组件，不包含业务逻辑。
pub struct EconomyReadFacade;

impl EconomyReadFacade {
    /// 获取实体的钱包。
    ///
    /// # Returns
    /// - `Some(&Wallet)` — 如果实体拥有 `Wallet` 组件
    /// - `None` — 如果实体不存在或无该组件
    ///
    /// # ReadFacade: 安全查询钱包
    pub fn get_wallet(world: &World, entity: Entity) -> Option<&Wallet> {
        world.get::<Wallet>(entity)
    }

    /// 检查实体是否拥有钱包组件。
    ///
    /// # ReadFacade: 检查钱包存在
    pub fn has_wallet(world: &World, entity: Entity) -> bool {
        world.get::<Wallet>(entity).is_some()
    }

    /// 获取实体指定货币的持有量。
    ///
    /// # Returns
    /// - 该货币的持有量（若无该货币则返回 0）
    ///
    /// # ReadFacade: 查询货币持有量
    pub fn get_currency_amount(world: &World, entity: Entity, currency: &CurrencyType) -> u64 {
        world
            .get::<Wallet>(entity)
            .and_then(|wallet| wallet.currencies.get(currency).copied())
            .unwrap_or(0)
    }

    /// 获取实体的商店实例。
    ///
    /// # Returns
    /// - `Some(&ShopInstance)` — 如果实体拥有 `ShopInstance` 组件
    /// - `None` — 如果实体不存在或无该组件
    ///
    /// # ReadFacade: 安全查询商店实例
    pub fn get_shop_instance(world: &World, entity: Entity) -> Option<&ShopInstance> {
        world.get::<ShopInstance>(entity)
    }

    /// 检查商店是否有指定物品的库存（不限数量）。
    ///
    /// # ReadFacade: 检查库存存在
    pub fn shop_has_stock(world: &World, entity: Entity, item_id: &str) -> bool {
        world
            .get::<ShopInstance>(entity)
            .is_some_and(|shop| shop.has_stock(item_id))
    }

    /// 检查商店指定物品是否有足够库存。
    ///
    /// # ReadFacade: 检查库存充足
    pub fn shop_has_enough_stock(
        world: &World,
        entity: Entity,
        item_id: &str,
        quantity: u32,
    ) -> bool {
        world
            .get::<ShopInstance>(entity)
            .is_some_and(|shop| shop.has_enough_stock(item_id, quantity))
    }

    /// 获取指定物品的供需系数。
    ///
    /// # Returns
    /// - `Some(&SupplyDemand)` — 如果该物品有供需记录
    /// - `None` — 无供需记录或实体无商店组件
    ///
    /// # ReadFacade: 安全查询供需系数
    pub fn get_supply_demand(world: &World, entity: Entity, item_id: &str) -> Option<SupplyDemand> {
        world
            .get::<ShopInstance>(entity)
            .and_then(|shop| shop.supply_demand.get(item_id).copied())
    }

    /// 获取经济系统全局配置。
    ///
    /// # ReadFacade: 安全查询全局配置
    pub fn get_economy_config(world: &World) -> &EconomyConfig {
        world.resource::<EconomyConfig>()
    }
}

// ─── EconomyWriteFacade ─────────────────────────────────────────────

/// WriteFacade — 可变操作 API
///
/// 提供对 Economy 域 ECS 组件的修改操作。
/// 不包含业务校验——校验应在调用前通过 domain rules 完成。
pub struct EconomyWriteFacade;

impl EconomyWriteFacade {
    // ── &mut World 方法（立即执行） ─────────────────────────────────

    /// 设置实体指定货币的持有量（覆盖写入）。
    ///
    /// # WriteFacade: 立即设置货币量
    pub fn set_currency_amount(
        world: &mut World,
        entity: Entity,
        currency: CurrencyType,
        amount: u64,
    ) {
        if let Some(mut wallet) = world.get_mut::<Wallet>(entity) {
            wallet.currencies.insert(currency, amount);
        }
    }

    /// 增加实体指定货币的持有量。
    ///
    /// # WriteFacade: 立即增加货币量
    pub fn add_currency(world: &mut World, entity: Entity, currency: CurrencyType, amount: u64) {
        if let Some(mut wallet) = world.get_mut::<Wallet>(entity) {
            wallet.add(currency, amount);
        }
    }

    /// 从商店移除指定数量的库存。
    ///
    /// # WriteFacade: 立即移除库存
    pub fn remove_stock(world: &mut World, entity: Entity, item_id: &str, quantity: u32) {
        if let Some(mut shop) = world.get_mut::<ShopInstance>(entity) {
            shop.remove_stock(item_id, quantity);
        }
    }

    /// 为商店添加指定数量的库存。
    ///
    /// # WriteFacade: 立即添加库存
    pub fn add_stock(world: &mut World, entity: Entity, item_id: &str, quantity: u32) {
        if let Some(mut shop) = world.get_mut::<ShopInstance>(entity) {
            shop.add_stock(item_id, quantity);
        }
    }

    /// 设置指定物品的供需系数。
    ///
    /// # WriteFacade: 立即设置供需系数
    pub fn set_supply_demand(
        world: &mut World,
        entity: Entity,
        item_id: String,
        supply_demand: SupplyDemand,
    ) {
        if let Some(mut shop) = world.get_mut::<ShopInstance>(entity) {
            shop.supply_demand.insert(item_id, supply_demand);
        }
    }

    // ── Commands 方法（延迟执行） ────────────────────────────────────

    /// 为实体插入钱包组件（通过 Commands 延迟执行）。
    ///
    /// # WriteFacade: 通过 Commands 插入钱包
    pub fn insert_wallet(commands: &mut Commands, entity: Entity, wallet: Wallet) {
        commands.entity(entity).insert(wallet);
    }

    /// 为实体插入商店实例组件（通过 Commands 延迟执行）。
    ///
    /// # WriteFacade: 通过 Commands 插入商店实例
    pub fn insert_shop_instance(commands: &mut Commands, entity: Entity, instance: ShopInstance) {
        commands.entity(entity).insert(instance);
    }

    /// 移除实体的钱包组件（通过 Commands 延迟执行）。
    ///
    /// # WriteFacade: 通过 Commands 移除钱包
    pub fn remove_wallet(commands: &mut Commands, entity: Entity) {
        commands.entity(entity).remove::<Wallet>();
    }

    /// 移除实体的商店实例组件（通过 Commands 延迟执行）。
    ///
    /// # WriteFacade: 通过 Commands 移除商店实例
    pub fn remove_shop_instance(commands: &mut Commands, entity: Entity) {
        commands.entity(entity).remove::<ShopInstance>();
    }
}
