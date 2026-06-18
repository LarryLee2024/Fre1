//! Ability Integration — combat 域接入 ability capability
//!
//! 通过 Facade 模式封装 ability capability 的调用，
//! 隔离 combat 域与 ability capability 的直接依赖。
//!
//! 详见 ADR-024 §2

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::capabilities::ability::foundation::{
    AbilityError, AbilityInstanceId, ActivationContext, ActivationType, CostEntry,
};
use crate::core::capabilities::ability::mechanism::{
    ActivationRequest, ActiveAbilityContainer, complete_ability, tick_cooldowns, try_activate,
};

// ─── Facade ────────────────────────────────────────────────────────

/// 战斗能力 Facade — 封装 ability capability 的战斗相关操作。
pub struct CombatAbilityFacade;

impl CombatAbilityFacade {
    /// 尝试激活一个战斗技能。
    pub fn try_activate_ability(
        container: &mut ActiveAbilityContainer,
        spec_id: &str,
        def_id: &str,
        caster_entity: Entity,
        target_entity: Entity,
        frame: u64,
        costs: Vec<CostEntry>,
    ) -> Result<AbilityInstanceId, AbilityError> {
        let request = ActivationRequest {
            spec_id: spec_id.to_string(),
            def_id: def_id.to_string(),
            activation: ActivationType::Instant,
            context: ActivationContext::new(format!("{:?}", caster_entity), frame)
                .with_target(format!("{:?}", target_entity)),
            costs,
        };
        try_activate(container, request)
    }

    /// 完成一个技能并进入冷却。
    pub fn complete_and_cooldown(
        container: &mut ActiveAbilityContainer,
        instance_id: &AbilityInstanceId,
        cooldown_turns: u32,
    ) -> Result<(), AbilityError> {
        complete_ability(container, instance_id, cooldown_turns)
    }

    /// 推进所有冷却。返回本回合到期的 spec_id 列表。
    pub fn tick_all_cooldowns(container: &mut ActiveAbilityContainer) -> Vec<String> {
        tick_cooldowns(container)
    }
}

// ─── SystemParam ───────────────────────────────────────────────────

/// 战斗能力 SystemParam — 在 System 中便捷访问 ability capability。
#[derive(SystemParam)]
pub struct CombatAbilityParam<'w, 's> {
    pub containers: Query<'w, 's, &'static mut ActiveAbilityContainer>,
}

impl<'w, 's> CombatAbilityParam<'w, 's> {
    /// 为指定实体激活技能。
    pub fn activate(
        &mut self,
        entity: Entity,
        spec_id: &str,
        def_id: &str,
        target: Entity,
        frame: u64,
        costs: Vec<CostEntry>,
    ) -> Result<AbilityInstanceId, AbilityError> {
        let mut container = self.containers.get_mut(entity).map_err(|e| {
            AbilityError::Runtime(format!(
                "entity {:?} has no ActiveAbilityContainer: {}",
                entity, e
            ))
        })?;
        CombatAbilityFacade::try_activate_ability(
            &mut container,
            spec_id,
            def_id,
            entity,
            target,
            frame,
            costs,
        )
    }
}
