//! Economy Domain — 单元测试
//!
//! 验证规则纯函数和组件行为。

use crate::core::domains::economy::components::{
    CurrencyType, Price, ShopInstance, SupplyDemand, Wallet,
};
use crate::core::domains::economy::rules::{
    calc_buy_price, calc_sell_price, can_afford, can_trade_with_reputation,
    reputation_to_price_modifier, should_restock,
};

// ============================================================================
// Wallet
// ============================================================================

#[test]
fn wallet_starts_empty() {
    let wallet = Wallet::new();
    assert_eq!(wallet.currencies.get(&CurrencyType::Gold), None);
}

#[test]
fn wallet_can_add_currency() {
    let mut wallet = Wallet::new();
    wallet.add(CurrencyType::Gold, 100);
    assert_eq!(*wallet.currencies.get(&CurrencyType::Gold).unwrap(), 100);
}

#[test]
fn wallet_can_afford_sufficient_funds() {
    let mut wallet = Wallet::new();
    wallet.add(CurrencyType::Gold, 100);
    let price = Price::new(50);
    assert!(wallet.can_afford(&price));
}

#[test]
fn wallet_cannot_afford_insufficient_funds() {
    let mut wallet = Wallet::new();
    wallet.add(CurrencyType::Gold, 10);
    let price = Price::new(50);
    assert!(!wallet.can_afford(&price));
}

#[test]
fn wallet_deduct_reduces_funds() {
    let mut wallet = Wallet::new();
    wallet.add(CurrencyType::Gold, 100);
    let price = Price::new(30);
    assert!(wallet.deduct(&price));
    assert_eq!(*wallet.currencies.get(&CurrencyType::Gold).unwrap(), 70);
}

// ============================================================================
// Price
// ============================================================================

#[test]
fn price_default_is_base() {
    let price = Price::new(100);
    assert_eq!(price.final_price(), 100);
}

#[test]
fn price_applies_modifiers() {
    let price = Price {
        base: 100,
        reputation_modifier: 0.9,
        supply_modifier: 1.5,
        stolen_modifier: 1.0,
    };
    assert_eq!(price.final_price(), 135); // 100 * 0.9 * 1.5 = 135
}

// ============================================================================
// ShopInstance
// ============================================================================

#[test]
fn shop_has_stock_when_available() {
    let mut shop = ShopInstance::new("shp_test".into());
    shop.add_stock("itm_test", 5);
    assert!(shop.has_stock("itm_test"));
}

#[test]
fn shop_has_no_stock_when_empty() {
    let shop = ShopInstance::new("shp_test".into());
    assert!(!shop.has_stock("itm_test"));
}

#[test]
fn shop_infinite_stock_always_available() {
    let mut shop = ShopInstance::new("shp_test".into());
    shop.current_stock.insert("itm_infinite".into(), -1);
    assert!(shop.has_stock("itm_infinite"));
}

// ============================================================================
// Rules
// ============================================================================

#[test]
fn can_afford_with_sufficient_gold() {
    assert!(can_afford(100, 50, false));
}

#[test]
fn can_afford_fails_with_insufficient_gold() {
    assert!(!can_afford(10, 50, false));
}

#[test]
fn should_restock_after_interval() {
    assert!(should_restock(100, 10, 120));
}

#[test]
fn should_not_restock_before_interval() {
    assert!(!should_restock(100, 30, 120));
}

#[test]
fn neutral_reputation_no_discount() {
    assert_eq!(reputation_to_price_modifier("Neutral"), 1.0);
}

#[test]
fn hostile_reputation_doubles_price() {
    assert_eq!(reputation_to_price_modifier("Hostile"), 2.0);
}

#[test]
fn hated_reputation_blocks_trade() {
    assert!(!can_trade_with_reputation("Hated"));
}

#[test]
fn friendly_reputation_discount() {
    assert_eq!(reputation_to_price_modifier("Friendly"), 0.9);
}

#[test]
fn calc_buy_price_stolen_applies_discount() {
    let price = calc_buy_price(100, 1.0, SupplyDemand::Balanced, true, 0.5);
    assert_eq!(price.final_price(), 50); // 100 * 1.0 * 1.0 * 0.5
}

#[test]
fn calc_sell_price_applies_sell_ratio() {
    let price = calc_sell_price(100, 1.0, false, 0.5, 0.5);
    assert_eq!(price.final_price(), 50); // 100 * 1.0 * 0.5 * 1.0
}
