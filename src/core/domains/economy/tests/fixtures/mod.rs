//! Economy Domain — 测试辅助
//!
//! 提供 Builder 模式和标准测试数据。

use crate::core::domains::economy::components::{CurrencyType, Price, Wallet};

/// 创建一个有指定金币的钱包。
pub fn wallet_with_gold(amount: u64) -> Wallet {
    let mut w = Wallet::new();
    w.add(CurrencyType::Gold, amount);
    w
}

/// 创建一个标准价格。
pub fn price(gold: u64) -> Price {
    Price {
        base: gold,
        reputation_modifier: 1.0,
        supply_modifier: 1.0,
        stolen_modifier: 1.0,
    }
}

/// 创建带有折扣的价格。
pub fn discounted_price(base: u64, reputation: f32, supply: f32) -> Price {
    Price {
        base,
        reputation_modifier: reputation,
        supply_modifier: supply,
        stolen_modifier: 1.0,
    }
}
