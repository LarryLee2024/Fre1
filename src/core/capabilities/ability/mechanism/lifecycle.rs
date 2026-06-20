//! Ability 生命周期管理
//!
//! 纯函数式实现技能激活、状态转换、取消打断、冷却管理的完整流程。
//! 遵循 docs/02-domain/capabilities/ability_domain.md §5（流程定义）中的约束。
//!
//! 核心流程：
//! 1. try_activate() — 条件检查 → 消耗 → 创建实例
//! 2. transition_to() — 状态机转换
//! 3. cancel_ability() — 取消/打断
//! 4. complete_ability() — 执行完成
//! 5. start_cooldown() — 冷却管理
//! 6. apply_block() / remove_block() — 封锁/恢复

use std::sync::Mutex;

use bevy::prelude::*;

use crate::core::capabilities::ability::events::{
    AbilityActivated, AbilityCancelled, AbilityCompleted, AbilityCooldownStarted,
};
use crate::core::capabilities::ability::foundation::{
    AbilityError, AbilityInstance, AbilityInstanceId, AbilityState, ActivationContext,
    ActivationType, BlockedRestoreState, CooldownEntry, CostEntry,
};
use crate::core::capabilities::ability::mechanism::components::ActiveAbilityContainer;
use crate::shared::ids::runtime_id::RuntimeIdAllocator;

/// 技能实例 ID 生成器（Resource）。
/// 通过 RuntimeIdAllocator 提供带 generation 保护的唯一 ID 分配。
#[derive(Resource, Debug)]
pub struct AbilityInstanceIdGenerator {
    allocator: Mutex<RuntimeIdAllocator>,
}

impl Default for AbilityInstanceIdGenerator {
    fn default() -> Self {
        Self {
            allocator: Mutex::new(RuntimeIdAllocator::new()),
        }
    }
}

impl AbilityInstanceIdGenerator {
    /// 分配一个新的唯一 AbilityInstanceId。
    pub fn next_id(&self) -> AbilityInstanceId {
        let mut alloc = self.allocator.lock().expect("lock poisoned");
        AbilityInstanceId::new(alloc.alloc())
    }

    /// 回收一个 AbilityInstanceId（generation 会在下次分配时递增）。
    pub fn free(&self, id: AbilityInstanceId) {
        let mut alloc = self.allocator.lock().expect("lock poisoned");
        alloc.free(id.runtime_id());
    }
}

// ============================================================================
// 激活流程
// ============================================================================

/// 技能激活请求的输入参数。
#[derive(Debug, Clone)]
pub struct ActivationRequest {
    /// 关联的 Spec ID
    pub spec_id: String,
    /// 引用的 AbilityDef ID
    pub def_id: String,
    /// 激活类型
    pub activation: ActivationType,
    /// 激活上下文（施法者、目标、帧号）
    pub context: ActivationContext,
    /// 消耗列表
    pub costs: Vec<CostEntry>,
}

/// 尝试激活一个技能。
///
/// 完整流程（docs/02-domain/capabilities/ability_domain.md §5.1）：
/// 1. 检查技能是否在冷却中（不变量 3.3）
/// 2. 检查是否有同 Spec 的活跃实例（不变量 V5）
/// 3. 创建 AbilityInstance
/// 4. 注册实例到容器
///
/// 注意：Condition 检查和 Cost 消耗由外部调用方负责，
/// 该函数仅负责状态管理（条件先于消耗的原则在外部保证）。
///
/// # Errors
///
/// 返回 AbilityError 的各种错误变体。
pub fn try_activate(
    container: &mut ActiveAbilityContainer,
    request: ActivationRequest,
    entity: Entity,
    commands: &mut Commands,
    generator: &AbilityInstanceIdGenerator,
) -> Result<AbilityInstanceId, AbilityError> {
    let spec_id = &request.spec_id;

    // 1. 冷却检查（不变量 3.3）
    if container.is_on_cooldown(spec_id) {
        let remaining = container.cooldown_remaining(spec_id);
        return Err(AbilityError::OnCooldown {
            spec_id: spec_id.clone(),
            remaining_turns: remaining,
        });
    }

    // 2. 唯一实例检查（不变量 V5）
    if container.has_active_instance(spec_id) {
        let existing = container
            .find_instance_by_spec(spec_id)
            .expect("has_active_instance returned true but no instance found");
        return Err(AbilityError::AlreadyActive {
            spec_id: spec_id.clone(),
            instance_id: existing.instance_id,
        });
    }

    // 3. 创建 AbilityInstance（使用 generator 分配 ID，确保 generation safety）
    let instance_id = generator.next_id();
    let mut instance = AbilityInstance::new(
        instance_id,
        spec_id.clone(),
        &request.def_id,
        request.activation,
        request.context,
    );

    // 4. 添加消耗追踪
    for cost in request.costs {
        instance.add_cost(cost);
    }

    // 5. 注册到容器
    let def_id = instance.def_id.clone();
    container.insert_instance(instance);

    commands.trigger(AbilityActivated {
        entity,
        spec_id: spec_id.clone(),
        def_id,
        instance_id,
        context_desc: format!("{:?}", request.activation),
    });

    Ok(instance_id)
}

// ============================================================================
// 状态转换
// ============================================================================

/// 执行合法的状态转换。
///
/// 校验规则（docs/02-domain/capabilities/ability_domain.md §1，状态转换表）：
/// - Ready → Casting/Active: 激活流程通过
/// - Casting → Active: 施法完成
/// - Casting → Ready: 施法被打断/取消
/// - Active → Cooldown: 执行完毕
/// - Cooldown → Ready: 冷却到期
/// - 任何 → Removed: 技能移除
///
/// # Errors
/// - `AbilityError::InvalidTransition` 当转换不合法时。
pub fn transition_to(
    container: &mut ActiveAbilityContainer,
    instance_id: &AbilityInstanceId,
    new_state: AbilityState,
) -> Result<(), AbilityError> {
    let instance = container
        .get_instance_mut(instance_id)
        .ok_or(AbilityError::InstanceNotFound(*instance_id))?;

    let old_state = instance.state;

    // 验证转换合法性
    validate_transition(old_state, new_state)?;

    instance.state = new_state;
    Ok(())
}

/// 校验状态转换的合法性（纯函数）。
fn validate_transition(from: AbilityState, to: AbilityState) -> Result<(), AbilityError> {
    let valid = match (from, to) {
        // 激活转换
        (AbilityState::Ready, AbilityState::Casting) => true,
        (AbilityState::Ready, AbilityState::Active) => true,
        // 施法完成
        (AbilityState::Casting, AbilityState::Active) => true,
        // 取消/打断
        (AbilityState::Casting, AbilityState::Ready) => true,
        // 执行完毕进入冷却
        (AbilityState::Active, AbilityState::Cooldown) => true,
        // 冷却到期
        (AbilityState::Cooldown, AbilityState::Ready) => true,
        // 任何状态到 Removed
        (_, AbilityState::Removed) => true,
        // Blocked <-> 原状态（由 apply_block/remove_block 管理，不在普通转换中处理）
        _ => false,
    };

    if valid {
        Ok(())
    } else {
        Err(AbilityError::InvalidTransition {
            from,
            to,
            reason: format!(
                "transition from {} to {} is not allowed by ability state machine",
                from.name(),
                to.name()
            ),
        })
    }
}

// ============================================================================
// 取消/打断
// ============================================================================

/// 取消/打断一个技能。
///
/// 流程（docs/02-domain/capabilities/ability_domain.md §5.3）：
/// - Casting 阶段: 打点 → 回到 Ready（Cost 回退由外部负责）
/// - Active 阶段: 标记为 Removed
///
/// # Errors
/// - `AbilityError::InstanceNotFound` 当实例不存在时。
/// - `AbilityError::InvalidTransition` 当实例状态不支持取消时（如 Ready/Cooldown）。
pub fn cancel_ability(
    container: &mut ActiveAbilityContainer,
    instance_id: &AbilityInstanceId,
    entity: Entity,
    commands: &mut Commands,
) -> Result<(), AbilityError> {
    let instance = container
        .get_instance_mut(instance_id)
        .ok_or(AbilityError::InstanceNotFound(*instance_id))?;

    match instance.state {
        AbilityState::Casting => {
            // 打断施法，回到 Ready
            instance.state = AbilityState::Ready;
            instance.cast_progress = 0;
            let spec_id = instance.spec_id.clone();
            let def_id = instance.def_id.clone();
            commands.trigger(AbilityCancelled {
                entity,
                spec_id,
                def_id,
                instance_id: *instance_id,
                reason: "cancelled during casting".into(),
            });
            Ok(())
        }
        AbilityState::Active => {
            // 终止执行
            instance.state = AbilityState::Removed;
            let spec_id = instance.spec_id.clone();
            let def_id = instance.def_id.clone();
            commands.trigger(AbilityCancelled {
                entity,
                spec_id,
                def_id,
                instance_id: *instance_id,
                reason: "cancelled during execution".into(),
            });
            Ok(())
        }
        _ => Err(AbilityError::InvalidTransition {
            from: instance.state,
            to: AbilityState::Removed,
            reason: format!(
                "ability in state {} cannot be cancelled",
                instance.state.name()
            ),
        }),
    }
}

// ============================================================================
// 完成执行
// ============================================================================

/// 标记技能执行完毕，进入冷却阶段。
///
/// 流程（docs/02-domain/capabilities/ability_domain.md §5.2）：
/// 1. 校验状态为 Active
/// 2. 移除实例（执行完毕）
/// 3. 创建冷却条目
///
/// # Errors
/// - `AbilityError::InstanceNotFound` 当实例不存在时。
/// - `AbilityError::InvalidTransition` 当实例状态不是 Active 时。
pub fn complete_ability(
    container: &mut ActiveAbilityContainer,
    instance_id: &AbilityInstanceId,
    cooldown_turns: u32,
    entity: Entity,
    commands: &mut Commands,
) -> Result<(), AbilityError> {
    let instance = container
        .get_instance(instance_id)
        .ok_or(AbilityError::InstanceNotFound(*instance_id))?;

    // 校验状态
    if instance.state != AbilityState::Active {
        return Err(AbilityError::InvalidTransition {
            from: instance.state,
            to: AbilityState::Cooldown,
            reason: format!("cannot complete ability in state {}", instance.state.name()),
        });
    }

    let spec_id = instance.spec_id.clone();
    let def_id = instance.def_id.clone();

    // 移除实例
    container.remove_instance(instance_id);

    // 创建冷却
    if cooldown_turns > 0 {
        let cooldown = CooldownEntry::new(&spec_id, cooldown_turns);
        container.set_cooldown(cooldown);
    }

    commands.trigger(AbilityCompleted {
        entity,
        spec_id: spec_id.clone(),
        def_id,
        instance_id: *instance_id,
        result: "success".into(),
    });

    if cooldown_turns > 0 {
        commands.trigger(AbilityCooldownStarted {
            entity,
            spec_id,
            cooldown_duration: cooldown_turns,
            shared_group: None,
        });
    }

    Ok(())
}

// ============================================================================
// 冷却管理
// ============================================================================

/// 手动启动冷却（用于技能执行完毕后额外触发冷却的场景）。
///
/// 如果已经存在同 spec_id 的冷却，则覆盖（更新回合数）。
pub fn start_cooldown(
    container: &mut ActiveAbilityContainer,
    commands: &mut Commands,
    entity: Entity,
    spec_id: impl Into<String>,
    turns: u32,
) {
    if turns > 0 {
        let spec: String = spec_id.into();
        let cooldown = CooldownEntry::new(&spec, turns);
        container.set_cooldown(cooldown);
        commands.trigger(AbilityCooldownStarted {
            entity,
            spec_id: spec,
            cooldown_duration: turns,
            shared_group: None,
        });
    }
}

/// 推进所有冷却 1 回合。返回本回合到期的 spec_id 列表。
///
/// 自动处理：
/// - 到期的单技能冷却被移除
/// - 到期的共享冷却组被移除
pub fn tick_cooldowns(container: &mut ActiveAbilityContainer) -> Vec<String> {
    let expired = container.tick_all_cooldowns();
    container.tick_shared_cooldowns();
    expired
}

/// 强制清除冷却（用于调试或特殊效果）。
pub fn force_reset_cooldown(container: &mut ActiveAbilityContainer, spec_id: &str) {
    container.remove_cooldown(spec_id);
}

/// 批量添加冷却（从 SpecContainer 中读取多个 Spec 的冷却配置）。
pub fn start_multiple_cooldowns(
    container: &mut ActiveAbilityContainer,
    commands: &mut Commands,
    entity: Entity,
    cooldowns: Vec<(String, u32)>,
) {
    for (spec_id, turns) in cooldowns {
        start_cooldown(container, commands, entity, spec_id, turns);
    }
}

// ============================================================================
// Blocked 状态管理
// ============================================================================

/// 应用封锁效果（沉默/眩晕/石化等）。
///
/// 流程（docs/02-domain/capabilities/ability_domain.md §1，Blocked 状态）：
/// 1. 找到所有活跃实例
/// 2. 记录每个实例的当前状态（用于恢复）
/// 3. 将实例状态设为 Blocked
pub fn apply_block(container: &mut ActiveAbilityContainer) {
    let to_block: Vec<(AbilityInstanceId, AbilityState)> = container
        .active_instances
        .iter()
        .filter(|(_, inst)| matches!(inst.state, AbilityState::Casting | AbilityState::Active))
        .map(|(id, inst)| (*id, inst.state))
        .collect();

    for (instance_id, original_state) in to_block {
        let spec_id = {
            let instance = container
                .get_instance_mut(&instance_id)
                .expect("instance disappeared during block");
            instance.state = AbilityState::Blocked;
            instance.paused = true;
            instance.spec_id.clone()
        };

        let restore = match original_state {
            AbilityState::Casting => BlockedRestoreState::Casting,
            AbilityState::Active => BlockedRestoreState::Active,
            _ => BlockedRestoreState::Ready,
        };
        container.set_blocked_restore(spec_id, restore);
    }
}

/// 移除封锁效果，恢复技能到之前的状态。
///
/// 流程：
/// 1. 找到所有被 Blocked 的实例
/// 2. 根据记录的 restore 状态恢复
/// 3. 清理 restore 记录
pub fn remove_block(container: &mut ActiveAbilityContainer) {
    let to_restore: Vec<(AbilityInstanceId, String)> = container
        .active_instances
        .iter()
        .filter(|(_, inst)| inst.state == AbilityState::Blocked)
        .map(|(id, inst)| (*id, inst.spec_id.clone()))
        .collect();

    for (instance_id, spec_id) in to_restore {
        let restore = container.take_blocked_restore(&spec_id);

        let instance = container
            .get_instance_mut(&instance_id)
            .expect("instance disappeared during restore");

        instance.paused = false;
        instance.state = restore.map(|r| r.to_state()).unwrap_or(AbilityState::Ready);
    }
}

/// 检查实体是否有被封锁的技能。
pub fn is_blocked(container: &ActiveAbilityContainer) -> bool {
    container.has_blocked_abilities()
}

// ============================================================================
// 施法进度管理
// ============================================================================

/// 推进指定实例的施法进度。
///
/// 返回 true 表示施法完成（可转换到 Active）。
pub fn advance_cast_progress(
    container: &mut ActiveAbilityContainer,
    instance_id: &AbilityInstanceId,
    delta: u64,
) -> Result<bool, AbilityError> {
    let instance = container
        .get_instance_mut(instance_id)
        .ok_or(AbilityError::InstanceNotFound(*instance_id))?;

    if instance.state != AbilityState::Casting {
        return Err(AbilityError::InvalidTransition {
            from: instance.state,
            to: AbilityState::Active,
            reason: format!(
                "cannot advance cast progress in state {}",
                instance.state.name()
            ),
        });
    }

    Ok(instance.advance_cast(delta))
}

// ============================================================================
// 查询函数
// ============================================================================

/// 获取所有非冷却状态的技能 Spec ID。
pub fn get_ready_abilities(
    container: &ActiveAbilityContainer,
    all_specs: &[String],
) -> Vec<String> {
    all_specs
        .iter()
        .filter(|spec_id| {
            !container.is_on_cooldown(spec_id) && !container.has_active_instance(spec_id)
        })
        .cloned()
        .collect()
}
