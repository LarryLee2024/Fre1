---
id: 01-architecture.ADR-044
title: ADR-044 — Pipeline Engine Architecture
status: approved
owner: architect
created: 2026-06-17
updated: 2026-06-17
supersedes: none
---

# ADR-044: Pipeline 引擎架构

## 状态

**Approved** — 依赖 ADR-010（Ability Pipeline）、ADR-011（Modifier Pipeline）、ADR-020（CombatIntent Pipeline）和 `docs/04-data/infrastructure/pipeline_schema.md`，本架构决策正式生效。

## 背景

项目存在多条管线：Ability Pipeline、Modifier Pipeline、CombatIntent Pipeline。每条管线都有 Stage → Step → Execute 的执行模式，但当前各自实现，缺乏统一的编排引擎。需要一个通用 Pipeline 引擎来：

1. 统一管线执行模式（注册 → 排序 → 执行 → 日志）
2. 支持前置/后置 Hook（用于录制、调试、观测）
3. 支持失败策略（Abort / SkipAndContinue / Retry）
4. 保证执行顺序确定性（Replay 兼容）

## 引用的领域规则与数据架构

- `docs/04-data/infrastructure/pipeline_schema.md` — Pipeline 数据 Schema
- ADR-010 — Ability Pipeline（Effect Pipeline 的上游）
- ADR-011 — Modifier Pipeline（属性修改管线）
- ADR-020 — CombatIntent Pipeline（战斗伤害管线）
- ADR-041 — Replay Determinism（管线执行顺序必须确定）

## 决策

### 1. Pipeline 引擎定位

```
┌─────────────────────────────────────────────────────────┐
│  C3 Runtime — Pipeline Engine                            │
│                                                          │
│  不包含任何业务逻辑，只提供：                              │
│  ① Stage 注册与排序                                      │
│  ② Step 执行调度                                         │
│  ③ 前置/后置 Hook 注册                                   │
│  ④ 执行日志记录                                          │
│  ⑤ 失败策略执行                                          │
│                                                          │
│  业务管线（Ability/Modifier/Combat）通过组合 Stage 使用    │
└─────────────────────────────────────────────────────────┘
```

### 2. 核心 Trait 设计

```rust
/// Pipeline Stage 的执行单元
pub trait PipelineStepExecutor: Send + Sync {
    /// 步骤名称（用于日志和回放）
    fn name(&self) -> &str;

    /// 执行步骤，返回成功/失败
    fn execute(&self, context: &mut PipelineContext) -> Result<(), StepError>;
}

/// Pipeline Hook — 前置/后置回调
pub trait PipelineHook: Send + Sync {
    /// Hook 名称
    fn name(&self) -> &str;

    /// 在 Stage 执行前调用
    fn on_stage_start(&self, _stage: &str, _context: &PipelineContext) {}

    /// 在 Stage 执行后调用
    fn on_stage_end(&self, _stage: &str, _context: &PipelineContext, _result: &StepResult) {}

    /// 在 Step 执行前调用
    fn on_step_start(&self, _stage: &str, _step: &str, _context: &PipelineContext) {}

    /// 在 Step 执行后调用
    fn on_step_end(&self, _stage: &str, _step: &str, _context: &PipelineContext, _result: &StepResult) {}
}
```

### 3. 执行流程

```
PipelineExecutor::run(pipeline_id, context)
    │
    ├── 注册的 Hooks: on_stage_start()
    │
    ├── for stage in pipeline.stages:
    │   │
    │   ├── Hooks: on_stage_start(stage)
    │   │
    │   ├── for step in stage.steps:
    │   │   ├── Hooks: on_step_start(step)
    │   │   ├── step.execute(context)
    │   │   ├── Hooks: on_step_end(step, result)
    │   │   └── if failed: apply FailureStrategy
    │   │
    │   ├── Hooks: on_stage_end(stage, result)
    │   └── if aborted: break
    │
    └── return final result
```

### 4. Hook 使用场景

| Hook | 使用者 | 用途 |
|------|--------|------|
| ReplayRecorder | `infra/replay` | 录制每个 Step 的输入/输出 |
| DebugLogger | `tools/` (dev) | 调试模式下打印执行详情 |
| MetricsCollector | `tools/` | 性能分析（Step 耗时统计） |
| ValidationHook | `tests/` | 测试模式下校验执行路径 |

### 5. 管线注册

业务管线通过 `PipelineRegistry` 注册，而非硬编码：

```rust
#[derive(Resource)]
pub struct PipelineRegistry {
    pipelines: HashMap<String, PipelineDefinition>,
    hooks: Vec<Box<dyn PipelineHook>>,
}

impl PipelineRegistry {
    pub fn register(&mut self, id: &str, definition: PipelineDefinition);
    pub fn add_hook(&mut self, hook: Box<dyn PipelineHook>);
    pub fn get(&self, id: &str) -> Option<&PipelineDefinition>;
}
```

### 6. 确定性保证

- Stage 执行顺序由注册顺序决定（先注册先执行）
- 同一 Frame 内相同注册顺序 → 相同执行路径 → Replay 兼容
- 🟥 禁止运行时动态调整 Stage 顺序

## Module Design

```
src/infra/pipeline/
  ├── plugin.rs              — PipelinePlugin
  ├── executor.rs            — PipelineExecutor（核心执行引擎）
  ├── registry.rs            — PipelineRegistry（管线注册中心）
  ├── hooks.rs               — PipelineHook trait + 内置 Hook
  ├── context.rs             — PipelineContext（执行上下文）
  └── integration/           — 跨域访问 ACL（ADR-046）
```

## Communication Design

| 通信 | 机制 | 说明 |
|------|------|------|
| 业务代码 → Pipeline | `PipelineRegistry::get()` + `executor.run()` | 同步调用 |
| Pipeline → Hooks | trait 回调 | 前置/后置通知 |
| Pipeline → 日志 | `ExecutionLogEntry` | 执行追踪 |
| Pipeline → Replay | Hook（ReplayRecorder） | 录制执行路径 |

## 边界定义

### 允许
- 业务管线通过注册 Stage 组合使用 Pipeline 引擎
- 通过 Hook 扩展录制、调试、观测能力
- Pipeline 引擎内部使用 HashMap 存储注册信息

### 🟥 禁止
- Pipeline 引擎包含任何业务逻辑（只编排不执行）
- 运行时动态增删 Stage（破坏确定性）
- Hook 修改 PipelineContext 中的业务数据（Hook 只读）
- 跳过 Pipeline 引擎直接执行 Stage（绕过编排）

## Forbidden

| 禁止行为 | 理由 |
|---------|------|
| Pipeline 引擎中写 if/else 业务分支 | 违反"引擎不含业务逻辑"原则 |
| 运行时注册新 Stage（Frame 中途） | 破坏 Replay 确定性 |
| Hook 返回 `Result` 阻断执行 | Hook 是观察者，不是守门人 |
| 多线程并行执行同一 Pipeline 的 Steps | 破坏顺序确定性 |

## Definition / Instance Design

- **Definition**: `PipelineDefinition`（配置层，定义 Stage/Step 结构）
- **Instance**: `PipelineRegistry`（Resource，运行时注册中心）, `PipelineContext`（运行时执行上下文）
- **Persistence**: 不持久化。Pipeline 是运行时编排机制，活跃的 PipelineState 由 AbilityInstance 管理

## 后果

### 正面
- 统一所有管线的执行模式，减少重复代码
- Hook 机制使录制、调试、观测无侵入式接入
- 注册制管线，新增管线不需要修改引擎代码

### 负面
- 增加一层间接调用（Pipeline → Stage → Step → 业务函数）
- 所有管线必须遵循相同执行模式，灵活性受限
- Hook 链过长时可能影响性能（可通过 dev-only Hook 缓解）

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 每条管线独立实现 | 重复代码多，行为不一致 |
| 使用 Bevy 的 State + OnEnter/OnExit | 状态机不适合多步骤顺序执行 |
| 使用 async/await 管线 | 破坏确定性，Replay 不兼容 |

## 评审要点

- [ ] Pipeline 引擎是否需要支持条件分支（Conditional Step）？
- [ ] 失败策略 Retry 的重试间隔如何处理（同 Frame 还是下一 Frame）？
- [ ] PipelineContext 的 `stage_data: HashMap<String, Box<dyn Any>>` 类型安全如何保证？
