//! Economy Domain — 不变量测试
//!
//! 验证 docs/02-domain/domains/economy_domain.md §3 定义的不变量。

use crate::core::domains::economy::components::{CurrencyType, Price, Wallet};
use crate::core::domains::economy::rules::can_trade_with_reputation;

/// 不变量 3.1：货币非负 — 任意货币变更后，所有货币持有量 >= 0。
#[test]
fn wallet_balance_never_negative() {
    let mut wallet = Wallet::new();
    wallet.add(CurrencyType::Gold, 100);

    // deduct 超过余额 → 应返回 false 且余额不变化
    let price = Price::new(200);
    let result = wallet.deduct(&price);
    assert!(!result, "deduct should fail when insufficient");
    assert_eq!(
        *wallet.currencies.get(&CurrencyType::Gold).unwrap(),
        100,
        "balance unchanged"
    );
}

/// 不变量 3.1（退化测试）：空钱包 deduct 不应 panic 或产生负数。
#[test]
fn empty_wallet_deduct_does_not_panic() {
    let mut wallet = Wallet::new();
    let price = Price::new(1);
    assert!(!wallet.deduct(&price));
    // 余额隐式为 0（不存在 key），视为 non-negative
}

/// 不变量 3.3：价格计算确定性 — 相同条件价格一致。
#[test]
fn price_deterministic_repeatable() {
    let p1 = Price {
        base: 100,
        reputation_modifier: 0.9,
        supply_modifier: 1.5,
        stolen_modifier: 1.0,
    };
    let p2 = Price {
        base: 100,
        reputation_modifier: 0.9,
        supply_modifier: 1.5,
        stolen_modifier: 1.0,
    };
    assert_eq!(
        p1.final_price(),
        p2.final_price(),
        "same inputs must produce same price"
    );
}

/// 不变量 3.5：Hated 声望不可交易。
#[test]
fn hated_reputation_blocks_all_trade() {
    assert!(
        !can_trade_with_reputation("Hated"),
        "Hated faction cannot trade"
    );
}

/// 衍生不变量：Price 非零 base 产生非零价格。
#[test]
fn price_nonzero_base_produces_nonzero_price() {
    let p = Price::new(100);
    assert!(p.final_price() > 0, "price with nonzero base must be nonzero");

    let p2 = Price {
        base: 0,
        reputation_modifier: 1.0,
        supply_modifier: 1.0,
        stolen_modifier: 1.0,
    };
    assert_eq!(
        p2.final_price(),
        0,
        "price with zero base and neutral modifiers must be zero"
    );
}
