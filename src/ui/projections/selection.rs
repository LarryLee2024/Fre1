//! Selection Projection — 选择状态 → ViewModel / Visual / Camera 映射
//!
//! 监听 `Selection` 资源变化，联动：
//! - UiStore（CharacterCard / Battle HUD / ActionMenu / SkillPanel）
//! - Sprite 高亮（选中色）
//! - Camera 跟随（TargetPose）
//!
//! 详见 ADR-067 §Phase 2–3。

use bevy::prelude::*;

use crate::core::domains::combat::components::{CombatParticipant, HitPoints, UnitIdComponent};
use crate::core::domains::tactical::components::GridPos;
use crate::infra::camera::foundation::pose::{CameraPose, TargetPose};
use crate::infra::picking::selection::Selection;
use crate::ui::binding::Dirty;
use crate::ui::view_models::{
    UiStore,
    battle_hud::BattleHudVm,
    character_panel::CharacterPanelVm,
    skill_panel::{SkillPanelVm, SkillSlotVm},
};
use crate::ui::widgets::action_menu::components::{ActionMenuItem, ActionMenuState, ActionType};

/// 选中高亮颜色
const SELECTED_COLOR: Color = Color::srgb(0.2, 1.0, 1.0);

/// 网格单元大小（与 render.rs 保持一致）
const CELL_SIZE: f32 = 80.0;

/// Selection 变更投影系统 — UiStore / ActionMenu / SkillPanel
///
/// 每帧检测 `Selection` 是否变更。当玩家选中新单位时：
/// 1. 更新 UiStore.character_panel → 驱动 CharacterCard 刷新
/// 2. 更新 UiStore.battle_hud → 驱动 HUD 刷新
/// 3. 更新 ActionMenuState → 启用/禁用行动项
/// 4. 更新 UiStore.skill_panel → 更新技能显示
/// 5. 标记所有相关 Dirty 标记
pub fn on_selection_changed(
    selection: Res<Selection>,
    mut store: ResMut<UiStore>,
    unit_query: Query<(&UnitIdComponent, &HitPoints)>,
    mut panel_dirty: Query<&mut Dirty<CharacterPanelVm>>,
    mut hud_dirty: Query<&mut Dirty<BattleHudVm>>,
    mut skill_dirty: Query<&mut Dirty<SkillPanelVm>>,
    mut action_menu_query: Query<&mut ActionMenuState>,
) {
    if !selection.is_changed() {
        return;
    }

    let Some(entity) = selection.selected_unit else {
        // 选中被清除 → 重置行动菜单为默认状态
        for mut menu in action_menu_query.iter_mut() {
            menu.actions.iter_mut().for_each(|a| a.enabled = true);
        }
        return;
    };

    let Ok((uid, hp)) = unit_query.get(entity) else {
        return;
    };

    // ── 更新 CharacterPanelVm（驱动 CharacterCard）──
    store.character_panel.character_id = entity.to_bits() as u32;
    store.character_panel.name_key = uid.id.clone();
    store.character_panel.level = 1;
    store.character_panel.hp = hp.current as f32;
    store.character_panel.max_hp = hp.maximum as f32;
    store.character_panel.mp = 50.0;
    store.character_panel.max_mp = 50.0;

    // ── 更新 BattleHudVm（驱动 HUD 血条/蓝条）──
    store.battle_hud.hp = hp.current as f32;
    store.battle_hud.max_hp = hp.maximum as f32;
    store.battle_hud.mp = 50.0;
    store.battle_hud.max_mp = 50.0;
    store.battle_hud.ap = 1.0;
    store.battle_hud.max_ap = 1.0;

    // ── 更新 SkillPanelVm──
    // 为选中单位填充默认技能。后续由 domain SkillSlots 组件驱动。
    let mut skills = std::collections::HashMap::new();
    skills.insert(
        1,
        SkillSlotVm {
            skill_id: 1,
            name_key: "ui.skill.attack",
            cooldown_remaining: 0,
            max_cooldown: 0,
            is_usable: true,
            ap_cost: 1,
        },
    );
    skills.insert(
        2,
        SkillSlotVm {
            skill_id: 2,
            name_key: "ui.skill.fireball",
            cooldown_remaining: 0,
            max_cooldown: 3,
            is_usable: true,
            ap_cost: 2,
        },
    );
    skills.insert(
        3,
        SkillSlotVm {
            skill_id: 3,
            name_key: "ui.skill.heal",
            cooldown_remaining: 0,
            max_cooldown: 2,
            is_usable: true,
            ap_cost: 1,
        },
    );
    store.skill_panel = SkillPanelVm { skills };

    // ── 更新 ActionMenuState（根据选中单位启用/禁用行动）──
    for mut menu in action_menu_query.iter_mut() {
        // MVP：选中单位后所有行动可用。后续根据领域状态（AP、状态效果等）过滤。
        menu.actions.iter_mut().for_each(|a| a.enabled = true);
    }

    // ── 标记 Dirty ──
    for mut d in &mut panel_dirty {
        d.mark_dirty();
    }
    for mut d in &mut hud_dirty {
        d.mark_dirty();
    }
    for mut d in &mut skill_dirty {
        d.mark_dirty();
    }

    info!(
        target: "ui",
        "[Selection] Projected '{}' → CharacterCard + SkillPanel + ActionMenu (HP: {}/{})",
        uid.id,
        hp.current,
        hp.maximum,
    );
}

/// 选中高亮视觉系统
///
/// 检测 Selection 变更，恢复上一个单位的队伍色，
/// 为新选中的单位应用选中高亮色（青色）。
pub fn on_selection_visual(
    selection: Res<Selection>,
    mut sprites: Query<&mut Sprite>,
    participants: Query<&CombatParticipant>,
    mut prev: Local<Option<Entity>>,
) {
    if !selection.is_changed() {
        return;
    }

    // 恢复上一个选中单位的队伍色
    if let Some(prev_entity) = prev.take() {
        if let Ok(participant) = participants.get(prev_entity) {
            if let Ok(mut sprite) = sprites.get_mut(prev_entity) {
                sprite.color = match participant.team_id.as_str() {
                    "Player" => Color::srgb(0.2, 0.5, 0.9),
                    "Enemy" => Color::srgb(0.9, 0.2, 0.2),
                    _ => Color::srgb(0.5, 0.5, 0.5),
                };
            }
        }
    }

    // 为新选中单位应用高亮色
    if let Some(entity) = selection.selected_unit {
        if let Ok(mut sprite) = sprites.get_mut(entity) {
            sprite.color = SELECTED_COLOR;
        }
        *prev = Some(entity);
    }
}

/// Camera 跟随选中单位
///
/// 检测 Selection 变更，将 Camera 的 TargetPose 设置到选中单位的网格位置。
/// 现有的 lerp 插值系统自动处理平滑移动。
pub fn camera_follow_selection(
    selection: Res<Selection>,
    grid_positions: Query<&GridPos>,
    mut camera_query: Query<&mut TargetPose, With<crate::infra::camera::components::MainCamera>>,
) {
    if !selection.is_changed() {
        return;
    }

    let Some(entity) = selection.selected_unit else {
        return;
    };
    let Ok(grid_pos) = grid_positions.get(entity) else {
        return;
    };
    let Ok(mut target) = camera_query.single_mut() else {
        return;
    };

    // 网格坐标 → 世界坐标（与 render.rs 公式一致）
    let world_x = grid_pos.x as f32 * CELL_SIZE + CELL_SIZE / 2.0;
    let world_y = grid_pos.y as f32 * CELL_SIZE + CELL_SIZE / 2.0;

    target.0.position = Vec2::new(world_x, world_y);

    info!(
        target: "camera",
        "[Camera] Following selection to grid ({}, {}) → world ({:.1}, {:.1})",
        grid_pos.x, grid_pos.y, world_x, world_y,
    );
}
