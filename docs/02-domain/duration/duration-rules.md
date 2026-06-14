---
id: 02-domain.duration.duration-rules
title: Duration Rules
status: draft
owner: domain-designer
created: 2026-06-14
updated: 2026-06-14
tags:
  - domain
  - duration
---

# 持续策略领域

Version: 1.0
Status: Proposed

持续策略领域管理 Buff/Effect 的持续时长规则，包括回合倒计时、条件终止和战斗结束终止等多种策略。

核心原则：
- 🟩 持续策略独立于 Buff 定义，是可组合的策略组件（宪法 1.1.6 组合优先）
- 🟩 持续结算在 TurnEnd 阶段统一执行，禁止在其他阶段递减
- 🟩 每个 Buff 必须有明确的 DurationPolicy，禁止永久存在（宪法 11.3.1 Buff 四阶段生命周期）
- 🟩 过期时必须清理对应的 Modifier，保证临时状态干净（宪法 11.3.1 Remove 阶段）
- 🟥 禁止在非 TurnEnd 阶段递减回合倒计时
- 🟥 禁止跳过 Modifier 清理（宪法 8.0.3）

---

# 宪法合规声明

本领域遵循以下宪法条款：

| 条款编号 | 条款名称 | 合规状态 | 说明 |
|----------|----------|----------|------|
| 11.3.1 | Buff 四阶段标准化 | 🟩 已合规 | Duration 管理 Apply→Tick→Expire→Remove 中的 Tick 和 Expire |
| 1.1.6 | 组合优先于继承 | 🟩 已合规 | DurationPolicy 作为可组合策略组件，独立于 Buff 定义 |
| 1.1.2 | 定义与实例分离 | 🟩 已合规 | DurationPolicy（定义态）与 DurationMarker（运行态）分离 |
| 8.0.3 | 属性修改规范 | 🟩 已合规 | 过期时清理 Modifier，保证属性值正确还原 |
| 2.2.4 | Message 跨域广播 | 🟩 已合规 | BuffExpired/DurationTerminated 用于跨域通知 |
| 2.2.6 | 领域事件是唯一事实源 | 🟩 已合规 | 过期事件作为 Buff 移除的业务事实 |
| 11.7.1 | 读路径无副作用 | 🟩 已合规 | Duration 检查为纯读操作 |
| 18.4.1 | 战斗完全可重现 | 🟩 已合规 | Duration 结算结果确定，支持回放 |

---

# 四级通信机制（宪法 2.2）

持续策略领域在四级通信机制中的定位：

| 通信层级 | 用途 | 持续策略领域应用 |
|----------|------|-------------|
| Hook（2.2.1） | 组件生命周期 | DurationMarker 组件添加/移除时的副作用 |
| Trigger（2.2.2） | Feature 内事件链 | 条件终止（UntilXxx）通过 Trigger 事件驱动 |
| Observer（2.2.3） | 局部状态变化响应 | Duration 变化触发的 UI 层数刷新 |
| Message（2.2.4） | 跨域广播 | BuffExpired/DurationTerminated 通知 Buff 领域移除 |

禁止事项（宪法 2.2.5）：
- 🟥 禁止将 Duration 检查逻辑事件化（纯函数直接调用即可）
- 🟥 禁止为临时状态清理单独创建领域事件

---

# 术语定义

## 持续策略（DurationPolicy）

Buff/Effect 的持续时长规则，定义 Buff 何时过期。

不是 Buff 本身。不是 Trigger。不是回合数。

关键属性：
- 策略类型：Turns(n) / UntilDeath / UntilMove / UntilAttack / UntilDamaged / BattleEnd / Permanent
- 每个 Buff 实例携带一个 DurationPolicy 实例
- DurationPolicy 是 Value Object，不可变
- 从 BuffDef 的 duration 字段反序列化

> **优化来源**: docs/01-architecture/skill-buff-abstraction.md §4.6 — Duration 作为十大正交子系统之一，与 StackPolicy 完全独立

---

## 回合倒计时（Turn Countdown）

基于回合数的递减机制，每个 TurnEnd 事件触发一次 tick。

不是真实时间。不是毫秒。不是帧数。

关键属性：
- tick 值从 Turns(n) 的 n 开始
- 每个 TurnEnd 阶段调用 tick() 递减 1
- tick 值归零时 Buff 过期
- tick 值为 0 时不递减（已在 TurnEnd 前过期）

---

## 条件终止（Conditional Termination）

满足特定条件时提前终止 Duration 的机制。

不是过期。不是驱散。不是回合倒计时。

关键属性：
- 触发条件：UntilDeath（目标死亡）/ UntilMove（Buff 持有者移动）/ UntilAttack（Buff 持有者攻击）/ UntilDamaged（Buff 持有者受伤）
- 条件满足时立即终止，不等待 TurnEnd
- 条件终止与回合倒计时互斥，同一 Buff 只使用一种
- 条件终止通过 Trigger 事件驱动

---

## 持续标记（Duration Marker）

记录 Buff 剩余持续时间的运行时标记。

不是 Buff 类型。不是 Component 数据。不是定义态配置。

关键属性：
- 附着在 Buff 实例上，随 Buff 生命周期存在
- 包含当前 tick 值或条件状态
- 通过 DurationPolicy 的 tick() 方法更新
- 过期时标记被清除，Buff 被移除

---

## 过期（Expiration）

Duration 结束后 Buff 被自动移除的事件。

不是驱散。不是手动移除。不是条件终止。

关键属性：
- 回合倒计时过期：tick 归零时触发
- 战斗结束过期：BattleEnd 事件触发
- 过期触发 BuffRemove 事件
- 过期时必须清理对应的 Modifier（参见 `attribute_modifier_rules.md#修饰器来源精确清理`）

---

## 无限持续（Infinite Duration）

Permanent 策略下的持续状态，Buff 永不过期直到手动移除。

不是永久存在。不是不可移除。不是无 Duration。

关键属性：
- DurationPolicy::Permanent 的运行时表现
- 不会因回合倒计时过期
- 不会因战斗结束过期
- 仅通过手动移除（Dispel / Cleanse）或死亡终止

---

# 领域边界

## 本领域负责

- DurationPolicy 的 7 种策略类型定义
- 回合倒计时的 tick 递减逻辑
- 条件终止的事件监听和提前终止
- 持续标记的创建、更新和过期检测
- 过期后的 Buff 移除通知

## 本领域不负责

- Buff 的施加和移除逻辑（由 Buff 领域负责）
- Buff 重复施加时的叠层策略（由 StackPolicy 领域负责）
- Buff 效果的触发时机（由 Trigger 领域负责）
- Modifier 的添加和清理（由 Attribute Modifier 领域负责）
- 回合状态机和行动顺序（由 Turn 领域负责）
- 战斗生命周期管理（由 Battle 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| Duration 过期 | Message（BuffExpired） | Buff 领域（移除 Buff） |
| 条件终止触发 | Message（DurationTerminated） | Buff 领域（移除 Buff） |
| 回合倒计时 tick | 函数调用（tick） | Turn 领域（TurnEnd 阶段） |
| 战斗结束终止 | Message（BattleEnded） | Buff 领域（批量移除） |

---

# 生命周期

## 状态列表

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Active | Duration 生效中 | Expired, Terminated |
| Expired | 回合倒计时归零 | —（终态） |
| Terminated | 条件满足提前终止 | —（终态） |

## 状态转换图

```
Active → Tick 递减 → Expired
Active → 条件满足 → Terminated
```

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| Active | Expired | Turns(n) 策略下 tick 归零 |
| Active | Expired | BattleEnd 事件触发 |
| Active | Terminated | UntilDeath：持有者死亡 |
| Active | Terminated | UntilMove：持有者移动 |
| Active | Terminated | UntilAttack：持有者攻击 |
| Active | Terminated | UntilDamaged：持有者受伤 |

---

# 不变量

## 不变量1：每个 Buff 必须有 DurationPolicy

任意时刻：

每个 Buff 实例必须携带一个 DurationPolicy，禁止无 Duration 的 Buff 存在。

违反表现：

Buff 实例的 DurationPolicy 为空或 None，导致 Buff 永久存在无法过期。

---

## 不变量2：回合倒计时仅在 TurnEnd 递减

回合生命周期：

Turns(n) 策略的 tick 递减仅在 TurnEnd 阶段执行，不在其他阶段调用。

违反表现：

在 TurnStart 或 Action 阶段调用 tick()，导致回合倒计时不一致。

---

## 不变量3：条件终止必须携带上下文

任意时刻：

UntilDeath / UntilMove / UntilAttack / UntilDamaged 的条件终止必须由 Trigger 事件驱动，事件必须携带 TriggerContext（参见 `trigger_rules.md#触发上下文`）。

违反表现：

无 TriggerContext 的条件终止，无法确定终止原因。

---

## 不变量4：过期 Buff 必须清理 Modifier

任意时刻：

Buff 过期或条件终止时，必须通知 Attribute Modifier 领域清理对应的 Modifier（参见 `attribute_modifier_rules.md#修饰器来源精确清理`）。

违反表现：

Buff 过期后 Modifier 残留，属性值与实际状态不一致。

---

## 不变量5：Permanent 策略必须手动移除

任意时刻：

DurationPolicy::Permanent 的 Buff 不会因回合或战斗结束过期，仅通过 Dispel / Cleanse / 死亡手动移除。

违反表现：

Permanent Buff 被回合倒计时意外移除。

---

# 业务规则

## 规则1：DurationPolicy 类型选择

禁止：
- 为 Buff 同时指定 Turns(n) 和 UntilXxx 条件（互斥）
- 使用 Turns(0) 作为持续策略（等价于不施加）
- 为 Debuff 指定 Permanent 策略（Debuff 必须有过期机制）

必须：
- Turns(n) 的 n 必须 ≥ 1
- UntilDeath 策略仅用于标记类/诅咒类 Buff
- BattleEnd 策略用于场地效果和临时增益

允许：
- Permanent 策略用于被动光环（需手动移除）
- Turns(1) 用于单回合效果

---

## 规则2：回合倒计时递减

禁止：
- 在 TurnEnd 以外的阶段递减 tick
- 递减后 tick 值为负数
- 跳过已过期 Buff 的清理

必须：
- 每个 TurnEnd 阶段遍历所有 Active Buff 的 Duration
- tick 递减使用 saturating_sub(1)
- tick 归零时立即标记为 Expired 并触发过期流程

允许：
- tick 递减后立即检查过期（在同一帧内完成）

---

## 规则3：条件终止处理

禁止：
- 条件终止后不清理 Modifier
- 条件终止后继续触发该 Buff 的后续 Trigger
- 在条件终止前执行 Buff 效果

必须：
- 条件终止时先清理 Modifier，再移除 Buff
- 条件终止后该 Buff 的所有 Trigger 立即失效
- 条件终止触发 BuffRemove 事件

允许：
- 条件终止时记录日志（终止原因 + Buff ID）

---

## 规则4：战斗结束处理

禁止：
- BattleEnd 后仍递减回合倒计时
- BattleEnd 后 Permanent Buff 被意外移除
- BattleEnd 后新施加 Buff

必须：
- BattleEnd 时批量终止所有 BattleEnd 策略的 Buff
- BattleEnd 时保留 Permanent 策略的 Buff（跨战斗保留）
- BattleEnd 时清理所有过期 Buff 的 Modifier

允许：
- BattleEnd 后 Permanent Buff 的状态保留到下一场战斗

---

# 流程管线

## 持续结算管线

```
TurnEnd 事件 → 遍历所有 Active Buff → 检查 DurationPolicy → tick 递减/条件检查 → 过期判断 → 移除 Buff + 清理 Modifier
```

### Step1：TurnEnd 事件触发

输入：TurnEnd 消息
处理：等待 TurnEnd 阶段开始
输出：持续结算管线启动
禁止：在 TurnEnd 以外的阶段启动持续结算

### Step2：遍历所有 Active Buff

输入：所有携带 DurationMarker 的 Buff 实例
处理：过滤出 Active 状态的 Duration
输出：待结算的 Duration 列表
禁止：跳过任何 Active Duration

### Step3：检查 DurationPolicy

输入：DurationPolicy 实例
处理：判断策略类型（Turns / UntilXxx / BattleEnd / Permanent）
输出：决定执行 tick 递减或条件检查
禁止：对 Permanent 策略执行 tick 递减

### Step4：tick 递减 / 条件检查

输入：DurationPolicy + TriggerContext（条件终止需要）
处理：Turns 策略调用 tick()；UntilXxx 策略检查条件是否满足
输出：更新后的 Duration 状态
禁止：在非 TurnEnd 阶段递减 Turns 策略

### Step5：过期判断

输入：更新后的 Duration 状态
处理：判断 tick 是否归零或条件是否满足
输出：过期/未过期标记
禁止：未过期的 Buff 被错误标记为过期

### Step6：移除 Buff + 清理 Modifier

输入：过期的 Buff 实例
处理：发送 BuffExpired 消息，通知 Attribute Modifier 领域清理 Modifier
输出：Buff 被移除，Modifier 被清理
禁止：跳过 Modifier 清理

---

## 条件终止管线

```
Trigger 事件（AfterDamaged / Move / Attack / Death）→ 检查 DurationPolicy → UntilXxx 匹配？ → 提前终止 + 清理 Modifier
```

### Step1：Trigger 事件触发

输入：AfterDamaged / AfterMove / AfterAttack / Death 消息
处理：获取事件上下文（TriggerContext）
输出：条件终止管线启动
禁止：无 TriggerContext 时启动条件终止

### Step2：检查 DurationPolicy

输入：持有者的 Buff 列表 + DurationPolicy
处理：遍历所有 Active Duration，检查是否为 UntilXxx 策略
输出：匹配的 Duration 列表
禁止：对非 UntilXxx 策略执行条件终止

### Step3：UntilXxx 匹配

输入：DurationPolicy + TriggerContext
处理：UntilDeath 检查 Death 事件；UntilMove 检查 Move 事件；UntilAttack 检查 Attack 事件；UntilDamaged 检查 Damaged 事件
输出：匹配成功标记
禁止：跨类型匹配（UntilMove 不能匹配 Damaged 事件）

### Step4：提前终止 + 清理 Modifier

输入：匹配成功的 Duration
处理：标记为 Terminated，发送 DurationTerminated 消息，清理 Modifier
输出：Buff 被移除，Modifier 被清理
禁止：跳过 Modifier 清理

---

# 数据结构

## DurationPolicy（持续策略定义）

职责：定义 Buff 的持续时长规则

结构：
- Turns(n)：回合倒计时，n 为回合数
- UntilDeath：直到持有者死亡
- UntilMove：直到持有者移动
- UntilAttack：直到持有者攻击
- UntilDamaged：直到持有者受伤
- BattleEnd：持续到战斗结束
- Permanent：永久持续

要求：
- 是 Value Object，不可变
- 从 BuffDef 的 duration 字段反序列化
- Turns(n) 的 n 必须 ≥ 1
- 每个 Buff 实例恰好携带一个 DurationPolicy

---

## DurationMarker（持续标记组件）

职责：记录 Buff 剩余持续时间的运行时状态

结构：
- tick：u32 — 回合倒计时剩余值（仅 Turns 策略使用）
- policy：DurationPolicy — 关联的持续策略
- is_active：bool — 是否处于 Active 状态

要求：
- 附着在 Buff 实例上
- Turns 策略使用 tick 字段递减
- UntilXxx / BattleEnd / Permanent 策略 tick 字段为 0
- 过期时 is_active 设为 false

---

## DurationDef（持续策略定义-反序列化用）

职责：RON 反序列化中间态，从 BuffDef 的 duration 字段解析

结构：
- Turns(n)：回合数
- UntilDeath / UntilMove / UntilAttack / UntilDamaged：条件类型
- BattleEnd：战斗结束
- Permanent：永久

要求：
- 通过 From trait 转换为 DurationPolicy
- Turns(n) 的 n 缺失时默认为 1

---

# 禁止事项

禁止：Buff 无 DurationPolicy

原因：无 Duration 的 Buff 会永久存在，导致属性无限增长

违反后果：Buff 堆积导致属性值异常，战斗平衡崩溃

---

禁止：在非 TurnEnd 阶段递减回合倒计时

原因：回合倒计时必须与回合生命周期同步，提前递减会导致持续时间不一致

违反后果：Buff 提前过期或延迟过期，影响战斗平衡

---

禁止：条件终止后不清理 Modifier

原因：Buff 移除后 Modifier 残留会导致属性值与实际状态不一致

违反后果：脱下装备后攻击力未恢复，Buff 过期后属性未还原

---

禁止：对 Permanent 策略执行 tick 递减

原因：Permanent 策略语义为永不过期，递减会破坏语义

违反后果：Permanent Buff 被意外移除，被动光环失效

---

禁止：Buff 同时指定 Turns 和 UntilXxx 策略

原因：两种策略互斥，同时指定导致终止条件不明确

违反后果：持续结算管线无法判断使用哪种终止逻辑

---

禁止：Turns(0) 作为持续策略

原因：Turns(0) 等价于不施加 Buff，语义错误

违反后果：Buff 施加后立即过期，效果不生效

---

禁止：BattleEnd 后新施加 Buff

原因：战斗结束后新施加的 Buff 无意义，且可能导致下一场战斗状态异常

违反后果：下一场战斗携带未清理的 Buff 状态

---

# AI 修改规则

## 如果新增 DurationPolicy 类型

允许：
- 在 DurationPolicy 枚举中新增变体
- 添加对应的 tick 或条件检查逻辑

禁止：
- 修改现有策略类型的终止语义
- 在 Turns 策略中引入条件逻辑
- 在 UntilXxx 策略中引入 tick 递减

优先检查：
- 新类型与现有 7 种策略是否互斥
- 过期时 Modifier 清理是否正确
- 新类型的 Trigger 事件是否在 Trigger 领域中定义

---

## 如果修改回合倒计时逻辑

允许：
- 调整 tick 递减的时机（仅限 TurnEnd 阶段内）
- 添加 tick 递减的日志记录

禁止：
- 在非 TurnEnd 阶段递减 tick
- 修改 saturating_sub(1) 的递减语义
- 跳过 tick 归零后的过期检查

优先检查：
- tick 递减后是否立即检查过期
- 过期 Buff 的 Modifier 是否被清理
- 多个 Buff 同时过期时的处理顺序

---

## 如果修改条件终止逻辑

允许：
- 添加新的 UntilXxx 条件类型
- 调整条件检查的触发时机

禁止：
- 条件终止后不清理 Modifier
- 条件终止前执行 Buff 效果
- 跨类型匹配（UntilMove 匹配 Damaged 事件）

优先检查：
- TriggerContext 是否携带足够的上下文数据
- 条件终止后 Trigger 是否立即失效
- 条件终止与回合倒计时的互斥性

---

## 如果测试失败

排查顺序：
1. 检查 DurationPolicy 类型是否正确（Turns / UntilXxx / BattleEnd / Permanent）
2. 检查 tick 递减是否在 TurnEnd 阶段执行
3. 检查条件终止的 TriggerContext 是否完整
4. 检查过期 Buff 的 Modifier 是否被清理
5. 检查 Permanent 策略是否被意外 tick 递减

---

# 宪法禁止事项

以下禁止事项源自 AI 开发宪法，持续策略领域必须严格遵守：

## 禁止：过期 Buff 不清理 Modifier（宪法 8.0.3）

原因：Buff 过期或条件终止时，必须通知 Attribute Modifier 领域清理对应的 Modifier。

违反后果：属性值与实际状态不一致，Buff 过期后属性未还原。

---

## 禁止：在非 TurnEnd 阶段递减回合倒计时（宪法 11.3.1）

原因：Buff 四阶段生命周期中 Tick 阶段固定在 TurnEnd 执行。

违反后果：Buff 提前过期或延迟过期，影响战斗平衡。

---

## 禁止：Duration 检查逻辑事件化（宪法 2.2.5）

原因：Duration 检查是纯函数直接调用，无需事件化。仅条件终止（UntilXxx）需要通过 Trigger 事件驱动。

违反后果：过度事件化导致调试困难、性能下降。

---

## 禁止：读路径产生副作用（宪法 11.7.1）

原因：Duration 过期检查为纯读操作，不修改游戏状态（仅在确认过期后才执行移除）。

违反后果：检查过程改变游戏状态、仿真结果不准确。

---

## 禁止：为未来需求过度设计 Duration 策略（宪法 1.1.7）

原因：当前 7 种策略类型已覆盖所有已知场景，禁止为未明确需求提前设计更多策略。

违反后果：架构复杂度上升、维护成本增加。

---

## 禁止：跳过临时状态清理（宪法 11.3.1 Remove 阶段）

原因：Buff 移除时必须清理所有临时状态（Modifier、Marker、StackCount），保证干净移除。

违反后果：残留的 Modifier 或 Marker 导致属性值异常、状态污染。

| 主题 | 详细文档 |
|------|----------|
| Modifier 添加和清理 | `docs/02-domain/attribute_modifier_rules.md#修饰器来源精确清理` |
| Trigger 事件和上下文 | `docs/02-domain/trigger_rules.md#触发上下文` |
| 回合生命周期 | `docs/02-domain/turn_rules.md` |
| Buff 施加和移除 | `docs/02-domain/buff_rules.md` |
| Buff 叠层策略 | `docs/02-domain/stack_policy_rules.md` |
