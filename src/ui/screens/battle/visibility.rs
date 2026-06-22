//! BattleScreen Zone Visibility — controlled by UiStore.battle_hud.is_in_battle
//!
//! Data source: UiStore (ViewModel firewall), no direct domain state queries.
//! Zone visibility is driven by the `is_in_battle` projection field set on
//! BattleStarted and cleared on screen despawn.
//!
//! mark_battle_ui_passthrough — 给所有 BattleScreen 后代的 UI 节点加 Pickable::IGNORE
//! 解决 bevy_ui::UiPickingPlugin 拦截 sprite picking 的问题。

use bevy::prelude::*;

use crate::ui::view_models::UiStore;

use super::layout::BattleZone;

/// Zone visibility system — runs in Update
///
/// Rules:
/// - Z1 (TurnIndicator): Always visible
/// - Z2 (PhaseText): Always visible
/// - Z3 (UnitSummary): Always visible (empty until P2)
/// - Z5 (CharacterCard): Visible during BattlePhase::Battle AND a unit is selected [P1]
/// - Z6 (ActionMenu): Visible during BattlePhase::Battle (player's controllable turn)
/// - Z7 (EndTurn button + SkillPanel): Visible during BattlePhase::Battle
/// - Z8 (TurnOrderBar): Always visible (empty until P2)
pub fn battle_zone_visibility_system(
    store: Res<UiStore>,
    zone_query: Query<(Entity, &BattleZone)>,
    mut visibility_query: Query<&mut Visibility>,
) {
    let in_battle = store.battle_hud.is_in_battle;

    for (entity, zone) in zone_query.iter() {
        let visible = match zone {
            BattleZone::Z1TopLeft | BattleZone::Z2TopCenter | BattleZone::Z3TopRight => true,
            // TODO[P1][UI][2026-07-21]: Also check unit selection state
            BattleZone::Z5BottomLeft => in_battle,
            BattleZone::Z6BottomCenter => in_battle,
            BattleZone::Z7BottomRight => in_battle,
            BattleZone::Z8BottomBar => true,
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
