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
    AbilityInstanceIdGenerator, ActivationIssue, ActivationRequest, ActiveAbilityContainer,
    complete_ability, tick_cooldowns, try_activate,
};

// ─── Facade ────────────────────────────────────────────────────────

/// 战斗能力 Facade — 封装 ability capability 的战斗相关操作。
pub struct CombatAbilityFacade;

impl CombatAbilityFacade {
    // ─── WriteFacade ──────────────────────────────────────────────────

    /// 创建一个空的活跃技能容器（用于组件插入）。
    pub fn empty_container() -> ActiveAbilityContainer {
        ActiveAbilityContainer::empty()
    }

    /// 尝试激活一个战斗技能。
    pub fn try_activate_ability(
        container: &mut ActiveAbilityContainer,
        spec_id: &str,
        def_id: &str,
        caster_entity: Entity,
        target_entity: Entity,
        frame: u64,
        costs: Vec<CostEntry>,
        commands: &mut Commands,
        generator: &AbilityInstanceIdGenerator,
    ) -> Result<AbilityInstanceId, ActivationIssue> {
        let request = ActivationRequest {
            spec_id: spec_id.to_string(),
            def_id: def_id.to_string(),
            activation: ActivationType::Instant,
            context: ActivationContext::new(format!("{:?}", caster_entity), frame)
                .with_target(format!("{:?}", target_entity)),
            costs,
        };
        try_activate(container, request, caster_entity, commands, generator)
    }

    /// 完成一个技能并进入冷却。
    pub fn complete_and_cooldown(
        container: &mut ActiveAbilityContainer,
        instance_id: &AbilityInstanceId,
        cooldown_turns: u32,
        entity: Entity,
        commands: &mut Commands,
    ) -> Result<(), AbilityError> {
        complete_ability(container, instance_id, cooldown_turns, entity, commands)
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
    pub generator: Res<'w, AbilityInstanceIdGenerator>,
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
        commands: &mut Commands,
    ) -> Result<AbilityInstanceId, ActivationIssue> {
        let mut container = self.containers.get_mut(entity).map_err(|e| {
            ActivationIssue::Error(AbilityError::ContainerMissing {
                detail: format!("entity {:?} has no ActiveAbilityContainer: {}", entity, e),
            })
        })?;
        CombatAbilityFacade::try_activate_ability(
            &mut container,
            spec_id,
            def_id,
            entity,
            target,
            frame,
            costs,
            commands,
            &self.generator,
        )
    }

    /// 推进指定实体的所有冷却（技能冷却 + 共享冷却）。
    ///
    /// 返回本回合到期的技能 spec_id 列表。
    pub fn tick_cooldowns_for_unit(&mut self, entity: Entity) -> Vec<String> {
        if let Ok(mut container) = self.containers.get_mut(entity) {
            CombatAbilityFacade::tick_all_cooldowns(&mut container)
        } else {
            Vec::new()
        }
    }
}
