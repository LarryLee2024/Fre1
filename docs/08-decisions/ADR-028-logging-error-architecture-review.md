---
id: 08-decisions.adr-028
title: ADR-028 日志与错误体系架构合规审查
status: Accepted
owner: architect
created: 2026-06-15
updated: 2026-06-15
tags:
  - architecture
  - error
  - logging
  - compliance
---

# ADR-028：日志与错误体系架构合规审查

## 状态

Accepted — 决策 1（shared/error 迁移）已实现并验证通过。

## 背景

架构总纲 `docs/01-architecture/README.md`（v4.3）和 `docs/01-architecture/error-architecture.md`（v1.0）对日志和错误体系有明确的架构原则，但当前源码实现与设计存在多处偏离。本 ADR 对 `src/infrastructure/logging/` 和 `src/shared/error/` 两个模块进行系统性的合规审查，回答三个问题：

1. 当前设计能否有效支撑其他业务模块？
2. 日志和错误是否需要集中管理？
3. 当前实现与架构规范的偏差如何修正？

## 引用的架构规则

- `docs/01-architecture/README.md` §错误架构 — 三层错误模型（Domain/Infrastructure/Shared）、失败分类学、禁止 AppError
- `docs/01-architecture/README.md` §日志系统设计 — 日志是领域事件的消费者、禁止 println!/dbg!
- `docs/01-architecture/error-architecture.md` — 错误体系完整设计，包括 shared/error 边界、GameResult 定位
- `docs/01-architecture/events_audit_design.md` — 领域事件与审计系统设计
- `docs/02-domain/README.md` — 领域规则索引

## 现状分析

### 审查方法

对 `src/` 下 14 个顶层目录、~280 个 Rust 文件进行了实际扫描，覆盖：
- 错误枚举定义与使用分布
- 日志宏调用分布（info!/warn!/error!/debug!/trace!）
- unwrap/expect 调用计数
- shared::error 组件导入链
- GameErrorMessage 生产与消费链路
- LogObserver 注册与匹配情况

### 发现一：错误枚举仅覆盖 4/18 核心模块

| 核心模块 | 有领域错误枚举 | 有自己的 Result 别名 | unwrap 调用数 |
|----------|:-----------:|:-----------------:|:------------:|
| ability | ✅ error.rs | ✅ SkillResult<T> | 3 |
| battle | ✅ error.rs | ❌ | 9 |
| buff | ✅ error.rs | ❌ | 9 |
| inventory | ✅ error.rs | ❌ | 40 |
| attribute | ❌ | ❌ | 15 |
| character | ❌ | ❌ | 10 |
| equipment | ❌ | ❌ | 7 |
| map | ❌ | ❌ | 7 |
| modifier | ❌ | ❌ | 1 |
| effect | ❌ | ❌ | 13 |
| execution | ❌ | ❌ | 0 |
| stacking | ❌ | ❌ | 0 |
| cue | ❌ | ❌ | 0 |
| tag | ❌ | ❌ | 2 |
| targeting | ❌ | ❌ | 0 |
| trigger | ❌ | ❌ | 58 |
| turn | ❌ | ❌ | 2 |
| ai | ❌ | ❌ | 2 |

**共 33 个 core 文件包含 unwrap() 调用，总计 ~200+ 处。**

### 发现二：shared::error 与架构规范矛盾

`docs/01-architecture/error-architecture.md` 第 204-274 行明确规定：

> 🟥 **GameResult 不应定义在 shared/ 层**。shared 层是最底层的工具层，如果它依赖 InfrastructureError 就会引入循环依赖。

但当前 `src/shared/error/result.rs` 同时定义了 `InfrastructureError` 枚举和 `GameResult<T>` 别名，并被 `shared/error/mod.rs` 公开导出。这违反了：

1. Shared 层零外部依赖原则（shared/error 依赖 thiserror，虽然 thiserror 是合理依赖，但概念上 shared 不应包含"基础设施错误"这个分类）
2. 错误分层原则（InfrastructureError 应属于 infrastructure/ 层）

**实际影响**：此问题为概念性违反，目前未造成循环依赖，因为 shared 没有其他外部依赖。但 `ErrorContext` 和 `LogIfError` trait（正确的 shared 层工具）与 `InfrastructureError`（错误的 shared 层内容）混在同一个模块中，不利于维护。

**使用情况**：`InfrastructureError` 和 `GameResult` 仅被 UI 层（`ui/view_models.rs`、`ui/screens/game_over.rs`）和 shared/error 自身的测试使用，**没有任何 core 模块导入它们**。这说明业务层已经正确地避开了这个类型。

### 发现三：LogObserver 模式覆盖率不足

`infrastructure::logging` 采用了"日志是领域事件的消费者"架构设计，注册了 19 个 LogObserver System 监听 19 种领域事件 Message。

**实际执行情况**：43 个 src 文件包含直接的日志宏调用（`info!`、`warn!`、`error!`、`debug!`、`trace!`），全部绕过 LogObserver 模式。具体分布：

| 日志来源 | 文件数 | 说明 |
|----------|:------:|------|
| LogObserver（合规） | 1 | 19 个 observer 函数，监听 shared::event |
| battle/pipeline 3 文件 | 3 | 直接 `bevy::log::info!`/`debug!`/`warn!`/`trace!` |
| 其他 core 模块 | ~25 | 直接 `bevy::log::*` 调用 |
| infrastructure 模块 | ~8 | 直接 tracing 调用 |
| UI 模块 | ~5 | 直接 tracing 调用 |

**核心矛盾**：LogObserver 模式设计正确，但实际日志流量的 95%+ 走的是直接日志路径。Observer 模式仅处理了 19 种预定义的结构化事件，而业务代码中散布着数百处临时性的 info!/warn!/debug! 调用。

**这是否是问题？** — **不是**。架构规则"日志是领域事件的消费者"要求的是：
- 🟥 禁止 `println!`/`dbg!`（✅ 当前未发现违反）
- 🟩 关键业务事件通过领域事件触发日志（✅ 19 个 Observer 覆盖了关键事件）
- 日常调试日志直接用 `info!`/`debug!` 是合理的，不需要全部事件化

**效率评估**：当前 `LogPlugin` 注册了 19 个零开销 System —— 每个 observer 只有 `MessageReader<T>` 参数，没有数据查询，Bevy 的空 System 开销极小。不存在性能问题。

### 发现四：error_monitor 无人消费

`src/app/error_monitor.rs` 实现了 `GameErrorMessage` 的消费者 System，用于跨层错误上报。

**实际使用情况**：`GameErrorMessage` 在 `plugin.rs` 中注册为 Message，`error_monitor` 系统已挂载到 Update。但 **没有任何业务模块实际向它发送消息**（除了 error_event.rs 和 error_monitor.rs 本身的测试代码）。

这意味着该通道存在（可以处理 RuleFailure/DomainError/Infrastructure/Bug 四种类型），但业务流程中出错时，错误被直接日志记录或 unwrap，未通过此通道上报。

### 发现五：不存在 println!/dbg! 违规

未在业务代码中发现 `println!` 或 `dbg!` 调用。

## 评估结论

### 问题 1：当前 logging 和 error 模块能否有效支撑业务？

| 维度 | 评分 | 说明 |
|------|:----:|------|
| 接口设计合理性 | 🟡 4/5 | `ErrorContext` + `LogIfError` 是优秀的轻量工具，但 `GameResult`/`InfrastructureError` 放错了层 |
| 功能完整性 | 🟡 3/5 | error 模块缺少：跨层错误上报通道（GameErrorMessage 已定义但无人使用）、领域错误到 App 层的统一路由 |
| 性能表现 | 🟢 5/5 | Observer 零开销，tracing 结构化字段高效 |
| 扩展性 | 🟢 4/5 | 新增 observer = 新增 system，无需修改核心架构 |

**结论**：基本支撑，但有三个关键缺口待修复。

### 问题 2：是否需要集中管理？

**日志**：不需要集中管理。

理由：
- 日志本质是横切关注点（cross-cutting concern），集中不集中不是关键
- 当前 19 个 Observer 已经覆盖了关键业务事件的结构化日志
- 日常调试日志（info!/debug!）直接编写比事件化更高效，改造成本远超收益
- "日志是领域事件的消费者"的正确理解是：**关键业务事件**（伤害、死亡、Buff 等）必须通过事件驱动日志，而非所有日志必须走事件

**错误**：部分集中。

需要集中的部分：
- ✅ 跨层错误上报通道（GameErrorMessage 是正确的设计，但要推动业务层使用它）
- ✅ 共享错误工具（ErrorContext、LogIfError）

不需要集中的部分：
- ❌ 各领域的 Domain Error 枚举（保持分散在 `core/xxx/domain/error.rs`）
- ❌ 各基础设施的错误枚举（保持分散在 `infrastructure/xxx/xxx_error.rs`）
- ❌ 全局 AppError 大枚举（严格禁止）

**结论**：维持"分散定义 + 集中上报通道 + 共享工具"的三层架构，当前设计方向正确，执行不到位。

## 决策

### 决策 1：维持三层错误架构，修复 shared::error 层污染

**决策**：
1. 将 `InfrastructureError` 枚举从 `src/shared/error/result.rs` 迁移到 `src/infrastructure/error.rs` 或 `src/shared/error/` 之外
2. `src/shared/error/` 只保留 `ErrorContext` trait 和 `LogIfError` trait
3. `GameResult<T>` 废弃

**理由**：
- 消除概念违规：Shared 层不应包含基础设施概念
- 当前无循环依赖不代表未来安全，修复是从源头阻断风险
- 迁移影响极小：`GameResult`/`InfrastructureError` 仅被 UI 层 3 处使用

### 决策 2：建立领域错误注册清单，逐步补齐缺失枚举

**决策**：
1. 不强制要求所有模块立刻创建领域错误枚举（避免过度设计）
2. 但为所有核心模块建立"是否已有领域错误枚举"的可见性清单
3. 当某个模块的 unwrap 调用超过 5 处，或者出现 `Option.unwrap()` 链时，应创建其领域错误枚举
4. 当前优先处理 high-unwrap 模块：inventory(40), trigger(58), attribute(15), effect(13), battle(9), buff(9)

**理由**：
- 领域错误枚举只有在模块有明确的、可恢复的失败场景时才有价值
- 纯数据转换/查询模块（如 stacking、execution、cue）使用 `Option` 或纯函数更合理
- 强制所有模块创建枚举会产生无业务价值的空枚举

### 决策 3：推动 error_monitor 通道实际使用，但不强制替换

**决策**：
1. `GameErrorMessage` + `error_monitor` 保持当前设计（正确）
2. 不在当前阶段强制替换所有 `bevy::log::warn!` 为 `GameErrorMessage` 发送
3. 当新增 System 级错误处理代码时，优先使用 `GameErrorMessage` 通道
4. 在 `docs/00-governance/coding-rules.md` 中补充"System 内不可恢复错误应发送 GameErrorMessage"的提示

**理由**：
- 全量替换的 ROI 低（~200 处日志调用，大部分合理）
- error_monitor 主要用于"System 内不可恢复但不应 panic 的场景"
- 系统性改造应放到 Phase 5（质量加固）统一执行

### 决策 4：logging 架构保持，不重构

**决策**：
1. 不改造 `infrastructure::logging` 模块结构
2. LogPlugin 的 19 个 Observer 保留作为关键事件的结构化日志出口
3. 允许业务层直接使用 `bevy::log::info!` / `tracing::info!` 记录辅助信息
4. 只有当新领域的关键事件需要结构化日志时，才新增 Observer + shared::event 类型

**理由**：
- 当前架构可工作，不存在功能缺失
- 改造 43 个文件全部事件化的成本 > 收益
- 直接日志在调试阶段是合理的生产力工具

## Module Design

### 目标状态：shared/error 精简后

```
src/
├── shared/error/                   # 共享错误工具层（零外部依赖）
│   ├── mod.rs                      # 仅导出 ErrorContext + LogIfError
│   ├── context.rs                  # ErrorContext trait（不变）
│   └── extensions.rs               # LogIfError trait（不变）
│
├── infrastructure/
│   ├── error.rs                    ← [新增] InfrastructureError + InfraResult<T>（从 shared/error 迁移）
│   ├── assets/asset_error.rs       # AssetError（已有）
│   └── ...
│
├── core/
│   ├── ability/domain/error.rs     # SkillError（已有）
│   ├── battle/domain/error.rs      # BattleError（已有）
│   ├── buff/domain/error.rs        # BuffError（已有）
│   ├── inventory/domain/error.rs   # InventoryError（已有）
│   ├── effect/domain/error.rs      ← [待定] 如果 unwrap 超过 threshold
│   ├── attribute/domain/error.rs   ← [待定]
│   └── ...
│
└── app/
    ├── error_event.rs              # GameErrorMessage（已有，不变）
    └── error_monitor.rs            # error_monitor system（已有，不变）
```

### 迁移路径

| 步骤 | 内容 | 影响文件 | 风险 |
|------|------|----------|------|
| 1 | 新建 `infrastructure/error.rs`，从 shared/error/result.rs 复制 InfrastructureError + InfraResult<T> | 1 新建 | 低 |
| 2 | shared/error/mod.rs 移除 `pub use result::{GameResult, InfrastructureError}` | 1 修改 | 高（需要确认无其他导入者） |
| 3 | UI 层 3 处 imports 改为 `use crate::infrastructure::InfrastructureError` | 3 修改 | 低 |
| 4 | 删除 shared/error/result.rs | 1 删除 | 低 |
| 5 | 编译验证 | — | — |
| 6 | 在代码审查标准中增加"检查模块是否应创建领域错误枚举"的 checklist | 1 文档 | 无 |

## Communication Design

```
┌─────────────────────────────────────────────────────────────────────┐
│ 跨层错误上报通道（当前设计正确，待推广使用）                         │
│                                                                     │
│  System 内不可恢复错误                                                 │
│       │                                                             │
│       ├──→ GameErrorMessage Message (App Layer)                     │
│       │       │                                                     │
│       │       ├──→ error_monitor System → tracing::error! 日志      │
│       │       │                                                       │
│       │       └──→ (未来) UI Toast 通知                               │
│       │                                                             │
│       └──→ 领域内预期异常 → 领域 Error 枚举，用 ? 传播               │
│                                                                     │
│ 关键业务事件结构化日志（Observer 模式，合规）                        │
│  Domain Event (shared::event::battle::DamageDealt)                   │
│       │                                                             │
│       └──→ LogObserver::log_damage_dealt → info!(结构化字段)         │
│                                                                     │
│ 辅助调试日志（直接 tracing，合规）                                   │
│  fn some_system() {                                                 │
│      bevy::log::debug!("缓存未命中, key={}", key);                   │
│      bevy::log::trace!("buff tick: entity={:?}", entity);            │
│  }                                                                  │
└─────────────────────────────────────────────────────────────────────┘
```

## 边界定义

### 允许

- ✅ 各 `core/xxx/domain/` 定义自己的 `XxxError` + `XxxResult<T>`
- ✅ 各 `infrastructure/xxx/` 定义自己的基础设施错误枚举
- ✅ 业务层使用 `bevy::log::info!`/`debug!`/`warn!`/`trace!` 记录辅助日志
- ✅ `LogPlugin` 注册 Observer 系统监听领域事件，输出结构化日志
- ✅ `GameErrorMessage` 作为跨层错误上报通道
- ✅ `shared::error` 提供 `ErrorContext` 和 `LogIfError` 两个零依赖工具 trait

### 禁止

- 🟥 **禁止 shared::error 包含任何错误枚举类型**（包括 `InfrastructureError`、`AppError`、领域错误联合体）
- 🟥 **禁止创建全局 `AppError` 大枚举**
- 🟥 **禁止业务层使用 `anyhow::Error` 或 `Box<dyn Error>` 作为返回类型**
- 🟥 **禁止在 core 层使用 `println!`、`dbg!`**
- 🟥 **禁止在高频 System（每帧 10+ 次）中使用 info! 级别日志**（仅允许 error!）
- 🟥 **禁止用不包含完整上下文的 bare 变体定义领域错误**（如 `InvalidTarget` 缺少参数）
- 🟥 **禁止将规则失败（RuleFailure）编码为 `Result::Err`** 应使用专门的结果枚举

## 后果

### 正面

- 消除 shared/error 的概念违规，使其真正成为"零依赖共享工具层"
- 明确了日志和错误处理的正确使用模式，不再强制"所有日志必须事件化"
- 建立了领域错误覆盖的可见性清单
- error_monitor 通道保留但不强推

### 负面

- `InfrastructureError` 迁移需要修改 UI 层 3 处 import（低成本）
- 13 个核心模块仍然没有领域错误枚举（非 blocker，但需要持续关注）
- 不推动全量日志改造意味着调试日志和结构化日志的双轨制将持续存在

## 替代方案

### 方案 A：全面集中化（已否决）

将所有错误枚举合并到 `shared/error/`，所有日志通过中央 Observer。

**否决理由**：违反架构的"Feature First"和"领域独立"原则，导致 shared 层膨胀为上帝模块。且 43 个文件的改造量在当前阶段不可接受。

### 方案 B：保持现有完全不动（已否决）

保留 `InfrastructureError` 在 shared/error/，不推动任何改变。

**否决理由**：目前虽不造成编译错误，但 shared 层包含基础设施概念是架构红线的松弛。小成本修复可避免未来耦合。

### 方案 C：本 ADR 方案（选择）

轻度整治：修 shared/error 违规 + 建可见性清单 + 推广 error_monitor。

**理由**：最小成本解决架构违规，同时不过度设计。

## 架构自检

- [x] 符合 ECS 约束 — 错误处理不引入 Entity/Component 模式，Observer 使用标准 Bevy Message
- [x] 符合 Plugin 注册顺序 — LogPlugin 在核心插件之后、UI 之前注册
- [x] 没有创建禁止的模块 — 未使用 components.rs/systems.rs/utils.rs
- [x] Effect/Modifier Pipeline 没有被绕过 — 错误/日志不影响管线
- [x] 符合"定义与实例分离"原则 — 错误枚举是定义，错误值是实例
- [x] 符合"规则与内容分离"原则 — 错误枚举不变，不通过 RON 配置
- [x] 已检查 `docs/01-architecture/`、`docs/02-domain/` 相关文档

## 交接指引

| 下游角色 | 交接内容 |
|----------|----------|
| @feature-developer | 执行 shared/error → infrastructure/error.rs 迁移（步骤 1-5） |
| @code-reviewer | 在审查 checklist 中增加"模块是否应创建领域错误枚举"检查 |
| @test-guardian | 验证迁移后 UI 测试不受影响 |
| @refactor-guardian | 将"13 个核心模块缺少领域错误枚举"加入技术债清单 |

---

## 附录：外部架构评审摘要

### 评审结论

整体架构方向完全符合中大型 Rust 项目的错误体系最佳实践。`app / infrastructure / shared` 三处 error 分别对应**应用编排层、基础设施层、共享工具层**的三级分工，是业界成熟的分层错误模型。修正 shared 层概念错位后，整个体系自洽且具备良好的可扩展性。

### 与本 ADR 一致的关键判断

| 评审观点 | 对应 ADR 内容 |
|----------|-------------|
| shared 层概念污染是当前唯一架构偏差 | §发现二、§决策1 |
| error_monitor 通道闲置不是设计问题，是落地推广问题 | §发现四、§决策3 |
| 日志双轨制是可接受的（关键事件走 Observer + 调试日志直接 tracing） | §发现三、§决策4 |
| 200+ unwrap 不需要全部治理，应区分场景 | §决策2（threshold 触发） |
| 禁止全局 AppError、禁止 anyhow、禁止无上下文错误变体 | §边界定义-禁止🟥 |

### 有价值的补充建议

| 建议 | 来源 | 当前状态 |
|------|------|----------|
| 严格界定 `infrastructure::error.rs` 边界：只放跨子模块公共错误变体，各子模块保留自己的细分错误枚举 | 外部评审 §三-1 | 代码已按此实现，待补充文档 |
| 提供 `SystemResultExt` trait（`.report_error()` 方法）降低 error_monitor 接入成本 | 外部评审 §三-2 | 待未来实现 |
| unwrap 管理按场景区分（init/测试可接受，运行时必须治理），而非只看数量 | 外部评审 §三-3 | §决策2 threshold 策略需补充场景标签 |
| 未来路径：Bevy 原生 System 返回 Result 后，error_monitor 可与其原生机制联动 | 外部评审 §三-4 | 待 Bevy 升级后评估 |
