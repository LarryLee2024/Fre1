---
id: OBSERVABILITY-CONSTITUTION
title: 可观测性宪法
status: accepted
stability: stable
layer: governance
related:
  - ai-constitution-complete.md
tags:
  - observability
  - logging
  - error
  - debugging
---

> **原文来源**：`ai-constitution-complete.md` 第十一编（L1113-L1207）
> **锚定总宪法**：第十一编

## 第十一编 可观测性宪法
### 11.1 日志框架与核心定位
- 🟩 统一使用 `tracing` 库进行日志记录，🟥 绝对禁止使用 `println!`、`dbg!` 输出运行时信息
- 🟩 日志核心定位：领域事件履历
  - 日志必须记录业务事件事实，🟥 绝对禁止记录技术执行流水
  - 一个完整业务动作最多输出一条 INFO 级别日志；DEBUG / TRACE 级可按需细化

### 11.2 日志分级规范
- 🟥 **ERROR**：理论上不应发生的程序异常，必须附带完整复现上下文
- 🟩 **WARN**：可继续运行的异常情况
- 🟩 **INFO**：核心业务事件边界
- 🟦 **DEBUG**：开发调试辅助信息，发布版默认关闭
- 🟦 **TRACE**：极细粒度算法细节，仅专项调试时临时开启

### 11.3 结构化日志强制要求
- 🟩 所有 INFO 级别日志必须携带 `code = ?LogCode::XXX` 字段，LogCode 是事件唯一标识
- 🟩 `event` 字段是 LogCode 的可选人类别名，**值必须使用英文**（`"level_up"`、`"battle_started"`），结构化日志是机器消费的，禁止使用中文
- 🟩 `message`（字符串消息）可用中文或英文，这是给人读的
- 🟩 日志 `target` 必须遵循 `domain.module.submodule` 层级格式（如 `domain.combat`、`domain.ability.activation`、`infra.save`），支持按粒度过滤
- 🟩 所有结构化字段必须使用 ID（`entity_id`、`spec_id`、`item_id`），**禁止使用自然语言文本**（`context_desc` 等高基数字段会破坏日志聚合分析）

### 11.4 日志禁令
- 🟥 绝对禁止记录函数进入、退出、系统执行等技术流水账
- 🟥 Release 版本绝对禁止在每帧执行的系统中输出 INFO / DEBUG 级别日志
- 🟥 绝对禁止在循环、迭代器内部输出 INFO 级别日志
- 🟥 绝对禁止业务代码直接调用 `info!` 输出核心业务事件（必须走领域事件链路）
- 🟥 绝对禁止在 `info!()` 中重复 `#[instrument(fields(...))]` 已覆盖的 `code`、`event` 等不变字段（span 负责不变量，event 只放变量）
- 🟥 绝对禁止 `event` 字段值使用中文（`"技能激活"`）——结构化日志是机器消费的，必须使用英文
- 🟥 绝对禁止使用 `context_desc` 等自然语言文本作为结构化字段（高基数，破坏日志聚合分析），必须使用 ID

### 11.5 日志架构规范（领域事件驱动）
- 核心模式：**领域事件触发 → 统一 Log Observer 监听 → 输出 tracing 日志**
- 所有 INFO 级别的核心业务事件，必须通过触发领域事件的方式生成日志
- 日志 Observer 统一放在基础设施层，绝对不侵入业务模块
- Battle Replay、战斗履历 UI、成就系统、任务系统与日志共用同一套领域事件源

### 11.6 Observer 实现规范（两要素模式）

Observer 包含两个要素：

1. 🟩 **`#[tracing::instrument]` span（不变量）**：LogCode 和事件名（`code = ?LogCode::PRG002`, `event = "level_up"`）——所有日志实例共用的固定值
2. 🟩 **`emit_info!`/`emit_warn!`/`emit_debug!`（统一入口）**：宏内部自动完成 `telemetry::record(LogCode)`（度量）+ `tracing::info!`（结构化日志）

```rust
// ✅ Observer 两要素模式
#[tracing::instrument(skip_all, target = "domain.progression", fields(
    code = ?LogCode::PRG002,
    event = "level_up",
))]
pub(crate) fn on_level_up(trigger: On<LevelUp>) {
    let e = trigger.event();
    // 两要素 = #[instrument] span + emit_info!宏
    emit_info!(
        LogCode::PRG002,
        entity = ?e.entity,
        old = e.old_level,
        new = e.new_level,
        "角色升级",
    );
}
```

**使用限制**（铁律）：
- 🟥 禁止在 Domain Service、Domain Model、Ability Executor、Pipeline 中使用 `emit_info!`/`emit_warn!`/`emit_debug!`
- 🟩 只在 Observer、Adapter、Infra 层使用
- DDD 原则：Domain 不知道 Observability 的存在，领域只产生事件

### 11.7 错误体系规范
#### 11.7.1 分领域错误原则
- 🟩 每个领域定义独立错误枚举，🟥 绝对禁止使用全局统一的 `AppError` 大枚举、`anyhow::Error`、`Box<dyn Error>` 作为业务层返回错误类型
- 🟨 基础设施层可定义通用错误转换 Trait，不包含任何业务错误变体

#### 11.7.2 失败分类学（强制区分）
1. **规则失败**：业务规则的正常不满足，不属于程序错误，用专门结果枚举表达，禁止用 `Result::Err` 返回
2. **领域错误**：领域内预期内的异常，用对应领域错误枚举的 `Result::Err` 返回
3. **基础设施错误**：底层通用能力异常
4. **程序 Bug**：非法状态、逻辑断言失败，属于代码缺陷

#### 11.7.3 错误强制要求
- 🟩 所有错误必须携带完整上下文信息，🟥 绝对禁止仅返回无上下文的错误变体
- 🟩 推荐使用带编号的错误码，便于快速定位问题

#### 11.7.4 业务层 Panic 禁令
- 🟥 绝对禁止在核心业务领域代码中使用 `unwrap()`、`expect()`、`panic!()`
- 仅允许测试代码、工具代码、编辑器代码、原型验证代码使用

### 11.8 调试工具与可观测性
- 🟩 优先使用 Inspector、Replay、Debug Panel 进行问题排查，🟥 绝对禁止通过堆砌临时日志定位问题
- 🟨 核心战斗系统优先支持单步执行与状态回溯

### 11.9 领域事件与审计轨迹
- 🟩 所有核心业务事实必须通过领域事件表达，所有下游能力共用同一事件源
- 🟩 所有正式领域事件必须收录在白名单文档中
- 🟩 核心战斗流程必须生成结构化的审计轨迹，支撑回放、Bug 复现、自动化测试、数值平衡分析
