//! BattleScreen Zone Visibility — driven by BattleHudVm ViewModel
//!
//! Data source: UiStore (ViewModel firewall), no direct domain state queries.
//! Visibility rules per ui-layout-system-plan.md §5:
//!
//! | Zone | PlayerPhase | EnemyPhase |
//! |------|-------------|------------|
//! | Z1 (TurnIndicator) | ✅ Visible | ✅ Visible |
//! | Z2 (PhaseText) | ✅ Visible | ✅ Visible |
//! | Z3 (UnitSummary) [P2] | ✅ Visible | ✅ Visible |
//! | Z5 (CharacterCard) | ✅ if selected [P1] | 🔲 Hidden |
//! | Z6 (ActionMenu) | ✅ **Visible** | 🔲 **Hidden** |
//! | Z7 (EndTurn+SkillPanel) | ✅ Visible | 🔲 Hidden |
//! | Z8 (TurnOrderBar) [P2] | ✅ Visible | ✅ Visible |
//!
//! SkillPanel visibility is independently toggled inside Z7 via
//! `skill_panel_visibility_system`: it is visible only when
//! `skill_panel_open && player_controlled`. This allows the SkillPanel
//! to be hidden while EndTurnButton remains visible during player turn.
//!
//! mark_battle_ui_passthrough — 给所有 BattleScreen 后代的 UI 节点加 Pickable::IGNORE
//! 解决 bevy_ui::UiPickingPlugin 拦截 sprite picking 的问题。

use bevy::prelude::*;

use crate::ui::view_models::UiStore;

use super::SkillPanelToggle;
use super::layout::BattleZone;

/// Zone visibility system — runs in Update
///
/// Rules:
/// - Z1 (TurnIndicator): Always visible
/// - Z2 (PhaseText): Always visible
/// - Z3 (UnitSummary): Always visible (empty until P2)
/// - Z5 (CharacterCard): Visible during BattlePhase::Battle AND a unit is selected [P1]
/// - Z6 (ActionMenu): Visible during BattlePhase::Battle (player's controllable turn)
/// - Z7 (EndTurn button + SkillPanel): Visible during BattlePhase::Battle.
///   SkillPanel has independent visibility toggle via `skill_panel_visibility_system`.
/// - Z8 (TurnOrderBar): Always visible (empty until P2)
pub fn battle_zone_visibility_system(
    store: Res<UiStore>,
    zone_query: Query<(Entity, &BattleZone)>,
    mut visibility_query: Query<&mut Visibility>,
) {
    let in_battle = store.battle_hud.is_in_battle;
    let player_controlled = store.battle_hud.is_player_controlled;
    let game_over = store.battle_hud.is_game_over;

    for (entity, zone) in zone_query.iter() {
        let visible = if game_over {
            // Game over: only Z2 shows the result text, all other zones hidden
            matches!(zone, BattleZone::Z2TopCenter)
        } else if in_battle {
            match zone {
                // Z1-Z3: Always visible in battle
                BattleZone::Z1TopLeft | BattleZone::Z2TopCenter | BattleZone::Z3TopRight => true,

                // Z5: CharacterCard — visible when a unit is selected (current_unit_id != 0)
                BattleZone::Z5BottomLeft => store.battle_hud.current_unit_id != 0,

                // Z6: ActionMenu — player's controllable turn only
                BattleZone::Z6BottomCenter => player_controlled,

                // Z7: SkillPanel + EndTurn — player's controllable turn only
                BattleZone::Z7BottomRight => player_controlled,

                // Z8: TurnOrderBar — always visible in battle (empty until P2)
                BattleZone::Z8BottomBar => true,
            }
        } else {
            false
        };

        if let Ok(mut vis) = visibility_query.get_mut(entity) {
            *vis = if visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

/// SkillPanel visibility system — runs in Update
///
/// Controls SkillPanel's individual visibility within Z7.
/// SkillPanel is visible only when `skill_panel_open` is true AND
/// the player is in control (enemy phase hides everything via Z7 zone).
///
/// This allows the EndTurnButton (also in Z7) to remain visible during
/// player turn even when the SkillPanel is closed.
pub fn skill_panel_visibility_system(
    store: Res<UiStore>,
    mut query: Query<&mut Visibility, With<SkillPanelToggle>>,
) {
    let visible = store.battle_hud.skill_panel_open && store.battle_hud.is_player_controlled;
    for mut vis in query.iter_mut() {
        *vis = if visible {
            // Inherited — follows Z7 zone visibility (Visible during player turn)
            Visibility::Inherited
        } else {
            // Explicitly hidden — overrides Z7's visibility for this child
            Visibility::Hidden
        };
    }
}

/// 给所有 BattleScreen 后代的 UI 节点加 Pickable::IGNORE
///
/// 解决 bevy_ui::UiPickingPlugin 默认拦截 sprite picking 的问题。
/// 无 Pickable 组件的 UI 节点默认 `should_block_lower: true`，会阻断棋子 Sprite 的点击。
///
/// 首次进入 Combat 时运行一次（由 Local<bool> 控制），
/// 在 on_selection_changed 之前执行，确保点击事件能穿透 UI 到达 Sprite。
pub fn mark_battle_ui_passthrough(
    mut commands: Commands,
    screen_query: Query<Entity, With<super::BattleScreen>>,
    ui_node_query: Query<Entity, (With<Node>, Without<Pickable>)>,
    child_of: Query<&ChildOf>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }

    for root in screen_query.iter() {
        for ui_entity in ui_node_query.iter() {
            if child_of.iter_ancestors(ui_entity).any(|a| a == root) {
                commands.entity(ui_entity).insert(Pickable::IGNORE);
            }
        }
    }

    *done = true;
}
