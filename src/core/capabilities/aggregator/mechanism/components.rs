//! Aggregator ECS 组件

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

/// 挂载在实体上的聚合状态组件。
///
/// 缓存当前聚合结果并跟踪需要重算的脏属性。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct AggregatorState {
    /// 属性标识符 → 缓存的聚合最终值
    pub cached_values: HashMap<String, f32>,
    /// 当前脏属性集合（需要重算）
    pub dirty_attributes: HashSet<String>,
    /// 上次聚合计算的帧号
    pub last_aggregation_frame: u64,
    /// 聚合总次数（诊断/回放校验用）
    pub aggregation_count: u64,
}

impl AggregatorState {
    /// 创建一个空的聚合状态。
    pub fn empty() -> Self {
        Self {
            cached_values: HashMap::new(),
            dirty_attributes: HashSet::new(),
            last_aggregation_frame: 0,
            aggregation_count: 0,
        }
    }

    /// 检查指定属性是否为脏。
    pub fn is_dirty(&self, attribute_id: &str) -> bool {
        self.dirty_attributes.contains(attribute_id)
    }

    /// 获取缓存的最终值（如果存在且非脏）。
    pub fn get_cached(&self, attribute_id: &str) -> Option<f32> {
        if self.is_dirty(attribute_id) {
            return None;
        }
        self.cached_values.get(attribute_id).copied()
    }

    /// 是否有任何脏属性。
    pub fn has_dirty(&self) -> bool {
        !self.dirty_attributes.is_empty()
    }
}

impl Default for AggregatorState {
    fn default() -> Self {
        Self::empty()
    }
}
