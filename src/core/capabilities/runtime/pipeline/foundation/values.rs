//! Pipeline 值对象：管线定义与阶段容器

use super::types::{PipelineContext, PipelineStage};

/// 管线定义——由一组有序阶段组成的执行计划。
///
/// 🟥 禁止运行时动态调整阶段顺序（破坏 Replay 确定性）。
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineDefinition {
    /// 管线标识
    pub id: String,
    /// 各执行阶段（按顺序）
    pub stages: Vec<PipelineStage>,
}

impl PipelineDefinition {
    /// 创建新的管线定义，id 必填。
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            stages: Vec::new(),
        }
    }

    /// 追加一个执行阶段到管线末尾。
    pub fn stage(mut self, s: PipelineStage) -> Self {
        self.stages.push(s);
        self
    }

    /// 按阶段名称查找，返回不可变引用。
    pub fn find_stage(&self, name: &str) -> Option<&PipelineStage> {
        self.stages.iter().find(|s| s.name == name)
    }

    /// 返回阶段总数。
    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }
}

/// 管线运行时状态。
///
/// 记录当前执行进度和上下文，由 Executor 驱动推进。
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineState {
    /// 管线定义 ID
    pub pipeline_id: String,
    /// 当前执行到的阶段索引
    pub current_stage_index: usize,
    /// 当前阶段内的步骤索引
    pub current_step_index: usize,
    /// 执行上下文
    pub context: PipelineContext,
    /// 是否已完成
    pub completed: bool,
}

impl PipelineState {
    /// 从管线定义创建初始运行时状态，当前阶段索引和步骤索引均为 0。
    pub fn new(definition: &PipelineDefinition) -> Self {
        Self {
            pipeline_id: definition.id.clone(),
            current_stage_index: 0,
            current_step_index: 0,
            context: PipelineContext::new(&definition.id),
            completed: false,
        }
    }

    /// 标记管线为已完成，不再接受进一步推进。
    pub fn mark_completed(&mut self) {
        self.completed = true;
    }

    /// 推进到当前阶段内的下一个步骤。
    pub fn advance_step(&mut self) {
        self.current_step_index += 1;
    }

    /// 推进到下一个阶段，步骤索引重置为 0。
    pub fn advance_stage(&mut self) {
        self.current_stage_index += 1;
        self.current_step_index = 0;
    }
}
