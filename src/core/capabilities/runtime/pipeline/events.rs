//! Pipeline 领域事件

use bevy::prelude::*;

/// 管线执行开始时触发。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct PipelineStarted {
    /// 管线 ID
    pub pipeline_id: String,
    /// 阶段总数
    pub total_stages: u32,
}

/// 管线步骤完成时触发。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct PipelineStepCompleted {
    /// 管线 ID
    pub pipeline_id: String,
    /// 阶段名称
    pub stage: String,
    /// 步骤名称
    pub step: String,
    /// 是否成功
    pub success: bool,
}

/// 管线执行失败时触发。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct PipelineFailed {
    /// 管线 ID
    pub pipeline_id: String,
    /// 失败的阶段
    pub stage: String,
    /// 失败的步骤
    pub step: String,
    /// 错误原因
    pub reason: String,
}

/// 管线完成时触发。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct PipelineCompleted {
    /// 管线 ID
    pub pipeline_id: String,
    /// 总步骤数
    pub total_steps: u32,
    /// 失败的步骤数
    pub failed_steps: u32,
}
