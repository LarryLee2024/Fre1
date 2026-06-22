//! CharacterCard 更新系统
//!
//! 每帧同步 CharacterCardState 到子控件（ProgressBar）的状态，
//! 以及从 UiStore.character_panel 刷新 CharacterCard 数据。
//! 由 CharacterCardPlugin 注册到 Update 调度。

use bevy::prelude::*;

use crate::ui::binding::Dirty;
use crate::ui::primitives::progress_bar::components::ProgressBarState;
use crate::ui::view_models::{UiStore, character_panel::CharacterPanelVm};

use super::components::{CharacterCardLevelLabel, CharacterCardNameLabel, CharacterCardState};

/// 角色卡片更新系统
///
/// 每帧对每个 CharacterCard 实体：
/// 1. 读取 CharacterCardState（hp_current, hp_max, mp_current, mp_max）
/// 2. 遍历子实体，更新 HP ProgressBar 和 MP ProgressBar 的 current/maximum
///
/// 进度条的视觉更新由 progress_bar_update_system 负责（它读取
/// ProgressBarState 并更新填充条宽度和标签文本），本系统仅修改数值。
pub fn character_card_update_system(
    parent_query: Query<(&CharacterCardState, &Children), Changed<CharacterCardState>>,
    mut progress_bar_query: Query<&mut ProgressBarState>,
) {
    for (card_state, children) in parent_query.iter() {
        for child in children.iter() {
            if let Ok(mut pb_state) = progress_bar_query.get_mut(child) {
                // 根据进度条变体路由值
                match pb_state.variant {
                    crate::ui::primitives::progress_bar::components::ProgressBarVariant::Hp => {
                        pb_state.current = card_state.hp_current;
                        pb_state.maximum = card_state.hp_max;
                    }
                    crate::ui::primitives::progress_bar::components::ProgressBarVariant::Mp => {
                        pb_state.current = card_state.mp_current;
                        pb_state.maximum = card_state.mp_max;
                    }
                    _ => {
                        // 非 HP/MP 进度条不由此系统更新
                    }
                }
            }
        }
    }
}

/// 从 UiStore.character_panel 刷新 CharacterCard 的系统
///
/// 检测 Dirty<CharacterPanelVm> 标记，当 UiStore 中角色面板数据发生
/// 变化时，读取最新数据并更新到 CharacterCardState 和子 Text 控件。
/// 子 ProgressBar 的更新由 character_card_update_system 自动处理
/// （检测到 CharacterCardState 变化后传播）。
///
/// 这是一个轮询系统，通过 consume() 保证每帧最多消费一次 dirty 标记。
pub fn refresh_character_card_from_vm(
    store: Res<UiStore>,
    mut card_query: Query<(
        &mut Dirty<CharacterPanelVm>,
        &mut CharacterCardState,
        &Children,
    )>,
    mut text_query: Query<(&mut Text, &CharacterCardNameLabel), Without<CharacterCardLevelLabel>>,
    mut level_text_query: Query<(&mut Text, &CharacterCardLevelLabel)>,
) {
    for (mut dirty, mut state, children) in card_query.iter_mut() {
        if !dirty.consume() {
            continue;
        }

        let vm = &store.character_panel;
        state.name = vm.name_key.clone();
        state.level = vm.level;
        state.hp_current = vm.hp;
        state.hp_max = vm.max_hp;
        state.mp_current = vm.mp;
        state.mp_max = vm.max_mp;

        // 更新子级名称文本（标记了 CharacterCardNameLabel）
        for child in children.iter() {
            if let Ok((mut text, _)) = text_query.get_mut(child) {
                text.0 = vm.name_key.clone();
            }
            if let Ok((mut text, _)) = level_text_query.get_mut(child) {
                text.0 = format!("Lv.{}", vm.level);
            }
        }
    }
}
