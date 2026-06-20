//! Reputation System — 声望变更处理系统
//!
//! 监听声望变更请求，验证关键角色保护，发布 ReputationChanged / ReputationLevelUp 事件。
//!
//! 详见 docs/02-domain/domains/faction_domain.md §5.1

use bevy::prelude::*;

use crate::core::domains::faction::components::{KeyCharacter, Reputation, ReputationLevel};
use crate::core::domains::faction::events::{ReputationChanged, ReputationLevelUp};
use crate::core::domains::faction::rules::reputation::{
    check_level_change, safe_reputation_change,
};

/// 声望变更请求事件（内部触发用）。
///
/// 外部系统通过触发此事件来请求声望变更，本 Observer 处理实际的数值修改和事件发布。
#[derive(Event, Debug, Clone)]
pub struct ReputationChangeRequest {
    /// 目标实体
    pub entity: Entity,
    /// 目标阵营
    pub faction_id: crate::core::domains::faction::components::FactionId,
    /// 声望变化量
    pub delta: i32,
    /// 变化原因
    pub reason: String,
}

/// 响应声望变更请求，执行变更并发布事件。
pub(crate) fn on_reputation_change_request(
    trigger: On<ReputationChangeRequest>,
    mut query: Query<(&mut Reputation, Option<&KeyCharacter>)>,
    mut commands: Commands,
) {
    let req = trigger.event();
    let entity = req.entity;

    let Ok((mut reputation, key_char)) = query.get_mut(entity) else {
        tracing::warn!(target: "faction", 
            event = "faction.reputation_change.missing_component",
            entity = ?entity,
            "ReputationChangeRequest: 实体 {:?} 没有 Reputation 组件",
            entity
        );
        return;
    };

    let faction = req.faction_id.clone();
    let current = reputation.get(&faction);
    let is_key = key_char.is_some();

    // 关键角色保护检查
    let Some(new_value) = safe_reputation_change(current, req.delta, is_key) else {
        return;
    };

    // 应用变更
    reputation.values.insert(faction.clone(), new_value);

    let new_level = ReputationLevel::from_value(new_value);

    // 发布 ReputationChanged 事件
    commands.trigger(ReputationChanged {
        entity,
        faction_id: faction.clone(),
        old_value: current,
        new_value,
        new_level,
        reason: req.reason.clone(),
    });

    // 检查是否跨越等级
    if let Some((old_level, new_level)) = check_level_change(current, new_value) {
        commands.trigger(ReputationLevelUp {
            entity,
            faction_id: faction,
            old_level,
            new_level,
        });
    }
}
