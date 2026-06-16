---
id: 02-domain.execution.execution-rules
title: Execution Rules
status: draft
owner: domain-designer
created: 2026-06-15
updated: 2026-06-15
tags:
  - domain
  - execution
---

# 执行算式领域

Version: 1.0
Status: Proposed
Source: `docs/其他/77.md` §五（缺失模块3：Execution）— Execution 作为独立一级领域
Changelog: v1.0 — 初始版本，定义 Execution Trait、ExecutionRegistry、Execution 计算管线，对齐 SRPG Lite-GAS 冻结架构

执行算式领域管理"效果执行时如何计算数值"——将伤害/治疗/护盾等计算公式抽离为独立的 Execution Trait，消灭巨型 match 分支，实现公式与业务解耦。

**核心原则**：
- 🟩 Execution = Trait 分发的公式执行层，每个计算类型独立实现
- 🟩 Effect 只负责声明"做什么"，Execution 负责"怎么算"
- 🟩 新增计算类型只需实现 Trait 并注册，不修改 Effect 或 Pipeline 代码
- 🟥 禁止在 Effect 内部写计算公式或硬编码 match 分支
- 🟥 禁止 Execution 直接修改游戏状态（纯计算，副作用由 Modifier 层后续处理）

**领域定位**：

```
Effect ── 意图（Intent）：产生什么效果（Damage/Heal/Shield）
  ↓
Stacking ── 堆叠（Stack）：同类型效果的叠加策略
  ↓
Execution ── 计算（Calculation）：具体数值怎么算 ← 本领域
  ↓
Modifier ── 修饰（Modification）：属性修改器挂载
  ↓
Attribute ── 属性（Attribute）：基础→派生属性刷新
```

---

# 术语定义

## 执行算式（Execution）

将效果的计算逻辑封装为独立 Trait 的执行单元。每种计算类型（伤害、治疗、护盾等）对应一个 Execution 实现，通过 ExecutionRegistry 注册和分发。

不是 Effect。不是 Formula。不是 Modifier。

关键属性：
- 实现 Execution trait（type_name / calculate）
- 注册到 ExecutionRegistry，运行时通过 type_name 查找分发
- 新增计算类型只需实现 trait 并注册，不修改 Effect 或 Pipeline 代码
- 纯计算层，不产生副作用（不修改 HP、不触发事件）
- Effect 在 Execute 阶段调用 Execution，而非内联公式

---

## 执行器（Executor）

Execution trait 的具体实现，封装一种计算类型的完整公式逻辑。

不是 Execution trait 本身。不是 EffectHandler。不是 Formula。

关键属性：
- 每个 Executor 对应一种计算类型（Damage / Heal / Shield）
- Executor 接收 ExecutionContext，输出 ExecutionResult
- Executor 内部可调用 Formula 进行表达式求值（可选）
- Executor 是无状态的，公式逻辑不依赖 ECS World

```rust
pub trait Execution: Send + Sync {
    /// 执行器的唯一标识，用于 Registry 查找
    fn type_name(&self) -> &'static str;

    /// 核心计算：接收上下文，返回计算结果
    fn calculate(&self, ctx: &ExecutionContext) -> ExecutionResult;
}
```

---

## 执行注册表（ExecutionRegistry）

全局唯一的执行器注册表 Resource，管理所有 Execution trait object 的注册和查找。

不是 FormulaRegistry。不是 EffectHandlerRegistry。不是 ModifierRuleRegistry。

关键属性：
- HashMap<String, Box<dyn Execution>> 存储
- 通过 type_name 字符串查找
- 游戏初始化时注册所有内置执行器
- 新增计算类型只需注册，不修改管线代码

---

## 执行上下文（ExecutionContext）

Execution 计算所需的全部输入参数，由 Effect 在调用时构建。

不是 ExecutionResult。不是 PendingEffectData。不是 FormulaInput。

关键属性：
- source_entity：攻击者 Entity ID
- target_entity：目标 Entity ID
- source_attrs：攻击者属性快照（攻击力、暴击率等）
- target_attrs：目标属性快照（防御力、抗性等）
- base_value：EffectDef 中声明的基础值（如伤害倍率 multiplier）
- modifier_value：Modifier 阶段修饰后的附加值
- stack_count：当前堆叠层数（来自 Stacking 阶段）
- execution_params：Executor 专用参数（如 ignore_def_percent、crit_multiplier）
- terrain_id：地形 ID（用于地形伤害计算）
- 纯数据传递，不包含 ECS 引用

---

## 执行结果（ExecutionResult）

Execution 计算完成后的输出，包含计算值和计算过程记录。

不是 EffectResult。不是 FormulaOutput。不是 PendingEffectData。

关键属性：
- value：i32 — 计算结果值（如最终伤害、最终治疗量）
- breakdown：Vec<StepRecord> — 计算过程记录（用于 Debug 面板和回放审计）
- is_critical：bool — 是否暴击（由 Execution 计算判定）
- 纯计算结果，不包含执行逻辑
- 由 Effect 在 Execute 阶段消费，交由 Modifier 层后续处理

---

# 领域边界

## 本领域负责

- Execution trait 的接口定义
- ExecutionRegistry 的注册和查找
- ExecutionContext / ExecutionResult 的数据结构定义
- 具体 Executor 的公式实现（DamageExecution / HealExecution / ShieldExecution 等）
- 公式分发：通过 Registry 查找而非 match 分支
- 计算过程记录（breakdown 用于 Debug 和回放）

## 本领域不负责

- Effect 的意图生成和生命周期管理（由 Effect 领域负责）
- 目标选择逻辑（由 Targeting 领域负责）
- 堆叠策略和层数管理（由 Stacking 领域负责）
- 属性修改器的挂载和清理（由 Modifier 领域负责）
- 基础→派生属性的刷新计算（由 Attribute 领域负责）
- 表现层事件下发（由 Cue 领域负责）
- 公式的表达式求值（由 Formula 领域提供，Execution 可选调用）
- 最终伤害/治疗的 HP 扣除/恢复（由 Effect Execute 阶段的副作用负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 计算请求 | 函数调用（registry.get(type_name).calculate(ctx)） | Effect 领域（Execute 阶段） |
| 计算结果 | 返回值（ExecutionResult） | Effect 领域 → Modifier 领域 |
| 计算过程 | ExecutionResult.breakdown | Replay 领域（回放审计） |
| 执行器注册 | 函数调用（registry.register()） | 初始化阶段（App 启动） |

---

# 生命周期

## Execution 执行状态

本领域为纯计算层，无持久状态机。每次计算独立执行，无状态转换。

ExecutionRegistry 在 App 启动时注册所有内置执行器，运行时不变。Execution 计算是纯函数，无状态。

---

# 不变量

## 不变量1：Execution 通过 Registry 查找分发

任意时刻：

Execution 执行时通过 ExecutionRegistry 查找执行器，禁止在 Effect 或 Pipeline 代码中硬编码 match 分发。

违反表现：

新增计算类型需要修改 Effect 或 Pipeline 调度代码。

---

## 不变量2：Execution 是纯计算层

任意时刻：

Execution.calculate() 不修改游戏状态（不扣除 HP、不触发事件、不修改属性）。计算结果通过 ExecutionResult 返回，由 Effect 在 Execute 阶段统一处理副作用。

违反表现：

Execution 内部直接扣除 HP 或触发领域事件。

---

## 不变量3：每种计算类型有且只有一个 Executor

任意时刻：

同一个计算类型（如 Damage）在所有 Effect 中使用相同的 Executor 实现。禁止为不同 Effect 中的同一计算类型实现不同逻辑。

违反表现：

fireball 的 Damage 和 basic_attack 的 Damage 使用不同的伤害计算逻辑。

---

## 不变量4：Execution 不感知 Effect 来源

任意时刻：

Execution 不应该知道它是被 Skill 还是 Buff 触发的。Execution 只管"怎么算"，不问"谁让我算的"。

违反表现：

Execution 内部判断 is_from_skill / is_from_buff 等来源标志。

---

## 不变量5：ExecutionContext 所有字段在调用时确定

任意时刻：

ExecutionContext 的所有字段在构建时确定，Execution 执行期间不修改 ExecutionContext。ExecutionContext 是只读输入。

违反表现：

Execution 修改了 ExecutionContext 中的 source_attrs 值。

---

# 业务规则

## 规则1：Effect 调用 Execution 而非内联公式

禁止：
- Effect 内部硬编码计算逻辑（如 `attack - defense`）
- Effect 直接读取属性值进行计算
- Effect 跳过 ExecutionRegistry 直接写计算代码

必须：
- Effect 通过 type_name 查找 Execution
- Effect 构建 ExecutionContext 传递给 Execution
- Effect 使用 ExecutionResult 作为计算结果

允许：
- Effect 根据 ExecutionResult 决定后续执行逻辑（如判断是否暴击触发 Cue）

---

## 规则2：Execution 纯计算约束

禁止：
- Execution 内部修改全局状态
- Execution 读取 ECS World
- Execution 产生随机数
- Execution 调用其他 Execution（无递归）
- Execution 调用自身（无自递归）

必须：
- Execution.calculate(ctx) 仅依赖 ExecutionContext
- Execution 输出仅通过 ExecutionResult 返回
- Execution 不持有运行时状态

允许：
- Execution 调用辅助计算函数（纯函数）
- Execution 调用 Formula 进行表达式求值（可选）

---

## 规则3：ExecutionRegistry 管理

禁止：
- 运行时动态注册 Execution
- 注册重复的 type_name
- 注册后修改已注册的 Execution

必须：
- App 启动时通过 register_defaults() 注册所有内置 Execution
- 新增计算类型需实现 Execution trait 并注册
- 注册时检查 key 是否存在（防止重复）

允许：
- 测试时临时注册额外 Execution

---

## 规则4：计算过程记录

禁止：
- ExecutionResult 不记录 breakdown（计算过程不可审计）
- breakdown 记录虚假或不一致的步骤

必须：
- ExecutionResult.breakdown 记录完整计算步骤
- 每步记录包含：步骤名称、输入值、输出值
- breakdown 用于 Debug 面板展示和回放审计

允许：
- breakdown 在 Release 构建中为空（性能优化）

---

# 流程管线

## Execution 在 GAS 链路中的位置

```
Ability ── 技能定义 + 施法校验
  ↓
Targeting ── 目标选取（纯函数）
  ↓
Effect ── 效果意图（Damage/Heal/Shield + 参数）
  ↓
Stacking ── 堆叠策略（覆写/刷新/叠加/上限）
  ↓
Execution ── 公式执行：计算具体数值 ← 本领域
  ↓
Modifier ── 属性修改器挂载
  ↓
Attribute ── 基础→派生属性刷新
  ↓
Tag ── 标签增减、状态判定
  ↓
Cue ── 表现事件下发
  ↓
Replay ── 指令+种子快照持久化
```

---

## Execution 计算管线

```
Effect 调用 → 查找 ExecutionRegistry → 构建 ExecutionContext → executor.calculate() → 返回 ExecutionResult
```

### Step1：Effect 调用

输入：EffectDef + GenerateContext + Stacking 结果
处理：从 EffectDef 中提取计算类型（type_name），构建 ExecutionContext
输出：type_name + ExecutionContext
禁止：EffectDef 中未指定计算类型时使用默认执行器

### Step2：查找 ExecutionRegistry

输入：type_name + ExecutionRegistry
处理：通过 ExecutionRegistry.get(type_name) 查找 Executor 实现
输出：Execution trait object
禁止：未注册的 type_name 直接调用 calculate

### Step3：构建 ExecutionContext

输入：GenerateContext + Stacking 结果 + 属性快照 + 地形信息
处理：构建 ExecutionContext 结构，填充所有字段
输出：ExecutionContext
禁止：ExecutionContext 缺少必需字段

### Step4：executor.calculate()

输入：ExecutionContext
处理：调用 Execution.calculate(ctx) 纯函数计算
输出：ExecutionResult
禁止：在 calculate 内部修改状态、生成随机数或访问 ECS World

### Step5：返回 ExecutionResult

输入：ExecutionResult
处理：传递给 Effect，Effect 在 Execute 阶段消费结果并处理副作用
输出：计算结果
禁止：修改 ExecutionResult 的 value 值

---

## Execution 注册管线

```
定义新 Executor → 实现 Execution trait → 注册到 ExecutionRegistry → Effect 通过 type_name 调用
```

### Step1：定义新 Executor

输入：新的计算需求
处理：创建 Executor 结构体，确定 type_name
输出：Executor struct
禁止：type_name 与已有 Executor 重复

### Step2：实现 Execution trait

输入：Executor struct + 计算逻辑
处理：实现 Execution trait 的 type_name 和 calculate 方法
输出：Executor 实现
禁止：calculate 内部包含随机逻辑或访问 ECS World

### Step3：注册到 ExecutionRegistry

输入：Executor struct + ExecutionRegistry
处理：调用 ExecutionRegistry.register(executor)
输出：注册完成
禁止：注册重复的 type_name

### Step4：Effect 通过 type_name 调用

输入：EffectDef 中的 type_name
处理：EffectHandler 在 Execute 阶段查找并调用 Execution
输出：ExecutionResult
禁止：EffectHandler 硬编码计算逻辑

---

# 数据结构

## Execution（执行算式 trait）

职责：定义执行器的计算接口

结构：
- type_name() → &'static str — 执行器唯一标识
- calculate(ctx: &ExecutionContext) → ExecutionResult — 核心计算

要求：
- 纯函数：无副作用、无随机、无状态
- 相同 ExecutionContext 产生相同 ExecutionResult
- 不修改 ExecutionContext
- 不访问 ECS World

---

## ExecutionRegistry（执行器注册表）

职责：存储所有 Execution trait object，通过 type_name 查找

结构：
- executors：String → Box<dyn Execution> 映射

要求：
- App 启动时通过 register_defaults() 注册内置执行器
- get(type_name) 返回 Option<&dyn Execution>
- 未注册的 type_name 返回 None（不 panic）
- 运行时不修改

---

## ExecutionContext（执行上下文）

职责：封装 Execution 计算所需的全部参数

结构：
- source_entity：Entity — 攻击者 Entity ID
- target_entity：Entity — 目标 Entity ID
- source_attrs：AttributeSnapshot — 攻击者属性快照
- target_attrs：AttributeSnapshot — 目标属性快照
- base_value：i32 — EffectDef 中声明的基础值
- modifier_value：i32 — Modifier 阶段修饰后的附加值
- stack_count：u32 — 当前堆叠层数
- execution_params：HashMap<String, f32> — Executor 专用参数
- terrain_id：Option<u32> — 地形 ID

要求：
- 纯数据传递，不存储持久状态
- 所有字段在构建时确定
- 不包含随机数（随机由 Random 系统注入）
- 是只读输入，Execution 执行期间不修改

---

## ExecutionResult（执行结果）

职责：封装 Execution 计算的结果

结构：
- value：i32 — 计算结果值
- breakdown：Vec<StepRecord> — 计算过程记录
- is_critical：bool — 是否暴击

要求：
- 纯计算结果，不包含执行逻辑
- breakdown 用于 Debug 面板展示和回放审计
- 不修改游戏状态

---

## 具体 Executor 示例

### DamageExecution

职责：计算伤害值

公式逻辑：
- 普通伤害：`Attack - Defense`（Attack 为 source_attrs.attack，Defense 为 target_attrs.defense）
- 真实伤害：`Attack`（忽略防御）
- 暴击伤害：`Attack * CritMultiplier`
- 地形伤害：`MaxHP * 10%`（基于目标最大生命值）

参数来源：
- execution_params["ignore_def_percent"]：忽略防御百分比
- execution_params["crit_multiplier"]：暴击倍率
- source_attrs.attack：攻击力
- target_attrs.defense：防御力
- target_attrs.max_hp：最大生命值

### HealExecution

职责：计算治疗量

公式逻辑：
- 基础治疗：`HealPower`（来自 source_attrs 或 base_value）
- 治疗加成：`HealPower * (1 + HealBonus)`

### ShieldExecution

职责：计算护盾吸收量

公式逻辑：
- 基础护盾：`ShieldPower`
- 护盾加成：`ShieldPower * (1 + ShieldBonus)`

---

# 禁止事项

- 🟥 禁止：Effect 内部硬编码计算公式 — 理由：计算逻辑应在 Execution 中实现
- 🟥 禁止：在 Effect 或 Pipeline 代码中硬编码 match 分发 — 理由：必须通过 ExecutionRegistry
- 🟥 禁止：Execution 直接修改游戏状态（扣血/回血/触发事件） — 理由：纯计算层，副作用由 Effect 处理
- 🟥 禁止：Execution 感知 Effect 来源（Skill/Buff） — 理由：职责单一，只做计算
- 🟥 禁止：为不同 Effect 中的同一计算类型实现不同 Executor — 理由：统一实现
- 🟥 禁止：新增计算类型时修改 Pipeline 或 Effect 调度代码 — 理由：扩展性
- 🟥 禁止：Execution 内部生成随机数 — 理由：随机性由 Random 系统注入
- 🟥 禁止：Execution 访问 ECS World — 理由：纯函数，独立测试
- 🟥 禁止：运行时动态注册 Execution — 理由：Registry 在启动时固定

---

# AI 修改规则

## 宪法合规检查清单

修改本领域代码前，必须逐项确认：
- 🟩 Execution 通过 Registry 查找分发，禁止 match 硬编码
- 🟩 Execution 是纯函数，不依赖 ECS World
- 🟩 Execution 不产生副作用，不修改游戏状态
- 🟩 ExecutionResult.breakdown 记录完整计算过程
- 🟩 新增 Executor 只注册不改管线

## 如果新增 Executor

允许：
- 实现 Execution trait 的 type_name 和 calculate 方法
- 在 ExecutionRegistry 中注册

禁止：
- 复用已有 type_name
- 修改现有 Executor 的计算逻辑
- 在 calculate 内部包含随机逻辑或访问 ECS World

优先检查：
- type_name 是否在 ExecutionRegistry 中注册
- calculate 是否为纯函数（无副作用）
- ExecutionContext 是否包含所有必需字段
- ExecutionResult.breakdown 是否正确记录

---

## 如果修改 Executor 计算逻辑

允许：
- 调整 calculate 方法内的计算公式
- 添加新的计算步骤（在 breakdown 中记录）

禁止：
- 修改 Execution trait 的接口签名
- 在 calculate 内部访问 ECS World
- 在 calculate 内部生成随机数

优先检查：
- 修改后是否保持纯函数性质
- 相同 ExecutionContext 是否仍产生相同 ExecutionResult
- breakdown 是否正确记录计算过程

---

## 如果修改 ExecutionRegistry

允许：
- 在 register_defaults() 中添加新 Executor 注册
- 调整注册顺序

禁止：
- 运行时动态注册
- 注册重复的 type_name
- 修改已注册 Executor 的实现

优先检查：
- 新 type_name 是否与现有 ID 冲突
- 注册后 get(type_name) 是否返回正确实现
- 未注册的 type_name 是否返回 None

---

## 如果测试失败

排查顺序：
1. 检查 type_name 是否在 ExecutionRegistry 中注册
2. 检查 Execution.calculate 是否为纯函数（无副作用、无随机）
3. 检查 ExecutionContext 是否包含所有必需字段
4. 检查 ExecutionResult 是否被正确使用
5. 检查 Execution 是否访问了 ECS World
6. 检查 breakdown 是否正确记录计算过程

---

# 交叉引用

| 主题 | 详细文档 |
|------|----------|
| Effect Pipeline（Generate → Modify → Execute） | `docs/02-domain/effect/effect-rules.md` |
| EffectHandler trait 分发 | `docs/02-domain/effect/effect-rules.md#效果处理器` |
| Modifier 修饰链 | `docs/02-domain/attribute-modifier/attribute-modifier-rules.md` |
| 公式表达式求值 | `docs/02-domain/formula/formula-rules.md` |
| 堆叠策略 | `docs/02-domain/stack-policy/stack-policy-rules.md` |
| 触发器和上下文 | `docs/02-domain/trigger/trigger-rules.md` |
| 属性定义与派生 | `docs/02-domain/attribute-modifier/attribute-modifier-rules.md` |
| SRPG Lite-GAS 冻结架构 | `docs/其他/77.md` §九（最终推荐架构）、§十（100分方案） |

---

## 附录：铃兰参考数据

> 领域：Execution | 来源：78铃兰.md §二、补充6、补充11 | 数据层：Runtime

#### ExecutionContext（Runtime层）

| 字段名 | 类型 | 说明 |
|--------|------|------|
| `attacker` | Entity | 攻击者 |
| `defender` | Entity | 防御者 |
| `skill_multiplier` | f32 | 技能倍率 |
| `damage_type` | DamageTypeTag | 伤害类型 |
| `is_crit` | bool | 是否暴击 |
| `is_backstab` | bool | 是否背击 |
| `height_advantage` | Option<HeightMod> | 高低地修正 |

#### 伤害类型分类

| 类型 | 防御减免 | 护盾交互 | 说明 |
|------|----------|----------|------|
| `Physical` | 受物防减免 | 物理护盾可挡 | 物理攻击 |
| `Magical` | 受魔防减免 | 魔法护盾可挡 | 魔法攻击 |
| `Pierce` | 无视防御 | 部分护盾可挡 | 穿透攻击 |
| `True` | 无视防御无视护盾 | 不可挡 | 真实伤害 |

#### 四段伤害公式

**第一段：攻击值结算**

```
最终攻击值 = (基础攻击 × (1 + Σ攻击百分比加算) + Σ固定攻击加成 + 属性转换攻击) × 属性克制倍率 × 高低地修正
```

| 输入 | 类型 | 叠加方式 |
|------|------|----------|
| 基础攻击 | f32 | - |
| 攻击百分比 | AddPercent | 加算 |
| 固定攻击加成 | Add | 加算 |
| 属性转换攻击 | Add | 加算 |
| 属性克制倍率 | MulPercent | 乘算 |
| 高低地修正 | MulPercent | 乘算 |

**第二段：防御值结算**

```
有效防御 = 目标防御 × Π(1 - 降防效果) × Π(1 - 无视防御效果)
```

| 输入 | 类型 | 叠加方式 |
|------|------|----------|
| 目标防御 | f32 | - |
| 降防效果 | MulPercent | 乘算 |
| 无视防御 | MulPercent | 乘算 |

**第三段：基础伤害计算**

```
基础伤害 = (最终攻击值 - 有效防御) × 技能倍率
```

**第四段：最终伤害修正**

```
最终伤害 = 基础伤害 × Π(1 + 增伤效果) × Π(1 + 易伤效果) × Π(1 - 减伤效果) × 暴击倍率
```

| 输入 | 类型 | 叠加方式 |
|------|------|----------|
| 基础伤害 | f32 | 第三段输出 |
| 增伤效果 | AddPercent | 加算 |
| 易伤效果 | MulPercent | 乘算 |
| 减伤效果 | MulPercent | 乘算 |
| 暴击倍率 | f32 | 暴击时=crit_dmg，否则=1.0 |

#### 治疗计算公式

```
基础治疗量 = 攻击 × 技能倍率 / 治疗强度
最终治疗量 = 基础治疗量 × (1 + 受治疗提升) × 暴击倍率 × AOE衰减
```

#### 护盾吸收顺序

```
伤害 → 先扣通用护盾 → 再扣物理/魔法专属护盾 → 最后扣HP
```

#### 数值边界强制规则

| 规则 | 约束 |
|------|------|
| 取整 | 向下取整，最小为1 |
| 防御下限 | ≥0 |
| 移动下限 | ≥1 |
| 暴击率上限 | ≤95% |
| 闪避率上限 | ≤80% |
| 减伤上限 | ≤90% |

#### Schema草案

```yaml
# execution_config.ron
(
  damage_formula: (
    stages: [
      (name: "attack_resolve",
       inputs: ["base_atk", "atk_pct_sum", "atk_flat_sum", "convert_atk"],
       formula: "(base_atk * (1 + atk_pct_sum) + atk_flat_sum + convert_atk) * element_mod * height_mod"),
      (name: "defense_resolve",
       inputs: ["target_def", "def_break_product", "armor_pen_product"],
       formula: "target_def * def_break_product * armor_pen_product"),
      (name: "base_damage",
       inputs: ["final_atk", "effective_def", "skill_multiplier"],
       formula: "(final_atk - effective_def) * skill_multiplier"),
      (name: "final_damage",
       inputs: ["base_dmg", "dmg_up_sum", "vuln_product", "dmg_red_product", "crit_multiplier"],
       formula: "base_dmg * (1 + dmg_up_sum) * vuln_product * dmg_red_product * crit_multiplier"),
    ],
    boundaries: (
      min_damage: 1,
      max_damage_reduction: 0.9,
      max_crit_rate: 0.95,
      max_dodge_rate: 0.8,
      max_hit_rate: 1.0,
      min_defense: 0.0,
      min_move_range: 1,
      rounding: Floor,
    ),
  ),
)
```
