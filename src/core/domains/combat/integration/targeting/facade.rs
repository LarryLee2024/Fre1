//! Targeting Integration — combat 域接入 targeting capability
//!
//! 封装 targeting capability 的目标选择功能，
//! 用于战斗中的技能目标校验与选择流程。
//!
//! 详见 ADR-024 §2

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::capabilities::targeting::foundation::{
    TargetContext, TargetData, TargetShape, TargetType, TargetingDef, TargetingError,
};
use crate::core::capabilities::targeting::mechanism::CandidateTarget;
use crate::core::capabilities::targeting::mechanism::{select_targets, validate_targeting_def};

// ─── Facade ────────────────────────────────────────────────────────

/// 战斗目标选择 Facade — 封装 targeting capability 的战斗相关操作。
pub struct CombatTargetingFacade;

impl CombatTargetingFacade {
    /// 执行目标选择，返回符合 TargetingDef 的目标列表。
    ///
    /// # Errors
    /// - `TargetingError::NoValidTargets` — 无合法目标
    /// - `TargetingError::InvalidShapeParameter` — 配置参数非法
    pub fn select_targets(
        def: &TargetingDef,
        candidates: Vec<CandidateTarget>,
        context: TargetContext,
    ) -> Result<TargetData, TargetingError> {
        select_targets(def, candidates, context)
    }

    /// 校验 TargetingDef 参数合法性。
    pub fn validate_def(def: &TargetingDef) -> Result<(), TargetingError> {
        validate_targeting_def(def)
    }

    /// 创建默认的单体目标定义（对敌方，可指定射程）。
    pub fn single_target_def(range: Option<f32>) -> TargetingDef {
        TargetingDef::new(TargetType::Enemy, TargetShape::Single, range, 1)
            .expect("default single target def should be valid")
    }

    /// 创建目标选择上下文。
    pub fn create_target_context(
        caster_entity: impl Into<String>,
        caster_faction: impl Into<String>,
        frame: u64,
    ) -> TargetContext {
        TargetContext::new(caster_entity, caster_faction, frame)
    }
}

// ─── SystemParam ───────────────────────────────────────────────────

/// 战斗目标选择 SystemParam — 在 System 中便捷访问 target 选择能力。
#[derive(SystemParam)]
pub struct CombatTargetingParam<'w, 's> {
    _marker: std::marker::PhantomData<(&'w (), &'s ())>,
}

impl<'w, 's> CombatTargetingParam<'w, 's> {
    /// 创建候选目标。
    pub fn create_candidate(entity_id: impl Into<String>) -> CandidateTarget {
        CandidateTarget::new(entity_id)
    }
}
