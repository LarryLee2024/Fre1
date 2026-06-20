//! Dialogue System — 对话流程控制系统
//!
//! 处理对话开始、分支选择、对话结束。
//! 详见 docs/02-domain/domains/narrative_domain.md §5.1-5.3

use bevy::prelude::*;

use crate::core::domains::narrative::components::{
    ChoiceOption, DialogueHistory, DialogueState, DialogueTreeRegistry, StoryFlags,
};
use crate::core::domains::narrative::events::{ChoiceMade, DialogueStarted, StoryFlagSet};
use crate::core::domains::narrative::rules::filter_visible_choices;

/// 对话开始请求事件。
#[derive(Event, Debug, Clone)]
pub struct DialogueStartRequest {
    /// 启动对话的实体
    pub entity: Entity,
    /// NPC 实体
    pub npc: Entity,
    /// 对话树 ID
    pub tree_id: String,
}

/// 分支选择请求事件。
#[derive(Event, Debug, Clone)]
pub struct ChoiceSelectRequest {
    /// 做出选择的实体
    pub entity: Entity,
    /// 选择的分支 ID
    pub choice_id: String,
}

/// 响应对话开始请求。
pub(crate) fn on_dialogue_start(
    trigger: On<DialogueStartRequest>,
    tree_registry: Option<Res<DialogueTreeRegistry>>,
    mut history: Option<ResMut<DialogueHistory>>,
    flag_query: Query<&mut StoryFlags>,
    mut commands: Commands,
) {
    let req = trigger.event();
    let Some(registry) = tree_registry else {
        tracing::warn!(target: "narrative", 
            event = "narrative.dialogue_start.missing_registry",
            "DialogueStartRequest: 没有 DialogueTreeRegistry"
        );
        return;
    };

    let Some(entry) = registry.entry_node(&req.tree_id) else {
        tracing::warn!(target: "narrative", 
            event = "narrative.dialogue_start.tree_not_found",
            tree_id = %req.tree_id,
            "DialogueStartRequest: 对话树 '{}' 未找到",
            req.tree_id
        );
        return;
    };

    // 获取实体的 StoryFlag
    let flags = flag_query.get(req.entity).ok();
    let choices = match flags {
        Some(f) => filter_visible_choices(entry, f),
        None => entry
            .choices
            .iter()
            .map(|c| ChoiceOption {
                choice_id: c.id.clone(),
                text: c.text.clone(),
                visible: c.condition_ref.is_none(),
            })
            .collect(),
    };

    // 记录历史
    if let Some(ref mut h) = history {
        h.visit_node(&req.tree_id, &entry.id);
    }

    // 添加 DialogueState 组件到玩家实体
    commands.entity(req.entity).insert(DialogueState::new(
        &req.tree_id,
        &entry.id,
        choices.clone(),
        0.0,
    ));

    // 发布事件
    commands.trigger(DialogueStarted {
        entity: req.entity,
        npc: req.npc,
        tree_id: req.tree_id.clone(),
        entry_node_id: entry.id.clone(),
        available_choices: choices,
    });
}

/// 响应分支选择请求。
pub(crate) fn on_choice_select(
    trigger: On<ChoiceSelectRequest>,
    mut dialogue_query: Query<&mut DialogueState>,
    tree_registry: Option<Res<DialogueTreeRegistry>>,
    mut flag_query: Query<&mut StoryFlags>,
    mut history: Option<ResMut<DialogueHistory>>,
    mut commands: Commands,
) {
    let req = trigger.event();
    let entity = req.entity;

    let Ok(mut state) = dialogue_query.get_mut(entity) else {
        tracing::warn!(target: "narrative", 
            event = "narrative.choice_select.missing_state",
            entity = ?entity,
            "ChoiceSelectRequest: 实体 {:?} 没有 DialogueState",
            entity
        );
        return;
    };

    let Some(registry) = tree_registry else {
        tracing::warn!(target: "narrative", 
            event = "narrative.choice_select.missing_registry",
            "ChoiceSelectRequest: 没有 DialogueTreeRegistry"
        );
        return;
    };

    // 查找当前节点
    let Some(node) = registry.get_node(&state.current_node_id) else {
        tracing::warn!(target: "narrative", 
            event = "narrative.choice_select.node_not_found",
            node_id = %state.current_node_id,
            "ChoiceSelectRequest: 节点 '{}' 未找到",
            state.current_node_id
        );
        return;
    };

    // 查找选择的分支
    let Some(choice) = node.choices.iter().find(|c| c.id == req.choice_id) else {
        tracing::warn!(target: "narrative", 
            event = "narrative.choice_select.choice_not_found",
            choice_id = %req.choice_id,
            node_id = %state.current_node_id,
            "ChoiceSelectRequest: 分支 '{}' 不在节点 '{}' 中",
            req.choice_id, state.current_node_id
        );
        return;
    };

    // 记录历史
    if let Some(ref mut h) = history {
        h.record_choice(&state.tree_id, &choice.id);
    }

    // 设置 StoryFlag
    let mut story_flags_set = Vec::new();
    if let Ok(mut flags) = flag_query.get_mut(entity) {
        for (flag_id, value) in &choice.set_flags {
            if flags.set_flag(flag_id.clone(), value.clone()) {
                story_flags_set.push((flag_id.clone(), value.clone()));
                commands.trigger(StoryFlagSet {
                    entity,
                    flag_id: flag_id.clone(),
                    value: value.clone(),
                    source: "dialogue".into(),
                });
            }
        }
    }

    // 跳转到下一节点或结束
    match &choice.next_node_id {
        Some(next_id) => {
            let next_choices = registry
                .get_node(next_id)
                .map(|next_node| {
                    let flags = flag_query.get(entity).ok();
                    match flags {
                        Some(f) => filter_visible_choices(next_node, f),
                        None => next_node
                            .choices
                            .iter()
                            .map(|c| ChoiceOption {
                                choice_id: c.id.clone(),
                                text: c.text.clone(),
                                visible: c.condition_ref.is_none(),
                            })
                            .collect(),
                    }
                })
                .unwrap_or_default();
            state.advance(next_id.clone(), next_choices);

            if let Some(ref mut h) = history {
                h.visit_node(&state.tree_id, next_id);
            }
        }
        None => {
            state.end();
            // 对话结束，移除 DialogueState
            commands.entity(entity).remove::<DialogueState>();
        }
    }

    commands.trigger(ChoiceMade {
        entity,
        dialogue_id: state.tree_id.clone(),
        choice_id: req.choice_id.clone(),
        story_flags_set,
    });
}
