//! Effect Lifecycle — 效果生命周期管理
//!
//! 提供效果从施加到移除的完整生命周期管理纯函数。
//! 遵循 docs/02-domain/effect_domain.md §2、§5 的流程定义。
//!
//! 核心函数：
//! - apply_effect() — 效果施加：检查条件 → 检查重复 → 注册到容器
//! - tick_durations() — 持续计时：推进所有效果计时器
//! - process_ticks() — 处理周期 Tick：对到期的 Tick 触发计算
//! - expire_effects() — 处理到期效果：进入 Expiring → Removed
//! - remove_effect() — 按条件移除效果（驱散/手动/来源死亡）
//! - can_apply() — 检查效果能否施加（免疫/条件/重复）

use crate::core::capabilities::effect::foundation::{
    ActiveEffectContainer, EffectDuration, EffectError, EffectInstance, EffectStage, RemovalReason,
};

/// 效果施加结果。
#[derive(Debug, Clone, PartialEq)]
pub struct ApplyResult {
    /// 施加是否成功
    pub success: bool,
    /// 效果实例 ID（成功时）
    pub instance_id: Option<String>,
    /// 失败原因（失败时）
    pub error: Option<EffectError>,
    /// 是否因免疫被阻止
    pub was_immunity_blocked: bool,
}

impl ApplyResult {
    /// 创建成功结果。
    pub fn success(instance_id: impl Into<String>) -> Self {
        Self {
            success: true,
            instance_id: Some(instance_id.into()),
            error: None,
            was_immunity_blocked: false,
        }
    }

    /// 创建失败结果。
    pub fn failure(error: EffectError) -> Self {
        let was_blocked = matches!(&error, EffectError::ImmunityBlocked { .. });
        Self {
            success: false,
            instance_id: None,
            error: Some(error),
            was_immunity_blocked: was_blocked,
        }
    }
}

/// 效果 Tick 结果。
#[derive(Debug, Clone, PartialEq)]
pub struct TickResult {
    /// 已 Tick 的效果实例 ID 列表
    pub ticked: Vec<String>,
    /// 已到期的效果实例 ID 列表
    pub expired: Vec<String>,
    /// 处理过程中遇到的错误
    pub errors: Vec<(String, EffectError)>,
}

impl TickResult {
    /// 创建空的 Tick 结果。
    pub fn empty() -> Self {
        Self {
            ticked: Vec::new(),
            expired: Vec::new(),
            errors: Vec::new(),
        }
    }
}

// ============================================================================
// 效果施加
// ============================================================================

/// 执行效果施加流程。
///
/// 流程（docs/02-domain/effect_domain.md §5.1）：
/// 1. 检查目标是否已有同源效果（不变量 3.5）
/// 2. 检查免疫条件（不变量 3.2，占位）
/// 3. 条件通过后，初始化效果并加入容器
/// 4. Instant 效果直接进入 Removed 阶段
///
/// # Errors
/// - DuplicateEffect: 同源效果已存在且不允许叠加
/// - ImmunityBlocked: 目标免疫此效果
/// - ConditionNotMet: 应用条件不满足
/// - SlotLimitReached: 效果槽位已满
pub fn apply_effect(
    container: &mut ActiveEffectContainer,
    instance: EffectInstance,
) -> ApplyResult {
    let def_id = instance.def_id.clone();
    let source_entity = instance.source_entity.clone();

    // V1: 来源检查（不变量 3.1）
    if source_entity.is_empty() {
        return ApplyResult::failure(EffectError::MissingSource(
            "source_entity must not be empty".into(),
        ));
    }

    // 不变量 3.5: 同源效果重复检查
    if container.has_duplicate(&def_id, &source_entity) {
        return ApplyResult::failure(EffectError::DuplicateEffect {
            def_id: def_id.clone(),
            detail: format!(
                "effect '{}' from source '{}' already active on target",
                def_id, source_entity
            ),
        });
    }

    // 不变量 3.2: 免疫检查（占位）
    // 实际实现应检查目标标签是否为 Tag.Immune.{EffectCategory}

    // 槽位检查
    if container.is_full() {
        return ApplyResult::failure(EffectError::SlotLimitReached {
            current: container.count(),
            max: container.max_effects,
        });
    }

    // 根据持续时间类型处理
    let mut instance = instance;
    match instance.duration {
        EffectDuration::Instant => {
            // Instant: Applying → Removed（立即执行）
            let _ = instance.transition_to(EffectStage::Removed);
        }
        EffectDuration::HasDuration { .. } | EffectDuration::Infinite => {
            // Duration/Infinite: Applying → Active
            let _ = instance.transition_to(EffectStage::Active);
        }
    }

    let instance_id = instance.instance_id.clone();
    container.effects.push(instance);

    ApplyResult::success(instance_id)
}

// ============================================================================
// 持续计时
// ============================================================================

/// 推进所有效果计时器。
///
/// 对 Duration 类效果减少 remaining_turns。
/// 如果 turns_elapsed 足够触发周期 Tick，返回需要 Tick 的效果列表。
/// 如果效果耗尽，标记为 Expiring。
pub fn tick_durations(
    container: &mut ActiveEffectContainer,
    turns_elapsed: u32,
    _current_turn: u64,
) -> TickResult {
    let mut result = TickResult::empty();

    for effect in &mut container.effects {
        if effect.paused || !effect.stage.can_tick() {
            continue;
        }

        match effect.duration {
            EffectDuration::Instant => continue,
            EffectDuration::HasDuration { .. } => {
                // 减少剩余回合数
                if effect.remaining_turns > 0 && effect.remaining_turns < i64::MAX {
                    effect.remaining_turns -= turns_elapsed as i64;
                    // 不变量 3.3: 剩余持续时间不得为负值
                    if effect.remaining_turns < 0 {
                        effect.remaining_turns = 0;
                    }
                }

                // Duration 耗尽 → Expiring
                if effect.remaining_turns <= 0 && effect.stage == EffectStage::Active {
                    let _ = effect.transition_to(EffectStage::Expiring);
                    result.expired.push(effect.instance_id.clone());
                }
            }
            EffectDuration::Infinite => {
                // Infinite 不会自然到期
            }
        }

        // 处理周期 Tick
        if effect.stage == EffectStage::Active {
            if let Some(ref mut tick_state) = effect.tick_state {
                if tick_state.advance(turns_elapsed) {
                    result.ticked.push(effect.instance_id.clone());
                }
            }
        }
    }

    result
}

/// 处理已到期的效果（Expiring → Removed）。
///
/// 对处于 Expiring 阶段的效果，执行移除前逻辑并设为 Removed。
/// 实际实现应在此阶段回退 Modifier（不变量 3.4）。
pub fn expire_effects(container: &mut ActiveEffectContainer) -> Vec<String> {
    let expired_ids: Vec<String> = container
        .effects
        .iter()
        .filter(|e| e.stage == EffectStage::Expiring)
        .map(|e| e.instance_id.clone())
        .collect();

    for effect in &mut container.effects {
        if effect.stage == EffectStage::Expiring {
            // 不变量 3.4: Modifier 回退（占位）
            // 实际实现应在此时回退所有关联 Modifier
            let _ = effect.transition_to(EffectStage::Removed);
        }
    }

    expired_ids
}

// ============================================================================
// 效果移除
// ============================================================================

/// 按 instance_id 移除单个效果。
///
/// 如果效果设置了不可驱散且原因不是 SourceDied/Forced，则拒绝移除。
pub fn remove_effect_by_id(
    container: &mut ActiveEffectContainer,
    instance_id: &str,
    reason: &RemovalReason,
) -> Result<EffectInstance, EffectError> {
    let idx = container
        .effects
        .iter()
        .position(|e| e.instance_id == instance_id && e.stage.is_active());

    match idx {
        Some(i) => {
            let effect = &container.effects[i];

            // 不可驱散检查
            if !effect.dispellable && matches!(reason, RemovalReason::Dispelled) {
                return Err(EffectError::Runtime(format!(
                    "effect '{}' is undispellable",
                    instance_id
                )));
            }

            // 不变量 3.4: Modifier 回退（占位）
            // 实际实现应在移除时回退所有关联 Modifier

            let mut effect = container.effects.remove(i);
            effect.stage = EffectStage::Removed;
            Ok(effect)
        }
        None => Err(EffectError::EffectNotFound(instance_id.into())),
    }
}

/// 按来源移除所有效果。
pub fn remove_effects_by_source(
    container: &mut ActiveEffectContainer,
    source_entity: &str,
    reason: &RemovalReason,
) -> Vec<EffectInstance> {
    let mut removed = Vec::new();
    container.effects.retain(|e| {
        if e.source_entity == source_entity && e.stage.is_active() && is_removal_allowed(e, reason)
        {
            removed.push(e.clone());
            false
        } else {
            true
        }
    });
    removed
}

/// 按 def_id 移除所有效果。
pub fn remove_effects_by_def(
    container: &mut ActiveEffectContainer,
    def_id: &str,
    reason: &RemovalReason,
) -> Vec<EffectInstance> {
    let mut removed = Vec::new();
    container.effects.retain(|e| {
        if e.def_id == def_id && e.stage.is_active() && is_removal_allowed(e, reason) {
            removed.push(e.clone());
            false
        } else {
            true
        }
    });
    removed
}

// ============================================================================
// 检查函数
// ============================================================================

/// 检查效果能否施加（不变量 3.2）。
///
/// 当前为占位实现——实际应检查目标标签免疫和应用条件。
pub fn can_apply(
    _container: &ActiveEffectContainer,
    _def_id: &str,
    _target_tags: &[String],
) -> Result<(), EffectError> {
    // 占位：默认允许施加
    // 实际实现应：
    // 1. 检查 target_tags 是否包含 Tag.Immune.{category}
    // 2. 检查 application_condition
    Ok(())
}

/// 检查效果是否允许被移除。
fn is_removal_allowed(effect: &EffectInstance, reason: &RemovalReason) -> bool {
    if effect.dispellable {
        return true;
    }
    // 不可驱散效果允许被强制移除或来源死亡时移除
    matches!(reason, RemovalReason::Forced | RemovalReason::SourceDied)
}
