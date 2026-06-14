---
id: history.archive.skill_rules_v1
title: skill_rules_v1
status: archived
owner: domain-designer
created: 2026-06-14
updated: 2026-06-14
superseded_by: ../../02-domain/skill/skill-rules.md
---

# Skill 领域

Version: 1.1

Skill 领域管理技能的定义、槽位、冷却、条件校验和效果预览。遵循 Definition / Instance 分离和 Rule / Content 分离。

核心原则：
- 🟥 Definition / Instance 分离（宪法 1.1.2）
- 🟥 Rule / Content 分离（宪法 1.1.3）
- 🟥 数据驱动（宪法 1.1.5）
- 🟥 纯函数校验（宪法 2.1.2 数据与行为分离）

---

# 术语定义

## SkillData

技能的静态定义，描述技能"是什么"。

不是 SkillCooldowns。SkillData 不可变，SkillCooldowns 是运行时状态。

关键属性：
- id / name / description：标识和展示
- cost_mp / range / targeting / effects / tags / conditions / cooldown：技能参数

---

## SkillSlots

单位拥有的技能 ID 列表，表示"能使用什么技能"。

不是 SkillData。SkillSlots 是引用，SkillData 是定义。

关键属性：
- skill_ids：字符串列表，第一个为默认攻击

---

## SkillCooldowns

技能冷却的运行时追踪，记录每个技能的剩余冷却回合。

不是 SkillData。Cooldowns 是实例状态，SkillData 是配置。

关键属性：
- cooldowns：skill_id → 剩余回合数映射

---

## SkillCondition

技能使用条件，决定"什么时候能用"。

不是 SkillTargeting。Condition 是前置检查，Targeting 是目标选择方式。

关键属性：
- MpCost / RequireTag / TargetRequireTag / HpBelow / HpAbove

---

## SkillPreview

技能效果预览，不修改任何状态的纯计算结果。

不是实际伤害。Preview 是估算，实际伤害走 Effect Pipeline。

关键属性：
- predictions：EffectPreview 列表

---

## effective_skill_range

技能有效射程，考虑技能自身 range 和单位基础攻击范围。

不是 SkillData.range。effective_skill_range 是计算结果，range 是配置值。

关键属性：
- range > 0 时使用技能自身射程
- range == 0 时使用单位基础攻击范围

---

# 领域边界

## 本领域负责

- SkillData 定义和注册表（SkillRegistry）
- SkillSlots 槽位管理
- SkillCooldowns 冷却追踪和递减
- SkillCondition 条件校验（can_use）
- 技能效果预览（SkillPreview）
- 有效射程计算（effective_skill_range）

## 本领域不负责

- 效果的生成/修饰/执行（由 effect_pipeline / battle_rules 领域负责）
- 技能目标查找（由 battle_rules 领域负责）
- Buff 的生命周期（由 buff_rules 领域负责）
- AI 技能选择（由 ai_rules 领域负责）
- UI 技能面板展示（由 ui_rules 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 技能使用 | CombatIntent 设置 | battle |
| 冷却设置 | 直接函数调用 | battle（execute_action） |
| 技能标签 | source_tags 传递 | battle（Modify 阶段） |

---

# 生命周期

本领域无状态机，为纯函数式计算。

can_use() 和 preview_skill_effects() 均为纯函数，不修改状态。

冷却生命周期：
```
使用技能 → set(skill_id, cooldown) → 每回合 tick() → 归零移除
```

---

# 不变量

## 不变量1：can_use 纯函数 🟥

宪法依据：2.1.2（数据与行为分离）

任意时刻：

can_use() 不修改任何状态。

违反表现：

调用 can_use() 后产生副作用（如修改冷却、扣除 MP）。

---

## 不变量2：冷却归零自动清理 🟥

tick() 完成后：

冷却为 0 的条目必须从 cooldowns 中移除。

违反表现：

HashMap 中存在值为 0 的条目，get() 返回 0 但条目仍在。

---

## 不变量3：set(0) 不产生记录 🟩

set(skill_id, 0) 调用后：

cooldowns 中不存在该 skill_id 的条目。

违反表现：

冷却为 0 的条目被记录，与"无冷却"状态混淆。

---

## 不变量4：预览不修改状态 🟥

宪法依据：1.1.4（逻辑与表现分离）

preview_skill_effects() 调用后：

所有 ECS 组件和 Resource 无变化。

违反表现：

预览后 HP 被扣减、冷却被设置。

架构违规检测：

发现预览修改 ECS 状态时，必须停止。必须输出：

```
ARCHITECTURE VIOLATION: 预览修改 ECS 状态，违反"预览是纯函数"原则。
```

---

# 业务规则

## 规则1：技能定义 🟥

禁止：
- 🟥 硬编码技能效果
- 🟥 运行时修改 SkillData

必须：
- 新增技能修改 RON 配置
- 技能标签作为 source_tags 传入效果管线
- 多效果按 effects 列表顺序处理

允许：
- 🟩 无 RON 文件时使用内置默认技能
- 🟩 register_defaults() 幂等调用

---

## 规则2：条件校验 🟥

禁止：
- 🟥 跳过冷却检查
- 🟥 can_use() 中修改状态

必须：
- 校验顺序：冷却 → conditions（按定义顺序，短路返回）
- TargetRequireTag 在 target_tags 为 None 时跳过
- 返回具体失败原因（SkillUseError）

允许：
- 🟩 纯函数调用

---

## 规则3：冷却管理 🟥

禁止：
- 🟥 跳过冷却直接使用技能
- 🟥 冷却不递减

必须：
- 使用技能后 set(skill_id, skill_data.cooldown)
- 每回合结束调用 tick()
- 归零后自动移除

允许：
- 🟩 clear() 清除所有冷却

---

## 规则4：有效射程 🟩

禁止：
- 硬编码射程

必须：
- range > 0 使用技能自身射程
- range == 0 使用单位基础攻击范围

---

# 流程管线

## 条件校验管线

冷却检查 → conditions 逐条检查 → 全部通过

### Step1：冷却检查

输入：skill_id + SkillCooldowns
处理：get(skill_id)，> 0 则失败
输出：OnCooldown 或 继续
🟥 禁止：修改冷却值

### Step2：conditions 检查

输入：conditions 列表 + source/target 属性和标签
处理：按定义顺序逐条检查，短路返回第一个失败
输出：SkillUseError 或 Ok(())
🟥 禁止：修改任何状态

---

## 预览管线

构建上下文 → 遍历效果 → Handler 生成预览 → 合并结果

### Step1：构建上下文

输入：source/target 实体
处理：从 ECS 查询构建 SkillExecutionContext 快照
输出：SkillExecutionContext
🟥 禁止：修改 ECS 状态

### Step2：遍历效果

输入：SkillData.effects + Context
处理：通过 EffectHandlerRegistry 分发，每个 Handler 生成 EffectPreview
输出：EffectPreview 列表
🟥 禁止：修改 ECS 状态

---

# 数据结构

## SkillData（Definition）

职责：技能的静态定义

结构：
- id / name / description：标识和展示
- cost_mp：MP 消耗
- range：射程（0 = 使用基础攻击范围）
- targeting：目标类型（SingleEnemy / SingleAlly / SelfOnly / AoeEnemies / AoeAllies / NoTarget）
- effects：效果定义列表
- tags：技能标签列表
- conditions：使用条件列表
- cooldown：冷却回合数
- priority：AI 决策优先级

要求：
- 🟥 不可变，加载后不修改（宪法 1.1.2）
- 🟥 RON 配置路径：assets/skills/（宪法 1.1.5）

---

## SkillSlots（Instance）

职责：单位拥有的技能 ID 列表

结构：
- skill_ids：字符串列表

要求：
- 🟩 第一个为默认攻击
- 🟩 空列表时 default_attack() 回退 BASIC_ATTACK_ID

---

## SkillCooldowns（Instance）

职责：冷却追踪

结构：
- cooldowns：skill_id → 剩余回合数映射

要求：
- 🟩 get() 未记录返回 0
- 🟩 set(0) 不产生记录
- 🟥 tick() 归零后移除

---

## SkillCondition（值对象）

职责：技能使用条件

结构：
- MpCost(i32)：MP 不足时不可用
- RequireTag(GameplayTag)：自身缺少标签时不可用
- TargetRequireTag(GameplayTag)：目标缺少标签时不可用
- HpBelow(f32) / HpAbove(f32)：HP 阈值检查

要求：
- 🟩 can_use() 中按顺序检查
- 🟩 TargetRequireTag 在无目标时跳过

---

## SkillPreview（值对象）

职责：技能效果预览结果

结构：
- skill_id / skill_name：技能标识
- predictions：EffectPreview 列表

要求：
- 🟥 纯函数生成，不修改状态
- 🟩 伤害预览最低 1，治疗预览不超过 MaxHp

---

# 禁止事项

🟥 禁止：硬编码技能效果

原因：技能效果由 RON 配置驱动，硬编码违反 Rule/Content 分离（宪法 1.1.3）

违反后果：新增技能需要修改代码，无法通过配置扩展

---

🟥 禁止：跳过冷却检查

原因：冷却是技能平衡的核心机制

违反后果：技能无限使用，游戏平衡被破坏

---

🟥 禁止：can_use() 中修改状态

原因：can_use() 是纯函数，用于 UI 和 AI 安全调用（宪法 2.1.2）

违反后果：UI 悬停时触发副作用，MP 被扣减

---

🟥 禁止：预览修改状态

原因：预览是估算，不是实际执行（宪法 1.1.4）

违反后果：预览后 HP 被扣减，玩家未确认就受伤

架构违规检测：

```
ARCHITECTURE VIOLATION: 预览修改 ECS 状态，违反"预览是纯函数"原则。
```

---

🟥 禁止：运行时修改 SkillData

原因：SkillData 是共享定义，修改影响所有引用（宪法 1.1.2）

违反后果：多个单位共享技能定义时产生意外副作用

架构违规检测：

```
ARCHITECTURE VIOLATION: 运行时修改 SkillData，违反 Definition/Instance 分离原则。
```

---

# AI 修改规则

## 如果新增技能

允许：
- 新增 RON 配置文件
- 新增 EffectDef 变体（需配套 Handler）

禁止：
- 🟥 修改 SkillData 结构
- 🟥 修改 can_use() 逻辑
- 🟥 修改 SkillRegistry 加载流程

优先检查：
- EffectHandlerRegistry 是否有对应 Handler
- SkillCondition 是否覆盖新条件
- tags 是否在 GameplayTag 枚举中

---

## 如果新增技能条件

允许：
- 新增 SkillCondition 变体
- 在 can_use() 中添加检查分支

禁止：
- 🟥 修改现有条件的逻辑
- 🟥 改变条件检查顺序

优先检查：
- SkillConditionDef RON 反序列化
- can_use() 短路逻辑

---

## 如果新增效果类型

允许：
- 新增 EffectDef 变体
- 新增 EffectHandler 实现并注册

禁止：
- 🟥 修改 preview_skill_effects 流程
- 🟥 修改 generate_combat_effects 流程

优先检查：
- EffectHandlerRegistry 注册
- 预览 Handler 是否配套

---

## 如果测试失败

排查顺序：
1. 检查 can_use() 条件是否全部通过
2. 检查冷却是否正确递减
3. 检查 effective_skill_range 计算
4. 检查 RON 配置是否合法
5. 检查 EffectHandler 是否正确注册

测试要求（宪法 13.0.1-13.0.3）：
- 🟩 单元测试：验证 can_use() 条件校验
- 🟩 集成测试：验证完整技能使用流程
- 🟩 Bug 修复必须先编写重现测试（宪法 13.0.2）

---

# 宪法条款映射

| 宪法条款 | 本领域对应 |
|----------|-----------|
| 1.1.2 Definition/Instance 分离 | SkillData(Definition) vs SkillCooldowns/SkillSlots(Instance) |
| 1.1.3 Rule/Content 分离 | can_use() 是规则，RON 配置是内容 |
| 1.1.4 逻辑与表现分离 | 预览是表现，不修改逻辑状态 |
| 1.1.5 数据驱动 | SkillData 从 RON 加载 |
| 2.1.2 数据与行为分离 | can_use() 是纯函数 |

---

# 架构违规检测

| 违规行为 | 检测方式 | 输出 |
|----------|----------|------|
| 预览修改 ECS 状态 | 代码审查 | ARCHITECTURE VIOLATION: 预览修改 ECS 状态，违反"预览是纯函数"原则。 |
| 运行时修改 SkillData | 代码审查 | ARCHITECTURE VIOLATION: 运行时修改 SkillData，违反 Definition/Instance 分离原则。 |
