//! Aggregator 领域事件
//!
//! 定义属性聚合生命周期中的核心事件。
//! Bevy 0.19+ 使用 observer-based 事件系统，通过 commands.trigger() 触发。
//!
//! 详见 docs/02-domain/capabilities/aggregator_domain.md §6。

use bevy::prelude::*;

/// 属性聚合计算完成时触发（聚合最终值已写入 Component）。
///
/// 订阅者：UI（刷新属性面板）、日志、调试工具。
#[derive(Event, Debug, Clone)]
pub struct AggregationComplete {
    /// 目标实体
    pub entity: Entity,
    /// 目标属性 ID
    pub attribute_id: String,
    /// 聚合计算后的最终值
    pub final_value: f32,
    /// 基础值（不含任何修饰器）
    pub base_value: f32,
    /// 发生帧号
    pub frame: u64,
}

/// 属性被标记为需要重新聚合时触发。
///
/// 订阅者：AggregatorSystem（将实体加入待聚合队列）。
#[derive(Event, Debug, Clone)]
pub struct AggregateDirty {
    /// 目标实体
    pub entity: Entity,
    /// 需要重算的属性 ID
    pub attribute_id: String,
    /// 触发来源（哪个 Modifier 变更导致的）
    pub trigger_source: String,
}

/// 检测到聚合闭环（循环依赖）时触发（严重告警）。
///
/// 订阅者：日志、平衡分析工具。
#[derive(Event, Debug, Clone)]
pub struct PipelineCycleDetected {
    /// 循环链上的属性 ID 序列
    pub cycle_chain: Vec<String>,
    /// 发生帧号
    pub frame: u64,
}
