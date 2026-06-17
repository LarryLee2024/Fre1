//! Trigger 评估器
//!
//! 纯函数触发器评估与触发逻辑（无副作用）。
//! 遵循领域规则 §5.2：检查频率限制 → 检查触发条件 → 创建 TriggerContext。
//!
//! 详见 docs/02-domain/trigger_domain.md §5.2-5.4。

use crate::core::capabilities::trigger::foundation::{
    TriggerEntry, TriggerFrequency, TriggerParams, TriggerType,
};

/// 触发器评估结果。
#[derive(Debug, Clone)]
pub enum TriggerEvalResult {
    /// 条件满足，允许触发
    Ready(TriggerContext),
    /// 条件不满足（附带原因）
    Blocked(String),
}

/// 触发上下文——当触发器被激活时携带的数据载荷。
///
/// 传递给 Ability 系统，供目标技能在执行时使用。
#[derive(Debug, Clone)]
pub struct TriggerContext {
    /// 触发源触发器 ID
    pub trigger_id: String,
    /// 触发类型
    pub trigger_type: TriggerType,
    /// 目标 AbilityDef ID
    pub target_ability_def_id: String,
    /// 来源实体标识（字符串跨域）
    pub source_entity: String,
    /// 触发载荷（key-value，语义由触发类型定义）
    pub payload: TriggerParams,
}

impl TriggerContext {
    /// 创建触发上下文。
    pub fn new(
        trigger_id: impl Into<String>,
        trigger_type: TriggerType,
        target_ability_def_id: impl Into<String>,
        source_entity: impl Into<String>,
    ) -> Self {
        Self {
            trigger_id: trigger_id.into(),
            trigger_type,
            target_ability_def_id: target_ability_def_id.into(),
            source_entity: source_entity.into(),
            payload: TriggerParams::new(),
        }
    }

    /// 添加载荷数据。
    pub fn with_payload(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.payload.insert(key.into(), value.into());
        self
    }
}

/// 评估触发器是否可触发。
///
/// 三阶段检查：
/// 1. 触发类型匹配（由 event_type 指定）
/// 2. 频率限制检查（不变量 §3.3）
/// 3. 条件委托检查（通过 condition_check 回调）
///
/// # 参数
/// - `entry`: 待评估的触发器条目
/// - `event_type`: 当前发生的事件类型
/// - `condition_check`: 条件评估回调（None = 视为条件满足）。
///   参数为 condition_id，返回 true = 条件通过。
///
/// # 无副作用保证
/// 纯函数——不修改 entry 的状态。调用方需在确认触发后调用 `record_trigger`。
pub fn can_trigger(
    entry: &TriggerEntry,
    event_type: &TriggerType,
    condition_check: Option<&dyn Fn(&str) -> bool>,
) -> TriggerEvalResult {
    // 1. 触发类型匹配
    if &entry.trigger_type != event_type {
        return TriggerEvalResult::Blocked(format!(
            "trigger type mismatch: expected {:?}, got {:?}",
            entry.trigger_type, event_type
        ));
    }

    // 2. 频率限制检查
    if !entry.can_trigger() {
        return TriggerEvalResult::Blocked(format!(
            "trigger '{}' exceeded frequency limit ({}/{})",
            entry.id, entry.frequency.current_turn_count, entry.frequency.max_per_turn,
        ));
    }

    // 3. 条件检查（委托 Condition 领域）
    if let Some(cond_id) = &entry.condition.condition_id {
        if let Some(check) = condition_check {
            if !check(cond_id) {
                return TriggerEvalResult::Blocked(format!(
                    "trigger condition '{}' not met",
                    cond_id
                ));
            }
        }
    }

    TriggerEvalResult::Ready(TriggerContext {
        trigger_id: entry.id.clone(),
        trigger_type: entry.trigger_type.clone(),
        target_ability_def_id: entry.target_ability_def_id.clone(),
        source_entity: String::new(), // caller fills this
        payload: TriggerParams::new(),
    })
}

/// 创建已就绪的 TriggerContext（简化版——当条件已由外部确认时使用）。
///
/// 调用方负责：确认 can_trigger 返回 Ready 之后，再调用此函数生成上下文。
pub fn build_trigger_context(
    entry: &TriggerEntry,
    source_entity: impl Into<String>,
    extra_payload: TriggerParams,
) -> TriggerContext {
    let mut ctx = TriggerContext::new(
        entry.id.clone(),
        entry.trigger_type.clone(),
        entry.target_ability_def_id.clone(),
        source_entity,
    );
    ctx.payload = extra_payload;
    ctx
}

/// 回合结束时重置所有触发器的频率计数。
///
/// 应在 TurnEnd System 中调用。
pub fn reset_all_frequencies(entries: &mut [TriggerEntry]) {
    for entry in entries {
        entry.reset_turn_count();
    }
}

/// 检查触发器是否满足触发上限（不变量 §3.3 前置检查）。
///
/// 返回 None = 可触发，Some(msg) = 被抑制及原因。
pub fn check_frequency_limit(frequency: &TriggerFrequency, trigger_id: &str) -> Option<String> {
    if !frequency.can_trigger() {
        Some(format!(
            "trigger '{}' suppressed: count {}/{}",
            trigger_id, frequency.current_turn_count, frequency.max_per_turn
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::capabilities::trigger::foundation::{
        TriggerCondition, TriggerEntry, TriggerFrequency, TriggerType,
    };

    // ── TriggerType matching ───────────────────────────────

    #[test]
    fn unit_001_trigger_type_match_passes() {
        let entry = TriggerEntry::new("trig_001", TriggerType::OnDamaged, "abl_000001");
        let result = can_trigger(&entry, &TriggerType::OnDamaged, None);
        assert!(matches!(result, TriggerEvalResult::Ready(_)));
    }

    #[test]
    fn unit_002_trigger_type_mismatch_blocked() {
        let entry = TriggerEntry::new("trig_001", TriggerType::OnDamaged, "abl_000001");
        let result = can_trigger(&entry, &TriggerType::OnHealed, None);
        assert!(matches!(result, TriggerEvalResult::Blocked(_)));
    }

    // ── Frequency limit ────────────────────────────────────

    #[test]
    fn unit_003_unlimited_frequency_always_allows() {
        let mut entry = TriggerEntry::new("trig_001", TriggerType::OnDamaged, "abl_000001");
        entry.record_trigger();
        entry.record_trigger();
        entry.record_trigger();
        let result = can_trigger(&entry, &TriggerType::OnDamaged, None);
        assert!(matches!(result, TriggerEvalResult::Ready(_)));
    }

    #[test]
    fn unit_004_limited_frequency_blocks_after_exceed() {
        let mut entry =
            TriggerEntry::new("trig_001", TriggerType::OnDamaged, "abl_000001").with_frequency(2);
        entry.record_trigger();
        entry.record_trigger();
        let result = can_trigger(&entry, &TriggerType::OnDamaged, None);
        assert!(matches!(result, TriggerEvalResult::Blocked(_)));
    }

    #[test]
    fn unit_005_limited_frequency_allows_within_limit() {
        let mut entry =
            TriggerEntry::new("trig_001", TriggerType::OnDamaged, "abl_000001").with_frequency(3);
        entry.record_trigger();
        let result = can_trigger(&entry, &TriggerType::OnDamaged, None);
        assert!(matches!(result, TriggerEvalResult::Ready(_)));
    }

    #[test]
    fn unit_006_frequency_reset_allows_retrigger() {
        let mut entry =
            TriggerEntry::new("trig_001", TriggerType::OnDamaged, "abl_000001").with_frequency(1);
        entry.record_trigger();
        // After reset
        entry.reset_turn_count();
        let result = can_trigger(&entry, &TriggerType::OnDamaged, None);
        assert!(matches!(result, TriggerEvalResult::Ready(_)));
    }

    // ── Condition delegation ───────────────────────────────

    #[test]
    fn unit_007_condition_check_passes() {
        let entry = TriggerEntry::new("trig_001", TriggerType::OnDamaged, "abl_000001")
            .with_condition(TriggerCondition::with_condition("hp_below_30"));
        let result = can_trigger(
            &entry,
            &TriggerType::OnDamaged,
            Some(&|cond_id| cond_id == "hp_below_30"),
        );
        assert!(matches!(result, TriggerEvalResult::Ready(_)));
    }

    #[test]
    fn unit_008_condition_check_blocks() {
        let entry = TriggerEntry::new("trig_001", TriggerType::OnDamaged, "abl_000001")
            .with_condition(TriggerCondition::with_condition("hp_below_30"));
        let result = can_trigger(
            &entry,
            &TriggerType::OnDamaged,
            Some(&|cond_id| cond_id == "is_raging"),
        );
        assert!(matches!(result, TriggerEvalResult::Blocked(_)));
    }

    #[test]
    fn unit_009_no_condition_always_passes() {
        let entry = TriggerEntry::new("trig_001", TriggerType::OnTurnStart, "abl_000001");
        let result = can_trigger(&entry, &TriggerType::OnTurnStart, None);
        assert!(matches!(result, TriggerEvalResult::Ready(_)));
    }

    // ── TriggerContext building ────────────────────────────

    #[test]
    fn unit_010_build_context_has_correct_fields() {
        let entry = TriggerEntry::new("trig_001", TriggerType::OnDamaged, "abl_000001");
        let mut payload = TriggerParams::new();
        payload.insert("damage_amount".into(), "42".into());
        let ctx = build_trigger_context(&entry, "entity_001", payload);
        assert_eq!(ctx.trigger_id, "trig_001");
        assert_eq!(ctx.target_ability_def_id, "abl_000001");
        assert_eq!(ctx.source_entity, "entity_001");
        assert_eq!(ctx.payload.get("damage_amount").unwrap(), "42");
    }

    // ── Frequency limits (standalone check) ────────────────

    #[test]
    fn unit_011_check_frequency_returns_none_when_ok() {
        let freq = TriggerFrequency::limited(3);
        // count=0 → ok
        assert!(check_frequency_limit(&freq, "trig_001").is_none());
    }

    #[test]
    fn unit_012_check_frequency_returns_msg_when_exceeded() {
        let mut freq = TriggerFrequency::limited(1);
        freq.record_trigger();
        let msg = check_frequency_limit(&freq, "trig_001");
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("suppressed"));
    }

    #[test]
    fn unit_013_unlimited_frequency_check_passes() {
        let freq = TriggerFrequency::unlimited();
        assert!(check_frequency_limit(&freq, "trig_001").is_none());
    }

    // ── TriggerEntry builder ───────────────────────────────

    #[test]
    fn unit_014_entry_builder_creates_valid_entry() {
        let entry = TriggerEntry::new("trig_001", TriggerType::OnTurnEnd, "abl_000001")
            .with_condition(TriggerCondition::with_condition("cond_001"))
            .with_frequency(1);
        assert_eq!(entry.id, "trig_001");
        assert_eq!(entry.trigger_type, TriggerType::OnTurnEnd);
        assert_eq!(entry.condition.condition_id.unwrap(), "cond_001");
        assert_eq!(entry.frequency.max_per_turn, 1);
    }

    #[test]
    fn unit_015_reset_all_frequencies_works() {
        let mut entries = vec![
            TriggerEntry::new("trig_001", TriggerType::OnDamaged, "abl_000001").with_frequency(3),
            TriggerEntry::new("trig_002", TriggerType::OnHealed, "abl_000002").with_frequency(1),
        ];
        entries[0].record_trigger();
        entries[0].record_trigger();
        entries[1].record_trigger();
        reset_all_frequencies(&mut entries);
        assert_eq!(entries[0].frequency.current_turn_count, 0);
        assert_eq!(entries[1].frequency.current_turn_count, 0);
    }

    // ── TriggerType name ───────────────────────────────────

    #[test]
    fn unit_016_trigger_type_name() {
        assert_eq!(TriggerType::OnDamaged.name(), "OnDamaged");
        assert_eq!(TriggerType::OnCustom("test".into()).name(), "OnCustom");
    }
}
