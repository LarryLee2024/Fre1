//! SkillSlot 更新系统
//!
//! 每帧同步 SkillSlotState 到子控件（ProgressBar、Button）的状态，
//! 以及从 UiStore.skill_panel 刷新 SkillSlot 数据。
//! 由 SkillSlotPlugin 注册到 Update 调度。

use bevy::prelude::*;

use crate::ui::binding::Dirty;
use crate::ui::primitives::button::components::ButtonState;
use crate::ui::primitives::progress_bar::components::ProgressBarState;
use crate::ui::view_models::{UiStore, skill_panel::SkillPanelVm};

use super::components::{SkillSlotNameLabel, SkillSlotState};

/// 技能槽更新系统
///
/// 每帧对每个 SkillSlot 实体：
/// 1. 读取 SkillSlotState（cooldown_current, cooldown_max, is_ready）
/// 2. 遍历子实体，更新 ProgressBar 的 current/maximum
/// 3. 遍历子实体，更新 Button 的 disabled 状态
///
/// 进度条的视觉更新由 progress_bar_update_system 负责（它读取
/// ProgressBarState 并更新填充条宽度和标签文本），本系统仅修改数值。
pub fn skill_slot_update_system(
    parent_query: Query<(&SkillSlotState, &Children)>,
    mut progress_bar_query: Query<&mut ProgressBarState>,
    mut button_query: Query<&mut ButtonState>,
) {
    for (slot_state, children) in parent_query.iter() {
        for child in children.iter() {
            if let Ok(mut pb_state) = progress_bar_query.get_mut(child) {
                pb_state.current = slot_state.cooldown_current as f32;
                pb_state.maximum = slot_state.cooldown_max as f32;
            }

            if let Ok(mut btn_state) = button_query.get_mut(child) {
                btn_state.disabled = !slot_state.is_ready;
            }
        }
    }
}

/// 从 UiStore.skill_panel 刷新 SkillSlot 的系统
///
/// 检测 Dirty<SkillPanelVm> 标记，当技能面板数据发生变化时，
/// 读取 UiStore 中对应的 SkillSlotVm 数据并更新到 SkillSlotState、
/// 子 Text 控件、ProgressBar 和 Button。
///
/// 技能匹配方式：遍历 UiStore.skill_panel.skills 的所有条目，
/// 按顺序（skill_id 升序）与 SkillSlot 实体一一对应。每个 SkillSlot
/// 实体的 skill_id 字段用于精确匹配。
pub fn refresh_skill_slot_from_vm(
    store: Res<UiStore>,
    mut slot_query: Query<(&mut Dirty<SkillPanelVm>, &mut SkillSlotState, &Children)>,
    mut text_query: Query<(&mut Text, &SkillSlotNameLabel)>,
    mut progress_bar_query: Query<&mut ProgressBarState>,
    mut button_query: Query<&mut ButtonState>,
) {
    for (mut dirty, mut state, children) in slot_query.iter_mut() {
        if !dirty.consume() {
            continue;
        }

        // 通过 skill_id 从 UiStore 查找匹配的 SkillSlotVm
        let vm = store.skill_panel.skills.get(&state.skill_id).or_else(|| {
            // 回退：如果 skill_id 为 0（工厂默认值），使用第一个条目
            if state.skill_id == 0 {
                store.skill_panel.skills.values().next()
            } else {
                None
            }
        });

        let Some(vm) = vm else {
            continue;
        };

        state.name = vm.name_key.to_string();
        state.cooldown_max = vm.max_cooldown;
        state.cooldown_current = vm.cooldown_remaining;
        state.is_ready = vm.is_usable;

        // 更新子级 Widget
        for child in children.iter() {
            // 更新技能名称文本
            if let Ok((mut text, _)) = text_query.get_mut(child) {
                text.0 = vm.name_key.to_string();
            }

            // 更新冷却进度条
            if let Ok(mut pb_state) = progress_bar_query.get_mut(child) {
                pb_state.current = vm.cooldown_remaining as f32;
                pb_state.maximum = vm.max_cooldown as f32;
            }

            // 更新按钮禁用状态
            if let Ok(mut btn_state) = button_query.get_mut(child) {
                btn_state.disabled = !vm.is_usable;
            }
        }
    }
}
