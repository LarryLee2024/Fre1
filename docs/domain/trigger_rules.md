# 触发器系统领域

Version: 1.0
Status: Proposed

触发器系统领域管理 Buff 效果的触发时机、触发上下文和触发链，是 Buff 规则引擎的核心。

核心原则：
- Trigger 是 Buff 的核心——定义"什么时候触发"
- 触发链必须有限（最多 3 级），防止无限递归
- 同一 Buff 在同一 Trigger 点只能触发一次

---

# 术语定义

## 触发器（Trigger）

Buff 效果触发的时机点，定义效果在何时执行。

不是 Effect。不是 Duration。不是 Buff 本身。

关键属性：
- 11 种触发时机：TurnStart / TurnEnd / BeforeAttack / AfterAttack / BeforeDamaged / AfterDamaged / BeforeMove / AfterMove / KillTarget / Death / BattleStart / BattleEnd
- 每个 Buff 声明一个或多个 Trigger
- Trigger 是 Value Object，不可变
- 从 BuffDef 的 triggers 字段反序列化

---

## 触发上下文（TriggerContext）

Trigger 触发时携带的上下文数据，提供 Effect 执行所需的输入。

不是 ECS World。不是 Skill 定义。不是全局状态。

关键属性：
- trigger：触发的 Trigger 类型
- source_entity：触发源 Entity
- target_entity：触发目标 Entity
- damage_dealt：造成的伤害值（AfterAttack / AfterDamaged 需要）
- is_critical：是否暴击
- 纯数据传递，不存储持久状态

---

## 触发时机点（Trigger Point）

回合/战斗生命周期中的具体时机。

不是游戏事件。不是 Message。不是 System。

关键属性：
- 回合相关：TurnStart / TurnEnd
- 攻击相关：BeforeAttack / AfterAttack
- 受伤相关：BeforeDamaged / AfterDamaged
- 移动相关：BeforeMove / AfterMove
- 击杀相关：KillTarget
- 死亡相关：Death
- 战斗相关：BattleStart / BattleEnd
- 每个时机点有明确的生命周期阶段

---

## 触发链（Trigger Chain）

一个 Trigger 触发的 Effect 可能引发新的 Trigger 的链式反应。

不是递归。不是无限循环。不是事件广播。

关键属性：
- 最大链深度为 3 级
- 第 1 级：Buff Trigger 触发 Effect
- 第 2 级：Effect 产生的事件触发新 Buff 的 Trigger
- 第 3 级：新 Effect 产生的事件触发更多 Buff 的 Trigger
- 超过 3 级时终止，输出警告日志

---

## 触发匹配（Trigger Matching）

遍历 Buff 列表，匹配当前 Trigger 点的过程。

不是 Trigger 执行。不是 Effect 生成。不是条件检查。

关键属性：
- 输入：所有 Active Buff + 当前 Trigger 点
- 处理：遍历 Buff，检查 triggers 是否包含当前 Trigger 点
- 输出：匹配的 Buff 列表
- 匹配失败的 Buff 被跳过

---

## 触发链深度（Trigger Chain Depth）

触发链的当前嵌套级别。

不是 Buff 层数。不是回合数。不是链表长度。

关键属性：
- 初始深度为 0（外部事件触发）
- 每进入下一级 +1
- 最大深度为 3
- 超过最大深度时终止触发链

---

# 领域边界

## 本领域负责

- 11 种 Trigger 时机点的定义
- TriggerContext 的数据结构定义
- 触发匹配逻辑（遍历 Buff 匹配 Trigger 点）
- 触发链深度控制（最多 3 级）
- 同一 Buff 在同一 Trigger 点的去重

## 本领域不负责

- Buff 的施加和移除逻辑（由 Buff 领域负责）
- Duration 的 tick 递减（由 Duration 领域负责）
- Effect 的执行逻辑（由 Effect Pipeline 领域负责）
- 效果数值计算（由 Formula 领域负责）
- 回合状态机和行动顺序（由 Turn 领域负责）
- 战斗生命周期管理（由 Battle 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| Trigger 匹配完成 | 函数调用（match_triggers） | Buff 领域（获取匹配的 Buff） |
| Effect 生成请求 | Message（TriggerEffectReady） | Effect Pipeline 领域（执行 Effect） |
| 触发链深度超限 | Message（TriggerChainExceeded） | Debug 领域（记录警告） |
| TriggerContext 构建 | 函数调用 | 各生命周期阶段（Turn / Battle / Combat） |

---

# 生命周期

## 状态列表

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Waiting | Trigger 未触发 | Triggered |
| Triggered | Trigger 已触发，Effect 执行中 | Completed |
| Completed | Effect 执行完成 | Waiting |

## 状态转换图

```
Waiting → 事件到达 → Triggered → Effect 执行 → Completed → Waiting
```

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| Waiting | Triggered | 对应生命周期事件到达 |
| Triggered | Completed | Effect Pipeline 执行完成 |
| Completed | Waiting | 触发链重置 |

---

# 不变量

## 不变量1：触发链必须有限（最多 3 级）

任意时刻：

Trigger Chain 的嵌套深度不得超过 3 级。超过时终止触发链并输出警告日志。

违反表现：

触发链深度 > 3，导致无限递归。

---

## 不变量2：同一 Buff 在同一 Trigger 点只能触发一次

回合生命周期：

同一 Buff 实例在同一 Trigger 点（如 TurnStart）只能触发一次，防止重复执行。

违反表现：

同一 Buff 在 TurnStart 触发了两次效果。

---

## 不变量3：TriggerContext 必须包含足够的上下文数据

任意时刻：

TriggerContext 必须包含 Effect 执行所需的全部数据（source_entity / target_entity / damage_dealt 等）。

违反表现：

AfterAttack 触发的 TriggerContext 缺少 damage_dealt，吸血效果无法计算回复量。

---

## 不变量4：Trigger 时机点与生命周期阶段严格对应

任意时刻：

TurnStart / TurnEnd 仅在回合生命周期中触发；BattleStart / BattleEnd 仅在战斗生命周期中触发。

违反表现：

在 Action 阶段触发 TurnStart 事件。

---

# 业务规则

## 规则1：Trigger 时机点选择

禁止：
- 为 Buff 声明不存在的 Trigger 时机点
- 在非对应生命周期阶段触发 Trigger
- 为同一 Buff 声明重复的 Trigger 时机点

必须：
- Trigger 时机点必须是 11 种枚举值之一
- TurnStart / TurnEnd 仅在回合生命周期触发
- BeforeAttack / AfterAttack 仅在攻击生命周期触发

允许：
- 一个 Buff 声明多个不同的 Trigger 时机点
- Buff 声明空 Trigger 列表（纯被动效果）

---

## 规则2：触发链深度控制

禁止：
- 触发链深度 > 3 时继续触发
- 无深度检查的触发链执行
- 无限递归的触发链

必须：
- 每次进入下一级触发链时深度 +1
- 深度达到 3 时终止并输出警告日志
- 触发链完成后重置深度

允许：
- 深度为 0-3 的正常触发链执行
- 触发链终止时记录日志（Buff ID + 深度）

---

## 规则3：TriggerContext 数据完整性

禁止：
- TriggerContext 缺少 source_entity 或 target_entity
- AfterAttack 触发时 damage_dealt 为空
- BeforeDamaged 触发时缺少伤害预估

必须：
- TriggerContext 包含 trigger / source_entity / target_entity
- AfterAttack / AfterDamaged 必须携带 damage_dealt
- KillTarget 必须携带被击杀的 Entity

允许：
- 部分字段为 Option（如 is_critical 可选）

---

## 触发匹配规则

禁止：
- 跳过 Trigger 匹配直接执行 Effect
- 匹配时忽略 Buff 的 Active 状态
- 匹配时忽略同一 Trigger 点的去重

必须：
- 遍历所有 Active Buff，匹配当前 Trigger 点
- 已匹配的 Buff 在同一 Trigger 点不再重复匹配
- 匹配失败的 Buff 被跳过

允许：
- 匹配时记录日志（Buff ID + Trigger 点）

---

# 流程管线

## 触发匹配管线

```
Trigger 点到达 → 遍历所有 Active Buff → 匹配 Trigger → Condition 检查 → Effect 生成 → Effect Pipeline
```

### Step1：Trigger 点到达

输入：生命周期事件（TurnEnd / AfterAttack / AfterDamaged 等）
处理：构建 TriggerContext
输出：Trigger 匹配管线启动
禁止：无 TriggerContext 时启动匹配

### Step2：遍历所有 Active Buff

输入：目标 Entity 的 Buff 列表
处理：过滤出 Active 状态的 Buff
输出：待匹配的 Buff 列表
禁止：包含非 Active 状态的 Buff

### Step3：匹配 Trigger

输入：Buff 的 triggers 列表 + 当前 Trigger 点
处理：检查 triggers 是否包含当前 Trigger 点
输出：匹配的 Buff 列表
禁止：跳过匹配直接执行

### Step4：Condition 检查

输入：匹配的 Buff + Condition 列表
处理：检查 Condition 是否满足（如 HpBelow / HasBuff 等）
输出：通过 Condition 检查的 Buff
禁止：跳过 Condition 检查

### Step5：Effect 生成

输入：通过检查的 Buff + TriggerContext
处理：从 Buff 的 effects 列表生成 Effect[]
输出：待执行的 Effect 列表
禁止：在 Effect 生成阶段执行效果

### Step6：Effect Pipeline

输入：Effect 列表 + TriggerContext
处理：进入 Effect Pipeline（Generate → Modify → Execute）
输出：EffectResult 列表
禁止：跳过 Effect Pipeline 直接执行

---

## 触发链管线

```
Effect 执行完成 → 产生新事件 → 检查链深度 → 深度 < 3？ → 触发匹配管线 → 递归
```

### Step1：Effect 执行完成

输入：EffectResult 列表
处理：检查 Effect 是否产生新事件（如 KillTarget / DamageApplied）
输出：新事件列表
禁止：无新事件时启动触发链

### Step2：检查链深度

输入：当前链深度
处理：判断深度是否 < 3
输出：是否允许继续
禁止：深度 ≥ 3 时继续触发

### Step3：触发匹配管线

输入：新事件 + 链深度 + 1
处理：进入触发匹配管线
输出：Effect 列表
禁止：跳过深度检查

---

# 数据结构

## Trigger（触发器枚举）

职责：定义 Buff 效果的触发时机

结构：
- TurnStart — 回合开始
- TurnEnd — 回合结束
- BeforeAttack — 攻击前
- AfterAttack — 攻击后
- BeforeDamaged — 受伤前
- AfterDamaged — 受伤后
- BeforeMove — 移动前
- AfterMove — 移动后
- KillTarget — 击杀目标时
- Death — 死亡时
- BattleStart — 战斗开始
- BattleEnd — 战斗结束

要求：
- 是 Value Object，不可变
- 从 BuffDef 的 triggers 字段反序列化
- 每个 Trigger 有明确的生命周期阶段

---

## TriggerContext（触发上下文）

职责：封装 Trigger 触发时的全部输入数据

结构：
- trigger：Trigger — 触发的 Trigger 类型
- source_entity：Entity — 触发源
- target_entity：Entity — 触发目标
- damage_dealt：Option<i32> — 造成的伤害值
- is_critical：Option<bool> — 是否暴击
- chain_depth：u32 — 当前触发链深度

要求：
- 纯数据传递，不存储持久状态
- source_entity 和 target_entity 必须存在
- damage_dealt 在 AfterAttack / AfterDamaged 时必须存在
- chain_depth 初始为 0

---

## TriggerDef（触发器定义-反序列化用）

职责：RON 反序列化中间态，从 BuffDef 的 triggers 字段解析

结构：
- 触发时机名称列表

要求：
- 通过 From trait 转换为 Vec<Trigger>
- 支持多个 Trigger 时机点

---

# 禁止事项

禁止：触发链深度超过 3 级

原因：超过 3 级会导致无限递归，耗尽栈空间

违反后果：程序崩溃，栈溢出

---

禁止：同一 Buff 在同一 Trigger 点触发多次

原因：重复触发会导致效果重复执行，破坏游戏平衡

违反后果：中毒效果触发两次，伤害翻倍

---

禁止：TriggerContext 缺少 source_entity 或 target_entity

原因：Effect 执行需要明确的来源和目标

违反后果：Effect 无法确定施放者和目标，执行失败

---

禁止：在非对应生命周期阶段触发 Trigger

原因：Trigger 时机点与生命周期阶段严格对应，错位触发导致逻辑混乱

违反后果：TurnStart 在 Action 阶段触发，回合状态机异常

---

禁止：跳过 Trigger 匹配直接执行 Effect

原因：Trigger 匹配是 Effect 执行的前提，跳过会导致未匹配的 Buff 也执行效果

违反后果：无 Trigger 的 Buff 也执行效果

---

禁止：触发链无深度检查

原因：无检查的触发链可能无限递归

违反后果：程序崩溃，栈溢出

---

# AI 修改规则

## 如果新增 Trigger 时机点

允许：
- 在 Trigger 枚举中新增变体
- 添加对应的生命周期阶段映射

禁止：
- 修改现有 Trigger 的语义
- 新增与现有 Trigger 重叠的时机点
- 在非生命周期阶段新增 Trigger

优先检查：
- 新 Trigger 是否在生命周期中有对应阶段
- TriggerContext 是否需要新增字段
- 现有 Buff 是否需要适配新 Trigger

---

## 如果修改触发链逻辑

允许：
- 调整链深度上限（当前为 3）
- 添加触发链的日志记录

禁止：
- 移除链深度检查
- 修改深度递增逻辑
- 允许无限递归

优先检查：
- 链深度检查是否在每次进入下一级前执行
- 深度超限时是否终止并输出警告
- 触发链完成后是否重置深度

---

## 如果修改 TriggerContext

允许：
- 添加新的上下文字段（如 element_type）
- 调整字段的 Option/非 Option

禁止：
- 移除 source_entity 或 target_entity
- 修改字段的语义
- 使 TriggerContext 可变

优先检查：
- 新字段是否在所有 Trigger 时机点中都有意义
- Effect 执行是否依赖新字段
- 现有 TriggerContext 构建代码是否兼容

---

## 如果测试失败

排查顺序：
1. 检查 Trigger 时机点是否与生命周期阶段对应
2. 检查 TriggerContext 是否包含足够的上下文数据
3. 检查触发链深度是否超过 3 级
4. 检查同一 Buff 是否在同一 Trigger 点触发多次
5. 检查 Trigger 匹配是否跳过了非 Active 状态的 Buff

---

# 交叉引用

| 主题 | 详细文档 |
|------|----------|
| 回合生命周期和 TurnStart/TurnEnd | `docs/domain/turn_rules.md` |
| 战斗生命周期和 BattleStart/BattleEnd | `docs/domain/battle_rules.md` |
| Duration 持续策略 | `docs/domain/duration_rules.md` |
| Effect Pipeline（Generate → Modify → Execute） | `docs/domain/attribute_modifier_rules.md#效果管线` |
| Buff 叠层策略 | `docs/domain/stack_policy_rules.md` |
| 效果数值计算 | `docs/domain/formula_rules.md` |
