---
id: infrastructure.pipeline.schema.v1
title: Pipeline Schema — 执行管线数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: runtime
replay-safe: true
---

# Pipeline Schema — 执行管线数据架构

> **领域归属**: Infrastructure — C3 Runtime | **依赖 Schema**: 全部 Capabilities | **定义依据**: `docs/00-governance/Fre项目架构设计.md` §6.5 C3 Runtime

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `PipelineStage` | Definition | 执行阶段定义 |
| `PipelineContext` | Runtime | 跨阶段执行上下文 |
| `PipelineState` | Instance | 管线运行时状态 |

---

## 2. Schema Design

### 2.1 PipelineStage（Definition 层）

```rust
/// 管线的执行阶段。
struct PipelineStage {
    /// 阶段名称（如 "skill_activation", "damage_calculation"）
    name: String,

    /// 阶段内步骤列表（按顺序执行）
    steps: Vec<PipelineStep>,

    /// 失败策略
    on_failure: FailureStrategy,

    /// 是否可跳过
    skippable: bool,
}

enum PipelineStep {
    /// 执行一个 System 函数
    System(SystemId),
    /// 执行一个领域规则
    Rule(RuleId),
    /// 执行一个子管线
    SubPipeline(String),
    /// 条件分支
    Conditional {
        condition: Condition,
        if_true: Box<PipelineStep>,
        if_false: Box<PipelineStep>,
    },
}

enum FailureStrategy {
    /// 失败时立即终止整条管线
    Abort,
    /// 跳过失败的步骤，继续后续步骤
    SkipAndContinue,
    /// 重试 N 次
    Retry { max_retries: u8 },
}
```

### 2.2 PipelineContext（Runtime 层）

```rust
/// 管线执行上下文。
struct PipelineContext {
    /// 当前管线 ID
    pipeline_id: String,

    /// 参与实体
    participants: PipelineParticipants,

    /// 各阶段的数据（上一个阶段的输出是下一个阶段的输入）
    stage_data: HashMap<String, Box<dyn Any>>,

    /// 已执行的步骤追踪
    execution_log: Vec<ExecutionLogEntry>,

    /// 是否已中止
    aborted: bool,

    /// 中止原因
    abort_reason: Option<String>,
}

struct PipelineParticipants {
    source: EntityId,
    targets: Vec<EntityId>,
    context: GameplayContextData,
}

struct ExecutionLogEntry {
    stage: String,
    step: String,
    status: StepStatus,
    duration_us: u64,
    error: Option<String>,
}

enum StepStatus {
    Success,
    Skipped,
    Failed,
    Retried,
}
```

### 2.3 PipelineState（Instance 层 — ECS Component）

```rust
/// 管线的运行时状态组件。
struct PipelineState {
    /// 当前正在执行的管线
    active_pipelines: Vec<ActivePipeline>,

    /// 管线执行队列（待执行的管线）
    pending_queue: Vec<PipelineRequest>,
}

struct ActivePipeline {
    /// 管线 ID
    pipeline_id: String,
    /// 当前阶段索引
    current_stage: usize,
    /// 当前步骤索引
    current_step: usize,
    /// 已执行时长
    elapsed_frames: u64,
}

struct PipelineRequest {
    pipeline_id: String,
    participants: PipelineParticipants,
    priority: u8,
    submitted_frame: u64,
}
```

### 2.4 PipelineConfig（Definition 层 — 示例）

```yaml
# RON 配置 — 技能激活管线
PipelineConfig:
  pipelines:
    skill_activation:
      stages:
        - name: "condition_check"
          steps:
            - System: "check_activation_conditions"
            - System: "check_immunity"
          on_failure: Abort

        - name: "cost_consumption"
          steps:
            - System: "consume_resources"
          on_failure: Abort

        - name: "target_selection"
          steps:
            - System: "select_targets"
            - System: "validate_targets"
          on_failure: SkipAndContinue

        - name: "execution"
          steps:
            - System: "execute_damage"
            - Conditional:
                condition: 
                  AttributeCheck:
                    attribute_id: "attr_000030"  # HP
                    operator: LessOrEqual
                    threshold: 0.0
                if_true:
                  System: "handle_death"
                if_false: ~
          on_failure: SkipAndContinue

        - name: "effect_application"
          steps:
            - System: "apply_effects"
            - System: "trigger_cues"
          on_failure: SkipAndContinue
```

---

## 3. Dependency Analysis

Pipeline 是 C3 Runtime 的编排层，依赖所有 Capabilities Schema 提供的 System/Rule。Pipeline 本身不包含业务逻辑，只编排执行顺序。

---

## 4. Replay Compatibility

| 场景 | 兼容性 | 说明 |
|------|--------|------|
| 管线执行顺序 | 🟩 完全确定 | 按 PipelineConfig 定义的顺序执行 |
| 条件分支 | 🟩 完全确定 | Condition 确定 → 分支路径确定 |
| 失败策略 | 🟩 确定 | Abort/Skip/Retry 行为确定 |

---

## 5. Save Compatibility

Pipelining 是运行时编排机制，不单独持久化。只在存档中记录当前活跃的 PipelineState（如正在施法的技能实例由 AbilityInstance 管理）。

---

## 6. Constitution Check

| 条款 | 合规 | 说明 |
|------|------|------|
| C3 Runtime 不包含业务逻辑 | ✅ | Pipeline 只编排不执行业务 |
| Replay First | ✅ | 管线执行顺序确定 |
