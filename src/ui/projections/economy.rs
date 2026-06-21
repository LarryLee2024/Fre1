//! 经济领域事件到 ViewModel 的投影骨架
//!
//! 纯函数骨架，将经济领域事件（物品购买、出售等）转换为 UiStore 上的
//! ViewModel 更新。这些函数无状态、确定性且可独立测试。
//!
//! TODO[P3][Economy][2026-06-21]: 当 EconomyViewModel 和领域事件定义完成后补全
//!   - 创建 EconomyVm 结构（gold, items 等字段）
//!   - 添加到 UiStore
//!   - 创建 EconomyProjection::on_purchase_complete / on_sell_complete
//!   - 添加 Observer 包装器并注册到 plugin.rs
//!   Completion criteria: 购买/出售事件到来时更新 UiStore 中的金币和库存 VM
//!
//! 参见 `docs/06-ui/04-data-flow/projection-viewmodel.md` §4

// TODO[P3][Economy][2026-06-21]: 当经济领域事件就绪时，添加对应的 Observer 系统
//   - on_purchase_complete_projection(trigger, store, dirty)
//   - on_sell_complete_projection(trigger, store, dirty)
//   - 注册到 plugin.rs 的 projections 部分

use bevy::prelude::*;

use crate::ui::view_models::UiStore;

/// 经济投影 — 经济领域事件的无状态投影逻辑。
///
/// 所有方法都是纯函数，接收 `&mut UiStore` 和事件。
/// 无 ECS 依赖，无副作用，完全确定性。
pub struct EconomyProjection;

impl EconomyProjection {
    /// 购买交易完成后的投影更新
    ///
    /// TODO[P3][Economy][2026-06-21]: 更新金币显示和物品列表
    ///   - 需要一个 EconomyVm（含 gold 字段）添加到 UiStore
    ///   - 需要 PurchaseCompleted 领域事件
    ///   Completion criteria: 购买事件到来时更新 UiStore 中的金币和库存 VM
    pub fn on_purchase_complete(store: &mut UiStore) {
        // Placeholder: no-op until ViewModel is defined
        let _ = store;
        info!(
            target: "ui",
            "[EconomyProjection] Purchase complete — stub, no-op"
        );
    }

    /// 出售交易完成后的投影更新
    ///
    /// TODO[P3][Economy][2026-06-21]: 更新金币显示和物品列表
    ///   - 需要一个 EconomyVm（含 gold 字段）添加到 UiStore
    ///   - 需要 SellCompleted 领域事件
    ///   Completion criteria: 出售事件到来时更新 UiStore 中的金币和库存 VM
    pub fn on_sell_complete(store: &mut UiStore) {
        // Placeholder: no-op until ViewModel is defined
        let _ = store;
        info!(
            target: "ui",
            "[EconomyProjection] Sell complete — stub, no-op"
        );
    }
}
