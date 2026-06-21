//! 战斗领域事件到 BattleHudVm / SkillPanelVm 的投影
//!
//! 纯函数，将战斗领域事件（BattleStarted、TurnStarted、
//! TurnEnded、EffectApplied）转换为 UiStore 上的 ViewModel 更新。这些函数无状态、
//! 确定性且可独立测试 — 不直接操作 ECS。
//!
//! 每个函数接收 `&mut UiStore` 和领域事件，执行投影逻辑后返回。
//! 本模块中的 Observer 包装器桥接 Bevy 的 Trigger<T> 事件系统与纯函数。
//!
//! 参见 `docs/06-ui/04-data-flow/projection-viewmodel.md` §4

use bevy::ecs::observer::On;
use bevy::prelude::*;

use crate::core::capabilities::effect::events::EffectApplied;
use crate::core::events::{BattleStarted, TurnEnded, TurnStarted};
use crate::ui::binding::Dirty;
use crate::ui::view_models::{UiStore, battle_hud::BattleHudVm, skill_panel::SkillPanelVm};

// ─── Pure Projection Functions ───────────────────────────────────────────

/// 战斗投影 — 战斗领域事件的无状态投影逻辑。
///
/// 所有方法都是纯函数，接收 `&mut UiStore` 和事件。
/// 无 ECS 依赖，无副作用，完全确定性。
pub struct BattleProjection;

impl BattleProjection {
    /// 将 `BattleStarted` 事件投影到 `UiStore.battle_hud`。
    ///
    /// 将回合计数器初始化为 1，并将阶段键设置为玩家阶段。
    pub fn on_battle_started(store: &mut UiStore, _event: &BattleStarted) {
        let hud = &mut store.battle_hud;
        hud.turn_number = 1;
        hud.phase_key = "ui.battle.phase.player";
        info!(target: "ui", "[BattleProjection] Battle started — HUD initialized");
    }

    /// 将 `TurnStarted` 事件投影到 `UiStore.battle_hud`。
    ///
    /// 递增回合计数器并将阶段键设置为玩家阶段
    /// （当前简化模型中回合开始后的第一个阶段）。
    pub fn on_turn_started(store: &mut UiStore, _event: &TurnStarted) {
        let hud = &mut store.battle_hud;
        hud.turn_number += 1;
        hud.phase_key = "ui.battle.phase.player";
    }

    /// 将 `TurnEnded` 事件投影到 `UiStore.battle_hud`。
    ///
    /// 将阶段键设置为敌方阶段，反映玩家的
    /// 活跃回合已结束。
    pub fn on_turn_ended(store: &mut UiStore, _event: &TurnEnded) {
        store.battle_hud.phase_key = "ui.battle.phase.enemy";
        info!(target: "ui", "[BattleProjection] Turn ended — phase set to enemy");
    }

    /// 将 `EffectApplied` 事件投影到 `UiStore.skill_panel`。
    ///
    /// 当前为占位符，仅记录事件。未来实现将匹配效果的 def_id 与已知技能
    /// 效果，并更新技能面板中的冷却状态。
    pub fn on_effect_applied(store: &mut UiStore, event: &EffectApplied) {
        // Placeholder: log effect application
        // TODO[P3][Projection][2026-06-21]: Implement skill cooldown update
        //   by matching event.def_id against UiStore.skill_panel skills
        //   and setting cooldown_remaining = max_cooldown for the matched skill.
        //   Completion criteria: EffectApplied with a matching def_id
        //   marks the corresponding SkillSlotVm's cooldown_remaining = max_cooldown.
        let _ = store; // Placeholder until real logic is implemented
        info!(
            target: "ui",
            "[BattleProjection] Effect applied: def_id={}, target={}",
            event.def_id,
            event.target_entity,
        );
    }
}

// ─── Observer Systems (ECS bridge) ───────────────────────────────────────

/// Observer：监听 `BattleStarted` 领域事件并通过
/// `BattleProjection::on_battle_started` 将其投影到 `UiStore.battle_hud`。
pub fn on_battle_started_projection(
    trigger: On<BattleStarted>,
    mut store: ResMut<UiStore>,
    mut dirty_query: Query<&mut Dirty<BattleHudVm>>,
) {
    BattleProjection::on_battle_started(&mut store, trigger.event());

    for mut dirty in dirty_query.iter_mut() {
        dirty.mark_dirty();
    }
}

/// Observer：监听 `TurnStarted` 领域事件并通过
/// `BattleProjection::on_turn_started` 将其投影到 `UiStore.battle_hud`。
///
/// 同时标记所有 `Dirty<BattleHudVm>` 组件为脏，以便消费此 ViewModel 的 Widget
/// 在下一帧刷新。
pub fn on_turn_started_projection(
    trigger: On<TurnStarted>,
    mut store: ResMut<UiStore>,
    mut dirty_query: Query<&mut Dirty<BattleHudVm>>,
) {
    BattleProjection::on_turn_started(&mut store, trigger.event());

    // Mark all BattleHudVm consumers dirty
    for mut dirty in dirty_query.iter_mut() {
        dirty.mark_dirty();
    }
}

/// Observer：监听 `TurnEnded` 领域事件并通过
/// `BattleProjection::on_turn_ended` 将其投影到 `UiStore.battle_hud`。
pub fn on_turn_ended_projection(
    trigger: On<TurnEnded>,
    mut store: ResMut<UiStore>,
    mut dirty_query: Query<&mut Dirty<BattleHudVm>>,
) {
    BattleProjection::on_turn_ended(&mut store, trigger.event());

    for mut dirty in dirty_query.iter_mut() {
        dirty.mark_dirty();
    }
}

/// Observer：监听 `EffectApplied` 领域事件并通过
/// `BattleProjection::on_effect_applied` 将其投影到 `UiStore.skill_panel`。
///
/// 同时标记所有 `Dirty<SkillPanelVm>` 组件为脏，以便技能
/// 槽位 Widget 在下一帧刷新。
pub fn on_effect_applied_projection(
    trigger: On<EffectApplied>,
    mut store: ResMut<UiStore>,
    mut dirty_query: Query<&mut Dirty<SkillPanelVm>>,
) {
    BattleProjection::on_effect_applied(&mut store, trigger.event());

    // Mark all SkillPanelVm consumers dirty
    for mut dirty in dirty_query.iter_mut() {
        dirty.mark_dirty();
    }
}
