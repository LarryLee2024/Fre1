---
id: 02-domain.ability.ability-rules
title: Ability Rules
status: draft
owner: domain-designer
created: 2026-06-14
updated: 2026-06-14
tags:
  - domain
  - ability
  - skill
---

# 能力系统领域

Version: 1.0
Status: Proposed

> **注意**: 本文档是对 `docs/02-domain/skill/skill-rules.md` 的概念升级。"Ability" 是 "Skill" 的领域概念重命名，对齐 UE GAS（Gameplay Ability System）的 SRPG-GAS 裁切版架构。概念重命名不改变现有代码结构，仅统一领域术语。

能力系统（Ability）管理 Ability 槽位、冷却、学习/遗忘、目标选择和使用验证。Ability 效果执行由 Effect Pipeline 负责。

核心原则：
- 🟩 Ability 是 Rule，火球术是 Content（Rule/Content 分离）
- 🟩 Ability 只负责验证和路由，不执行效果
- 🟩 Ability 释放前必须通过验证管线（Requirement → Cost → Targeting → Effect → Settlement）
- 🟩 所有操作入口为标准化命令，Ability 释放通过 CastAbilityCommand

SRPG-GAS 能力链路：Character → Ability → Targeting → Effect → Buff → Modifier → Tag

本领域的高层模型：Ability = Requirement（能不能放）+ Cost（消耗什么）+ Targeting（对谁放）+ Effect[]（放什么效果）。

参见：condition-rules.md, targeting-rules.md, cost-rules.md, requirement-rules.md 提供各个子系统的详细规则。

---

# 统一术语

## 能力（Ability）

单位可执行的战斗能力，包含效果列表和消耗。

不是 Effect。不是 Buff。不是修饰器。

关键属性：
- 定义态为 AbilityDef（RON 反序列化用），运行态为 AbilityData
- 包含：id、name、description、cost_mp、range、targeting、effects、tags、conditions、cooldown、priority
- 效果列表中的每个 EffectDef 描述一种战斗效果（Damage / Heal / ApplyBuff / Cleanse）
- 通过 AbilityRegistry 按 ID 查询

参见：
- `docs/02-domain/requirement/requirement-rules.md` — Ability 释放的前提条件系统
- `docs/02-domain/cost/cost-rules.md` — Ability 的消耗统一管理系统
- `docs/02-domain/targeting/targeting-rules.md` — Ability 的目标选择器系统

---

## 能力定义（AbilityDef / AbilityData）

RON 文件中的 Ability 配置，包含效果列表、消耗、冷却、范围和条件。

不是运行时实例。不是效果管线。不是 AbilitySlots。

关键属性：
- AbilityDef 是 RON 反序列化中间态（使用 TagName 替代 GameplayTag）
- AbilityData 是运行时数据（使用 GameplayTag）
- AbilityDef 通过 From trait 转换为 AbilityData
- 从 assets/skills/*.ron 加载（Rule/Content 分离）

---

## 能力槽位（AbilitySlot）

单位装备的 Ability 位，包含 Ability ID 列表。

不是 Ability 本身。不是 Buff 槽位。不是 AbilityCooldowns。

关键属性：
- 存储为 AbilitySlots 组件（Vec<String> ability_ids）
- 第一个槽位为默认攻击（default_attack）
- 第二个槽位为特殊技能（special_skill）
- 空列表时 default_attack 回退到 BASIC_ATTACK_ID

---

## 冷却（Cooldown）

Ability 使用后进入的不可用倒计时。

不是消耗。不是资源不足。不是规则失败。

关键属性：
- 存储为 AbilityCooldowns 组件（HashMap<String, u32>）
- set(ability_id, turns) 设置冷却回合数
- set(0) 等价于清除该 Ability 冷却
- tick() 在回合结束时递减所有冷却
- clear() 清除所有冷却

---

## 能力消耗（AbilityCost）

使用 Ability 消耗的 MP / HP / Stamina。

不是冷却。不是施法条件。不是 AbilityCondition。

关键属性：
- cost_mp：Ability 的 MP 消耗值
- 通过 AbilityCondition::MpCost 检查是否满足
- 消耗检查是验证管线的第一步

---

## 能力目标（AbilityTargeting）

Ability 的作用范围类型，决定 Ability 可以作用于谁。

不是目标选择 UI。不是瞄准系统。不是 CursorPosition。

关键属性：
- AbilityTargeting 枚举：SingleEnemy / SingleAlly / SelfOnly / AoeEnemies / AoeAllies / NoTarget
- requires_target_selection()：SingleEnemy 和 SingleAlly 需要选择目标
- AoeEnemies / AoeAllies 范围由 range 字段决定
- NoTarget 直接对自身生效，无需选择目标

---

## 能力学习（Ability Learning）

单位获得新 Ability 的方式（等级解锁 / 装备提供 / 道具）。

不是 Ability 使用。不是槽位分配。不是 AbilityCastResult。

关键属性：
- 通过 AbilitySlots 的 ability_ids 列表管理
- 学习新 Ability 需要检查槽位上限
- 学习/遗忘必须通知 UI（ViewModel 更新）

---

## 能力释放结果（AbilityCastResult）

释放 Ability 的规则验证结果，包含成功或失败原因。验证阶段是五阶段管线的第一步。

不是 Error。不是 EffectResult。不是 PendingEffect。

关键属性：
- AbilityUseError 枚举标识失败原因：OnCooldown / InsufficientMp / MissingTag / TargetMissingTag / HpNotBelow / HpNotAbove
- can_use() 方法纯函数验证，不修改状态
- 验证通过返回 Ok(())，失败返回 Err(AbilityUseError)

---

## 能力效果预览（AbilityPreview）

Ability 释放前的效果预览，用于 UI 展示伤害预览。预览与执行分离，预览路径无副作用。

不是实际效果执行。不是 EffectResult。不是 PendingEffect。

关键属性：
- AbilityPreview 包含 ability_id、ability_name、predictions 列表
- EffectPreview 枚举：Damage / Heal / BuffApplied / Cleanse
- preview_ability_effects() 纯函数，不修改状态
- 通过 EffectHandlerRegistry 查找处理器进行预览

---

## 能力执行上下文（AbilityExecutionContext）

封装一次 Ability 释放的所有输入数据。

不是全局状态。不是 ECS World。不是 Resource。

关键属性：
- source：发起者 Entity
- target：目标 Entity
- ability_id：使用的 Ability ID
- source_attrs / target_attrs：双方属性快照
- source_tags / target_tags：双方标签快照
- terrain_defense_bonus：地形防御加成
- 纯数据传递，不存储持久状态

---

## 效果执行器（Effect Executor）

一种效果类型的完整执行逻辑封装，500+ Ability 收敛为 20-30 个可复用的 Effect Executor。

不是 Ability。不是 Buff。不是修饰器。

关键属性：
- 每种效果类型（Damage / Heal / ApplyBuff / Cleanse 等）对应一个 Effect Executor
- 通过 EffectHandler trait 定义接口：type_name / generate / preview / execute
- 注册到 EffectHandlerRegistry，运行时通过 type_name 查找分发
- 新增效果类型只需实现 trait 并注册，不修改管线代码
- 500 Ability ≈ 20-30 种原子 Effect 组合，不需要为每个 Ability 写独立逻辑

---

## 组合效果（Composite Effect）

多个效果的组合执行模式，支持顺序、并行、选择、条件组合。

不是单个 Effect。不是 Ability。不是修饰器。

关键属性：
- Sequence：按顺序依次执行多个 Effect
- Parallel：同时执行多个 Effect
- Select：根据条件选择执行哪个 Effect
- ConditionalEffect：满足条件时才执行的 Effect（如 HP<30% 时触发 Execute）
- 组合效果让 Ability 配置更灵活，不需要为每种组合写新代码

---

## 能力模板与参数覆盖（Ability Template + Override）

Ability 的基础模板定义通用结构，具体 Ability 通过参数覆盖实现差异化。

不是硬编码逻辑。不是 Effect 执行器。不是修饰器。

关键属性：
- AbilityDef 定义通用结构（targeting + effects + costs + requirements + tags）
- 具体 Ability（火球术、治疗术）只是参数不同的配置实例
- 新增 Ability 只需新建 RON 配置文件，不修改 Rust 代码
- Ability 间的差异体现在参数、组合、条件，不在执行逻辑

---

# 领域边界

## 本领域负责

- Ability 槽位管理（AbilitySlots：Ability ID 列表）
- Ability 冷却管理（AbilityCooldowns：冷却递减和清除）
- Ability 使用条件验证（can_use：冷却 / 消耗 / 标签 / HP 阈值）
- Ability 目标类型定义（AbilityTargeting 枚举）
- Ability 注册表管理（AbilityRegistry：ID → AbilityData 查询）
- Ability 效果预览（preview_ability_effects：纯函数预览）
- Ability 学习/遗忘（AbilitySlots 增删）

## 本领域不负责

- 效果管线执行（由 Effect Pipeline 领域负责：Generate → Modify → Execute）
- Ability 伤害的实际计算（由 Effect Handler 执行）
- Buff 的生命周期管理（由 Buff 领域负责）
- Ability UI 展示（由 UI 架构领域负责）
- 输入处理和目标选择（由 Input 领域负责）
- 回合阶段转换（由 Turn Battle 领域负责）
- 具体条件逻辑评估（由 `docs/02-domain/condition/condition-rules.md` 负责）
- 具体消耗校验和扣除（由 `docs/02-domain/cost/cost-rules.md` 负责）
- 目标坐标到实体列表的解析（由 `docs/02-domain/targeting/targeting-rules.md` 负责）
- 释放前提的逐条验证（由 `docs/02-domain/requirement/requirement-rules.md` 负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| Ability 释放验证 | 函数调用（can_use） | Battle/Turn 领域 |
| Ability 冷却递减 | 组件方法（tick） | Turn 领域（TurnEnd 阶段） |
| Ability 效果预览 | 函数调用（preview_ability_effects） | UI 领域 |
| Ability 学习/遗忘 | 函数调用（AbilitySlots 增删） | Character/UI 领域 |
| Ability 效果路由 | 函数调用（Effect Pipeline） | Battle 领域 |

---

# 生命周期

## 状态列表

### Ability 执行状态

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Ready | Ability 可用，等待释放 | Casting |
| Casting | Ability 正在施放，验证通过 | Executing |
| Executing | Ability 效果执行中 | Cooldown |
| Cooldown | Ability 冷却中 | Ready |

### Ability 冷却状态

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Ready（cd = 0） | Ability 可用 | OnCooldown |
| OnCooldown（cd > 0） | Ability 冷却中 | Ready |

### Ability 槽位状态

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Equipped | Ability 已装备 | Unlearned |
| Unlearned | Ability 未装备 | Equipped |

## 状态转换图

```
Ready → 使用 Ability → Casting → 验证通过 → Executing → 效果完成 → Cooldown → tick() → Ready
Equipped → 遗忘 Ability → Unlearned → 学习 Ability → Equipped
```

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| Ready | Casting | 用户选择释放 Ability |
| Casting | Executing | can_use() 验证通过 |
| Casting | Ready | can_use() 验证失败（回退） |
| Executing | Cooldown | 效果执行完成，设置冷却 |
| Cooldown | Ready | tick() 递减后冷却归零 |
| Equipped | Unlearned | 遗忘 Ability（移除 AbilitySlots 中的 ID） |
| Unlearned | Equipped | 学习 Ability（添加 AbilitySlots 中的 ID） |

---

# 不变量

## 不变量1：Ability 释放前必须验证

回合阶段流转：

每次 Ability 释放前，必须调用 can_use() 验证所有条件（冷却 / 消耗 / 标签 / HP 阈值）。验证失败时返回 AbilityUseError，不执行效果。

违反表现：

冷却中的 Ability 被使用、MP 不足时释放消耗 Ability、缺少必需标签时释放职业专属 Ability。

---

## 不变量2：冷却在 TurnEnd 阶段递减

回合生命周期：

AbilityCooldowns.tick() 仅在 TurnEnd 阶段调用，不在其他阶段调用。

违反表现：

同一回合内多次调用 tick() 导致冷却提前结束、冷却递减时机不一致。

---

## 不变量3：set(0) 等价于清除冷却

任意时刻：

AbilityCooldowns.set(ability_id, 0) 调用后，该 ability_id 的冷却条目必须被移除（HashMap 中不存在该 key）。

违反表现：

set(0) 后 get(ability_id) 仍返回 0 而非默认值（语义上等价但实现不一致）。

---

## 不变量4：Ability 效果通过 Effect Pipeline 执行

任意时刻：

Ability 的效果列表（AbilityData.effects）中的每个 EffectDef 必须通过 Effect Pipeline 的 Generate → Modify → Execute 三步执行，禁止跳步或直接执行。

违反效果：

Ability 效果绕过 Modifier 修饰、Trait 触发、BattleRecord 记录。

---

## 不变量5：基础攻击 Ability 始终存在

任意时刻：

每个 Unit 的 AbilitySlots 中必须包含至少一个 Ability（default_attack）。空列表时回退到 BASIC_ATTACK_ID。

违反表现：

AbilitySlots 为空且无回退机制，导致单位无法执行基础攻击。

---

## 不变量6：Ability 条件检查顺序固定

任意时刻：

can_use() 的条件检查顺序为：冷却 → 消耗 → 标签 → 目标标签 → HP 阈值。冷却检查最先执行。

违反表现：

先检查消耗再检查冷却，导致 MP 不足的错误信息掩盖了冷却中的真实原因。

---

## 不变量7：Ability = 配置，不是代码

任意时刻：

每个 Ability 的执行逻辑不应是独立的硬编码函数。火球术、治疗术、猛击——在程序眼里只是配置数据。程序只认识 Effect Executor。500 Ability + 1000 Buff 收敛为 20-30 个 Effect Executor，新增内容只改配置不改代码。

违反表现：

为每个 Ability 编写独立的 execute_fireball()、execute_heal() 函数，100 个 Ability = 100 个 Bug 来源。

---

## 不变量8：Buff 的效果列表也走 Effect Pipeline

任意时刻：

Buff 触发产生的效果列表（BuffDef.effects）中的每个 EffectDef 必须通过 Effect Pipeline 的 Generate → Modify → Execute 三步执行，与 Ability 效果执行路径一致。

违反表现：

Buff 触发后直接修改属性而不经过 Effect Pipeline，跳过修饰规则和 BattleRecord 记录。

---

## 不变量9：Ability 管线阶段顺序固定

任意时刻：

Ability 执行必须严格按 Requirement → Cost → Targeting → Effect → Settlement 顺序执行，禁止跳步或乱序。

违反表现：

先执行效果再检查消耗、跳过 Targeting 直接对默认目标执行效果。

---

# 业务规则

## 规则1：Ability 释放验证

禁止：
- 跳过 can_use() 验证直接释放 Ability
- 验证失败后仍执行效果
- 忽略 AbilityUseError 返回值

必须：
- 释放前调用 can_use() 获取验证结果
- 验证失败时返回错误原因（AbilityUseError）
- 验证通过后才进入效果执行管线

允许：
- AI 跳过不需要选择目标的 Ability（SelfOnly / NoTarget）

---

## 规则2：冷却管理

禁止：
- 在 TurnEnd 以外的阶段调用 tick()
- 手动修改 cooldowns HashMap 的值
- Ability 释放后不设置冷却

必须：
- Ability 释放成功后调用 set(ability_id, cooldown) 设置冷却
- 每个回合结束时调用 tick() 递减所有冷却
- set(0) 清除条目（不是设置为 0）

允许：
- clear() 在特殊情况下清除所有冷却（如调试）

---

## 规则3：Ability 学习/遗忘

禁止：
- 不通知 UI 就修改 AbilitySlots
- 添加已存在的 Ability ID
- 移除不存在的 Ability ID

必须：
- 学习/遗忘后通知 UI（ViewModel 更新）
- 检查 Ability ID 是否在 AbilityRegistry 中存在
- 保持至少一个 Ability（default_attack）

允许：
- 通过 Equipment 领域的装备效果间接添加/移除 Ability

---

## 规则4：效果预览

禁止：
- 预览时修改游戏状态
- 预览结果用于实际伤害计算

必须：
- preview_ability_effects() 是纯函数
- 预览通过 EffectHandlerRegistry 分发
- 预览结果只用于 UI 展示

允许：
- 预览使用硬编码的属性快照（不查询实时 ECS）

---

## 规则5：数据驱动 Ability 定义

禁止：
- 在 Rust 代码中硬编码 Ability 属性（如 const FIREBALL_DAMAGE: i32 = 120）
- 为新增 Ability 修改 Rust 代码
- 在 Ability 执行逻辑中硬编码计算公式

必须：
- Ability 定义存储在 assets/skills/*.ron 配置文件中（Rule/Content 分离）
- 新增 Ability 只需新建 RON 文件，不修改 Rust 代码
- Ability 间的差异体现在配置参数（effects、costs、tags），不在执行逻辑
- Ability 模板（Ability Template）定义通用结构，具体 Ability 通过参数覆盖实现差异化

允许：
- 通过 RON 配置文件定义新 Ability
- 在 EffectHandlerRegistry 中注册新效果处理器

---

## 规则6：标签驱动 Ability 过滤

禁止：
- 硬编码 Ability 筛选逻辑（如 if ability_id == FIREBALL）
- 使用字符串匹配进行 Ability 分类

必须：
- Ability 分类和过滤通过 GameplayTag 位掩码实现
- 按 Tag 过滤 Ability 列表（如查询所有 Damage.Fire Ability 进行冷却缩减）
- Ability 的 targeting、interaction rules 通过 Tag 组合判定
- 新增 Ability 类型只需添加 Tag 定义和配置，不改代码

允许：
- 使用 GameplayTag.has() / has_any() / has_all() 进行标签查询
- 通过 ModifierRule 的 source_tag + target_tag 驱动 Ability 交互规则

---

## 规则7：Buff 容器化管理

禁止：
- Buff 直接修改属性而不通过 Effect Pipeline
- Buff 没有 Duration 策略导致永不过期
- Buff 移除时不清理对应的 Modifier

必须：
- Buff 作为 Effect 的持有者（Trigger[] + Effect[] + Duration + StackPolicy）
- Buff 的效果列表（effects）通过 Effect Pipeline 执行
- Buff 触发时机（Trigger）由 TriggerRegistry 统一注册和分发
- Buff 移除时通过 remove_modifiers_from(source) 精确清理修饰器

允许：
- 通过 TriggerHandler trait 实现自定义触发逻辑
- 通过 StackPolicy 配置叠层规则

---

# 流程定义

## Ability 释放管线（五阶段）

```
用户选择 Ability → Targeting → AbilityCastValidation → 消耗资源 → 进入冷却 → 路由到 Effect Pipeline
```

### Step1：用户选择 Ability

输入：UiCommand::Ability / UiCommand::Attack
处理：根据当前 TurnPhase 判断是否允许选择 Ability
输出：选中的 ability_id
禁止：在非玩家回合接受 Ability 选择

### Step2：Targeting（目标选择）

输入：AbilityTargeting 类型
处理：需要选择目标时进入 SelectTarget 阶段；SelfOnly / NoTarget / Aoe 类型跳过
输出：目标坐标或目标 Entity
禁止：对已死亡单位选择目标

### Step3：AbilityCastValidation

输入：AbilityData + Attributes + GameplayTags + cooldown
处理：调用 can_use() 验证所有条件
输出：AbilityUseError 或 Ok(())
禁止：验证失败后继续执行

### Step4：消耗资源

输入：验证通过的 Ability + Attributes
处理：扣除 MP（通过 set_vital）
输出：更新后的 Attributes
禁止：扣除后 HP/MP 为负数

### Step5：进入冷却

输入：ability_id + AbilityData.cooldown
处理：调用 AbilityCooldowns.set(ability_id, cooldown)
输出：冷却状态更新
禁止：cooldown 为 0 时不设置（已由 set(0) 语义处理）

### Step6：路由到 Effect Pipeline

输入：AbilityExecutionContext + AbilityData.effects
处理：遍历 effects，每个 EffectDef 路由到 Effect Pipeline 执行
输出：EffectResult 列表
禁止：跳过 Effect Pipeline 直接执行效果

---

## Ability 学习管线

```
触发学习条件 → 检查槽位 → 分配槽位 → 通知 UI
```

### Step1：触发学习条件

输入：等级提升 / 装备穿戴 / 道具使用
处理：判断是否触发 Ability 学习
输出：待学习的 ability_id
禁止：无触发条件时主动学习

### Step2：检查槽位

输入：AbilitySlots + 待学习 ability_id
处理：检查 Ability 是否已存在、槽位是否已满
输出：是否允许学习
禁止：重复添加相同 Ability

### Step3：分配槽位

输入：AbilitySlots + ability_id
处理：添加到 ability_ids 列表
输出：更新后的 AbilitySlots
禁止：移除现有 Ability

### Step4：通知 UI

输入：更新后的 AbilitySlots
处理：触发 ViewModel 更新
输出：UI 重绘 Ability 面板
禁止：跳过通知（UI 显示旧数据）

---

# 数据结构

## AbilityData（Ability 数据-运行时）

职责：存储从 RON 加载的 Ability 配置（运行时使用 GameplayTag）

结构：
- id：String — Ability ID（如 "fireball"）
- name：String — 显示名称
- description：String — 描述文本
- cost_mp：i32 — MP 消耗
- range：u32 — Ability 范围（0 表示使用单位基础攻击范围）
- targeting：AbilityTargeting — 目标类型
- effects：Vec — EffectDef 列表
- tags：Vec — GameplayTag 列表（用于修饰规则匹配）
- conditions：Vec — AbilityCondition 列表（使用条件）
- cooldown：u32 — 冷却回合数
- priority：u32 — 优先级（AI 使用）

要求：
- 从 AbilityDef 通过 From trait 转换
- can_use() 纯函数验证
- 通过 AbilityRegistry.get(id) 查询

---

## AbilityDef（Ability 定义-反序列化用）

职责：RON 反序列化中间态，使用 TagName 替代 GameplayTag

结构：
- version：u32 — 配置版本号（默认 0）
- 其余字段同 AbilityData（TagName 替代 GameplayTag）

要求：
- 实现 From<AbilityDef> for AbilityData
- version 缺失时默认为 0（兼容旧配置）

---

## AbilitySlots（Ability 槽位组件）

职责：管理单位的 Ability ID 列表

结构：
- ability_ids：Vec — 已装备的 Ability ID

要求：
- 是 Bevy Component
- new(ability_ids) 创建实例
- default_attack() 返回第一个 Ability 或回退到 BASIC_ATTACK_ID
- special_skill() 返回第二个 Ability
- iter() 返回所有 Ability ID

---

## AbilityCooldowns（Ability 冷却组件）

职责：追踪所有 Ability 的冷却状态

结构：
- cooldowns：HashMap — ability_id → 剩余冷却回合数

要求：
- 是 Bevy Component
- get(ability_id) 返回当前冷却（不存在返回 0）
- set(ability_id, turns) 设置冷却（0 时移除条目）
- tick() 递减所有冷却（saturating_sub(1)，归零移除）
- clear() 清除所有冷却

---

## AbilityTargeting（Ability 目标类型枚举）

职责：定义 Ability 的作用范围

结构：
- SingleEnemy — 对单个敌方
- SingleAlly — 对单个友方
- SelfOnly — 对自身
- AoeEnemies — 范围敌方（由 range 决定）
- AoeAllies — 范围友方
- NoTarget — 无需目标

要求：
- requires_target_selection() 仅 SingleEnemy 和 SingleAlly 返回 true
- label() 返回中文标签名

---

## AbilityCondition（Ability 使用条件枚举）

职责：定义 Ability 的使用前置条件

结构：
- MpCost(i32) — MP 消耗检查
- RequireTag(GameplayTag) — 施法者需要拥有指定标签
- TargetRequireTag(GameplayTag) — 目标需要拥有指定标签
- HpBelow(f32) — 施法者 HP 低于指定百分比
- HpAbove(f32) — 施法者 HP 高于指定百分比

要求：
- 所有条件必须全部满足（AND 逻辑）
- 通过 AbilityConditionDef 从 RON 转换

---

## AbilityUseError（Ability 使用失败枚举）

职责：标识 Ability 验证失败的原因

结构：
- OnCooldown { remaining } — 冷却中
- InsufficientMp { required, current } — MP 不足
- MissingTag { tag } — 缺少施法者标签
- TargetMissingTag { tag } — 缺少目标标签
- HpNotBelow { threshold } — HP 不低于阈值
- HpNotAbove { threshold } — HP 不高于阈值

要求：
- 由 can_use() 返回
- 携带足够的上下文信息用于 UI 展示

---

## AbilityExecutionContext（Ability 执行上下文）

职责：封装一次 Ability 释放的全部输入数据

结构：
- source：Entity — 发起者
- target：Entity — 目标
- ability_id：String — Ability ID
- source_attrs / target_attrs：Attributes — 双方属性快照
- source_tags / target_tags：GameplayTags — 双方标签快照
- terrain_defense_bonus：i32 — 地形防御加成

要求：
- 纯数据传递，不存储持久状态
- 通过 from_query() 从 ECS 查询构建
- 克隆属性和标签数据，避免借用冲突

---

# 禁止事项

禁止：跳过 can_use() 验证直接释放 Ability

原因：验证管线保证战斗公平性，跳过验证会导致不公平的 Ability 使用

违反后果：冷却中的 Ability 被使用、MP 不足时释放消耗 Ability、缺少标签时释放职业专属 Ability

---

禁止：Ability 效果绕过 Effect Pipeline 执行

原因：Effect Pipeline 是效果执行的唯一通道，绕过管线会跳过修饰、记录等关键步骤

违反后果：修饰规则不生效、伤害/治疗值异常、BattleRecord 缺少记录

---

禁止：在 TurnEnd 以外的阶段调用 AbilityCooldowns.tick()

原因：冷却递减必须与回合生命周期同步，提前递减会导致冷却期缩短

违反后果：Ability 冷却期不一致、同一回合内多次递减导致冷却提前结束

---

禁止：AbilitySlots 为空时单位无法行动

原因：基础攻击是单位的最低能力保障，空槽位必须回退到 BASIC_ATTACK_ID

违反后果：单位无法执行任何攻击，战斗陷入死循环

---

禁止：预览时修改游戏状态

原因：预览是纯函数，用于 UI 展示，修改状态会破坏游戏逻辑

违反后果：预览操作导致实际伤害、消耗不一致

---

禁止：修改 AbilityData 定义态

原因：AbilityData 是不可变配置，运行时变化通过冷却和消耗系统管理

违反后果：全局 Ability 配置被污染、多场战斗数据不一致

---

禁止：can_use() 条件检查顺序不固定

原因：检查顺序影响错误信息的准确性，固定顺序保证一致的用户体验

违反后果：错误信息不一致、调试困难

---

禁止：一个 Ability = 一个函数

原因：100 个 Ability = 100 个函数 = 100 个 Bug 来源，无法复用，违反 Rule/Content 分离。每个 Ability 的独立硬编码逻辑无法复用，修复一个 Bug 可能影响另一个 Ability。

违反后果：代码膨胀，新增 Ability 必须修改 Rust 代码，无法实现数据驱动配置。

---

禁止：为新增 Ability 修改 Rust 代码

原因：新增内容 = 新增 RON 文件，不改代码。Ability 的差异体现在配置参数（effects、costs、tags），不在执行逻辑。

违反后果：违反 Rule/Content 分离原则，每次新增 Ability 都需要修改核心代码。

---

禁止：Ability/Buff 硬编码效果逻辑

原因：效果逻辑应在 EffectHandler 中实现，不在 Ability/Buff 定义中。同一个 Effect 类型在不同 Ability 中必须有统一的实现。

违反后果：同一个 Effect 类型在不同 Ability 中有不同实现，无法统一维护。

---

禁止：跳过 Ability 管线阶段顺序

原因：Ability 执行必须严格按 Requirement → Cost → Targeting → Effect → Settlement 顺序，跳步或乱序破坏管线契约。

违反后果：先执行效果再检查消耗、跳过 Targeting 直接对默认目标执行效果。

---

# 领域事件

本领域产生的领域事件：
- `AbilityCastStarted` — Ability 施放开始，携带 caster_id、ability_id
- `AbilityCastFinished` — Ability 施放完成，携带 caster_id、ability_id、effect_results
- `AbilityLearned` — Ability 学习，携带 unit_id、ability_id
- `AbilityForgotten` — Ability 遗忘，携带 unit_id、ability_id

> 🟩 新增领域事件必须先更新白名单文档
> 🟩 所有核心业务事实通过领域事件表达，日志、回放、UI 均监听同一事件源

---

# AI 修改规则

## 宪法合规检查清单

修改本领域代码前，必须逐项确认：
- 🟩 Rule/Content 分离：Ability 定义通过 RON 配置，新增 Ability 只改配置不改代码
- 🟩 Ability 执行遵循 Requirement → Cost → Targeting → Effect → Settlement 五阶段管线
- 🟩 预览与执行分离：预览路径无副作用，纯函数计算
- 🟩 所有操作入口为标准化命令，Ability 释放通过 CastAbilityCommand
- 🟩 核心验证逻辑为纯函数，不依赖 ECS

## 如果新增 Ability

允许：
- 在 assets/skills/*.ron 中添加新的 AbilityDef RON 配置
- 确保 id 唯一且 effects 中的 EffectDef 类型已注册

禁止：
- 在代码中硬编码新 Ability 属性
- 新增 EffectDef 变体而不注册 EffectHandler

优先检查：
- AbilityData 中的 conditions 是否使用正确的 AbilityCondition 类型
- tags 中的 TagName 是否在 GameplayTag 中有对应定义
- cooldown 值是否合理（避免过强或过弱）

---

## 如果修改 Ability 验证逻辑

允许：
- 在 AbilityCondition 枚举中新增变体
- 在 can_use() 中添加新的验证分支

禁止：
- 改变 can_use() 的返回类型
- 修改条件检查顺序（冷却必须最先）
- 移除现有条件类型的检查

优先检查：
- 新条件类型是否影响现有 Ability 的 can_use 结果
- 新条件的 RON 反序列化是否兼容旧配置
- 新条件的错误信息是否清晰

---

## 如果修改冷却系统

允许：
- 调整 tick() 的递减逻辑
- 添加冷却清除的触发条件

禁止：
- 修改 set(ability_id, 0) 的语义（必须移除条目）
- 在 TurnEnd 以外的阶段调用 tick()
- 修改 AbilityCooldowns 的存储结构

优先检查：
- set(0) 后 get() 是否返回默认值 0
- tick() 后归零的条目是否被正确移除
- clear() 是否清空所有冷却

---

## 如果修改 Ability 预览

允许：
- 在 EffectPreview 枚举中新增变体
- 调整预览计算的精度

禁止：
- 预览时修改游戏状态
- 预览结果用于实际伤害计算
- 预览绕过 EffectHandlerRegistry

优先检查：
- 预览结果是否与实际效果一致
- 新增 EffectDef 类型是否在预览中有对应处理
- 预览的纯函数性质是否保持

---

## 如果扩展 Ability 效果类型

允许：
- 在 EffectHandlerRegistry 中注册新的 EffectHandler 实现
- 添加对应的 EffectDef 变体
- 通过 RON 配置文件使用新效果类型

禁止：
- 为新效果类型编写独立的 execute_xxx() 函数
- 在 Ability 定义中硬编码新效果的执行逻辑
- 修改 Effect Pipeline 调度代码

优先检查：
- 新 EffectHandler 是否实现 trait 的所有方法
- type_name 与 EffectDef::type_name 是否一致
- 新效果类型是否通过 Effect Pipeline 执行
- Ability 配置中的 effects 列表是否使用正确的 EffectDef 变体

---

## 如果测试失败

排查顺序：
1. 检查 can_use() 的条件检查顺序是否正确（冷却最先）
2. 检查 AbilityCooldowns.set(0) 是否正确移除条目
3. 检查 AbilityData 中的 conditions 是否使用正确的 GameplayTag
4. 检查 preview_ability_effects 是否通过 EffectHandlerRegistry 分发
5. 检查 AbilitySlots 是否至少包含一个 Ability（default_attack 回退）
6. 检查 Ability 是否通过 RON 配置定义（禁止硬编码）
7. 检查 Buff 效果是否通过 Effect Pipeline 执行（禁止绕过管线）

---

## 交叉引用

| 主题 | 详细文档 |
|------|----------|
| Targeting 系统（目标选择） | `docs/02-domain/targeting/targeting-rules.md` |
| Effect Pipeline（Generate → Modify → Execute） | `docs/02-domain/effect/effect-rules.md` |
| 触发器系统（Trigger） | `docs/02-domain/trigger/trigger-rules.md` |
| 条件系统（Condition） | `docs/02-domain/condition/condition-rules.md` |
| 消耗系统（Cost） | `docs/02-domain/cost/cost-rules.md` |
| 释放前提（Requirement） | `docs/02-domain/requirement/requirement-rules.md` |
| 原始 Skill 文档（已过时） | `docs/02-domain/skill/skill-rules.md` |

---

## 附录：铃兰参考数据

> 领域：Ability | 来源：78铃兰.md §三、补充8 | 数据层：Definition + Instance

#### AbilityDefinition（Definition层）

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| `id` | AbilityId | PK | 技能唯一标识 |
| `name_key` | String | - | 技能名称本地化Key |
| `desc_key` | String | - | 技能描述本地化Key |
| `ability_type` | AbilityType | - | 技能类型 |
| `cost` | CostDef | - | 消耗定义 |
| `cooldown` | Option<u32> | - | 冷却回合数 |
| `range` | u32 | ≥1 | 射程 |
| `targeting` | TargetingId | FK | 目标选择规则引用 |
| `effects` | Vec<EffectId> | FK | 效果列表引用 |
| `tags_required` | Vec<TagId> | - | 需具备的Tag |
| `tags_forbidden` | Vec<TagId> | - | 禁止具备的Tag |
| `special_rules` | Vec<SpecialRule> | - | 特殊规则 |

#### AbilityInstance（Instance层）

| 字段名 | 类型 | 说明 |
|--------|------|------|
| `entity` | Entity | 所属实体 |
| `ability_id` | AbilityId | 引用AbilityDefinition |
| `cooldown_remaining` | u32 | 剩余冷却回合 |
| `current_energy_cost` | f32 | 当前能量消耗 |

#### 技能五大分类

| 类型 | 行动消耗 | 触发方式 | 典型例子 |
|------|----------|----------|----------|
| `NormalAttack` | 消耗1次行动 | 玩家主动释放 | 普通攻击、远程射击 |
| `ActiveSkill` | 消耗能量+行动 | 玩家主动释放 | 火球、冲锋、治疗 |
| `ReactionSkill` | 不消耗行动 | 事件触发 | 反击、援护、回击 |
| `SupportSkill` | 瞬发，保留行动 | 玩家主动释放 | 加Buff、驱散、自身回血 |
| `PassiveSkill` | 无消耗 | 常驻/条件触发 | 个性、天赋、装备被动 |

> **Data Law 004 注意**：反应技(ReactionSkill)在铃兰中归为技能分类，但按Data Law 004，Ability不拥有行为，反应技应归属Trigger领域。建议实现时将ReactionSkill映射为Trigger+Ability组合。

#### 资源管控三层体系

**行动点（AP）**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| `base_ap` | u32 | =1 | 每回合默认行动点 |
| `current_ap` | u32 | ≤max_ap | 当前行动点 |
| `max_ap` | u32 | ≥1 | 单回合行动点上限 |

**能量（CP）**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| `current_cp` | f32 | ≤max_cp | 当前能量 |
| `max_cp` | f32 | >0 | 能量上限 |
| `regen_per_turn` | f32 | ≥0 | 每回合固定回复 |
| `bonus_on_kill` | f32 | ≥0 | 击杀额外回复 |
| `bonus_on_hit` | f32 | ≥0 | 受击额外回复 |

**CostDef 结构**

```yaml
cost:
  ap: 1           # 行动点消耗
  cp: 0           # 能量消耗（0=不消耗）
  hp_pct: 0.0     # 生命值百分比消耗（可选）
```

#### Schema草案

```yaml
# ability_config.ron
(
  abilities: [
    (id: "normal_attack", name_key: "skill.s_1000.name", desc_key: "skill.s_1000.desc", ability_type: NormalAttack,
     cost: (ap: 1, cp: 0), cooldown: None, range: 1,
     targeting: "single_enemy", effects: ["phys_damage"],
     tags_required: [], tags_forbidden: ["control_full"],
     special_rules: []),
    (id: "fireball", name_key: "skill.s_1001.name", desc_key: "skill.s_1001.desc", ability_type: ActiveSkill,
     cost: (ap: 1, cp: 30), cooldown: Some(3), range: 3,
     targeting: "single_enemy", effects: ["fire_damage", "apply_burn"],
     tags_required: [], tags_forbidden: ["silenced"],
     special_rules: []),
  ],
)
```
