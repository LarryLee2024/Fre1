//! CombatTriggerParam — Bevy SystemParam，封装触发器容器访问。
//!
//! Systems 通过此 param 查询和变更触发器状态，
//! 完全不知道 TriggerContainer 的具体实现。

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::capabilities::trigger::mechanism::TriggerContainer;

use super::CombatTriggerType;

/// 战斗触发器查询参数 — 封装触发器容器的所有操作。
///
/// System 签名中使用此类型替代裸 `Query<&mut TriggerContainer>`。
#[derive(SystemParam)]
pub struct CombatTriggerParam<'w, 's> {
    pub containers: Query<'w, 's, &'static mut TriggerContainer>,
}

impl CombatTriggerParam<'_, '_> {
    /// 评估并消耗指定实体的就绪触发器。
    ///
    /// 完成以下操作：
    /// 1. 查找该实体上指定类型的所有就绪触发器
    /// 2. 标记已触发的触发器（record_trigger）
    /// 3. 重置回合触发计数器
    ///
    /// 返回已触发的触发器 ID 列表。
    pub fn evaluate_and_consume(
        &mut self,
        entity: Entity,
        trigger_type: CombatTriggerType,
    ) -> Vec<String> {
        let Ok(mut container) = self.containers.get_mut(entity) else {
            return Vec::new();
        };

        let tt = trigger_type.to_trigger_type();
        let ready_ids: Vec<String> = container
            .find_ready(&tt)
            .into_iter()
            .map(|entry| entry.id.clone())
            .collect();

        for id in &ready_ids {
            if let Some(entry) = container.get_mut(id) {
                entry.record_trigger();
            }
        }

        container.reset_turn_counts();
        ready_ids
    }

    /// 仅查找就绪触发器（只读，不记录触发）。
    pub fn find_ready(&self, entity: Entity, trigger_type: CombatTriggerType) -> Vec<String> {
        let Ok(container) = self.containers.get(entity) else {
            return Vec::new();
        };

        let tt = trigger_type.to_trigger_type();
        container
            .find_ready(&tt)
            .into_iter()
            .map(|entry| entry.id.clone())
            .collect()
    }
}
