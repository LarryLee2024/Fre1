//! 经济/交易领域 — 组件定义
//!
//! 详见 docs/02-domain/domains/economy_domain.md
//! Schema: docs/04-data/domains/economy_schema.md

use crate::shared::localization_key::LocalizationKey;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── 货币类型 ───────────────────────────────────────────────────

/// 货币类型枚举。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum CurrencyType {
    Gold,            // 金币 GP
    Silver,          // 银币 SP (1 GP = 10 SP)
    Copper,          // 铜币 CP (1 SP = 10 CP)
    Special(String), // 特殊货币
}

impl CurrencyType {
    /// 获取货币的换算基数（以铜币为单位）。
    pub fn base_value(&self) -> u64 {
        match self {
            CurrencyType::Gold => 100,
            CurrencyType::Silver => 10,
            CurrencyType::Copper => 1,
            CurrencyType::Special(_) => 1,
        }
    }
}

// ─── 钱包 ───────────────────────────────────────────────────────

/// 角色的钱包——持有所有货币类型的数量。
#[derive(Debug, Clone, Component, Reflect)]
pub struct Wallet {
    /// 各货币类型的持有量
    pub currencies: HashMap<CurrencyType, u64>,
}

impl Wallet {
    pub fn new() -> Self {
        Self {
            currencies: HashMap::new(),
        }
    }

    /// 检查是否能支付给定价格（支持多货币换算）。
    pub fn can_afford(&self, price: &Price) -> bool {
        let cost_in_copper = price.final_price() * CurrencyType::Gold.base_value();
        let total_copper: u64 = self
            .currencies
            .iter()
            .map(|(currency, amount)| amount * currency.base_value())
            .sum();
        total_copper >= cost_in_copper
    }

    /// 扣款（优先扣除金币，不足时按换算比例扣其他货币）。返回是否成功。
    pub fn deduct(&mut self, price: &Price) -> bool {
        if !self.can_afford(price) {
            return false;
        }

        let mut remaining = price.final_price() * CurrencyType::Gold.base_value();

        // 按优先级扣款：金币 > 银币 > 铜币
        let priority = [
            CurrencyType::Gold,
            CurrencyType::Silver,
            CurrencyType::Copper,
        ];

        for currency in &priority {
            if remaining == 0 {
                break;
            }
            let base = currency.base_value();
            let amount = self.currencies.get(currency).copied().unwrap_or(0);
            let can_use = amount * base;
            if can_use >= remaining {
                // 向上取整确保扣款彻底
                let deduct_amount = remaining.div_ceil(base);
                if let Some(balance) = self.currencies.get_mut(currency) {
                    *balance -= deduct_amount;
                }
                remaining = 0;
            } else {
                if let Some(balance) = self.currencies.get_mut(currency) {
                    *balance = 0;
                }
                remaining -= can_use;
            }
        }

        // 处理特殊货币
        if remaining > 0 {
            for (currency, amount) in self.currencies.iter_mut() {
                if matches!(currency, CurrencyType::Special(_)) {
                    let base = currency.base_value();
                    let can_use = *amount * base;
                    if can_use >= remaining {
                        // 向上取整确保扣款彻底
                        let deduct_amount = remaining.div_ceil(base);
                        *amount -= deduct_amount;
                        remaining = 0;
                        break;
                    } else {
                        *amount = 0;
                        remaining -= can_use;
                    }
                }
            }
        }

        remaining == 0
    }

    /// 增加指定货币数量。
    pub fn add(&mut self, currency: CurrencyType, amount: u64) {
        *self.currencies.entry(currency).or_insert(0) += amount;
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}

// ─── 价格对象 ──────────────────────────────────────────────────

/// 价格值对象——封装价格计算逻辑。
#[derive(Debug, Clone, Reflect)]
pub struct Price {
    pub base: u64,
    pub reputation_modifier: f32,
    pub supply_modifier: f32,
    pub stolen_modifier: f32,
}

impl Price {
    pub fn new(base: u64) -> Self {
        Self {
            base,
            reputation_modifier: 1.0,
            supply_modifier: 1.0,
            stolen_modifier: 1.0,
        }
    }

    /// 计算最终价格。
    pub fn final_price(&self) -> u64 {
        (self.base as f32 * self.reputation_modifier * self.supply_modifier * self.stolen_modifier)
            as u64
    }
}

// ─── 供需系数 ──────────────────────────────────────────────────

/// 供需系数枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum SupplyDemand {
    Surplus,  // × 0.8
    Balanced, // × 1.0
    Scarce,   // × 1.5
    Shortage, // × 2.0
}

impl SupplyDemand {
    pub fn modifier(&self) -> f32 {
        match self {
            SupplyDemand::Surplus => 0.8,
            SupplyDemand::Balanced => 1.0,
            SupplyDemand::Scarce => 1.5,
            SupplyDemand::Shortage => 2.0,
        }
    }
}

// ─── 补货策略 ──────────────────────────────────────────────────

/// 补货策略。
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub enum RestockPolicy {
    Timed { interval_hours: u32 },
    OnVisit { full_restock: bool },
    Never,
}

// ─── 商店条目定义 ──────────────────────────────────────────────

/// 商店商品条目定义。
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct ShopEntryDef {
    /// 物品模板 ID
    pub item_id: String,
    /// 基础价格覆盖（None 则使用 ItemDef 默认）
    pub base_price: Option<u64>,
    /// 初始库存数量（-1 = 无限）
    pub initial_stock: i32,
    /// 每次补货恢复的数量
    pub restock_amount: u32,
    /// 是否收购赃物
    pub buys_stolen: bool,
}

// ─── 商店定义 ──────────────────────────────────────────────────

/// 商店定义。内容团队配置，运行时只读。
#[derive(Debug, Clone, Reflect, Asset, Serialize, Deserialize)]
pub struct ShopDef {
    pub id: String,
    #[reflect(ignore)]
    pub name_key: LocalizationKey,
    pub faction_id: String,
    pub inventory: Vec<ShopEntryDef>,
    pub restock_policy: RestockPolicy,
}

// ─── 商店实例 ──────────────────────────────────────────────────

/// 商店运行时实例。
#[derive(Debug, Clone, Component, Reflect)]
pub struct ShopInstance {
    pub shop_def_id: String,
    pub current_stock: HashMap<String, i32>,
    pub supply_demand: HashMap<String, SupplyDemand>,
    pub last_restock: u64,
}

impl ShopInstance {
    pub fn new(shop_def_id: String) -> Self {
        Self {
            shop_def_id,
            current_stock: HashMap::new(),
            supply_demand: HashMap::new(),
            last_restock: 0,
        }
    }

    /// 检查指定物品是否有库存。
    pub fn has_stock(&self, item_id: &str) -> bool {
        self.current_stock
            .get(item_id)
            .map(|&qty| qty > 0 || qty == -1)
            .unwrap_or(false)
    }

    /// 检查指定物品是否有足够库存。
    pub fn has_enough_stock(&self, item_id: &str, quantity: u32) -> bool {
        match self.current_stock.get(item_id) {
            Some(-1) => true, // 无限
            Some(&qty) => qty >= quantity as i32,
            None => false,
        }
    }

    /// 移除库存。
    pub fn remove_stock(&mut self, item_id: &str, quantity: u32) {
        if let Some(stock) = self.current_stock.get_mut(item_id)
            && *stock > 0
        {
            *stock = stock.saturating_sub(quantity as i32);
        }
    }

    /// 添加库存。
    pub fn add_stock(&mut self, item_id: &str, quantity: u32) {
        *self.current_stock.entry(item_id.to_string()).or_insert(0) += quantity as i32;
    }
}

// ─── 声望折扣系数 ──────────────────────────────────────────────

/// 声望等级对应的价格折扣系数。
pub fn reputation_discount(reputation_level: &str) -> f32 {
    match reputation_level {
        "Hostile" => 2.0,
        "Neutral" => 1.0,
        "Friendly" => 0.9,
        "Honored" => 0.8,
        "Revered" => 0.7,
        _ => 1.0,
    }
}
