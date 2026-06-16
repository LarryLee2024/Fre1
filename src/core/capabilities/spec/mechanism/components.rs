//! Spec ECS 组件
//!
//! SpecContainer 是挂载在实体上的 Spec 容器组件，管理该实体的所有
//! AbilitySpec 和 EffectSpec，并提供基于 def_id 的快速索引。
//!
//! 依赖 foundation::values::{AbilitySpec, EffectSpec}。
//! 详见 docs/04-data/capabilities/spec_schema.md §3.5。

use std::collections::HashMap;

use bevy::prelude::*;

use crate::core::capabilities::spec::foundation::{AbilitySpec, EffectSpec, SpecId};

/// 挂载在实体上的 Spec 容器组件。
///
/// 管理实体的所有 AbilitySpec 和 EffectSpec 实例，维护 def_id → spec_id 的
/// 快速索引以支持 O(1) 重复检测（不变量 V3）。
#[derive(Component, Debug, Clone)]
pub struct SpecContainer {
    /// 所有 AbilitySpec（keyed by spec_id）
    pub abilities: HashMap<SpecId, AbilitySpec>,
    /// 所有活跃的 EffectSpec（keyed by spec_id）
    pub effects: HashMap<SpecId, EffectSpec>,
    /// def_id → spec_id 的索引（快速查找 AbilitySpec）
    pub ability_by_def: HashMap<String, SpecId>,
    /// def_id → spec_ids 的索引（快速查找 EffectSpec，一对多）
    pub effect_by_def: HashMap<String, Vec<SpecId>>,
}

impl SpecContainer {
    /// 创建一个空的 SpecContainer。
    pub fn empty() -> Self {
        Self {
            abilities: HashMap::new(),
            effects: HashMap::new(),
            ability_by_def: HashMap::new(),
            effect_by_def: HashMap::new(),
        }
    }

    /// 获取指定 AbilitySpec 的可变引用。
    pub fn get_ability_mut(&mut self, spec_id: &SpecId) -> Option<&mut AbilitySpec> {
        self.abilities.get_mut(spec_id)
    }

    /// 获取指定 AbilitySpec 的不可变引用。
    pub fn get_ability(&self, spec_id: &SpecId) -> Option<&AbilitySpec> {
        self.abilities.get(spec_id)
    }

    /// 获取指定 EffectSpec 的不可变引用。
    pub fn get_effect(&self, spec_id: &SpecId) -> Option<&EffectSpec> {
        self.effects.get(spec_id)
    }

    /// 根据 def_id 查找 AbilitySpec 的 SpecId。
    pub fn find_ability_by_def(&self, def_id: &str) -> Option<SpecId> {
        self.ability_by_def.get(def_id).cloned()
    }

    /// 根据 def_id 查找所有 EffectSpec 的 SpecId 列表。
    pub fn find_effects_by_def(&self, def_id: &str) -> Vec<SpecId> {
        self.effect_by_def.get(def_id).cloned().unwrap_or_default()
    }

    /// 检查该实体是否已有同 def 的 AbilitySpec（不变量 V3）。
    pub fn has_ability_for_def(&self, def_id: &str) -> bool {
        self.ability_by_def.contains_key(def_id)
    }

    /// 插入一个 AbilitySpec，同时更新反向索引。
    pub fn insert_ability(&mut self, spec: AbilitySpec) {
        let def_id = spec.def_id.clone();
        let spec_id = spec.spec_id.clone();
        self.ability_by_def.insert(def_id, spec_id.clone());
        self.abilities.insert(spec_id, spec);
    }

    /// 移除一个 AbilitySpec，同时清理反向索引。
    ///
    /// 返回被移除的 AbilitySpec（如果存在）。
    pub fn remove_ability(&mut self, spec_id: &SpecId) -> Option<AbilitySpec> {
        if let Some(spec) = self.abilities.remove(spec_id) {
            // 清理反向索引：只有当前 def 指向该 spec_id 时才移除
            if let Some(idx_spec_id) = self.ability_by_def.get(&spec.def_id) {
                if idx_spec_id == spec_id {
                    self.ability_by_def.remove(&spec.def_id);
                }
            }
            return Some(spec);
        }
        None
    }

    /// 插入一个 EffectSpec，同时更新反向索引。
    pub fn insert_effect(&mut self, spec: EffectSpec) {
        let def_id = spec.def_id.clone();
        let spec_id = spec.spec_id.clone();
        self.effect_by_def
            .entry(def_id)
            .or_default()
            .push(spec_id.clone());
        self.effects.insert(spec_id, spec);
    }

    /// 移除一个 EffectSpec，同时清理反向索引。
    ///
    /// 返回被移除的 EffectSpec（如果存在）。
    pub fn remove_effect(&mut self, spec_id: &SpecId) -> Option<EffectSpec> {
        if let Some(spec) = self.effects.remove(spec_id) {
            if let Some(ids) = self.effect_by_def.get_mut(&spec.def_id) {
                ids.retain(|id| id != spec_id);
                if ids.is_empty() {
                    self.effect_by_def.remove(&spec.def_id);
                }
            }
            return Some(spec);
        }
        None
    }

    /// 获取所有 AbilitySpec 的数量。
    pub fn ability_count(&self) -> usize {
        self.abilities.len()
    }

    /// 获取所有 EffectSpec 的数量。
    pub fn effect_count(&self) -> usize {
        self.effects.len()
    }

    /// 清空所有 Spec 和索引。
    pub fn clear(&mut self) {
        self.abilities.clear();
        self.effects.clear();
        self.ability_by_def.clear();
        self.effect_by_def.clear();
    }
}

impl Default for SpecContainer {
    fn default() -> Self {
        Self::empty()
    }
}
