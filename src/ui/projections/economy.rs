//! 经济领域事件到 ViewModel 的投影
//!
//! 将经济领域事件（CurrencyChanged）转换为 UiStore 上的 EconomyVm 更新。
//! 这些函数无状态、确定性且可独立测试 — 不直接操作 ECS。
//!
//! 每个函数接收 `&mut UiStore` 和领域事件，执行投影逻辑后返回。
//! 本模块中的 Observer 包装器桥接 Bevy 的 Trigger<T> 事件系统与纯函数。
//!
//! TODO[P3][Economy][2026-06-21]: PurchaseCompleted/SellCompleted 领域事件就绪后
//!   补全 on_purchase_complete / on_sell_complete 的详细信息（物品种类、数量等）。
//!   当前仅通过 CurrencyChanged 事件跟踪金币变化。
//!
//! 参见 `docs/06-ui/04-data-flow/projection-viewmodel.md` §4

use bevy::ecs::observer::On;
use bevy::prelude::*;

use crate::core::domains::economy::CurrencyChanged;
use crate::ui::binding::Dirty;
use crate::ui::view_models::{UiStore, economy::EconomyVm};

// ─── 纯投影函数 ─────────────────────────────────────────────────────────

/// 经济投影 — 经济领域事件的无状态投影逻辑。
///
/// 所有方法都是纯函数，接收 `&mut UiStore` 和事件。
/// 无 ECS 依赖，无副作用，完全确定性。
pub struct EconomyProjection;

impl EconomyProjection {
    /// 将 `CurrencyChanged` 事件投影到 `UiStore.economy`。
    ///
    /// 仅处理 `currency_type == "Gold"` 的金币变更，
    /// 将 `new_amount` 同步到 `EconomyVm.player_gold`。
    pub fn on_currency_changed(store: &mut UiStore, event: &CurrencyChanged) {
        if event.currency_type != "Gold" {
            return;
        }
        store.economy.player_gold = event.new_amount as u32;
        info!(
            target: "ui",
            "[EconomyProjection] Gold changed: {} -> {} (delta: {}, reason: {})",
            event.old_amount,
            event.new_amount,
            event.delta,
            event.reason,
        );
    }

    /// 购买交易完成后的投影更新
    ///
    /// 当 `PurchaseCompleted` 领域事件就绪后，从事件中提取 item_def_id、
    /// quantity 和 total_cost，更新 `UiStore` 中的金币显示和物品列表。
    ///
    /// TODO[P3][Economy][2026-06-21]: 更新金币显示和物品列表
    ///   - 需要 PurchaseCompleted 领域事件
    ///
    ///   Completion criteria: 购买事件到来时更新 UiStore 中的金币和库存 VM
    pub fn on_purchase_complete(
        store: &mut UiStore,
        item_def_id: &str,
        quantity: u32,
        total_cost: u64,
    ) {
        info!(
            target: "ui",
            "[EconomyProjection] Purchase: {} x{} for {} gold",
            item_def_id, quantity, total_cost,
        );
        let _ = store;
    }

    /// 出售交易完成后的投影更新
    ///
    /// TODO[P3][Economy][2026-06-21]: 更新金币显示和物品列表
    ///   - 需要 SellCompleted 领域事件
    ///
    ///   Completion criteria: 出售事件到来时更新 UiStore 中的金币和库存 VM
    pub fn on_sell_complete(
        store: &mut UiStore,
        item_def_id: &str,
        quantity: u32,
        total_revenue: u64,
    ) {
        info!(
            target: "ui",
            "[EconomyProjection] Sold: {} x{} for {} gold",
            item_def_id, quantity, total_revenue,
        );
        let _ = store;
    }
}

// ─── Observer Systems (ECS bridge) ───────────────────────────────────────

/// Observer：监听 `CurrencyChanged` 领域事件并通过
/// `EconomyProjection::on_currency_changed` 将其投影到 `UiStore.economy`。
///
/// 仅处理 `currency_type == "Gold"` 的金币变更，将新余额同步到
/// `EconomyVm.player_gold`，并标记所有 `Dirty<EconomyVm>` 组件为脏，
/// 以便消费此 ViewModel 的 Widget 在下一帧刷新。
pub fn on_currency_changed_projection(
    trigger: On<CurrencyChanged>,
    mut store: ResMut<UiStore>,
    mut dirty_query: Query<&mut Dirty<EconomyVm>>,
) {
    let event = trigger.event();

    // 仅处理金币变更，忽略其他货币类型
    if event.currency_type != "Gold" {
        return;
    }

    EconomyProjection::on_currency_changed(&mut store, event);

    for mut dirty in dirty_query.iter_mut() {
        dirty.mark_dirty();
    }
}
