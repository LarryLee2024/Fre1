//! Aggregator Integration — combat 域接入 aggregator capability
//!
//! 封装 aggregator capability 的属性聚合功能，
//! 用于战斗中在 Modifier 变更后重新计算属性值。
//!
//! 详见 ADR-024 §2

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::capabilities::aggregator::events::AggregateDirty;
use crate::core::capabilities::aggregator::foundation::{
    AggregationResult, CalcPipeline, ModifierEntry, ModifierOp as AggregatorModifierOp,
    PipelineError, default_stages,
};
use crate::core::capabilities::aggregator::mechanism::pipeline::execute_aggregation;
use crate::core::capabilities::aggregator::mechanism::{AggregatorState, mark_dirty};

// ─── Facade ────────────────────────────────────────────────────────

/// 战斗属性聚合 Facade — 封装 aggregator capability 的战斗相关操作。
pub struct CombatAggregatorFacade;

impl CombatAggregatorFacade {
    // ─── ReadFacade ───────────────────────────────────────────────────

    /// 执行属性聚合——将 base_value 与 modifiers 计算为 final_value。
    ///
    /// 使用指定的聚合管线。
    ///
    /// # Errors
    /// - `PipelineError::EmptyModifiers` — 无可用的 modifier
    /// - `PipelineError::InvalidOperation` — 不支持的 modifier 操作
    pub fn execute_aggregation(
        attribute_id: &str,
        base_value: f32,
        modifiers: &[ModifierEntry],
        pipeline: &CalcPipeline,
        min_value: f32,
        max_value: f32,
        frame: u64,
        entity: Entity,
        commands: &mut Commands,
    ) -> Result<AggregationResult, PipelineError> {
        execute_aggregation(
            attribute_id,
            base_value,
            modifiers,
            pipeline,
            min_value,
            max_value,
            frame,
            entity,
            commands,
        )
    }

    /// 使用默认管线执行聚合（Add → Multiply → Override → Clamp, 无边界限制）。
    pub fn execute_default_aggregation(
        attribute_id: &str,
        base_value: f32,
        modifiers: &[ModifierEntry],
        frame: u64,
        entity: Entity,
        commands: &mut Commands,
    ) -> Result<AggregationResult, PipelineError> {
        let pipeline = CalcPipeline {
            attribute_id: attribute_id.to_string(),
            enabled_stages: default_stages(),
            priority_ascending: true,
            clamp_override: None,
            cycle_detection: true,
        };
        execute_aggregation(
            attribute_id,
            base_value,
            modifiers,
            &pipeline,
            f32::NEG_INFINITY,
            f32::INFINITY,
            frame,
            entity,
            commands,
        )
    }

    // ─── WriteFacade ──────────────────────────────────────────────────

    /// 标记指定属性为脏（需要重算）。
    pub fn mark_attribute_dirty(
        state: &mut AggregatorState,
        attribute_id: &str,
        trigger_source: &str,
        frame: u64,
        entity: Entity,
        commands: &mut Commands,
    ) {
        mark_dirty(state, attribute_id, trigger_source, frame, entity, commands);
    }

    // ─── ReadFacade ───────────────────────────────────────────────────

    /// 创建默认的聚合管线。
    pub fn default_pipeline(attribute_id: &str) -> CalcPipeline {
        CalcPipeline {
            attribute_id: attribute_id.to_string(),
            enabled_stages: default_stages(),
            priority_ascending: true,
            clamp_override: None,
            cycle_detection: true,
        }
    }

    /// 创建 ModifierEntry。
    pub fn create_modifier_entry(
        op: AggregatorModifierOp,
        magnitude: f32,
        priority: u8,
        target_attribute: String,
    ) -> ModifierEntry {
        ModifierEntry {
            op,
            magnitude,
            priority,
            target_attribute,
        }
    }
}

// ─── SystemParam ───────────────────────────────────────────────────

/// 战斗属性聚合 SystemParam — 在 System 中便捷访问 aggregator capability。
#[derive(SystemParam)]
pub struct CombatAggregatorParam<'w, 's> {
    pub commands: Commands<'w, 's>,
}

impl<'w, 's> CombatAggregatorParam<'w, 's> {
    /// 触发属性重算（通过 AggregateDirty 事件）。
    pub fn request_recalc(&mut self, entity: Entity, attribute_id: &str, trigger: &str) {
        self.commands.trigger(AggregateDirty {
            entity,
            attribute_id: attribute_id.to_string(),
            trigger_source: trigger.to_string(),
        });
    }
}
