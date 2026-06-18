//! Trigger Integration — combat 域接入 trigger capability
//!
//! 封装 trigger capability 的触发器评估，
//! 用于战斗事件（TurnStarted, DamageTaken 等）的触发器分发。

use crate::core::capabilities::trigger::foundation::{TriggerCondition, TriggerEntry, TriggerType};
use crate::core::capabilities::trigger::mechanism::{
    TriggerContext, TriggerEvalResult, can_trigger,
};

// ─── 战斗触发器类型 ────────────────────────────────────────────────

/// 战斗中常见的触发器类型。
pub enum CombatTriggerType {
    /// 回合开始
    TurnStarted,
    /// 回合结束
    TurnEnded,
    /// 受到伤害
    DamageTaken,
    /// 造成伤害（攻击）
    Attack,
    /// 生命值低于阈值
    HealthBelow(f32),
    /// 击杀敌人
    Kill,
}

impl CombatTriggerType {
    /// 转换为 TriggerType。
    pub fn to_trigger_type(&self) -> TriggerType {
        match self {
            CombatTriggerType::TurnStarted => TriggerType::OnTurnStart,
            CombatTriggerType::TurnEnded => TriggerType::OnTurnEnd,
            CombatTriggerType::DamageTaken => TriggerType::OnDamaged,
            CombatTriggerType::Attack => TriggerType::OnAttack,
            CombatTriggerType::HealthBelow(_) => TriggerType::OnConditionMet,
            CombatTriggerType::Kill => TriggerType::OnDeath,
        }
    }
}

// ─── Facade ────────────────────────────────────────────────────────

/// 战斗触发器 Facade — 封装 trigger capability 的战斗相关操作。
pub struct CombatTriggerFacade;

impl CombatTriggerFacade {
    /// 评估一个触发器是否可以触发。
    pub fn can_trigger_check(entry: &TriggerEntry, context: &TriggerContext) -> TriggerEvalResult {
        can_trigger(entry, context)
    }

    /// 评估一组触发器，返回已就绪的触发器列表。
    pub fn evaluate_triggers(
        entries: &[TriggerEntry],
        trigger_type: CombatTriggerType,
        context: &TriggerContext,
    ) -> Vec<TriggerEntry> {
        let tt = trigger_type.to_trigger_type();
        entries
            .iter()
            .filter(|entry| entry.trigger_type == tt)
            .filter(|entry| matches!(can_trigger(entry, context), TriggerEvalResult::Ready(_)))
            .cloned()
            .collect()
    }

    /// 创建战斗触发器条目。
    pub fn create_trigger_entry(
        id: &str,
        trigger_type: CombatTriggerType,
        target_ability_def_id: &str,
    ) -> TriggerEntry {
        TriggerEntry::new(id, trigger_type.to_trigger_type(), target_ability_def_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combat_trigger_type_converts_correctly() {
        assert_eq!(
            CombatTriggerType::TurnStarted.to_trigger_type(),
            TriggerType::OnTurnStart
        );
        assert_eq!(
            CombatTriggerType::DamageTaken.to_trigger_type(),
            TriggerType::OnDamaged
        );
    }
}
