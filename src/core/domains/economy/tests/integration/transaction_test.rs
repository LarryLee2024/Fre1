//! Economy Domain — 交易流程集成测试
//!
//! 验证 EconomyPlugin 注册后的交易 Observer 链路：
//! - 购买流程：钱包扣款、余额不足拒绝
//! - 出售流程：钱包收款

use bevy::prelude::*;

use crate::core::domains::economy::components::{CurrencyType, Wallet};
use crate::core::domains::economy::events::{
    PriceBreakdown, TransactionCompleted, TransactionType,
};
use crate::core::domains::economy::plugin::EconomyPlugin;

// ─── 辅助函数 ──────────────────────────────────────────────────────

fn spawn_wallet(world: &mut World, gold: u64) -> Entity {
    let mut wallet = Wallet::new();
    wallet.add(CurrencyType::Gold, gold);
    world.spawn(wallet).id()
}

fn get_gold(world: &World, entity: Entity) -> u64 {
    world
        .get::<Wallet>(entity)
        .and_then(|w| w.currencies.get(&CurrencyType::Gold).copied())
        .unwrap_or(0)
}

// ─── 购买流程 ──────────────────────────────────────────────────────

#[test]
fn buy_deducts_correct_amount() {
    let mut app = App::new();
    app.add_plugins(EconomyPlugin);

    let buyer = spawn_wallet(app.world_mut(), 100);
    app.world_mut().flush();

    app.world_mut().trigger(TransactionCompleted {
        entity: buyer,
        shop_id: "shop_potion".into(),
        item_id: "potion_hp".into(),
        total_price: 50,
        price_breakdown: PriceBreakdown {
            base_price: 50,
            reputation_modifier: 1.0,
            supply_modifier: 1.0,
            stolen_modifier: 0.0,
            final_price: 50,
        },
        transaction_type: TransactionType::Buy,
        quantity: 1,
    });
    app.world_mut().flush();

    assert_eq!(
        get_gold(app.world_mut(), buyer),
        50,
        "购买 50 金币物品后应剩余 50"
    );
}

#[test]
fn buy_insufficient_funds_leaves_wallet_unchanged() {
    let mut app = App::new();
    app.add_plugins(EconomyPlugin);

    let buyer = spawn_wallet(app.world_mut(), 30);
    app.world_mut().flush();

    app.world_mut().trigger(TransactionCompleted {
        entity: buyer,
        shop_id: "shop_potion".into(),
        item_id: "potion_hp".into(),
        total_price: 50,
        price_breakdown: PriceBreakdown {
            base_price: 50,
            reputation_modifier: 1.0,
            supply_modifier: 1.0,
            stolen_modifier: 0.0,
            final_price: 50,
        },
        transaction_type: TransactionType::Buy,
        quantity: 1,
    });
    app.world_mut().flush();

    assert_eq!(
        get_gold(app.world_mut(), buyer),
        30,
        "余额不足时钱包不应变动"
    );
}

#[test]
fn buy_exact_amount_empties_wallet() {
    let mut app = App::new();
    app.add_plugins(EconomyPlugin);

    let buyer = spawn_wallet(app.world_mut(), 50);
    app.world_mut().flush();

    app.world_mut().trigger(TransactionCompleted {
        entity: buyer,
        shop_id: "shop_potion".into(),
        item_id: "potion_hp".into(),
        total_price: 50,
        price_breakdown: PriceBreakdown {
            base_price: 50,
            reputation_modifier: 1.0,
            supply_modifier: 1.0,
            stolen_modifier: 0.0,
            final_price: 50,
        },
        transaction_type: TransactionType::Buy,
        quantity: 1,
    });
    app.world_mut().flush();

    assert_eq!(get_gold(app.world_mut(), buyer), 0, "恰好花光所有金币");
}

#[test]
fn sell_adds_correct_amount() {
    let mut app = App::new();
    app.add_plugins(EconomyPlugin);

    let seller = spawn_wallet(app.world_mut(), 0);
    app.world_mut().flush();

    app.world_mut().trigger(TransactionCompleted {
        entity: seller,
        shop_id: "shop_blacksmith".into(),
        item_id: "iron_sword".into(),
        total_price: 25,
        price_breakdown: PriceBreakdown {
            base_price: 25,
            reputation_modifier: 1.0,
            supply_modifier: 1.0,
            stolen_modifier: 0.0,
            final_price: 25,
        },
        transaction_type: TransactionType::Sell,
        quantity: 1,
    });
    app.world_mut().flush();

    assert_eq!(
        get_gold(app.world_mut(), seller),
        25,
        "出售物品后应收到 25 金币"
    );
}

#[test]
fn multiple_buys_accumulate_deductions() {
    let mut app = App::new();
    app.add_plugins(EconomyPlugin);

    let buyer = spawn_wallet(app.world_mut(), 100);
    app.world_mut().flush();

    // 两次购买
    for _ in 0..2 {
        app.world_mut().trigger(TransactionCompleted {
            entity: buyer,
            shop_id: "shop_potion".into(),
            item_id: "potion_hp".into(),
            total_price: 30,
            price_breakdown: PriceBreakdown {
                base_price: 30,
                reputation_modifier: 1.0,
                supply_modifier: 1.0,
                stolen_modifier: 0.0,
                final_price: 30,
            },
            transaction_type: TransactionType::Buy,
            quantity: 1,
        });
    }
    app.world_mut().flush();

    assert_eq!(
        get_gold(app.world_mut(), buyer),
        40,
        "两次购买 30 金币物品后应剩余 40"
    );
}

#[test]
fn buy_sell_cycle_net_correct() {
    let mut app = App::new();
    app.add_plugins(EconomyPlugin);

    let trader = spawn_wallet(app.world_mut(), 100);
    app.world_mut().flush();

    // 买 花 30
    app.world_mut().trigger(TransactionCompleted {
        entity: trader,
        shop_id: "shop_potion".into(),
        item_id: "potion_hp".into(),
        total_price: 30,
        price_breakdown: PriceBreakdown {
            base_price: 30,
            reputation_modifier: 1.0,
            supply_modifier: 1.0,
            stolen_modifier: 0.0,
            final_price: 30,
        },
        transaction_type: TransactionType::Buy,
        quantity: 1,
    });
    app.world_mut().flush();

    // 卖 得 15
    app.world_mut().trigger(TransactionCompleted {
        entity: trader,
        shop_id: "shop_blacksmith".into(),
        item_id: "iron_sword".into(),
        total_price: 15,
        price_breakdown: PriceBreakdown {
            base_price: 15,
            reputation_modifier: 1.0,
            supply_modifier: 1.0,
            stolen_modifier: 0.0,
            final_price: 15,
        },
        transaction_type: TransactionType::Sell,
        quantity: 1,
    });
    app.world_mut().flush();

    assert_eq!(
        get_gold(app.world_mut(), trader),
        85,
        "买 30 卖 15 后 100 → 85"
    );
}
