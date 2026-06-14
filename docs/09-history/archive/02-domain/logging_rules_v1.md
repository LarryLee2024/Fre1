---
id: history.archive.logging_rules_v1
title: logging_rules_v1
status: archived
owner: domain-designer
created: 2026-06-14
updated: 2026-06-14
superseded_by: ../../03-technical/logging-rules.md
---

# Logging 领域

Version: 1.0

Logging 领域管理程序运行时的可观测性，通过「领域事件驱动」架构实现日志与业务逻辑解耦。日志是领域事件的消费者，而非业务代码主动调用的功能。

核心原则：
- 🟥 Logic / Presentation 分离（宪法 1.1.4）
- 🟥 领域事件驱动日志（宪法 14.8.1）
- 🟥 战斗日志与运行日志分离（宪法 14.5）
- 🟩 日志 = 领域事件履历（宪法 14.1.2）

---

# 术语定义

## TracingLog

运行时可观测性日志，使用 tracing 库输出。

不是 BattleLog。TracingLog 面向开发者，用于程序运行监控和问题排查。

关键属性：
- level：日志级别（ERROR / WARN / INFO / DEBUG / TRACE）
- target：模块标识（battle / buff / skill 等）
- event：业务事件名（与领域事件一一对应）
- fields：结构化数据字段

---

## BattleLog

玩家可见的战斗履历日志，使用独立 BattleLogEvent 体系。

不是 TracingLog。BattleLog 面向玩家，用于 UI 战斗记录和录像回放。

关键属性：
- entries：战斗事件列表
- turn_number：当前回合

---

## DomainEvent

业务中发生的重要事情，是日志的唯一触发源。

不是直接调用 info!。DomainEvent 是业务语义，tracing 是技术输出。

关键属性：
- event_name：事件名（如 UnitAttacked、BuffApplied）
- payload：事件携带的数据

---

## LogObserver

监听 DomainEvent 并输出 TracingLog 的统一消费者。

不是业务代码中的日志调用。LogObserver 集中管理所有日志输出，一次修改全量生效。

关键属性：
- 监听的事件类型
- 日志级别映射
- 输出格式

---

## EventField

结构化日志字段，所有 INFO 级别日志必须携带。

不是纯字符串日志。EventField 支持按字段过滤和检索。

关键属性：
- event：事件名（必填，值与事件名完全一致）
- target：模块标识（必填，与 Feature 目录名一致）
- 其他业务字段（实体 ID、数值等）

---

# 领域边界

## 本领域负责

- TracingLog 的分级规范（ERROR / WARN / INFO / DEBUG / TRACE）
- 结构化日志字段规范（event、target）
- DomainEvent → LogObserver → TracingLog 的架构模式
- 日志例外范围定义（哪些场景可绕过事件链路）
- 日志禁令清单（哪些行为绝对禁止）

## 本领域不负责

- BattleLog 玩家可见日志（由 battle_rules 领域负责）
- 战斗事件录制（由 battle_rules 领域的 BattleRecord 负责）
- UI 战斗日志展示（由 ui_rules 领域负责）
- 基础设施层日志（由各基础设施模块自行管理）
- 调试工具（Inspector、Replay 等由 debug_rules 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 业务事件发生 | DomainEvent | logging（LogObserver 监听） |
| 战斗日志记录 | BattleLogEvent | battle（BattleRecord 录制） |
| 运行异常 | ERROR/WARN 日志 | 各领域自行处理 |

---

# 生命周期

## 日志输出生命周期

| 阶段 | 含义 | 执行者 |
|------|------|--------|
| 业务事件触发 | DomainEvent 被触发 | 业务代码 |
| Observer 监听 | LogObserver 接收事件 | logging 领域 |
| 字段组装 | 构建 EventField | LogObserver |
| tracing 输出 | 调用 tracing 宏 | LogObserver |

## 阶段图

```
业务代码 → DomainEvent → LogObserver → TracingLog
                                           ↓
                                    tracing::info!(...)
```

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| 业务事件触发 | Observer 监听 | 事件被注册的 Observer 订阅 |
| Observer 监听 | 字段组装 | 事件数据有效 |
| 字段组装 | tracing 输出 | 日志级别符合当前配置 |

---

# 不变量

## 不变量1：INFO 日志必须走领域事件链路 🟥

宪法依据：§14.8.2

任意时刻：

所有 INFO 级别核心业务事件日志，必须通过触发 DomainEvent → LogObserver 链路输出，业务代码禁止直接调用 info!()。

违反表现：

业务代码中出现 `info!(event = "Xxx", ...)` 调用。

架构违规检测：

发现业务代码直接调用 info!() 输出核心业务事件时，必须停止。必须输出：

```
ARCHITECTURE VIOLATION: 业务代码直接调用 info!() 输出核心业务事件 [event_name]，违反"INFO 日志必须走领域事件链路"原则。
```

---

## 不变量2：所有 INFO 日志必须携带 event 字段 🟥

宪法依据：§14.3.1

任意时刻：

所有 INFO 级别日志必须携带 `event` 字段，值与事件名完全一致。

违反表现：

日志缺少 event 字段，无法按事件类型过滤和检索。

---

## 不变量3：日志 target 必须与模块名一致 🟥

宪法依据：§14.3.2

任意时刻：

日志 `target` 必须与所属 Feature 目录名完全一致（battle / buff / skill / map 等）。

违反表现：

无法通过 `RUST_LOG=buff=debug` 精准过滤单一领域日志。

---

## 不变量4：战斗日志与运行日志分离 🟥

宪法依据：§14.5

任意时刻：

玩家可见的战斗履历使用独立 BattleLogEvent 体系，不依赖 tracing。TracingLog 仅负责程序运行时的可观测性，不承担玩家可见的日志功能。

违反表现：

战斗日志混入 tracing 输出，或 tracing 输出暴露给玩家。

---

## 不变量5：一个业务动作最多一条 INFO 日志 🟥

宪法依据：§14.1.2

任意时刻：

一个完整业务动作最多输出一条 INFO 级别日志。DEBUG / TRACE 级可按需细化，但发布版默认关闭。

违反表现：

单次攻击输出多条 INFO 日志，日志量爆炸且难以阅读。

---

## 不变量6：例外范围必须遵守 🟥

宪法依据：§14.8.3

以下场景可直接调用 tracing 宏，无需走事件链路：
- ERROR / WARN 级别异常日志
- DEBUG / TRACE 级别调试日志
- 基础设施层、工具层代码
- 测试代码

违反表现：

ERROR 级别日志强行走事件链路，增加不必要的复杂度。

---

# 业务规则

## 规则1：日志分级 🟥

禁止：
- 🟥 每帧执行的系统输出 INFO / DEBUG 日志（仅允许 ERROR）
- 🟥 循环 / 迭代器内部输出 INFO 日志
- 🟥 堆砌日志替代 Inspector / Replay 等调试工具

必须：
- ERROR：程序异常（Entity 丢失、配置缺失、非法状态、存档损坏），必须附带完整复现上下文
- WARN：可继续运行的异常（配置不全、数据异常、回退逻辑触发）
- INFO：核心业务事件边界（战斗开始/结束、回合切换、单位移动/攻击/死亡、Buff 增减、任务完成）
- DEBUG：开发调试细节（寻路路径长度、Modifier 计算过程），发布版默认关闭
- TRACE：极细算法细节（A* 节点探索），仅专项调试时临时开启

---

## 规则2：结构化日志 🟥

禁止：
- 🟥 纯字符串日志
- 🟥 缺少 event 字段的 INFO 日志
- 🟥 target 与模块名不一致

必须：
- 所有 INFO 日志必须携带 `event` 字段
- 日志 `target` 必须与所属 Feature 目录名一致
- 所有关联实体 ID、数值必须作为独立字段

---

## 规则3：日志禁令 🟥

绝对禁止：
- 记录函数进入、退出、系统执行等技术流水账
- 在每帧执行的系统中输出 INFO / DEBUG 级别日志（仅允许 ERROR）
- 在循环、迭代器内部输出 INFO 级别日志
- 堆砌日志替代专业调试工具
- 业务代码直接调用 `info!` 输出核心业务事件（必须走领域事件链路）
- 绕过 Modifier / Effect 管道时打日志掩盖违规

---

## 规则4：错误处理与日志 🟩

禁止：
- 🟥 只输出 "operation failed" 无上下文的 ERROR 日志
- 🟥 层层重复打日志

必须：
- 所有 ERROR 级别日志必须包含完整的上下文信息
- 可失败逻辑优先返回 Result，错误由上层统一记录

---

# 流程管线

## 日志输出管线

DomainEvent → LogObserver → EventField → tracing

### Step1：业务事件触发

输入：业务逻辑执行
处理：触发 DomainEvent（如 UnitAttacked、BuffApplied）
输出：DomainEvent 实例
🟥 禁止：跳过 DomainEvent 直接调用 info!()

---

### Step2：Observer 监听

输入：DomainEvent
处理：LogObserver 接收事件，检查日志级别配置
输出：待处理事件
🟥 禁止：Observer 中包含业务逻辑

---

### Step3：字段组装

输入：DomainEvent payload
处理：构建 EventField（event、target、业务字段）
输出：结构化日志字段
🟥 禁止：输出纯字符串日志

---

### Step4：tracing 输出

输入：EventField
处理：调用 tracing::info!() / tracing::warn!() 等
输出：TracingLog
🟥 禁止：在输出中修改业务状态

---

# 数据结构

## DomainEvent（Message）

职责：业务事件，日志的唯一触发源

结构：
- event_name：事件名（如 UnitAttacked）
- payload：事件携带的数据（实体 ID、数值等）

要求：
- 🟥 业务代码只触发事件，不直接打日志（§14.8.2）
- 🟥 事件名与日志 event 字段完全一致

---

## LogObserver（System）

职责：监听 DomainEvent，输出 TracingLog

结构：
- 监听的事件类型列表
- 日志级别映射（默认 INFO）
- 输出格式模板

要求：
- 🟥 放在基础设施层，绝对不侵入业务模块（§14.8.2）
- 🟥 一次修改全量生效

---

## EventField（值对象）

职责：结构化日志字段

结构：
- event：事件名（必填）
- target：模块标识（必填）
- 其他业务字段

要求：
- 🟥 所有 INFO 日志必须携带 event 字段（§14.3.1）
- 🟥 target 必须与 Feature 目录名一致（§14.3.2）

---

## CombatLog（Resource）

职责：玩家可见的战斗履历

结构：
- entries：战斗事件列表
- turn_number：当前回合

要求：
- 🟥 使用独立 BattleLogEvent 体系，不依赖 tracing（§14.5）
- 🟥 面向玩家，不面向开发者

---

# 禁止事项

🟥 禁止：业务代码直接调用 info!() 输出核心业务事件

原因：INFO 级别日志必须走领域事件链路（§14.8.2）

违反后果：
- 日志散落在业务代码中，无法统一管理
- 新增 BattleReplay、Analytics 等下游时需逐个修改业务代码
- 日志格式不一致，难以检索

架构违规检测：

```
ARCHITECTURE VIOLATION: 业务代码直接调用 info!() 输出核心业务事件 [event_name]，违反"INFO 日志必须走领域事件链路"原则。
```

---

🟥 禁止：记录函数进入、退出、系统执行等技术流水账

原因：日志 = 领域事件履历，不是技术流水账（§14.1.2）

违反后果：
- 日志量爆炸，核心业务事件被淹没
- 无法按业务流程检索和分析

---

🟥 禁止：每帧执行的系统输出 INFO / DEBUG 日志

原因：高频日志影响性能，且无业务价值（§14.4）

违反后果：
- 帧率下降
- 日志文件快速膨胀

---

🟥 禁止：在循环 / 迭代器内部输出 INFO 日志

原因：单次操作可能触发大量循环，日志量不可控（§14.4）

违反后果：
- 单次攻击可能输出数百条日志
- 性能严重下降

---

🟥 禁止：堆砌日志替代专业调试工具

原因：Inspector、Replay 等工具效率远高于文本日志（§14.7.1）

违反后果：
- 调试效率低下
- 临时日志残留代码库

---

🟥 禁止：绕过 Modifier / Effect 管道时打日志掩盖违规

原因：日志不应成为架构违规的遮羞布

违反后果：
- 架构违规被日志掩盖，问题隐藏更深

---

# AI 修改规则

## 如果新增业务事件日志

允许：
- 新增 DomainEvent 类型
- 在 LogObserver 中新增事件监听

禁止：
- 🟥 在业务代码中直接调用 info!()
- 🟥 修改现有 Observer 的日志格式

优先检查：
- 事件名是否与日志 event 字段一致
- target 是否与模块名一致
- 是否在例外范围内（ERROR/WARN 可直接调用）

---

## 如果新增日志级别

允许：
- 在 LogObserver 中调整级别映射

禁止：
- 🟥 修改现有事件的默认级别
- 🟥 在 INFO 级别输出技术流水账

优先检查：
- 新级别是否符合分级规范
- 是否会影响现有日志过滤

---

## 如果新增日志输出目标

允许：
- 新增 Observer 实现（如 ReplayRecorder、AnalyticsTracker）

禁止：
- 🟥 修改 LogObserver 的职责（它只负责 TracingLog）

优先检查：
- 新目标是否与现有 Observer 冲突
- 是否共享同一 DomainEvent 源

---

## 如果测试失败

排查顺序：
1. 检查业务代码是否直接调用 info!()
2. 检查 INFO 日志是否携带 event 字段
3. 检查 target 是否与模块名一致
4. 检查是否在例外范围内
5. 检查日志级别是否正确

测试要求（宪法 13.0.1-13.0.3）：
- 🟩 单元测试：验证 LogObserver 输出格式
- 🟩 集成测试：验证 DomainEvent → TracingLog 完整链路
- 🟩 Bug 修复必须先编写重现测试（宪法 13.0.2）

---

# 宪法条款映射

| 宪法条款 | 本领域对应 |
|----------|-----------|
| 1.1.4 Logic/Presentation 分离 | 日志输出不包含业务逻辑 |
| 14.1.1 日志框架统一 | 统一使用 tracing |
| 14.1.2 日志 = 领域事件履历 | 只记录业务事件，不记录技术流程 |
| 14.3.1 结构化日志 | 所有 INFO 日志必须携带 event 字段 |
| 14.3.2 Target 规范 | target 必须与模块名一致 |
| 14.4 日志禁令 | 禁止技术流水账、高频日志等 |
| 14.5 战斗日志分离 | BattleLog 与 TracingLog 独立 |
| 14.8.1 领域事件驱动架构 | DomainEvent → LogObserver → tracing |
| 14.8.2 强制适用范围 | INFO 核心事件必须走事件链路 |
| 14.8.3 例外范围 | ERROR/WARN/DEBUG/TRACE 可直接调用 |

---

# 架构违规检测

| 违规行为 | 检测方式 | 输出 |
|----------|----------|------|
| 业务代码直接调用 info!() | 代码审查 | ARCHITECTURE VIOLATION: 业务代码直接调用 info!() 输出核心业务事件 [event_name]，违反"INFO 日志必须走领域事件链路"原则。 |
| INFO 日志缺少 event 字段 | 代码审查 | ARCHITECTURE VIOLATION: INFO 日志缺少 event 字段，违反"所有 INFO 日志必须携带 event 字段"不变量。 |
| target 与模块名不一致 | 代码审查 | ARCHITECTURE VIOLATION: 日志 target [xxx] 与模块名 [yyy] 不一致，违反"target 必须与模块名一致"不变量。 |
| 记录技术流水账 | 代码审查 | ARCHITECTURE VIOLATION: 日志记录函数进入/退出/系统执行，违反"日志 = 领域事件履历"原则。 |
| 每帧系统输出 INFO 日志 | 代码审查 | ARCHITECTURE VIOLATION: 每帧执行的系统输出 INFO 日志，违反"每帧系统仅允许 ERROR"禁令。 |
| 循环内输出 INFO 日志 | 代码审查 | ARCHITECTURE VIOLATION: 循环/迭代器内部输出 INFO 日志，违反"循环内禁止 INFO"禁令。 |
