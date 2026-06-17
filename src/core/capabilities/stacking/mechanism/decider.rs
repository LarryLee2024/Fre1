//! Stacking Decider — 堆叠判定核心逻辑
//!
//! 提供纯粹的堆叠决策函数，判断当同一效果的第二个实例到达时应如何处理。
//! 此模块不依赖 ECS 或 Effect 类型，仅使用 Stacking 自有类型。
//!
//! 详见 docs/02-domain/stacking_domain.md §5。
//! 详见 docs/04-data/capabilities/stacking_schema.md §3.5。

use crate::core::capabilities::stacking::foundation::{
    StackingConfig, StackingDecision, StackingError, StackingType,
};

/// 堆叠判定所需的主题数据。
///
/// 精简自 EffectInstance，仅包含堆叠判定所需的字段。
/// 效果施加方（Effect lifecycle）负责提取这些数据并传入判定器。
#[derive(Debug, Clone, PartialEq)]
pub struct StackingSubject {
    /// 实例 ID
    pub instance_id: String,
    /// 效果定义 ID
    pub def_id: String,
    /// 来源实体
    pub source_entity: String,
    /// 当前剩余持续时间（回合数）
    pub remaining_turns: i64,
    /// 当前堆叠层数
    pub stack_count: u32,
}

impl StackingSubject {
    /// 创建堆叠判定主题。
    pub fn new(
        instance_id: impl Into<String>,
        def_id: impl Into<String>,
        source_entity: impl Into<String>,
        remaining_turns: i64,
        stack_count: u32,
    ) -> Self {
        Self {
            instance_id: instance_id.into(),
            def_id: def_id.into(),
            source_entity: source_entity.into(),
            remaining_turns,
            stack_count,
        }
    }
}

/// 检查两个效果是否属于同一堆叠。
///
/// 判定逻辑（per docs/02-domain/stacking_domain.md §1）：
/// Step 1: 检查 EffectDefId —— 不同 → NoMatch
/// Step 2: 检查 SourceEntity
///   - 同源 → FullMatch
///   - 不同源 → CrossSource
pub fn match_identity(existing_def_id: &str, incoming_def_id: &str) -> bool {
    existing_def_id == incoming_def_id
}

/// 执行堆叠判定。
///
/// 当已有实例存在且 `match_identity` 返回 true 时调用。
/// 根据 StackingConfig 的 stacking_type 返回对应的决策。
///
/// # 流程 (per docs/02-domain/stacking_domain.md §5.1)
/// 1. 根据 StackingType 选择策略
/// 2. None → Reject（忽略新实例）
/// 3. Aggregate → Accumulate（层数叠加，受 max_stacks 限制）
/// 4. RefreshDuration → Refresh（重置持续时间，层数不变）
/// 5. Replace → Replace（按优先级替换，当前使用新实例替换旧实例）
///
/// # 不变量
/// - 3.2: 不同 EffectDef 不参与堆叠（调用方保证 match_identity 通过）
/// - 3.1: 堆叠层数不得超过 max_stacks
/// - 3.4: 溢出策略必须明确配置
pub fn decide_stacking(
    existing: &StackingSubject,
    incoming: &StackingSubject,
    config: &StackingConfig,
) -> StackingDecision {
    // 不变量 3.2: 调用方已保证 def_id 匹配

    match config.stacking_type {
        StackingType::None => {
            // None: 不堆叠，忽略新实例
            StackingDecision::Reject
        }

        StackingType::Aggregate => {
            // Aggregate: 层数叠加，受 max_stacks 限制
            let current = existing.stack_count;
            let new_total = current.saturating_add(1); // +1 for the new instance
            let capped = new_total.min(config.max_stacks);
            let added = capped.saturating_sub(current);

            if added == 0 {
                // 已达到上限，按溢出策略处理
                match config.overflow_behavior {
                    crate::core::capabilities::stacking::foundation::OverflowBehavior::IgnoreNew => {
                        StackingDecision::Reject
                    }
                    crate::core::capabilities::stacking::foundation::OverflowBehavior::Refresh => {
                        StackingDecision::Refresh {
                            refreshed_instance_id: existing.instance_id.clone(),
                            new_duration: incoming.remaining_turns.max(existing.remaining_turns),
                        }
                    }
                    crate::core::capabilities::stacking::foundation::OverflowBehavior::Replace => {
                        StackingDecision::Replace {
                            replaced_instance_id: existing.instance_id.clone(),
                        }
                    }
                    crate::core::capabilities::stacking::foundation::OverflowBehavior::RemoveOldest => {
                        // RemoveOldest 在 Aggregate 上下文中等同于 Reject 加刷新
                        // 此处简化为 Reject + Refresh
                        StackingDecision::Refresh {
                            refreshed_instance_id: existing.instance_id.clone(),
                            new_duration: incoming.remaining_turns.max(existing.remaining_turns),
                        }
                    }
                }
            } else {
                StackingDecision::Accumulate {
                    new_stack_count: capped,
                    added_layers: added,
                }
            }
        }

        StackingType::RefreshDuration => {
            // RefreshDuration: 重置持续时间，取最大值
            let new_duration = incoming.remaining_turns.max(existing.remaining_turns);
            StackingDecision::Refresh {
                refreshed_instance_id: existing.instance_id.clone(),
                new_duration,
            }
        }

        StackingType::Replace => {
            // Replace: 新实例替换旧实例
            StackingDecision::Replace {
                replaced_instance_id: existing.instance_id.clone(),
            }
        }
    }
}

// ============================================================================
// 高级判定接口（供 Effect lifecycle 使用）
// ============================================================================

/// 堆叠判定的完整结果，包含决策和需要更新的状态。
#[derive(Debug, Clone, PartialEq)]
pub struct StackingOutcome {
    /// 堆叠决策
    pub decision: StackingDecision,
    /// 决策后新的堆叠层数（仅 Aggregate 有意义）
    pub new_stack_count: u32,
}

/// 执行完整的堆叠判定流程。
///
/// 与 `decide_stacking` 不同，此函数包含：
/// - 身份匹配检查（不同 def_id 直接返回 None 表示不参与堆叠）
/// - 结果封装为 StackingOutcome
///
/// 返回 None 表示两个效果不属于同一堆叠，不进行任何堆叠操作。
pub fn evaluate_stacking(
    existing: &StackingSubject,
    incoming: &StackingSubject,
    config: &StackingConfig,
) -> Option<StackingOutcome> {
    // Step 1: 检查 EffectDefId —— 不同 → 不进行堆叠
    if !match_identity(&existing.def_id, &incoming.def_id) {
        return None;
    }

    // Step 2: 执行堆叠判定
    let decision = decide_stacking(existing, incoming, config);

    // Step 3: 计算新的层数
    let new_stack_count = match &decision {
        StackingDecision::Accumulate {
            new_stack_count, ..
        } => *new_stack_count,
        StackingDecision::Replace { .. } => 1,
        _ => existing.stack_count,
    };

    Some(StackingOutcome {
        decision,
        new_stack_count,
    })
}

/// 验证堆叠配置是否合法。
///
/// 校验规则（per docs/04-data/capabilities/stacking_schema.md §6）：
/// - V1: max_stacks ≥ 1
/// - V2: Aggregate 类型 max_stacks 必须 ≥ 2
/// - V3: Replace 类型 max_stacks 必须 = 1
pub fn validate_config(config: &StackingConfig) -> Result<(), StackingError> {
    // V1: max_stacks ≥ 1
    if config.max_stacks < 1 {
        return Err(StackingError::InvalidConfig(
            "max_stacks must be ≥ 1".into(),
        ));
    }

    match config.stacking_type {
        StackingType::None | StackingType::RefreshDuration => {
            // 无特殊约束
        }
        StackingType::Aggregate => {
            // V2: Aggregate 类型 max_stacks 必须 ≥ 2
            if config.max_stacks < 2 {
                return Err(StackingError::InvalidConfig(
                    "Aggregate stacking requires max_stacks ≥ 2".into(),
                ));
            }
        }
        StackingType::Replace => {
            // V3: Replace 类型 max_stacks 必须 = 1
            if config.max_stacks != 1 {
                return Err(StackingError::InvalidConfig(
                    "Replace stacking requires max_stacks == 1".into(),
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::capabilities::stacking::foundation::{
        OverflowBehavior, StackingConfig, StackingType,
    };

    // ── Helpers ────────────────────────────────────────────

    fn subject(id: &str, def_id: &str, source: &str, turns: i64, stack: u32) -> StackingSubject {
        StackingSubject::new(id, def_id, source, turns, stack)
    }

    // ── match_identity ─────────────────────────────────────

    #[test]
    fn unit_030_match_identity_same_def() {
        assert!(match_identity("eff_poison", "eff_poison"));
    }

    #[test]
    fn unit_031_match_identity_different_def() {
        assert!(!match_identity("eff_poison", "eff_haste"));
    }

    // ── StackingType::None ─────────────────────────────────

    #[test]
    fn unit_032_decide_none_rejects() {
        let existing = subject("existing", "eff_poison", "caster_001", 3, 1);
        let incoming = subject("incoming", "eff_poison", "caster_001", 3, 1);
        let config = StackingConfig::none();

        let decision = decide_stacking(&existing, &incoming, &config);
        assert_eq!(decision, StackingDecision::Reject);
    }

    // ── StackingType::Aggregate ────────────────────────────

    #[test]
    fn unit_033_decide_aggregate_accumulates() {
        let existing = subject("existing", "eff_poison", "caster_001", 3, 1);
        let incoming = subject("incoming", "eff_poison", "caster_001", 3, 1);
        let config = StackingConfig::aggregate(5, false).unwrap();

        let decision = decide_stacking(&existing, &incoming, &config);
        assert_eq!(
            decision,
            StackingDecision::Accumulate {
                new_stack_count: 2,
                added_layers: 1,
            }
        );
    }

    #[test]
    fn unit_034_decide_aggregate_capped() {
        let existing = subject("existing", "eff_poison", "caster_001", 3, 5);
        let incoming = subject("incoming", "eff_poison", "caster_001", 3, 1);
        let config = StackingConfig::aggregate(5, false).unwrap();

        let decision = decide_stacking(&existing, &incoming, &config);
        assert_eq!(decision, StackingDecision::Reject); // at max, overflow=IgnoreNew
    }

    #[test]
    fn unit_035_decide_aggregate_overflow_refresh() {
        let existing = subject("existing", "eff_poison", "caster_001", 2, 5);
        let incoming = subject("incoming", "eff_poison", "caster_001", 5, 1);
        let config = StackingConfig {
            stacking_type: StackingType::Aggregate,
            max_stacks: 5,
            allow_cross_source: false,
            overflow_behavior: OverflowBehavior::Refresh,
            reapply_modifiers_on_stack: true,
        };

        let decision = decide_stacking(&existing, &incoming, &config);
        assert_eq!(
            decision,
            StackingDecision::Refresh {
                refreshed_instance_id: "existing".into(),
                new_duration: 5, // max(2, 5)
            }
        );
    }

    #[test]
    fn unit_036_decide_aggregate_overflow_replace() {
        let existing = subject("existing", "eff_poison", "caster_001", 2, 5);
        let incoming = subject("incoming", "eff_poison", "caster_001", 5, 1);
        let config = StackingConfig {
            stacking_type: StackingType::Aggregate,
            max_stacks: 5,
            allow_cross_source: false,
            overflow_behavior: OverflowBehavior::Replace,
            reapply_modifiers_on_stack: true,
        };

        let decision = decide_stacking(&existing, &incoming, &config);
        assert_eq!(
            decision,
            StackingDecision::Replace {
                replaced_instance_id: "existing".into(),
            }
        );
    }

    // ── StackingType::RefreshDuration ──────────────────────

    #[test]
    fn unit_037_decide_refresh_takes_max_duration() {
        let existing = subject("existing", "eff_haste", "caster_001", 3, 1);
        let incoming = subject("incoming", "eff_haste", "caster_001", 5, 1);
        let config = StackingConfig::refresh();

        let decision = decide_stacking(&existing, &incoming, &config);
        assert_eq!(
            decision,
            StackingDecision::Refresh {
                refreshed_instance_id: "existing".into(),
                new_duration: 5,
            }
        );
    }

    #[test]
    fn unit_038_decide_refresh_keeps_longer() {
        let existing = subject("existing", "eff_haste", "caster_001", 10, 1);
        let incoming = subject("incoming", "eff_haste", "caster_001", 3, 1);
        let config = StackingConfig::refresh();

        let decision = decide_stacking(&existing, &incoming, &config);
        assert_eq!(
            decision,
            StackingDecision::Refresh {
                refreshed_instance_id: "existing".into(),
                new_duration: 10, // existing is longer
            }
        );
    }

    // ── StackingType::Replace ──────────────────────────────

    #[test]
    fn unit_039_decide_replace_replaces() {
        let existing = subject("existing", "eff_buff", "caster_001", 3, 1);
        let incoming = subject("incoming", "eff_buff", "caster_002", 5, 1);
        let config = StackingConfig::replace().unwrap();

        let decision = decide_stacking(&existing, &incoming, &config);
        assert_eq!(
            decision,
            StackingDecision::Replace {
                replaced_instance_id: "existing".into(),
            }
        );
    }

    // ── evaluate_stacking ──────────────────────────────────

    #[test]
    fn unit_040_evaluate_stacking_different_def_returns_none() {
        let existing = subject("existing", "eff_poison", "caster_001", 3, 1);
        let incoming = subject("incoming", "eff_haste", "caster_001", 5, 1);
        let config = StackingConfig::aggregate(5, false).unwrap();

        let result = evaluate_stacking(&existing, &incoming, &config);
        assert!(result.is_none());
    }

    #[test]
    fn unit_041_evaluate_stacking_same_def_returns_outcome() {
        let existing = subject("existing", "eff_poison", "caster_001", 3, 1);
        let incoming = subject("incoming", "eff_poison", "caster_001", 3, 1);
        let config = StackingConfig::aggregate(5, false).unwrap();

        let result = evaluate_stacking(&existing, &incoming, &config);
        assert!(result.is_some());
        let outcome = result.unwrap();
        assert_eq!(outcome.new_stack_count, 2);
        assert!(matches!(
            outcome.decision,
            StackingDecision::Accumulate { .. }
        ));
    }

    #[test]
    fn unit_042_evaluate_stacking_replace_returns_stack_one() {
        let existing = subject("existing", "eff_buff", "caster_001", 3, 3);
        let incoming = subject("incoming", "eff_buff", "caster_002", 5, 1);
        let config = StackingConfig::replace().unwrap();

        let result = evaluate_stacking(&existing, &incoming, &config);
        assert!(result.is_some());
        let outcome = result.unwrap();
        assert_eq!(outcome.new_stack_count, 1); // Replace resets to 1
    }

    // ── validate_config ────────────────────────────────────

    #[test]
    fn unit_043_validate_config_none_ok() {
        assert!(validate_config(&StackingConfig::none()).is_ok());
    }

    #[test]
    fn unit_044_validate_config_aggregate_ok() {
        assert!(validate_config(&StackingConfig::aggregate(5, false).unwrap()).is_ok());
    }

    #[test]
    fn unit_045_validate_config_aggregate_too_small() {
        let config = StackingConfig {
            stacking_type: StackingType::Aggregate,
            max_stacks: 1,
            allow_cross_source: false,
            overflow_behavior: OverflowBehavior::IgnoreNew,
            reapply_modifiers_on_stack: true,
        };
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn unit_046_validate_config_replace_wrong_max() {
        let config = StackingConfig {
            stacking_type: StackingType::Replace,
            max_stacks: 3,
            allow_cross_source: true,
            overflow_behavior: OverflowBehavior::Replace,
            reapply_modifiers_on_stack: true,
        };
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn unit_047_validate_config_refresh_ok() {
        assert!(validate_config(&StackingConfig::refresh()).is_ok());
    }

    #[test]
    fn unit_048_validate_config_zero_max() {
        let config = StackingConfig {
            stacking_type: StackingType::None,
            max_stacks: 0,
            allow_cross_source: false,
            overflow_behavior: OverflowBehavior::IgnoreNew,
            reapply_modifiers_on_stack: false,
        };
        assert!(validate_config(&config).is_err());
    }
}
