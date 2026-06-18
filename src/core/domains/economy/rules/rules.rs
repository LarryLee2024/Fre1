//! 经济/交易业务规则 — 纯函数
//!
//! 包括定价、供需、赃物折扣等规则。
//! 详见 docs/02-domain/domains/economy_domain.md §5

use super::super::components::{Price, ShopInstance, SupplyDemand};

// ─── 价格计算规则 ──────────────────────────────────────────────

/// 计算购买价格。
///
/// 公式：基础价 × 声望折扣 × 供需系数 × 赃物折扣
pub fn calc_buy_price(
    base_price: u64,
    reputation_modifier: f32,
    supply_demand: SupplyDemand,
    is_stolen: bool,
    sell_price_ratio: f32,
) -> Price {
    let stolen_modifier = if is_stolen { sell_price_ratio } else { 1.0 };
    Price {
        base: base_price,
        reputation_modifier,
        supply_modifier: supply_demand.modifier(),
        stolen_modifier,
    }
}

/// 计算出售价格。
///
/// 公式：基础价 × 售价折扣系数(0.5) × 声望折扣 × 赃物折扣
pub fn calc_sell_price(
    base_price: u64,
    reputation_modifier: f32,
    is_stolen: bool,
    sell_price_ratio: f32,
    stolen_goods_ratio: f32,
) -> Price {
    let stolen_modifier = if is_stolen { stolen_goods_ratio } else { 1.0 };
    Price {
        base: base_price,
        reputation_modifier,
        supply_modifier: sell_price_ratio,
        stolen_modifier,
    }
}

// ─── 库存规则 ──────────────────────────────────────────────────

/// 检查商店是否有足够库存。
///
/// 不变量 3.2：交易物存在性。
pub fn check_stock_availability(shop: &ShopInstance, item_id: &str, quantity: u32) -> bool {
    shop.has_enough_stock(item_id, quantity)
}

/// 更新供需系数。
///
/// 根据库存比例更新供需状态。
pub fn update_supply_demand(shop: &mut ShopInstance, item_id: &str, initial_stock: i32) {
    let current = shop.current_stock.get(item_id).copied().unwrap_or(0);
    if initial_stock <= 0 || current < 0 {
        return; // 无限库存或异常
    }
    let ratio = current as f32 / initial_stock as f32;
    let new_state = if ratio >= 0.75 {
        SupplyDemand::Surplus
    } else if ratio >= 0.4 {
        SupplyDemand::Balanced
    } else if ratio >= 0.15 {
        SupplyDemand::Scarce
    } else {
        SupplyDemand::Shortage
    };
    shop.supply_demand.insert(item_id.to_string(), new_state);
}

// ─── 补货规则 ──────────────────────────────────────────────────

/// 检查补货周期是否已到。
///
/// 不变量 3.4：商店补货周期。
pub fn should_restock(last_restock: u64, interval_hours: u32, current_time: u64) -> bool {
    current_time >= last_restock + interval_hours as u64
}

/// 计算补货数量。
pub fn calc_restock_amount(initial_stock: i32, current_stock: i32, restock_amount: u32) -> u32 {
    if initial_stock == -1 {
        return 0; // 无限库存不补货
    }
    if current_stock >= initial_stock {
        return 0; // 已满
    }
    let deficit = initial_stock - current_stock;
    deficit.min(restock_amount as i32) as u32
}

// ─── 货币规则 ──────────────────────────────────────────────────

/// 检查钱包是否有足够货币。
///
/// 不变量 3.1：货币非负。不允许透支。
pub fn can_afford(gold: u64, cost: u64, allow_overdraft: bool) -> bool {
    if allow_overdraft { true } else { gold >= cost }
}

/// 检查赃物标记。
///
/// 不变量 3.5：赃物标记不可清除。
pub fn is_stolen_goods(stolen_flag: bool) -> bool {
    stolen_flag // 由外部系统标记，本领域只读取不修改
}

/// 声望折扣映射。
pub fn reputation_to_price_modifier(reputation_level: &str) -> f32 {
    match reputation_level {
        "Hostile" => 2.0,
        "Hated" => f32::MAX, // 不交易（用极大值表示不可交易）
        "Neutral" => 1.0,
        "Friendly" => 0.9,
        "Honored" => 0.8,
        "Revered" => 0.7,
        _ => 1.0,
    }
}

/// 检查声望是否允许交易。
pub fn can_trade_with_reputation(reputation_level: &str) -> bool {
    reputation_level != "Hated"
}
