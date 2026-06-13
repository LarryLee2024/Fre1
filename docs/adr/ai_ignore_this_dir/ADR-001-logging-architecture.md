# ADR-001: 领域事件驱动日志架构

## 状态

Proposed

## 背景

当前代码中存在 35 处直接 `info!()` 调用（如 `src/battle/pipeline/execute.rs`、`src/buff/resolve.rs`），违反宪法 §14.8.2「INFO 级别核心业务事件必须通过领域事件链路输出」。

直接调用 `info!()` 导致：
- 日志散落在业务代码中，无法统一管理
- 新增 BattleReplay、Analytics 等下游时需逐个修改业务代码
- 日志格式不一致，难以检索

需要建立统一的日志架构，实现：
1. 业务代码与日志输出解耦
2. 一次修改全量生效
3. 支持未来无限扩展下游（Replay、Analytics、Achievement 等）

## 引用的领域规则

- `docs/domain/logging_rules_v1.md` — Logging 领域规则 v1.0
  - 不变量1：INFO 日志必须走领域事件链路
  - 不变量4：战斗日志与运行日志分离
  - 流程管线：DomainEvent → LogObserver → EventField → tracing

- `docs/AI开发宪法.md` §14.8 — 日志架构规范
  - §14.8.1：核心架构模式 = 领域事件驱动
  - §14.8.2：强制适用范围 = INFO 核心事件
  - §14.8.3：例外范围 = ERROR/WARN/DEBUG/TRACE

## 决策

采用「领域事件驱动日志」架构：

```
业务代码 → DomainEvent → LogObserver → TracingLog
                                           ↓
                                    tracing::info!(...)
```

核心原则：
1. **日志是领域事件的消费者**，不是业务代码主动调用的功能
2. **一个完整业务动作最多输出一条 INFO 日志**
3. **战斗日志与运行日志完全分离**

## Module Design

```
src/
└── infrastructure/
    └──logging/
      ├── mod.rs              # 模块入口，注册 LogPlugin
      ├── observer.rs         # LogObserver 系统，监听 DomainEvent
      ├── events.rs           # 日志相关 DomainEvent 定义
      └── config.rs           # 日志级别配置（可选）
```

### 职责划分

| 文件 | 职责 |
|------|------|
| `mod.rs` | 暴露公共接口，注册 LogPlugin |
| `observer.rs` | 监听 DomainEvent，组装 EventField，调用 tracing |
| `events.rs` | 定义日志相关的 DomainEvent 类型 |
| `config.rs` | 日志级别映射配置（可选，初期可硬编码） |

## Communication Design

### Message（跨 Feature 广播）

| Message | 发送方 | 接收方 | 用途 |
|---------|--------|--------|------|
| UnitAttacked | battle/execute | logging/observer | 攻击事件日志 |
| UnitDied | battle/events | logging/observer | 死亡事件日志 |
| BuffApplied | buff/apply | logging/observer | Buff 施加日志 |
| BuffExpired | buff/resolve | logging/observer | Buff 过期日志 |
| TurnStarted | turn | logging/observer | 回合开始日志 |
| TurnEnded | turn | logging/observer | 回合结束日志 |
| DamageApplied | battle/execute | logging/observer | 伤害应用日志 |
| HealApplied | battle/execute | logging/observer | 治疗应用日志 |
| StunApplied | buff/resolve | logging/observer | 晕眩施加日志 |
| DotApplied | buff/resolve | logging/observer | DoT 结算日志 |
| HotApplied | buff/resolve | logging/observer | HoT 结算日志 |

### Observer（局部响应）

LogObserver 作为统一消费者，监听上述所有 Message，输出 TracingLog。

### Hook（组件固有行为）

不涉及。日志不需要组件级别的副作用。

### 函数调用

不涉及。LogObserver 只监听 Message，不主动调用业务函数。

## 边界定义

### 允许

- logging 模块监听所有业务领域的 DomainEvent
- logging 模块调用 tracing 宏输出日志
- 业务领域触发 DomainEvent 通知日志

### 禁止

- 🟥 业务代码直接调用 `info!()` 输出核心业务事件
- 🟥 logging 模块修改任何业务状态
- 🟥 logging 模块依赖业务模块的具体实现（只依赖 Message 类型）
- 🟥 在 LogObserver 中包含业务逻辑

## Forbidden（禁止事项）

- 🟥 禁止：业务代码直接调用 `info!()` 输出核心业务事件 — 理由：INFO 日志必须走领域事件链路（§14.8.2）
- 🟥 禁止：logging 模块修改业务状态 — 理由：日志是观察者，不是执行者
- 🟥 禁止：在 LogObserver 中包含业务逻辑 — 理由：Observer 只负责日志输出
- 🟥 禁止：战斗日志混入 TracingLog — 理由：战斗日志与运行日志必须分离（§14.5）
- 🟥 禁止：每帧系统输出 INFO/DEBUG 日志 — 理由：高频日志影响性能（§14.4）
- 🟥 禁止：循环内输出 INFO 日志 — 理由：日志量不可控（§14.4）

## Definition / Instance Design

### Definition（不可变配置）

- 日志级别映射配置（可选，初期可硬编码在 observer.rs）

### Instance（运行时状态）

不涉及。LogObserver 是无状态系统，每次执行都是独立的。

## 后果

### 正面

1. **统一管理**：所有日志输出收口到 LogObserver，一次修改全量生效
2. **可扩展**：新增 BattleReplay、Analytics 等下游只需监听同一套 DomainEvent
3. **可测试**：LogObserver 可独立测试，验证日志输出格式
4. **解耦**：业务代码不再依赖 tracing，只依赖 DomainEvent

### 负面

1. **迁移成本**：现有 35 处直接 `info!()` 调用需要迁移
2. **事件类型增多**：需要为每个日志事件定义 DomainEvent 类型
3. **间接层**：增加了一层间接调用，但性能影响可忽略（日志本身是低频操作）

## 替代方案

### 方案1：保留直接 `info!()` 调用

优点：零迁移成本

缺点：
- 违反宪法 §14.8.2
- 无法统一管理日志
- 新增下游时需逐个修改业务代码

**结论：否决** — 违反宪法，不可接受

### 方案2：使用 Bevy Observer 监听组件变化

优点：利用 Bevy 原生 Observer 能力

缺点：
- Observer 用于同一 Feature 内的局部响应，日志是跨 Feature 的全局关注点
- 每个组件变化都触发 Observer，性能开销大

**结论：否决** — 不适合跨 Feature 的全局日志

### 方案3：使用 tracing Subscriber 自定义过滤

优点：利用 tracing 生态

缺点：
- 无法实现「一个业务动作最多一条 INFO 日志」的约束
- 无法保证日志格式一致性

**结论：部分采用** — tracing 作为底层输出，但上层仍需 DomainEvent 架构

## 实现步骤

### Phase 1：建立 logging 模块

1. 创建 `src/logging/mod.rs`
2. 定义 LogPlugin
3. 实现 LogObserver 系统

### Phase 2：定义 DomainEvent 类型

1. 在 `src/logging/events.rs` 定义日志相关事件
2. 事件类型与现有 Message 对齐（DamageApplied、BuffApplied 等）

### Phase 3：迁移现有日志

1. 识别 35 处直接 `info!()` 调用
2. 按模块逐个迁移到 DomainEvent 链路
3. 验证日志输出格式一致

### Phase 4：验证与测试

1. 单元测试：验证 LogObserver 输出格式
2. 集成测试：验证 DomainEvent → TracingLog 完整链路
3. 确认所有 INFO 日志携带 event 字段

## 架构合规性检查

- [x] 符合 ECS 约束（Entity=ID, Component=数据, System=行为）
- [x] 符合 Feature First 原则（logging/ 作为独立模块）
- [x] 符合 Logic/Presentation 分离（日志不包含业务逻辑）
- [x] 符合 Message 跨 Feature 广播（使用 Message 通信）
- [x] 符合 Definition/Instance 分离（日志配置可选）
- [x] 没有创建禁止的模块（components.rs/systems.rs/utils.rs）
- [x] 所有禁止事项已明确列出
- [x] 已检查 docs/domain/logging_rules_v1.md
