---
id: 02-domain.stack-policy.stack-policy-rules
title: Stack Policy Rules
status: complete
owner: domain-designer
created: 2026-06-14
updated: 2026-06-15
tags:
  - domain
  - stack-policy
---

# 叠层策略领域

Version: 2.0
Status: **Complete** ✅

> 2026-06-15: Feature Developer 已实现 StackingRule 4-enum 模型（src/core/stacking/），17 测试通过。

叠层策略领域管理 Buff 重复施加时的叠层、刷新和替换规则，是后期 Buff 爆炸的核心抽象。

核心原则：
- 🟩 叠层策略独立于 Buff 定义，是可组合的策略组件（宪法 1.1.6 组合优先）
- 🟩 叠层与 Duration 刷新是两个独立操作，不可混淆
- 🟩 层数上限必须在施加时检查，防止无限叠加
- 🟩 StackingRule（定义态）与 StackCount（运行态）强制分离（宪法 1.1.2）
- 🟥 禁止跳过已有同类型 Buff 的检查直接施加

## Changelog

| 版本 | 日期 | 变更 |
|------|------|------|
| 1.0 | 2026-06-14 | 初始版本，3 变体模型（NoStack / Stackable / StackableNoRefresh） |
| 2.0 | 2026-06-15 | 重构为 4 变体模型（Replace / RefreshDuration / StackAdd / StackMax），命名统一为 StackingRule |

---

# 宪法合规声明

本领域遵循以下宪法条款：

| 条款编号 | 条款名称 | 合规状态 | 说明 |
|----------|----------|----------|------|
| 1.1.2 | 定义与实例分离 | 🟩 已合规 | StackingRule（定义态）与 StackCount（运行态）分离 |
| 1.1.6 | 组合优先于继承 | 🟩 已合规 | StackingRule 作为可组合策略组件，独立于 Buff 定义 |
| 2.2.4 | Message 跨域广播 | 🟩 已合规 | StackChanged/MaxStackReached 用于跨域通知 UI |
| 2.2.6 | 领域事件是唯一事实源 | 🟩 已合规 | 层数变化事件作为业务事实 |
| 11.7.1 | 读路径无副作用 | 🟩 已合规 | 叠层判定为纯读操作 |
| 11.7.2 | 写路径收口 | 🟩 已合规 | 层数修改通过统一流程处理 |

---

# 四级通信机制（宪法 2.2）

叠层策略领域在四级通信机制中的定位：

| 通信层级 | 用途 | 叠层策略领域应用 |
|----------|------|-------------|
| Hook（2.2.1） | 组件生命周期 | StackCount 组件添加/移除时的副作用 |
| Trigger（2.2.2） | Feature 内事件链 | 叠层判定结果触发的连锁操作（刷新/叠加） |
| Observer（2.2.3） | 局部状态变化响应 | 层数变化触发的 UI 刷新 |
| Message（2.2.4） | 跨域广播 | StackChanged/MaxStackReached 通知 UI 领域 |

禁止事项（宪法 2.2.5）：
- 🟥 禁止将叠层判定逻辑事件化（纯函数直接调用即可）
- 🟥 禁止为层数变化单独创建非白名单领域事件

---

# 术语定义

## 叠层策略（StackingRule）

Buff 重复施加时的处理规则，决定是替换、刷新 Duration、增加层数还是忽略。

不是 Buff 定义。不是 Duration。不是效果。

关键属性：
- 策略类型：Replace / RefreshDuration / StackAdd / StackMax(u32)
- 每个 Buff 实例携带一个 StackingRule 实例
- StackingRule 是 Value Object，不可变
- 从 BuffDef 的 stacking_rule 字段反序列化

> **优化来源**: docs/01-architecture/01-battle-gas/skill-buff-abstraction.md §4.7 — StackingRule 作为十大正交子系统之一，与 Duration 完全独立

---

## 刷新（Refresh）

重复施加时重置 Duration 的操作。

不是叠加。不是替换。不是合并。

关键属性：
- 刷新将 Duration 的 tick 重置为初始值
- RefreshDuration 策略执行刷新（不叠加）
- 刷新不影响当前层数（仅重置时间）

---

## 替换（Replace）

重复施加时用新 Buff 实例完全覆盖旧实例的操作。

不是刷新。不是叠加。不是合并。

关键属性：
- 旧 Buff 实例被移除（包括其 Modifier 和层数）
- 新 Buff 实例以初始状态（层数=1）施加
- Replace 策略执行替换
- 替换后旧 Buff 的所有状态归零

---

## 叠加（Stack）

重复施加时增加 Buff 层数的操作。

不是刷新。不是合并。不是替换。

关键属性：
- 叠加增加当前层数 +1
- StackAdd 和 StackMax(u32) 策略允许叠加
- 叠加效果与层数成正比（如中毒伤害 = 基础值 × 层数）
- StackMax(u32) 叠加上限后不再增加

---

## 层数上限（Max Stack）

允许的最大叠加层数。

不是 Damage 上限。不是等级上限。不是 Duration 值。

关键属性：
- StackMax(u32) 的参数为最大层数
- StackAdd 和 RefreshDuration 无上限概念（层数固定为 1 或无限）
- 超过上限时忽略叠加操作

---

## 层数（Stack Count）

Buff 当前的叠加层数。

不是 Buff 类型。不是持续时间。不是效果强度。

关键属性：
- 初始层数为 1
- 每次叠加 +1
- 移除时层数归零
- 效果计算时使用当前层数作为乘数

---

## 叠层判定（Stack Resolution）

新 Buff 施加时，检查已有同类型 Buff 并决定处理方式的流程。

不是 Buff 施加。不是 Duration 检查。不是 Trigger。

关键属性：
- 检查目标：同一 Entity 上的同类型 Buff
- 判定输入：StackingRule + 当前层数
- 判定输出：替换 / 刷新 / 叠加 / 忽略
- 判定时机：Buff 施加前

---

# 领域边界

## 本领域负责

- StackingRule 的 4 种策略类型定义
- 重复施加时的叠层判定逻辑
- 层数上限检查
- 刷新 Duration 的操作
- 层数与效果强度的映射关系

## 本领域不负责

- Buff 的施加和移除逻辑（由 Buff 领域负责）
- Duration 的 tick 递减（由 Duration 领域负责）
- Buff 效果的触发时机（由 Trigger 领域负责）
- Modifier 的添加和清理（由 Attribute Modifier 领域负责）
- 效果数值计算（由 Formula 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 叠层判定结果 | 函数调用（resolve_stack） | Buff 领域（决定施加/刷新/替换/忽略） |
| Duration 刷新 | 函数调用（refresh_duration） | Duration 领域（重置 tick） |
| 层数变化通知 | Message（StackChanged） | UI 领域（显示层数） |
| 层数上限达到 | Message（MaxStackReached） | UI 领域（提示上限） |

---

# 生命周期

## 状态列表

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Single | 层数为 1，不可叠加 | Stacked |
| Stacked | 层数 > 1，已叠加 | Single, Maxed |
| Maxed | 层数达到上限 | Stacked |

## 状态转换图

```
Single → 首次叠加 → Stacked → 叠加 → Maxed
Stacked → 移除一层 → Single
Maxed → 移除一层 → Stacked
```

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| Single | Stacked | StackAdd 或 StackMax 策略下重复施加，层数 < 上限 |
| Stacked | Maxed | 层数达到 StackMax(n) 的 n |
| Maxed | Stacked | 移除一层（Dispel / 过期） |
| Stacked | Single | 层数降为 1 |

---

# 不变量

## 不变量1：Replace 策略每次施加都覆盖

任意时刻：

Replace 策略的 Buff 重复施加时，旧实例被完全替换，新实例层数始终为 1。

违反表现：

Replace 策略的 Buff 层数 > 1，或旧实例未被移除。

---

## 不变量2：StackMax(u32) 不超过上限

任意时刻：

StackMax(u32) 策略的 Buff 层数不得超过参数值，超过时忽略叠加。

违反表现：

层数超过 StackMax(n) 的 n 值。

---

## 不变量3：RefreshDuration 不增加层数

任意时刻：

RefreshDuration 策略的 Buff 重复施加时，仅刷新 Duration，层数始终为 1。

违反表现：

RefreshDuration 策略的 Buff 层数 > 1。

---

## 不变量4：层数变化必须通知 UI

任意时刻：

层数发生变化（叠加 / 替换 / 移除一层 / 完全移除）时，必须发送 StackChanged 消息通知 UI 领域。

违反表现：

层数变化后 UI 未更新，显示旧层数。

---

# 业务规则

## 规则1：叠层判定逻辑

禁止：
- Replace 策略增加层数（应替换旧实例）
- RefreshDuration 策略增加层数（应仅刷新 Duration）
- StackMax(u32) 策略在未达上限时忽略重复施加
- 跳过叠层判定直接施加 Buff

必须：
- 新 Buff 施加前检查同一 Entity 上的同类型 Buff
- 根据 StackingRule 执行对应的叠层逻辑
- 叠层结果必须明确：替换 / 刷新 / 叠加 / 忽略

允许：
- StackAdd 在重复施加时仅叠加不刷新 Duration（Duration 刷新由 Duration 领域独立控制）

---

## 规则2：Duration 刷新规则

禁止：
- 刷新后 tick 值不等于初始值
- 刷新时重置层数

必须：
- RefreshDuration 策略重复施加时刷新 Duration
- 刷新操作不改变当前层数

允许：
- 刷新时记录日志（Buff ID + 新 tick 值）
- StackAdd / StackMax 策略的 Duration 刷新行为由 Duration 领域配置，叠层策略不干预

---

## 规则3：层数上限处理

禁止：
- 层数超过 StackMax(u32) 的参数值
- 超过上限后仍执行叠加操作
- 超过上限后不通知 UI

必须：
- 叠加前检查当前层数是否已达上限
- 达到上限时发送 MaxStackReached 消息
- StackMax(u32) 达到上限后忽略后续叠加

允许：
- StackAdd 策略无上限检查（无限叠加）

---

## 规则4：效果与层数映射

禁止：
- 效果计算忽略层数（所有层数效果相同）
- 层数为 0 时仍计算效果

必须：
- 效果值 = 基础值 × 当前层数（或按配置的比例）
- 层数为 1 时效果等于基础值
- 层数为 0 时不触发效果

允许：
- 特殊效果忽略层数（固定值效果）

---

# 流程管线

## 叠层判定管线

```
新 Buff 施加 → 检查已有同类型 Buff → StackingRule 判断 → 替换/刷新/叠加/忽略 → 更新 Buff 状态
```

### Step1：新 Buff 施加

输入：BuffDef + 目标 Entity
处理：获取待施加 Buff 的类型 ID
输出：Buff 施加请求
禁止：无 BuffDef 时启动叠层判定

### Step2：检查已有同类型 Buff

输入：目标 Entity 的 Buff 列表 + Buff 类型 ID
处理：遍历已有 Buff，查找同类型 Buff
输出：匹配的已有 Buff（或 None）
禁止：跳过检查直接施加

### Step3：StackingRule 判断

输入：StackingRule + 当前层数
处理：
- Replace → 替换旧实例
- RefreshDuration → 刷新 Duration
- StackAdd → 叠加（无上限检查）
- StackMax(n) → 检查上限后叠加
输出：叠层判定结果（替换 / 刷新 / 叠加 / 忽略）
禁止：未匹配到已有 Buff 时执行叠层逻辑

### Step4：执行叠层操作

输入：叠层判定结果
处理：
- 替换：移除旧 Buff 实例，以初始状态施加新实例
- 刷新：重置 Duration tick，不改变层数
- 叠加：层数 +1，Modifier 强度随层数调整
- 忽略：不执行任何操作
输出：更新后的 Buff 状态
禁止：叠加后层数超过上限

### Step5：更新 Buff 状态

输入：更新后的 Buff 状态
处理：更新 DurationMarker 和 StackCount，发送 StackChanged 消息
输出：Buff 状态更新完成
禁止：跳过 StackChanged 消息发送

---

## 层数移除管线

```
Buff 移除/Dispel → 检查当前层数 → 层数 -1 / 完全移除 → 清理 Modifier → 通知 UI
```

### Step1：Buff 移除/Dispel

输入：Buff 移除请求（过期 / Dispel / Cleanse）
处理：确定要移除的 Buff 实例
输出：Buff 移除指令
禁止：无移除请求时执行层数移除

### Step2：检查当前层数

输入：Buff 实例的 StackCount
处理：判断层数是否 > 1
输出：层数状态（多层 / 单层）
禁止：层数为 0 时执行移除

### Step3：层数 -1 / 完全移除

输入：层数状态
处理：
- 多层：层数 -1，不移除 Buff
- 单层：完全移除 Buff
输出：更新后的层数或移除结果
禁止：多层时完全移除 Buff

### Step4：清理 Modifier

输入：移除结果
处理：完全移除时清理 Modifier；层数 -1 时调整 Modifier 强度
输出：Modifier 清理完成
禁止：跳过 Modifier 清理

### Step5：通知 UI

输入：层数变化
处理：发送 StackChanged 消息
输出：UI 更新层数显示
禁止：跳过通知

---

# 数据结构

## StackingRule（叠层策略定义）

职责：定义 Buff 重复施加时的处理规则

结构：
- Replace：替换旧实例，新实例以初始状态施加
- RefreshDuration：仅刷新 Duration，不叠加
- StackAdd：叠加层数，无上限
- StackMax(u32)：叠加层数，上限为参数值

要求：
- 是 Value Object，不可变
- 从 BuffDef 的 stacking_rule 字段反序列化
- StackMax 的参数必须 ≥ 1
- 每个 Buff 实例恰好携带一个 StackingRule

---

## StackCount（层数标记组件）

职责：记录 Buff 当前的叠加层数

结构：
- count：u32 — 当前层数
- max：u32 — 最大层数（从 StackingRule 获取，Replace/RefreshDuration 为 1，StackAdd 为 u32::MAX，StackMax(n) 为 n）

要求：
- 附着在 Buff 实例上
- 初始 count 为 1
- count 不得超过 max
- 移除时 count 归零

---

## StackingRuleDef（叠层策略定义-反序列化用）

职责：RON 反序列化中间态，从 BuffDef 的 stacking_rule 字段解析

结构：
- Replace：无参数
- RefreshDuration：无参数
- StackAdd：无参数
- StackMax { max_stack }：最大层数

要求：
- 通过 From trait 转换为 StackingRule
- max_stack 缺失时默认为 1

---

# 禁止事项

禁止：Replace 策略增加层数

原因：Replace 语义为完全替换旧实例，增加层数破坏语义

违反后果：Replace Buff 层数 > 1，旧实例未被正确清理

---

禁止：RefreshDuration 策略增加层数

原因：RefreshDuration 语义为仅刷新 Duration，增加层数破坏语义

违反后果：RefreshDuration Buff 层数 > 1，效果计算错误

---

禁止：StackMax(u32) 超过上限仍叠加

原因：超过上限会导致属性无限增长，破坏游戏平衡

违反后果：Buff 层数无限增长，效果值异常

---

禁止：层数变化不通知 UI

原因：层数是 UI 展示的重要信息，不通知会导致显示不一致

违反后果：UI 显示旧层数，玩家无法判断 Buff 状态

---

禁止：层数为 0 时仍计算效果

原因：层数为 0 等价于 Buff 不存在，不应产生效果

违反后果：已移除的 Buff 仍产生效果

---

禁止：叠加前不检查已有同类型 Buff

原因：跳过检查会导致重复施加而非叠加，破坏叠层逻辑

违反后果：同一 Buff 多个实例并存，效果重复计算

---

# AI 修改规则

## 如果新增 StackingRule 类型

允许：
- 在 StackingRule 枚举中新增变体
- 添加对应的叠层判定逻辑

禁止：
- 修改现有策略类型的叠层语义
- 在 Replace 策略中引入叠加逻辑
- 在 RefreshDuration 策略中引入叠加逻辑

优先检查：
- 新类型与现有 4 种策略是否互斥
- Duration 刷新规则是否正确
- 层数上限检查是否生效

---

## 如果修改叠层判定逻辑

允许：
- 调整叠层判定的触发时机
- 添加叠层判定的日志记录

禁止：
- 跳过已有同类型 Buff 的检查
- 修改 StackingRule 的枚举变体语义
- 超过上限后仍执行叠加

优先检查：
- 叠层判定结果是否明确（替换 / 刷新 / 叠加 / 忽略）
- Duration 刷新规则与 StackingRule 是否一致
- 层数变化后 StackChanged 消息是否发送

---

## 如果修改层数上限

允许：
- 调整 StackMax(u32) 的参数值
- 添加层数上限的配置化支持

禁止：
- 参数值为 0（等价于 RefreshDuration）
- 运行时突破上限
- 层数上限为负数

优先检查：
- 新上限是否影响现有 Buff 的平衡性
- 层数达到上限后的行为是否正确
- 层数移除时 Modifier 清理是否正确

---

## 如果测试失败

排查顺序：
1. 检查 StackingRule 类型是否正确（Replace / RefreshDuration / StackAdd / StackMax）
2. 检查叠层判定是否跳过了已有同类型 Buff 的检查
3. 检查层数是否超过上限
4. 检查 Duration 刷新规则是否与 StackingRule 一致
5. 检查层数变化后 StackChanged 消息是否发送
6. 检查层数为 0 时效果是否被正确跳过

---

# 宪法禁止事项

以下禁止事项源自 AI 开发宪法，叠层策略领域必须严格遵守：

## 禁止：跳过已有同类型 Buff 检查（宪法 11.3.1）

原因：叠层判定是 Buff 四阶段生命周期中 Apply 阶段的核心逻辑，跳过检查会导致重复施加而非叠加。

违反后果：同一 Buff 多个实例并存，效果重复计算。

---

## 禁止：层数变化不通知 UI（宪法 2.2.4）

原因：层数是 UI 展示的重要信息，必须通过 Message 跨域通知。

违反后果：UI 显示旧层数，玩家无法判断 Buff 状态。

---

## 禁止：叠层判定逻辑事件化（宪法 2.2.5）

原因：叠层判定是纯函数直接调用，无需事件化。仅层数变化通知需要通过 Message。

违反后果：过度事件化导致调试困难、性能下降。

---

## 禁止：读路径产生副作用（宪法 11.7.1）

原因：叠层判定为纯读操作，不修改游戏状态（仅在确认结果后才执行叠加/刷新/替换）。

违反后果：判定过程改变游戏状态、仿真结果不准确。

---

## 禁止：为未来需求过度设计叠层策略（宪法 1.1.7）

原因：当前 4 种策略类型（Replace / RefreshDuration / StackAdd / StackMax）已覆盖所有已知场景。

违反后果：架构复杂度上升、维护成本增加。

---

## 禁止：修改 StackingRule 定义态（宪法 1.1.2）

原因：StackingRule 是不可变配置，运行时通过叠层判定逻辑处理。

违反后果：全局叠层配置被污染，多场战斗数据不一致。

| 主题 | 详细文档 |
|------|----------|
| Duration 持续策略 | `docs/02-domain/duration_rules.md` |
| Modifier 添加和清理 | `docs/02-domain/attribute_modifier_rules.md#修饰器来源精确清理` |
| Buff 施加和移除 | `docs/02-domain/buff_rules.md` |
| 触发器和上下文 | `docs/02-domain/trigger_rules.md` |
| 效果数值计算 | `docs/02-domain/formula_rules.md` |

---

## 附录：铃兰参考数据

> 领域：Stacking | 来源：78铃兰.md §五、§四.3 | 数据层：Definition

#### StackingDefinition（Definition层）

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| `id` | StackingId | PK | 堆叠策略唯一标识 |
| `stack_type` | StackType | - | 堆叠类型 |
| `max_stack` | u32 | ≥1 | 最大层数 |
| `on_max_action` | OnMaxAction | - | 达到最大层数时的行为 |
| `duration_refresh` | bool | - | 叠加时是否刷新持续时间 |

#### 基础堆叠规则

| 规则 | 说明 |
|------|------|
| 同名效果 | 按堆叠策略处理（刷新/叠层/取强） |
| 不同名效果 | 独立共存，互不影响 |

| 持续策略 | 说明 |
|----------|------|
| 刷新持续 | 重置为新的持续时间 |
| 取最大持续 | 保留较长的持续时间 |
| 独立计时 | 每层独立计时 |

#### StackType 枚举（8种堆叠类别）

| 类型 | 说明 | 典型场景 |
|------|------|----------|
| `RefreshDuration` | 刷新持续时间，不叠加层数 | 同一Buff重复施加 |
| `StackIndependent` | 独立叠加，每层效果独立 | 中毒（每层独立掉血） |
| `StackDecay` | 衰减叠加，层数增加但效果递减 | 燃烧（每层效果减半） |
| `StackUndispellable` | 不可驱散叠加，层数决定驱散难度 | 某些Boss Buff |
| `TakeStrongest` | 取最强效果，弱效果被覆盖 | 同类增伤取最大值 |
| `CounterStack` | 反击类堆叠，只触发最高优先级 | 多个反击效果 |
| `ShieldMaxRefresh` | 护盾取最大值+刷新时长 | 同类护盾 |
| `ShieldIndependent` | 不同类护盾独立共存 | 物理/魔法/通用护盾 |

#### 各类型详细配置

```yaml
# RefreshDuration — 刷新持续时间
stacking: RefreshDuration; max_stack: 1; duration_refresh: true

# StackIndependent — 独立叠加
stacking: StackIndependent; max_stack: 9; duration_refresh: false

# StackDecay — 衰减叠加（每层效果 = base_effect * (0.5 ^ (stack - 1))）
stacking: StackDecay; max_stack: 5; duration_refresh: true

# StackUndispellable — 不可驱散叠加
stacking: StackUndispellable; max_stack: 3; duration_refresh: true

# TakeStrongest — 取最强
stacking: TakeStrongest; max_stack: 1; duration_refresh: true

# CounterStack — 反击堆叠
stacking: CounterStack; max_stack: 8; duration_refresh: false
```

#### 特殊机制堆叠

**反击类堆叠**
| 规则 | 说明 |
|------|------|
| 多个反击共存 | 只触发优先级最高的一个 |
| 反击次数增加 | 对每个反击单独生效，分别加次数 |

**护盾类堆叠**
| 规则 | 说明 |
|------|------|
| 同类护盾 | 取最大值+刷新时长 |
| 不同类护盾 | 独立共存 |
| 吸收顺序 | 先通用→再物理/魔法专属 |

**控制类堆叠**
| 规则 | 说明 |
|------|------|
| 高级控制覆盖低级 | 强控>硬控>软控 |
| 同级控制刷新 | 刷新持续时间，不叠加 |
| 控制递减 | 连续同类控制→持续时间衰减 |

#### Schema草案

```yaml
# stacking_config.ron
(
  stacking_rules: [
    (id: "refresh_duration", stack_type: RefreshDuration,
     max_stack: 1, on_max_action: None, duration_refresh: true),
    (id: "stack_independent", stack_type: StackIndependent,
     max_stack: 9, on_max_action: None, duration_refresh: false),
    (id: "stack_decay", stack_type: StackDecay,
     max_stack: 5, on_max_action: None, duration_refresh: true),
    (id: "take_strongest", stack_type: TakeStrongest,
     max_stack: 1, on_max_action: None, duration_refresh: true),
    (id: "counter_stack", stack_type: CounterStack,
     max_stack: 8, on_max_action: None, duration_refresh: false),
    (id: "shield_max_refresh", stack_type: ShieldMaxRefresh,
     max_stack: 1, on_max_action: None, duration_refresh: true),
  ],
)
```
