//! Pipeline 基础类型与枚举
//!
//! 定义执行管线的阶段、步骤、上下文以及领域错误。
//!
//! 详见 docs/04-data/infrastructure/pipeline_schema.md §2。

use std::collections::HashMap;

/// 管线的执行阶段。
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineStage {
    /// 阶段名称
    pub name: String,
    /// 阶段内步骤列表（按顺序执行）
    pub steps: Vec<PipelineStep>,
    /// 失败策略
    pub on_failure: FailureStrategy,
    /// 是否可跳过
    pub skippable: bool,
}

impl PipelineStage {
    /// 创建新的执行阶段。
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            steps: Vec::new(),
            on_failure: FailureStrategy::Abort,
            skippable: false,
        }
    }

    /// 添加步骤。
    pub fn step(mut self, step: PipelineStep) -> Self {
        self.steps.push(step);
        self
    }

    /// 设置失败策略。
    pub fn on_failure(mut self, strategy: FailureStrategy) -> Self {
        self.on_failure = strategy;
        self
    }

    /// 标记为可跳过。
    pub fn skippable(mut self) -> Self {
        self.skippable = true;
        self
    }
}

/// 管线步骤类型。
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineStep {
    /// 执行一个命名的 System 函数
    System(String),
    /// 执行一个命名的领域规则
    Rule(String),
    /// 执行一个子管线
    SubPipeline(String),
    /// 条件分支
    Conditional {
        /// 条件名称（由执行器评估）
        condition: String,
        /// 条件为真时执行的步骤
        if_true: Box<PipelineStep>,
        /// 条件为假时执行的步骤
        if_false: Box<PipelineStep>,
    },
}

impl PipelineStep {
    /// 返回步骤的名称标识。
    pub fn name(&self) -> &str {
        match self {
            Self::System(id) => id.as_str(),
            Self::Rule(id) => id.as_str(),
            Self::SubPipeline(id) => id.as_str(),
            Self::Conditional { condition, .. } => condition.as_str(),
        }
    }
}

/// 步骤执行失败时的策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FailureStrategy {
    /// 失败时立即终止整条管线
    Abort,
    /// 跳过失败的步骤，继续后续步骤
    SkipAndContinue,
    /// 重试 N 次
    Retry { max_retries: u8 },
}

impl FailureStrategy {
    /// 返回策略名称。
    pub fn name(&self) -> &str {
        match self {
            Self::Abort => "Abort",
            Self::SkipAndContinue => "SkipAndContinue",
            Self::Retry { .. } => "Retry",
        }
    }
}

/// 管线执行结果。
#[derive(Debug, Clone, PartialEq)]
pub enum StepResult {
    /// 成功
    Success,
    /// 失败带错误信息
    Failure(String),
    /// 跳过（阶段被标记为可跳过且未执行）
    Skipped,
}

/// Pipeline 领域错误。
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineError {
    /// 阶段未找到
    StageNotFound(String),
    /// 步骤执行失败
    StepFailed {
        stage: String,
        step: String,
        detail: String,
    },
    /// 管线被中止
    Aborted(String),
    /// 上下文数据缺失
    MissingContext(String),
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StageNotFound(name) => write!(f, "stage '{}' not found", name),
            Self::StepFailed {
                stage,
                step,
                detail,
            } => {
                write!(f, "step '{}' in stage '{}' failed: {}", step, stage, detail)
            }
            Self::Aborted(reason) => write!(f, "pipeline aborted: {}", reason),
            Self::MissingContext(key) => write!(f, "missing context key: {}", key),
        }
    }
}

impl std::error::Error for PipelineError {}

/// 管线执行日志条目。
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionLogEntry {
    /// 阶段名称
    pub stage: String,
    /// 步骤名称
    pub step: String,
    /// 执行结果
    pub result: StepResult,
}

impl ExecutionLogEntry {
    /// 创建日志条目。
    pub fn new(stage: impl Into<String>, step: impl Into<String>, result: StepResult) -> Self {
        Self {
            stage: stage.into(),
            step: step.into(),
            result,
        }
    }
}

/// 执行上下文——跨阶段传递数据的容器。
///
/// 每个阶段的输出存储在此上下文中，供后续阶段读取。
/// 使用 String 键值对保持类型安全的同时保持通用性。
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineContext {
    /// 管线 ID
    pub pipeline_id: String,
    /// 上下文数据（阶段名 → JSON/序列化后的数据字符串）
    pub stage_data: HashMap<String, String>,
    /// 执行日志
    pub execution_log: Vec<ExecutionLogEntry>,
    /// 是否已中止
    pub aborted: bool,
    /// 中止原因
    pub abort_reason: Option<String>,
}

impl PipelineContext {
    /// 创建新的管线上下文。
    pub fn new(pipeline_id: impl Into<String>) -> Self {
        Self {
            pipeline_id: pipeline_id.into(),
            stage_data: HashMap::new(),
            execution_log: Vec::new(),
            aborted: false,
            abort_reason: None,
        }
    }

    /// 写入阶段数据。
    pub fn set_stage_data(&mut self, stage: impl Into<String>, data: impl Into<String>) {
        self.stage_data.insert(stage.into(), data.into());
    }

    /// 读取阶段数据。
    pub fn get_stage_data(&self, stage: &str) -> Option<&String> {
        self.stage_data.get(stage)
    }

    /// 记录执行日志。
    pub fn log(&mut self, entry: ExecutionLogEntry) {
        self.execution_log.push(entry);
    }

    /// 中止管线。
    pub fn abort(&mut self, reason: impl Into<String>) {
        self.aborted = true;
        self.abort_reason = Some(reason.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_001_stage_creation() {
        let stage = PipelineStage::new("damage_calculation")
            .step(PipelineStep::Rule("calc_base_damage".into()))
            .step(PipelineStep::Rule("apply_modifiers".into()));
        assert_eq!(stage.name, "damage_calculation");
        assert_eq!(stage.steps.len(), 2);
    }

    #[test]
    fn unit_002_stage_skippable() {
        let stage = PipelineStage::new("optional").skippable();
        assert!(stage.skippable);
    }

    #[test]
    fn unit_003_step_name() {
        assert_eq!(PipelineStep::System("physics".into()).name(), "physics");
        assert_eq!(
            PipelineStep::Rule("calc_damage".into()).name(),
            "calc_damage"
        );
        assert_eq!(PipelineStep::SubPipeline("combat".into()).name(), "combat");
    }

    #[test]
    fn unit_004_conditional_step_name() {
        let step = PipelineStep::Conditional {
            condition: "has_shield".into(),
            if_true: Box::new(PipelineStep::Rule("apply_shield".into())),
            if_false: Box::new(PipelineStep::Rule("apply_damage".into())),
        };
        assert_eq!(step.name(), "has_shield");
    }

    #[test]
    fn unit_005_failure_strategy_name() {
        assert_eq!(FailureStrategy::Abort.name(), "Abort");
        assert_eq!(FailureStrategy::SkipAndContinue.name(), "SkipAndContinue");
        assert_eq!(FailureStrategy::Retry { max_retries: 3 }.name(), "Retry");
    }

    #[test]
    fn unit_006_context_creation() {
        let ctx = PipelineContext::new("combat_pipeline");
        assert_eq!(ctx.pipeline_id, "combat_pipeline");
        assert!(!ctx.aborted);
    }

    #[test]
    fn unit_007_context_stage_data() {
        let mut ctx = PipelineContext::new("test");
        ctx.set_stage_data("generate", "damage=50,type=fire");
        assert_eq!(
            ctx.get_stage_data("generate"),
            Some(&"damage=50,type=fire".into())
        );
        assert_eq!(ctx.get_stage_data("nonexistent"), None);
    }

    #[test]
    fn unit_008_context_abort() {
        let mut ctx = PipelineContext::new("test");
        ctx.abort("critical failure");
        assert!(ctx.aborted);
        assert_eq!(ctx.abort_reason, Some("critical failure".into()));
    }

    #[test]
    fn unit_009_context_log() {
        let mut ctx = PipelineContext::new("test");
        ctx.log(ExecutionLogEntry::new(
            "damage",
            "calc_base",
            StepResult::Success,
        ));
        assert_eq!(ctx.execution_log.len(), 1);
    }

    #[test]
    fn unit_010_error_display() {
        let err = PipelineError::StageNotFound("combat".into());
        let msg = format!("{}", err);
        assert!(msg.contains("combat"));
    }

    #[test]
    fn unit_011_error_step_failed() {
        let err = PipelineError::StepFailed {
            stage: "damage".into(),
            step: "calc".into(),
            detail: "division by zero".into(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("calc"));
        assert!(msg.contains("division by zero"));
    }

    #[test]
    fn unit_012_execution_log_entry() {
        let entry = ExecutionLogEntry::new("stage_1", "step_a", StepResult::Success);
        assert_eq!(entry.stage, "stage_1");
        assert_eq!(entry.step, "step_a");
    }
}
