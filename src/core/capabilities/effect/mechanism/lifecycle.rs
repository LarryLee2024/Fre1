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

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::capabilities::effect::foundation::{
        EffectDuration, EffectInstance, EffectPeriod, EffectStage, TickState,
    };

    // ── Helpers ────────────────────────────────────────────

    fn make_instant_effect(id: &str) -> EffectInstance {
        EffectInstance::new(
            id,
            "eff_damage",
            "Damage",
            "caster_001",
            "target_001",
            EffectDuration::Instant,
            1,
        )
    }

    fn make_duration_effect(id: &str, turns: u32) -> EffectInstance {
        EffectInstance::new(
            id,
            "eff_poison",
            "Debuff",
            "caster_001",
            "target_001",
            EffectDuration::HasDuration {
                turns,
                calculation:
                    crate::core::capabilities::effect::foundation::DurationCalculation::Fixed,
            },
            1,
        )
    }

    fn make_infinite_effect(id: &str) -> EffectInstance {
        EffectInstance::new(
            id,
            "eff_aura",
            "Buff",
            "caster_001",
            "target_001",
            EffectDuration::Infinite,
            1,
        )
    }

    fn make_container() -> ActiveEffectContainer {
        ActiveEffectContainer::new()
    }

    // ── EffectInstance creation ────────────────────────────

    #[test]
    fn unit_001_instant_effect_starts_applying() {
        let effect = make_instant_effect("inst_001");
        assert_eq!(effect.stage, EffectStage::Applying);
        assert_eq!(effect.remaining_turns, 0);
    }

    #[test]
    fn unit_002_duration_effect_starts_applying() {
        let effect = make_duration_effect("dur_001", 3);
        assert_eq!(effect.stage, EffectStage::Applying);
        assert_eq!(effect.remaining_turns, 3);
    }

    #[test]
    fn unit_003_infinite_effect_starts_applying() {
        let effect = make_infinite_effect("inf_001");
        assert_eq!(effect.stage, EffectStage::Applying);
        assert_eq!(effect.remaining_turns, i64::MAX);
    }

    // ── Apply effect ───────────────────────────────────────

    #[test]
    fn unit_010_apply_instant_effect_success() {
        let mut container = make_container();
        let effect = make_instant_effect("inst_001");

        let result = apply_effect(&mut container, effect);
        assert!(result.success);
        assert_eq!(container.count(), 0); // Instant 效果直接到 Removed，不计入活跃
    }

    #[test]
    fn unit_011_apply_duration_effect_success() {
        let mut container = make_container();
        let effect = make_duration_effect("dur_001", 3);

        let result = apply_effect(&mut container, effect);
        assert!(result.success);
        assert_eq!(container.count(), 1); // Duration 进入 Active
    }

    #[test]
    fn unit_012_apply_infinite_effect_success() {
        let mut container = make_container();
        let effect = make_infinite_effect("inf_001");

        let result = apply_effect(&mut container, effect);
        assert!(result.success);
        assert_eq!(container.count(), 1);
    }

    #[test]
    fn unit_013_apply_duplicate_effect_rejected() {
        let mut container = make_container();
        let effect = make_duration_effect("dup_001", 3);

        let first = apply_effect(&mut container, effect);
        assert!(first.success);

        let second = make_duration_effect("dup_002", 3);
        let result = apply_effect(&mut container, second);
        assert!(!result.success);
        assert!(matches!(
            result.error,
            Some(EffectError::DuplicateEffect { .. })
        ));
    }

    #[test]
    fn unit_014_apply_effect_missing_source_rejected() {
        let mut container = make_container();
        let effect = EffectInstance::new(
            "no_source",
            "eff_test",
            "Test",
            "", // empty source
            "target_001",
            EffectDuration::Instant,
            1,
        );

        let result = apply_effect(&mut container, effect);
        assert!(!result.success);
        assert!(matches!(result.error, Some(EffectError::MissingSource(_))));
    }

    #[test]
    fn unit_015_apply_effect_slot_limit() {
        let mut container = ActiveEffectContainer::new().with_max_effects(1);
        let first = make_duration_effect("first", 3);
        let _ = apply_effect(&mut container, first);

        // Different def_id so duplicate check passes, but slot is full
        let second = EffectInstance::new(
            "second",
            "eff_other",
            "Buff",
            "caster_002",
            "target_001",
            EffectDuration::Infinite,
            1,
        );
        let result = apply_effect(&mut container, second);
        assert!(!result.success);
        assert!(matches!(
            result.error,
            Some(EffectError::SlotLimitReached { .. })
        ));
    }

    #[test]
    fn unit_016_apply_multiple_different_effects_ok() {
        let mut container = make_container();

        let a = make_duration_effect("a", 3);
        assert!(apply_effect(&mut container, a).success);

        let b = EffectInstance::new(
            "b",
            "eff_other",
            "Buff",
            "caster_002",
            "target_001",
            EffectDuration::Infinite,
            1,
        );
        assert!(apply_effect(&mut container, b).success);

        assert_eq!(container.count(), 2);
    }

    // ── Stage transitions ──────────────────────────────────

    #[test]
    fn unit_020_transition_applying_to_active() {
        let mut effect = make_duration_effect("test", 3);
        assert!(effect.transition_to(EffectStage::Active).is_ok());
        assert_eq!(effect.stage, EffectStage::Active);
    }

    #[test]
    fn unit_021_transition_applying_to_removed() {
        let mut effect = make_instant_effect("test");
        assert!(effect.transition_to(EffectStage::Removed).is_ok());
        assert_eq!(effect.stage, EffectStage::Removed);
    }

    #[test]
    fn unit_022_transition_active_to_expiring() {
        let mut effect = make_duration_effect("test", 3);
        let _ = effect.transition_to(EffectStage::Active);
        assert!(effect.transition_to(EffectStage::Expiring).is_ok());
    }

    #[test]
    fn unit_023_transition_expiring_to_removed() {
        let mut effect = make_duration_effect("test", 3);
        let _ = effect.transition_to(EffectStage::Active);
        let _ = effect.transition_to(EffectStage::Expiring);
        assert!(effect.transition_to(EffectStage::Removed).is_ok());
    }

    #[test]
    fn unit_024_invalid_transition_active_to_applying() {
        let mut effect = make_duration_effect("test", 3);
        let _ = effect.transition_to(EffectStage::Active);
        let result = effect.transition_to(EffectStage::Applying);
        assert!(result.is_err());
    }

    #[test]
    fn unit_025_invalid_transition_removed_to_any() {
        let mut effect = make_instant_effect("test");
        let _ = effect.transition_to(EffectStage::Removed);
        assert!(effect.transition_to(EffectStage::Active).is_err());
    }

    // ── Duration ticking ───────────────────────────────────

    #[test]
    fn unit_030_tick_duration_reduces_remaining_turns() {
        let mut container = make_container();
        let effect = make_duration_effect("test", 3);
        let _ = apply_effect(&mut container, effect);

        let result = tick_durations(&mut container, 1, 2);
        let instance = container.find_by_id("test").unwrap();
        assert_eq!(instance.remaining_turns, 2);
        assert!(result.ticked.is_empty());
        assert!(result.expired.is_empty());
    }

    #[test]
    fn unit_031_tick_duration_to_zero_triggers_expiring() {
        let mut container = make_container();
        let effect = make_duration_effect("test", 2);
        let _ = apply_effect(&mut container, effect);

        tick_durations(&mut container, 2, 2);
        let instance = container.find_by_id("test").unwrap();
        assert_eq!(instance.stage, EffectStage::Expiring);
        assert_eq!(instance.remaining_turns, 0);
    }

    #[test]
    fn unit_032_tick_duration_beyond_zero_clamps() {
        let mut container = make_container();
        let effect = make_duration_effect("test", 1);
        let _ = apply_effect(&mut container, effect);

        tick_durations(&mut container, 5, 2);
        let instance = container.find_by_id("test").unwrap();
        assert_eq!(instance.remaining_turns, 0);
        assert_eq!(instance.stage, EffectStage::Expiring);
    }

    #[test]
    fn unit_033_tick_infinite_does_not_expire() {
        let mut container = make_container();
        let effect = make_infinite_effect("test");
        let _ = apply_effect(&mut container, effect);

        tick_durations(&mut container, 100, 2);
        let instance = container.find_by_id("test").unwrap();
        assert_eq!(instance.stage, EffectStage::Active);
    }

    #[test]
    fn unit_034_tick_paused_effect_skipped() {
        let mut container = make_container();
        let mut effect = make_duration_effect("test", 3);
        effect.paused = true;
        let _ = apply_effect(&mut container, effect);

        tick_durations(&mut container, 2, 2);
        let instance = container.find_by_id("test").unwrap();
        assert_eq!(instance.remaining_turns, 3); // 未减少
    }

    // ── Periodic tick ──────────────────────────────────────

    #[test]
    fn unit_040_periodic_tick_triggers() {
        let mut container = make_container();
        let period = EffectPeriod::new(1).unwrap();
        let effect = make_duration_effect("dot", 5).with_period(period);
        let _ = apply_effect(&mut container, effect);

        let result = tick_durations(&mut container, 1, 2);
        assert!(result.ticked.contains(&"dot".to_string()));
    }

    #[test]
    fn unit_041_periodic_tick_not_at_wrong_interval() {
        let mut container = make_container();
        let period = EffectPeriod::new(2).unwrap();
        let effect = make_duration_effect("dot", 5).with_period(period);
        let _ = apply_effect(&mut container, effect);

        let result = tick_durations(&mut container, 1, 2);
        assert!(result.ticked.is_empty());
    }

    #[test]
    fn unit_042_periodic_tick_max_ticks() {
        let mut container = make_container();
        let period = EffectPeriod::new(1).unwrap().with_max_ticks(2).unwrap();
        let effect = make_duration_effect("dot", 10).with_period(period);
        let _ = apply_effect(&mut container, effect);

        // Tick 3 times, should only get 2
        let r1 = tick_durations(&mut container, 1, 2);
        assert_eq!(r1.ticked.len(), 1);

        let r2 = tick_durations(&mut container, 1, 3);
        assert_eq!(r2.ticked.len(), 1);

        let r3 = tick_durations(&mut container, 1, 4);
        assert!(r3.ticked.is_empty()); // max_ticks 已到
    }

    // ── Expire effects ─────────────────────────────────────

    #[test]
    fn unit_050_expire_effects_moves_to_removed() {
        let mut container = make_container();
        let effect = make_duration_effect("test", 1);
        let _ = apply_effect(&mut container, effect);
        tick_durations(&mut container, 1, 2); // → Expiring

        let expired = expire_effects(&mut container);
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0], "test");

        let instance = container.find_by_id("test").unwrap();
        assert_eq!(instance.stage, EffectStage::Removed);
    }

    #[test]
    fn unit_051_expire_only_expiring_effects() {
        let mut container = make_container();
        let _ = apply_effect(&mut container, make_duration_effect("a", 3));
        // Use different source to avoid duplicate check
        let mut b = make_duration_effect("b", 1);
        b.source_entity = "caster_002".into();
        let _ = apply_effect(&mut container, b);

        tick_durations(&mut container, 1, 2); // a→2, b→0→Expiring
        let expired = expire_effects(&mut container);
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0], "b");
    }

    // ── Remove effects ─────────────────────────────────────

    #[test]
    fn unit_060_remove_by_id_success() {
        let mut container = make_container();
        let effect = make_duration_effect("test", 3);
        let _ = apply_effect(&mut container, effect);

        let removed = remove_effect_by_id(&mut container, "test", &RemovalReason::Dispelled);
        assert!(removed.is_ok());
        assert_eq!(container.count(), 0);
    }

    #[test]
    fn unit_061_remove_by_id_not_found() {
        let mut container = make_container();
        let result = remove_effect_by_id(&mut container, "nonexistent", &RemovalReason::Manual);
        assert!(result.is_err());
    }

    #[test]
    fn unit_062_remove_undispellable_rejected() {
        let mut container = make_container();
        let mut effect = make_duration_effect("test", 3);
        effect.dispellable = false;
        let _ = apply_effect(&mut container, effect);

        let result = remove_effect_by_id(&mut container, "test", &RemovalReason::Dispelled);
        assert!(result.is_err());
    }

    #[test]
    fn unit_063_remove_undispellable_allowed_when_forced() {
        let mut container = make_container();
        let mut effect = make_duration_effect("test", 3);
        effect.dispellable = false;
        let _ = apply_effect(&mut container, effect);

        let result = remove_effect_by_id(&mut container, "test", &RemovalReason::Forced);
        assert!(result.is_ok());
    }

    #[test]
    fn unit_064_remove_by_source() {
        let mut container = make_container();

        let a = make_duration_effect("a", 3);
        let _ = apply_effect(&mut container, a);

        let b = EffectInstance::new(
            "b",
            "eff_other",
            "Buff",
            "caster_002",
            "target_001",
            EffectDuration::Infinite,
            1,
        );
        let _ = apply_effect(&mut container, b);

        let removed =
            remove_effects_by_source(&mut container, "caster_001", &RemovalReason::SourceDied);
        assert_eq!(removed.len(), 1);
        assert_eq!(container.count(), 1);
    }

    #[test]
    fn unit_065_remove_by_def() {
        let mut container = make_container();

        let a = make_duration_effect("a", 3);
        let _ = apply_effect(&mut container, a);

        let mut b = make_duration_effect("b", 3);
        b.source_entity = "caster_002".into();
        let _ = apply_effect(&mut container, b);

        let removed = remove_effects_by_def(&mut container, "eff_poison", &RemovalReason::Manual);
        assert_eq!(removed.len(), 2);
        assert_eq!(container.count(), 0);
    }

    // ── Container queries ──────────────────────────────────

    #[test]
    fn unit_070_container_find_by_def() {
        let mut container = make_container();

        let a = make_duration_effect("a", 3);
        let _ = apply_effect(&mut container, a);

        let found = container.find_by_def("eff_poison");
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn unit_071_container_find_by_source() {
        let mut container = make_container();
        let _ = apply_effect(&mut container, make_duration_effect("a", 3));

        let found = container.find_by_source("caster_001");
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn unit_072_container_is_empty() {
        let container = make_container();
        assert!(container.is_empty());
    }

    #[test]
    fn unit_073_container_get_tickable() {
        let mut container = make_container();
        let period = EffectPeriod::new(1).unwrap();
        let effect = make_duration_effect("dot", 5).with_period(period);
        let _ = apply_effect(&mut container, effect);

        let tickable = container.get_tickable();
        assert_eq!(tickable.len(), 1);
    }

    #[test]
    fn unit_074_container_has_duplicate() {
        let mut container = make_container();
        let _ = apply_effect(&mut container, make_duration_effect("a", 3));
        assert!(container.has_duplicate("eff_poison", "caster_001"));
        assert!(!container.has_duplicate("eff_other", "caster_001"));
    }

    // ── TickState ──────────────────────────────────────────

    #[test]
    fn unit_080_tick_state_advance_triggers() {
        let period = EffectPeriod::new(2).unwrap();
        let mut state = TickState::new(&period);

        assert!(!state.advance(1)); // remaining 2→1
        assert!(state.advance(1)); // remaining 1→0, triggers
        assert_eq!(state.tick_count, 1);
        assert_eq!(state.remaining_turns, 2); // reset
    }

    #[test]
    fn unit_081_tick_state_max_ticks_stops() {
        let period = EffectPeriod::new(1).unwrap().with_max_ticks(2).unwrap();
        let mut state = TickState::new(&period);

        assert!(state.advance(1)); // tick 1
        assert!(state.advance(1)); // tick 2
        assert!(!state.advance(1)); // stopped
        assert_eq!(state.tick_count, 2);
    }

    #[test]
    fn unit_082_tick_state_has_more() {
        let period = EffectPeriod::new(1).unwrap().with_max_ticks(3).unwrap();
        let mut state = TickState::new(&period);
        assert!(state.has_more());
        state.tick_count = 3;
        assert!(!state.has_more());
    }

    // ── EffectStage helpers ────────────────────────────────

    #[test]
    fn unit_090_stage_is_active() {
        assert!(EffectStage::Applying.is_active());
        assert!(EffectStage::Active.is_active());
        assert!(!EffectStage::Expiring.is_active());
        assert!(!EffectStage::Removed.is_active());
    }

    #[test]
    fn unit_091_stage_can_tick() {
        assert!(!EffectStage::Applying.can_tick());
        assert!(EffectStage::Active.can_tick());
        assert!(!EffectStage::Expiring.can_tick());
        assert!(!EffectStage::Removed.can_tick());
    }

    #[test]
    fn unit_092_stage_name() {
        assert_eq!(EffectStage::Active.name(), "Active");
        assert_eq!(EffectStage::Removed.name(), "Removed");
    }

    // ── EffectDuration helpers ─────────────────────────────

    #[test]
    fn unit_100_duration_is_instant() {
        assert!(EffectDuration::Instant.is_instant());
        assert!(
            !EffectDuration::HasDuration {
                turns: 3,
                calculation:
                    crate::core::capabilities::effect::foundation::DurationCalculation::Fixed
            }
            .is_instant()
        );
    }

    #[test]
    fn unit_101_duration_initial_remaining() {
        assert_eq!(EffectDuration::Instant.initial_remaining_turns(), 0);
        assert_eq!(
            EffectDuration::HasDuration {
                turns: 5,
                calculation:
                    crate::core::capabilities::effect::foundation::DurationCalculation::Fixed
            }
            .initial_remaining_turns(),
            5
        );
        assert_eq!(EffectDuration::Infinite.initial_remaining_turns(), i64::MAX);
    }

    // ── ApplyResult helpers ────────────────────────────────

    #[test]
    fn unit_110_apply_result_success() {
        let r = ApplyResult::success("inst_001");
        assert!(r.success);
        assert_eq!(r.instance_id, Some("inst_001".into()));
        assert!(r.error.is_none());
    }

    #[test]
    fn unit_111_apply_result_failure() {
        let r = ApplyResult::failure(EffectError::SlotLimitReached { current: 5, max: 5 });
        assert!(!r.success);
        assert!(r.instance_id.is_none());
        assert!(r.error.is_some());
    }

    // ── EffectPeriod validation ────────────────────────────

    #[test]
    fn unit_120_period_invalid_interval() {
        let result = EffectPeriod::new(0);
        assert!(result.is_err());
    }

    #[test]
    fn unit_121_period_invalid_max_ticks() {
        let period = EffectPeriod::new(1).unwrap();
        let result = period.with_max_ticks(0);
        assert!(result.is_err());
    }

    #[test]
    fn unit_122_period_valid() {
        let period = EffectPeriod::new(2).unwrap().with_max_ticks(5).unwrap();
        assert_eq!(period.interval_turns, 2);
        assert_eq!(period.max_ticks, Some(5));
    }

    // ── RemovalReason ──────────────────────────────────────

    #[test]
    fn unit_130_removal_reason_name() {
        assert_eq!(RemovalReason::Expired.name(), "Expired");
        assert_eq!(RemovalReason::Dispelled.name(), "Dispelled");
        assert_eq!(RemovalReason::SourceDied.name(), "SourceDied");
    }

    // ─── EffectCategory ───────────────────────────────────

    #[test]
    fn unit_140_category_name() {
        assert_eq!(
            crate::core::capabilities::effect::foundation::EffectCategory::Buff.name(),
            "Buff"
        );
        assert_eq!(
            crate::core::capabilities::effect::foundation::EffectCategory::Debuff.name(),
            "Debuff"
        );
        assert_eq!(
            crate::core::capabilities::effect::foundation::EffectCategory::Custom("test".into())
                .name(),
            "test"
        );
    }

    // ─── EffectInstance builder ────────────────────────────

    #[test]
    fn unit_150_instance_with_period() {
        let period = EffectPeriod::new(1).unwrap();
        let effect = make_duration_effect("test", 3).with_period(period);
        assert!(effect.tick_state.is_some());
    }

    #[test]
    fn unit_151_instance_with_modifiers() {
        let effect = make_duration_effect("test", 3).with_modifiers(2);
        assert_eq!(effect.modifier_count, 2);
    }

    #[test]
    fn unit_152_instance_undispellable() {
        let effect = make_duration_effect("test", 3).with_undispellable();
        assert!(!effect.dispellable);
    }

    #[test]
    fn unit_153_instance_with_stack() {
        let effect = make_duration_effect("test", 3).with_stack(3);
        assert_eq!(effect.stack_count, 3);
    }

    // ─── EffectError Display ───────────────────────────────

    #[test]
    fn unit_160_error_display() {
        let err = EffectError::DuplicateEffect {
            def_id: "eff_test".into(),
            detail: "already active".into(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("eff_test"));
    }
}
