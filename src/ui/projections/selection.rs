//! Selection Projection — 事件驱动的选择状态 → ViewModel / Visual / Camera 映射
//!
//! 消费 UnitClicked / SelectionCleared 事件，联动：
//! - UiStore（CharacterCard / Battle HUD / ActionMenu / SkillPanel）
//! - Sprite 高亮（选中色 / 队伍色）
//! - Camera 跟随（CameraRequest）
//!
//! 详见 ADR-068 §Phase 2–3。

use bevy::prelude::*;

use crate::core::domains::combat::components::{CombatParticipant, HitPoints, UnitIdComponent};
use crate::core::domains::tactical::components::GridPos;
use crate::infra::camera::foundation::request::CameraRequest;
use crate::infra::camera::foundation::target::CameraTarget;
use crate::ui::binding::Dirty;
use crate::ui::selection::{SelectionCleared, UnitClicked};
use crate::ui::view_models::{
    UiStore,
    battle_hud::BattleHudVm,
    character_panel::CharacterPanelVm,
    skill_panel::{SkillPanelVm, SkillSlotVm},
};
use crate::ui::widgets::action_menu::components::ActionMenuState;

// ─── 颜色常量 ────────────────────────────────────────────────────────

/// 玩家单位颜色
const PLAYER_COLOR: Color = Color::srgb(0.2, 0.5, 0.9);
/// 敌方单位颜色
const ENEMY_COLOR: Color = Color::srgb(0.9, 0.2, 0.2);
/// 其他单位颜色
const NEUTRAL_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);
/// 选中高亮颜色（青色）
const SELECTED_COLOR: Color = Color::srgb(0.2, 1.0, 1.0);

/// 网格单元大小（与 render.rs 保持一致）
const CELL_SIZE: f32 = 80.0;

// ─── 投影：UiStore 更新 ──────────────────────────────────────────────

/// Observer：监听 `UnitClicked` 事件，更新 UiStore 的 ViewModel。
///
/// 从 UnitClicked 中提取 unit_id，查找匹配的 ECS 实体，读取领域组件数据，
/// 写入 UiStore.character_panel / battle_hud / skill_panel，并标记 Dirty。
pub fn on_unit_clicked_projection(
    trigger: On<UnitClicked>,
    mut store: ResMut<UiStore>,
    unit_ids: Query<(Entity, &UnitIdComponent, &HitPoints)>,
    mut panel_dirty: Query<&mut Dirty<CharacterPanelVm>>,
    mut hud_dirty: Query<&mut Dirty<BattleHudVm>>,
    mut skill_dirty: Query<&mut Dirty<SkillPanelVm>>,
    mut action_menu_query: Query<&mut ActionMenuState>,
) {
    let unit_id = &trigger.event().unit_id;

    // 按 UnitIdComponent.id 查找匹配实体
    let Some((entity, uid, hp)) = unit_ids
        .iter()
        .find(|(_, uid_component, _)| uid_component.id == *unit_id)
    else {
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

    // ── 自动打开 SkillPanel ──
    // 选中单位时显示其技能面板，方便玩家查看和使用技能。
    store.battle_hud.skill_panel_open = true;

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

/// Observer：监听 `SelectionCleared` 事件，重置行动菜单状态，
/// 并清除当前单位 ID 以隐藏 CharacterCard（Z5）。
pub fn on_selection_cleared_projection(
    _trigger: On<SelectionCleared>,
    mut store: ResMut<UiStore>,
    mut action_menu_query: Query<&mut ActionMenuState>,
    mut dirty_query: Query<&mut Dirty<BattleHudVm>>,
) {
    // 清除当前单位 ID → Z5 CharacterCard 隐藏
    store.battle_hud.current_unit_id = 0;
    // 清除选中 → Z7 SkillPanel 隐藏
    store.battle_hud.skill_panel_open = false;

    for mut menu in action_menu_query.iter_mut() {
        menu.actions.iter_mut().for_each(|a| a.enabled = true);
    }

    for mut dirty in dirty_query.iter_mut() {
        dirty.mark_dirty();
    }
}

// ─── 投影：Camera 跟随 ──────────────────────────────────────────────

/// Observer：监听 `UnitClicked` 事件，通过 CameraRequest::MoveTo 驱动摄像机。
///
/// 替代旧的 `camera_follow_selection` 系统（直接写 TargetPose 的违规实现）。
pub fn on_unit_selected_follow(
    trigger: On<UnitClicked>,
    unit_ids: Query<(Entity, &UnitIdComponent)>,
    grid_positions: Query<&GridPos>,
    mut commands: Commands,
) {
    let unit_id = &trigger.event().unit_id;

    // 按 ID 查找实体
    let Some(entity) = unit_ids
        .iter()
        .find(|(_, uid)| uid.id == *unit_id)
        .map(|(e, _)| e)
    else {
        return;
    };

    // 获取网格位置
    let Ok(grid_pos) = grid_positions.get(entity) else {
        return;
    };

    // 网格坐标 → 世界坐标（与 render.rs 公式一致）
    let world_x = grid_pos.x as f32 * CELL_SIZE + CELL_SIZE / 2.0;
    let world_y = grid_pos.y as f32 * CELL_SIZE + CELL_SIZE / 2.0;

    commands.trigger(CameraRequest::MoveTo {
        target: CameraTarget::WorldPos(Vec2::new(world_x, world_y)),
        duration: 0.3,
    });

    info!(
        target: "camera",
        "[Camera] Following unit '{}' to grid ({}, {}) → world ({:.1}, {:.1})",
        unit_id,
        grid_pos.x,
        grid_pos.y,
        world_x,
        world_y,
    );
}

// ─── 投影：视觉高亮 ──────────────────────────────────────────────────

/// Observer：监听 `UnitClicked` 事件，设置选中单位高亮色。
///
/// 重置所有单位为队伍色，然后将选中单位设为青色高亮。
pub fn on_unit_selected_highlight(
    trigger: On<UnitClicked>,
    mut sprites: Query<&mut Sprite>,
    unit_ids: Query<(Entity, &UnitIdComponent, &CombatParticipant)>,
) {
    let unit_id = &trigger.event().unit_id;

    for (entity, uid, participant) in &unit_ids {
        if let Ok(mut sprite) = sprites.get_mut(entity) {
            if uid.id == *unit_id {
                // 选中单位 → 青色高亮
                sprite.color = SELECTED_COLOR;
            } else {
                // 其他单位 → 恢复队伍色
                sprite.color = match participant.team_id.as_str() {
                    "Player" => PLAYER_COLOR,
                    "Enemy" => ENEMY_COLOR,
                    _ => NEUTRAL_COLOR,
                };
            }
        }
    }
}

/// Observer：监听 `SelectionCleared` 事件，恢复所有单位为队伍色。
pub fn on_selection_cleared_highlight(
    _trigger: On<SelectionCleared>,
    mut sprites: Query<&mut Sprite>,
    unit_ids: Query<(Entity, &UnitIdComponent, &CombatParticipant)>,
) {
    for (entity, _, participant) in &unit_ids {
        if let Ok(mut sprite) = sprites.get_mut(entity) {
            sprite.color = match participant.team_id.as_str() {
                "Player" => PLAYER_COLOR,
                "Enemy" => ENEMY_COLOR,
                _ => NEUTRAL_COLOR,
            };
        }
    }
}
