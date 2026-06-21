//! Ability ECS 组件
//!
//! ActiveAbilityContainer 是挂载在实体上的技能运行时容器组件，
//! 管理该实体的所有活跃技能实例和冷却状态。
//!
//! 依赖 foundation::values::{AbilityInstance, CooldownEntry, BlockedRestoreState}。
//! 详见 docs/04-data/capabilities/ability_schema.md §3.7。

use std::collections::HashMap;

use bevy::prelude::*;

use crate::core::capabilities::ability::foundation::{
    AbilityInstance, AbilityInstanceId, BlockedRestoreState, CooldownEntry,
};

/// 挂载在实体上的活跃技能容器组件。
///
/// 管理实体的所有活跃技能实例、冷却状态、共享冷却组。
/// 提供基于 spec_id 和 instance_id 的快速查询。
#[derive(Component, Debug, Clone)]
pub struct ActiveAbilityContainer {
    /// 所有活跃的技能实例（包括 Casting/Active/Blocked 等）
    pub active_instances: HashMap<AbilityInstanceId, AbilityInstance>,
    /// 冷却状态（spec_id → CooldownEntry）
    pub cooldowns: HashMap<String, CooldownEntry>,
    /// 共享冷却组状态（group_name → 剩余回合数）
    pub shared_cooldowns: HashMap<String, u32>,
    /// blocked 状态下保存的原始状态（spec_id → 恢复目标状态）
    pub blocked_restore: HashMap<String, BlockedRestoreState>,
}

impl ActiveAbilityContainer {
    /// 创建一个空的 ActiveAbilityContainer。四个容器/映射全部零初始化。
    pub fn empty() -> Self {
        Self {
            active_instances: HashMap::new(),
            cooldowns: HashMap::new(),
            shared_cooldowns: HashMap::new(),
            blocked_restore: HashMap::new(),
        }
    }

    // ── 实例查询 ───────────────────────────────────────────

    /// 用于 AbilitySystem 查询实例状态（Casting/Active/Blocked）。
    pub fn get_instance(&self, instance_id: &AbilityInstanceId) -> Option<&AbilityInstance> {
        self.active_instances.get(instance_id)
    }

    /// 用于 AbilitySystem 修改实例阶段（Casting→Active→Completed）。
    pub fn get_instance_mut(
        &mut self,
        instance_id: &AbilityInstanceId,
    ) -> Option<&mut AbilityInstance> {
        self.active_instances.get_mut(instance_id)
    }

    /// 根据 spec_id 查找活跃实例。
    pub fn find_instance_by_spec(&self, spec_id: &str) -> Option<&AbilityInstance> {
        self.active_instances
            .values()
            .find(|inst| inst.spec_id == spec_id)
    }

    /// 根据 spec_id 查找活跃实例（可变）。
    pub fn find_instance_by_spec_mut(&mut self, spec_id: &str) -> Option<&mut AbilityInstance> {
        self.active_instances
            .values_mut()
            .find(|inst| inst.spec_id == spec_id)
    }

    /// 检查指定 spec_id 是否有活跃实例（Casting/Active 状态）。
    pub fn has_active_instance(&self, spec_id: &str) -> bool {
        self.active_instances
            .values()
            .any(|inst| inst.spec_id == spec_id && inst.is_active())
    }

    /// 活跃实例数量（不含已暂停或已完成的）。
    pub fn active_count(&self) -> usize {
        self.active_instances
            .values()
            .filter(|inst| inst.is_active())
            .count()
    }

    // ── 实例管理 ───────────────────────────────────────────

    /// 用于 AbilitySystem 在技能激活后注册新实例。
    pub fn insert_instance(&mut self, instance: AbilityInstance) {
        self.active_instances.insert(instance.instance_id, instance);
    }

    /// 移除指定实例，返回被移除的实例（如果存在）。
    pub fn remove_instance(&mut self, instance_id: &AbilityInstanceId) -> Option<AbilityInstance> {
        self.active_instances.remove(instance_id)
    }

    /// 移除指定 spec_id 对应的所有实例，返回被移除的实例列表。
    pub fn remove_instances_by_spec(&mut self, spec_id: &str) -> Vec<AbilityInstance> {
        let to_remove: Vec<AbilityInstanceId> = self
            .active_instances
            .iter()
            .filter(|(_, inst)| inst.spec_id == spec_id)
            .map(|(id, _)| *id)
            .collect();

        let mut removed = Vec::new();
        for id in to_remove {
            if let Some(inst) = self.active_instances.remove(&id) {
                removed.push(inst);
            }
        }
        removed
    }

    /// 清除所有活跃实例（如实体死亡时）。
    pub fn clear_instances(&mut self) {
        self.active_instances.clear();
    }

    // ── 冷却管理 ───────────────────────────────────────────

    /// 用于冷却系统在回合开始时查询冷却状态。
    pub fn get_cooldown(&self, spec_id: &str) -> Option<&CooldownEntry> {
        self.cooldowns.get(spec_id)
    }

    /// 由 CooldownSystem 在技能使用后写入冷却记录。
    pub fn set_cooldown(&mut self, entry: CooldownEntry) {
        let spec_id = entry.spec_id.clone();
        self.cooldowns.insert(spec_id, entry);
    }

    /// 手动清除冷却（如冷却缩减效果触发时）。
    pub fn remove_cooldown(&mut self, spec_id: &str) {
        self.cooldowns.remove(spec_id);
    }

    /// 检查指定 spec_id 是否在冷却中。
    pub fn is_on_cooldown(&self, spec_id: &str) -> bool {
        self.cooldowns.get(spec_id).is_some_and(|c| !c.is_expired())
    }

    /// 获取指定 spec_id 的剩余冷却回合数。
    pub fn cooldown_remaining(&self, spec_id: &str) -> u32 {
        self.cooldowns.get(spec_id).map_or(0, |c| c.remaining_turns)
    }

    /// 推进所有冷却 1 回合。返回已过期的 spec_id 列表。
    pub fn tick_all_cooldowns(&mut self) -> Vec<String> {
        let mut expired = Vec::new();
        for (spec_id, entry) in &mut self.cooldowns {
            if entry.remaining_turns > 0 {
                entry.tick();
                if entry.is_expired() {
                    expired.push(spec_id.clone());
                }
            }
        }
        // 批量清理已过期的冷却，避免迭代中修改集合
        for spec_id in &expired {
            self.cooldowns.remove(spec_id);
        }
        expired
    }

    // ── 共享冷却管理 ───────────────────────────────────────

    /// 由 CooldownSystem 在共享冷却启用时调用。
    pub fn set_shared_cooldown(&mut self, group: impl Into<String>, turns: u32) {
        self.shared_cooldowns.insert(group.into(), turns);
    }

    /// 查询共享冷却组剩余回合数。
    pub fn shared_cooldown_remaining(&self, group: &str) -> u32 {
        self.shared_cooldowns.get(group).copied().unwrap_or(0)
    }

    /// 推进所有共享冷却。
    pub fn tick_shared_cooldowns(&mut self) {
        let mut expired_groups = Vec::new();
        for (group, remaining) in &mut self.shared_cooldowns {
            if *remaining > 0 {
                *remaining = remaining.saturating_sub(1);
                if *remaining == 0 {
                    expired_groups.push(group.clone());
                }
            }
        }
        for group in expired_groups {
            self.shared_cooldowns.remove(&group);
        }
    }

    // ── Blocked 状态管理 ───────────────────────────────────

    /// 记录被阻塞的技能应恢复的状态。
    pub fn set_blocked_restore(
        &mut self,
        spec_id: impl Into<String>,
        restore: BlockedRestoreState,
    ) {
        self.blocked_restore.insert(spec_id.into(), restore);
    }

    /// 获取并移除阻塞恢复状态。
    pub fn take_blocked_restore(&mut self, spec_id: &str) -> Option<BlockedRestoreState> {
        self.blocked_restore.remove(spec_id)
    }

    /// 检查是否有被阻塞的技能。
    pub fn has_blocked_abilities(&self) -> bool {
        !self.blocked_restore.is_empty()
    }

    // ── 批量操作 ───────────────────────────────────────────

    /// 获取当前活跃实例的迭代器。
    pub fn iter_active(&self) -> impl Iterator<Item = &AbilityInstance> {
        self.active_instances.values()
    }

    /// 获取当前活跃实例的可变迭代器。
    pub fn iter_active_mut(&mut self) -> impl Iterator<Item = &mut AbilityInstance> {
        self.active_instances.values_mut()
    }
}

impl Default for ActiveAbilityContainer {
    fn default() -> Self {
        Self::empty()
    }
}
