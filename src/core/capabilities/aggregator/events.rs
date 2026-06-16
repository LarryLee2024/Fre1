//! Aggregator 领域事件

use bevy::prelude::*;

/// 属性聚合计算完成事件。
#[derive(Event, Debug, Clone)]
pub struct AggregationComplete {
    /// 目标实体
    pub entity: Entity,
    /// 目标属性 ID
    pub attribute_id: String,
    /// 聚合最终值
    pub final_value: f32,
    /// 基础值
    pub base_value: f32,
    /// 发生帧号
    pub frame: u64,
}

/// 属性被标记为需要重算事件。
#[derive(Event, Debug, Clone)]
pub struct AggregateDirty {
    /// 目标实体
    pub entity: Entity,
    /// 目标属性 ID
    pub attribute_id: String,
    /// 触发来源（哪个 Modifier 变更导致的）
    pub trigger_source: String,
}

/// 检测到聚合闭环（循环依赖）事件。
#[derive(Event, Debug, Clone)]
pub struct PipelineCycleDetected {
    /// 循环链上的属性 ID 序列
    pub cycle_chain: Vec<String>,
    /// 发生帧号
    pub frame: u64,
}
