//! GameplayContext Integration — combat 域接入 gameplay_context capability
//!
//! 封装 gameplay_context capability 的上下文构建功能，
//! 用于战斗中统一创建攻击/技能/效果的全链路上下文。
//!
//! 详见 ADR-024 §2

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::capabilities::gameplay_context::foundation::{
    ContextBuildError, ContextOrigin, GameplayContextData, SourceInfo, TargetInfo,
};
use crate::core::capabilities::gameplay_context::mechanism::ContextBuilder;

// ─── Facade ────────────────────────────────────────────────────────

/// 战斗上下文 Facade — 封装 gameplay_context capability 的战斗相关操作。
pub struct CombatContextFacade;

impl CombatContextFacade {
    /// 构建战斗攻击上下文（最常见的 combat context 类型）。
    ///
    /// # Errors
    /// - `ContextBuildError::MissingFields` — source 或 target 未设置
    pub fn build_attack_context(
        source_entity: Entity,
        source_faction: &str,
        source_pos: Option<(i32, i32)>,
        target_entity: Entity,
        target_faction: &str,
        target_pos: Option<(i32, i32)>,
        ability_id: Option<&str>,
        frame: u64,
    ) -> Result<GameplayContextData, ContextBuildError> {
        ContextBuilder::new(ContextOrigin::Direct, frame)
            .source(SourceInfo {
                entity: source_entity,
                faction: source_faction.to_string(),
                position: source_pos,
            })
            .target(TargetInfo {
                entity: target_entity,
                faction: target_faction.to_string(),
                position: target_pos,
                is_valid: true,
            })
            .ability(ability_id.unwrap_or(""))
            .build()
    }

    /// 构建反击/连锁反应上下文。
    pub fn build_reaction_context(
        source_entity: Entity,
        source_faction: &str,
        target_entity: Entity,
        target_faction: &str,
        origin: ContextOrigin,
        frame: u64,
    ) -> Result<GameplayContextData, ContextBuildError> {
        ContextBuilder::new(origin, frame)
            .source(SourceInfo {
                entity: source_entity,
                faction: source_faction.to_string(),
                position: None,
            })
            .target(TargetInfo {
                entity: target_entity,
                faction: target_faction.to_string(),
                position: None,
                is_valid: true,
            })
            .build()
    }

    /// 构建周期性效果（DoT/HoT）上下文。
    pub fn build_periodic_context(
        source_entity: Entity,
        target_entity: Entity,
        ability_id: Option<&str>,
        frame: u64,
    ) -> Result<GameplayContextData, ContextBuildError> {
        ContextBuilder::new(ContextOrigin::Periodic, frame)
            .source(SourceInfo {
                entity: source_entity,
                faction: String::new(),
                position: None,
            })
            .target(TargetInfo {
                entity: target_entity,
                faction: String::new(),
                position: None,
                is_valid: true,
            })
            .ability(ability_id.unwrap_or(""))
            .build()
    }

    /// 获取上下文中的发起者 entity。
    pub fn source_entity(context: &GameplayContextData) -> Entity {
        context.source.entity
    }

    /// 获取上下文中的目标 entity。
    pub fn target_entity(context: &GameplayContextData) -> Entity {
        context.target.entity
    }
}

// ─── SystemParam ───────────────────────────────────────────────────

/// 战斗上下文 SystemParam — 在 System 中便捷构建 gameplay context。
#[derive(SystemParam)]
pub struct CombatContextParam<'w, 's> {
    _marker: std::marker::PhantomData<(&'w (), &'s ())>,
}
