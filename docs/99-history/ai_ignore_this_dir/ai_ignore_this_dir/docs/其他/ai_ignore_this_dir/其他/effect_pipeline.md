# Effect Pipeline 领域

Version: 1.0

Effect Pipeline 领域管理战斗效果的数据流，从技能定义生成效果、修饰效果、执行效果。采用 Generate → Modify → Execute 三步管线。

核心原则：
- ECS 是数据流，不是调用链
- Handler 分发扩展
- Logic / Presentation 分离
- 数据驱动

---

# 术语定义

## EffectDef

技能定义中的效果配置，描述"产生什么效果"。

不是 PendingEffectData。EffectDef 是配置，PendingEffectData 是运行时数据。

关键属性：
- Damage：倍率 + 无视防御百分比
- Heal：固定治疗量
- ApplyBuff：buff_id + duration
- Cleanse：无参数

---

## PendingEffect

待处理效果，尚未执行，在 EffectQueue 中等待。

不是最终伤害。PendingEffect 是中间状态，必须经过 Modify → Execute。

关键属性：
- source / target：攻击者/目标实体
- data：PendingEffectData
- source_tags：技能标签
- terrain_id：地形 ID

---

## PendingEffectData

待处理效果的数据，包含具体数值。

不是 EffectDef。PendingEffectData 有计算后的数值，EffectDef 只有配置参数。

关键属性：
- Damage：amount / base_amount / modifiers
- Heal：amount / base_amount
- ApplyBuff：buff_id / duration
- Cleanse：无参数

---

## EffectQueue

战斗唯一效果缓冲区，管线三步共享。

不是直接执行通道。所有效果必须进入队列，禁止直接执行。

关键属性：
- pending：PendingEffect 列表

---

## EffectHandler

效果处理器 trait，负责 Generate 和 Preview。

不是 EffectDef。Handler 是执行者，Def 是配置。

关键属性：
- type_name()：分发键
- generate()：从 EffectDef 生成 PendingEffectData
- preview()：生成 EffectPreview

---

## GenerateContext

Generate 阶段的上下文，包含攻击者/目标的属性快照。

不是 ECS Query。Context 是快照，避免借用冲突。

关键属性：
- source/target 属性快照
- defense_bonus：地形防御加成
- skill_id / source_tags / terrain_id

---

# 领域边界

## 本领域负责

- EffectDef 定义
- EffectHandler 注册表（EffectHandlerRegistry）
- EffectQueue 管理
- Generate / Modify / Execute 三步管线的流程定义
- PendingEffect 和 PendingEffectData 的数据结构
- EffectPreview 预览
- DamageHandler 的伤害计算公式

## 本领域不负责

- ModifierRule 的定义和匹配（由 modifier_rules 领域负责）
- Buff 的施加和移除（由 buff_rules 领域负责）
- CombatIntent 的设置（由 battle_rules 领域负责）
- 死亡判定的 Hook/Observer/Message（由 battle_rules 领域负责）
- UI 展示（由 ui_rules 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 效果生成 | 推入 EffectQueue | battle |
| 效果修饰 | 调用 ModifierRuleRegistry | modifier_rules |
| 效果执行 | 修改 Attributes + 发送 Message | battle / ui |

---

# 生命周期

## 效果生命周期

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Defined | EffectDef 在 SkillData 中 | — |
| Generated | PendingEffect 在 EffectQueue 中 | Modified |
| Modified | 修饰完成 | Executed |
| Executed | 已消费，属性已变化 | — |

## 状态转换图

Defined → Generated → Modified → Executed

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| Defined | Generated | generate_combat_effects 调用 |
| Generated | Modified | modify_effects 调用 |
| Modified | Executed | execute_effects 调用 |

---

# 不变量

## 不变量1：管线严格顺序

任意时刻：

效果必须按 Generate → Modify → Execute 顺序处理。

违反表现：

Generate 直接扣血，Modify 发送死亡消息，Execute 创建新 CombatIntent。

---

## 不变量2：EffectQueue 执行后清空

Execute 阶段结束后：

EffectQueue.pending 必须为空。

违反表现：

残留效果在下一轮被重复执行。

---

## 不变量3：伤害下限

Generate 和 Modify 完成后：

伤害值 ≥ 1。

违反表现：

伤害为 0 或负数。

---

## 不变量4：base_amount 首次记录

Modify 阶段完成后：

base_amount 记录 Generate 阶段的原始值，后续 Modify 不覆盖。

违反表现：

多次 Modify 覆盖 base_amount，伤害分解显示错误。

---

## 不变量5：Generate 不修改 ECS

Generate 阶段完成后：

所有 ECS 组件和 Resource 无变化（除 EffectQueue）。

违反表现：

Generate 阶段扣减 HP 或发送 Message。

---

# 业务规则

## 规则1：Generate

禁止：
- 修改 ECS 状态（除 EffectQueue）
- 跳过前置检查（晕眩/冷却）
- 直接执行效果

必须：
- 通过 EffectHandlerRegistry 分发
- handler.generate() 生成 PendingEffectData
- 组装 PendingEffect 推入 EffectQueue
- 触发 OnAttack Trait

允许：
- 类型不匹配时 Handler 返回 None

---

## 规则2：Modify

禁止：
- 修改 ECS 状态
- 创建新效果
- 覆盖已记录的 base_amount

必须：
- Damage 走 apply_damage_modifiers_with_breakdown
- Heal 走 apply_heal_modifiers
- ApplyBuff / Cleanse 不修饰
- 首次记录 base_amount

---

## 规则3：Execute

禁止：
- 创建新 CombatIntent
- 跳过死亡判定

必须：
- drain(..) 消费所有效果
- Damage：扣血 + 死亡判定 + DamageApplied Message
- Heal：回血 + HealApplied Message
- ApplyBuff：施加 Buff
- Cleanse：驱散 Debuff

---

## 规则4：Handler 分发

禁止：
- match 分发效果类型

必须：
- 通过 type_name() 查找 Handler
- 新增效果类型只需实现 Handler 并注册
- 类型不匹配返回 None

---

# 流程管线

## 效果管线

EffectDef → Generate → Modify → Execute

### Step1：Generate

输入：CombatIntent + SkillData + GenerateContext
处理：前置检查 → 遍历 effects → Handler.generate() → 推入 EffectQueue → OnAttack Trait
输出：EffectQueue.pending 填充
禁止：修改 ECS 状态（除 EffectQueue）

### Step2：Modify

输入：EffectQueue + ModifierRuleRegistry
处理：遍历 pending → Damage/Heal 修饰 → 记录 base_amount 和 modifiers
输出：修饰后的 EffectQueue
禁止：修改 ECS 状态、创建新效果

### Step3：Execute

输入：修饰后的 EffectQueue
处理：drain(..) → 扣血/回血/施加Buff/净化 → 死亡判定 → 发送 Message
输出：属性变化 + Messages
禁止：创建新 CombatIntent

---

## 伤害计算管线

effective_atk → 减去防御 → 减去地形加成 → 乘以倍率 → 下限保护

### Step1：计算防御

输入：effective_def + base_def + ignore_def_percent
处理：def_ignored = base_def × (ignore_def_percent / 100)，final_def = effective_def - def_ignored
输出：final_def
禁止：ignore_def 基于 effective_def（必须基于 base_def）

### Step2：基础伤害

输入：effective_atk + final_def
处理：base_damage = effective_atk - final_def
输出：base_damage

### Step3：地形和倍率

输入：base_damage + terrain_defense_bonus + multiplier
处理：result = (base_damage - terrain_defense_bonus) × multiplier
输出：result

### Step4：下限保护

输入：result
处理：max(1, result)
输出：最终伤害
禁止：跳过下限保护

---

# 数据结构

## EffectDef（Definition）

职责：技能中的效果配置

结构：
- Damage：multiplier + ignore_def_percent
- Heal：amount
- ApplyBuff：buff_id + duration
- Cleanse：无参数

要求：
- type_name() 返回分发键
- 一个技能可有多个 EffectDef

---

## PendingEffectData（Instance）

职责：待处理效果的运行时数据

结构：
- Damage：amount / is_skill / base_amount / modifiers
- Heal：amount / base_amount
- ApplyBuff：buff_id / duration
- Cleanse：无参数

要求：
- base_amount 在 Modify 首次记录
- modifiers 记录每步修饰

---

## EffectQueue（Resource）

职责：战斗唯一效果缓冲区

结构：
- pending：PendingEffect 列表

要求：
- Generate 推入，Modify 修改，Execute drain 清空
- Execute 后 pending 必须为空

---

## EffectHandler（Trait）

职责：效果处理器接口

结构：
- type_name()：分发键
- generate()：从 EffectDef 生成 PendingEffectData
- preview()：生成 EffectPreview

要求：
- 新增效果类型只需实现并注册
- 内置四个 Handler

---

## GenerateContext（值对象）

职责：Generate 阶段上下文快照

结构：
- source/target 属性快照
- defense_bonus：地形防御加成
- skill_id / source_tags / terrain_id

要求：
- 纯数据快照，避免 ECS 借用冲突

---

# 禁止事项

禁止：跳过管线直接执行效果

原因：管线保证 Generate → Modify → Execute 严格顺序

违反后果：伤害未经修饰、未经记录、死亡判定被跳过

---

禁止：Generate 阶段修改 ECS 状态

原因：Generate 只生成效果，不执行效果

违反后果：伤害未经修饰直接生效

---

禁止：Modify 阶段发送 Message

原因：Modify 只修饰数值，不产生副作用

违反后果：UI 在修饰未完成时收到通知

---

禁止：Execute 阶段创建新 CombatIntent

原因：Execute 是管线终点

违反后果：无限循环攻击

---

禁止：match 分发效果类型

原因：Handler 通过 type_name() 分发

违反后果：新增效果类型需要修改分发代码

---

# AI 修改规则

## 如果新增效果类型

允许：
- 新增 EffectDef 变体
- 新增 PendingEffectData 变体
- 新增 EffectHandler 实现并注册

禁止：
- 修改 generate_combat_effects 流程
- 修改 modify_effects 流程
- 修改 execute_effects 流程

优先检查：
- EffectHandlerRegistry 注册
- Modify 阶段是否需要适配
- Execute 阶段是否需要适配
- Preview 是否需要配套

---

## 如果修改伤害公式

允许：
- 修改 DamageHandler.calculate

禁止：
- 修改管线三步流程
- 修改下限保护规则

优先检查：
- 伤害下限 ≥ 1
- ignore_def_percent 基于 base_def
- 地形防御加成在最后减去

---

## 如果测试失败

排查顺序：
1. 检查管线是否跳步
2. 检查 EffectQueue 是否在 Execute 后清空
3. 检查 Handler 是否正确注册
4. 检查 base_amount 是否首次记录
5. 检查伤害下限保护
