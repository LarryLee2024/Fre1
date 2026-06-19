//! Execution Integration — combat 域接入 execution capability
//!
//! 封装 execution capability 的执行计算功能，
//! 用于战斗中的伤害/治疗数值结算。
//!
//! 详见 ADR-024 §2

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::capabilities::execution::foundation::{
    DamageParams, ExecutionContext, ExecutionError, ExecutionResult, ExecutionType,
};
use crate::core::capabilities::execution::mechanism::{execute, validate_context};

// ─── Facade ────────────────────────────────────────────────────────

/// 战斗执行 Facade — 封装 execution capability 的战斗相关操作。
pub struct CombatExecutionFacade;

impl CombatExecutionFacade {
    /// 根据 ExecutionType 执行计算（自动分发）。
    ///
    /// # Errors
    /// - `ExecutionError::ContextMissing` — 上下文数据不完整
    /// - `ExecutionError::UnsupportedExecutionType` — 不支持的执行类型
    pub fn execute(ctx: &ExecutionContext, commands: &mut Commands) -> Result<ExecutionResult, ExecutionError> {
        execute(ctx, commands)
    }

    /// 校验执行上下文是否完整。
    pub fn validate_context(ctx: &ExecutionContext) -> Result<(), ExecutionError> {
        validate_context(ctx)
    }

    /// 构建默认的战斗伤害执行上下文。
    pub fn build_damage_context(
        source_entity: impl Into<String>,
        target_entity: impl Into<String>,
        damage_params: DamageParams,
        source_atk: f32,
        target_def: f32,
    ) -> ExecutionContext {
        let mut source_attrs = std::collections::HashMap::new();
        source_attrs.insert("attack".to_string(), source_atk);
        let mut target_attrs = std::collections::HashMap::new();
        target_attrs.insert("defense".to_string(), target_def);

        ExecutionContext::new(
            ExecutionType::Damage(damage_params),
            source_entity,
            target_entity,
        )
        .with_source_attributes(source_attrs)
        .with_target_attributes(target_attrs)
    }
}

// ─── SystemParam ───────────────────────────────────────────────────

/// 战斗执行 SystemParam — 在 System 中便捷访问 execution capability。
#[derive(SystemParam)]
pub struct CombatExecutionParam<'w, 's> {
    _marker: std::marker::PhantomData<(&'w (), &'s ())>,
}
