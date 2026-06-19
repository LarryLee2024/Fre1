//! Trigger 评估器
//!
//! 纯函数触发器评估与触发逻辑（无副作用）。
//! 遵循领域规则 §5.2：检查频率限制 → 检查触发条件 → 创建 TriggerContext。
//!
//! 详见 docs/02-domain/capabilities/trigger_domain.md §5.2-5.4。

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
