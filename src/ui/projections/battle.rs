//! Module Name: BattleProjection — Domain Event to BattleHudVm / SkillPanelVm projection
//!
//! Pure functions that transform battle-domain events (BattleStarted, TurnStarted,
//! TurnEnded, EffectApplied) into ViewModel updates on the UiStore.  These functions are stateless,
//! deterministic, and independently testable — they never touch ECS directly.
//!
//! Each function takes `&mut UiStore` and the domain event, performs the
//! projection logic, and returns.  Observer wrappers in this module bridge
//! between Bevy's Trigger<T> event system and the pure functions.
//!
//! See `docs/06-ui/04-data-flow/projection-viewmodel.md` §4

use bevy::ecs::observer::On;
use bevy::prelude::*;

use crate::core::capabilities::effect::events::EffectApplied;
use crate::core::events::{BattleStarted, TurnEnded, TurnStarted};
use crate::ui::binding::Dirty;
use crate::ui::view_models::{UiStore, battle_hud::BattleHudVm, skill_panel::SkillPanelVm};

// ─── Pure Projection Functions ───────────────────────────────────────────

/// BattleProjection — stateless projection logic for battle-domain events.
///
/// All methods are pure functions taking `&mut UiStore` and the event.
/// No ECS dependencies, no side effects, fully deterministc.
pub struct BattleProjection;

impl BattleProjection {
    /// Projects a `BattleStarted` event onto `UiStore.battle_hud`.
    ///
    /// Initializes turn counter to 1 and sets the phase key to the player phase.
    pub fn on_battle_started(store: &mut UiStore, _event: &BattleStarted) {
        let hud = &mut store.battle_hud;
        hud.turn_number = 1;
        hud.phase_key = "ui.battle.phase.player";
        info!(target: "ui", "[BattleProjection] Battle started — HUD initialized");
    }

    /// Projects a `TurnStarted` event onto `UiStore.battle_hud`.
    ///
    /// Increments turn counter and sets the phase key to the player phase
    /// (the first phase after a turn starts in the current simplified model).
    pub fn on_turn_started(store: &mut UiStore, _event: &TurnStarted) {
        let hud = &mut store.battle_hud;
        hud.turn_number += 1;
        hud.phase_key = "ui.battle.phase.player";
    }

    /// Projects a `TurnEnded` event onto `UiStore.battle_hud`.
    ///
    /// Sets the phase key to the enemy phase to reflect that the player's
    /// active turn has concluded.
    pub fn on_turn_ended(store: &mut UiStore, _event: &TurnEnded) {
        store.battle_hud.phase_key = "ui.battle.phase.enemy";
        info!(target: "ui", "[BattleProjection] Turn ended — phase set to enemy");
    }

    /// Projects an `EffectApplied` event onto `UiStore.skill_panel`.
    ///
    /// Currently a no-op placeholder that logs the event.  Future
    /// implementation will match the effect's def_id against known skill
    /// effects and update cooldown state in the skill panel.
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

/// Observer: listens for `BattleStarted` domain events and projects them into
/// `UiStore.battle_hud` via `BattleProjection::on_battle_started`.
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

/// Observer: listens for `TurnStarted` domain events and projects them into
/// `UiStore.battle_hud` via `BattleProjection::on_turn_started`.
///
/// Also marks all `Dirty<BattleHudVm>` components as dirty so that widgets
/// consuming this ViewModel will refresh on the next frame.
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

/// Observer: listens for `TurnEnded` domain events and projects them into
/// `UiStore.battle_hud` via `BattleProjection::on_turn_ended`.
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

/// Observer: listens for `EffectApplied` domain events and projects them into
/// `UiStore.skill_panel` via `BattleProjection::on_effect_applied`.
///
/// Also marks all `Dirty<SkillPanelVm>` components as dirty so that skill
/// slot widgets will refresh on the next frame.
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
