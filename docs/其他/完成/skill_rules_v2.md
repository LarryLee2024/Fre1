# Skill 领域

Version: 2.0

## Purpose

Skill 领域管理技能的定义、槽位、冷却、条件校验和效果预览。遵循 Definition / Instance 分离和 Rule / Content 分离。本领域为纯函数式计算，无状态机。

---

## Glossary

| 术语 | 定义 | 易混淆项 |
|------|------|----------|
| SkillData | 技能的静态定义，描述技能"是什么" | ≠ SkillCooldowns：Data 不可变，Cooldowns 是运行时状态 |
| SkillSlots | 单位拥有的技能 ID 列表，表示"能使用什么技能" | ≠ SkillData：Slots 是引用，Data 是定义 |
| SkillCooldowns | 技能冷却的运行时追踪 | ≠ SkillData：Cooldowns 是实例状态，Data 是配置 |
| SkillCondition | 技能使用条件，决定"什么时候能用" | ≠ SkillTargeting：Condition 是前置检查，Targeting 是目标选择方式 |
| SkillPreview | 技能效果预览，不修改任何状态的纯计算结果 | ≠ 实际伤害：Preview 是估算，实际伤害走 Effect Pipeline |
| effective_skill_range | 技能有效射程，考虑技能自身 range 和单位基础攻击范围 | ≠ SkillData.range：effective 是计算结果，range 是配置值 |

---

## Responsibilities

### Owns

- SkillData 定义和注册表
- SkillSlots 槽位管理
- SkillCooldowns 冷却追踪和递减
- SkillCondition 条件校验（can_use）
- 技能效果预览（SkillPreview）
- 有效射程计算

### Does Not Own

- 效果的生成/修饰/执行 → effect_pipeline / battle_rules
- 技能目标查找 → battle_rules
- Buff 的生命周期 → buff_rules
- AI 技能选择 → ai_rules
- UI 技能面板展示 → ui_rules

---

## Invariants

### INV-SKL-01：can_use 纯函数 🟥

宪法：2.1.2

can_use() 不修改任何状态。

违反：调用 can_use() 后产生副作用（如修改冷却、扣除 MP）。

### INV-SKL-02：冷却归零自动清理 🟥

tick() 完成后，冷却为 0 的条目必须从 cooldowns 中移除。

违反：HashMap 中存在值为 0 的条目。

### INV-SKL-03：预览不修改状态 🟥

宪法：1.1.4

preview_skill_effects() 调用后，所有 ECS 组件和 Resource 无变化。

违反：预览后 HP 被扣减、冷却被设置。

### INV-SKL-04：SkillData 不可变 🟥

宪法：1.1.2

SkillData 加载后不可修改，多个实例共享同一定义。

违反：修改定义影响所有引用。

### INV-SKL-05：Rule/Content 分离 🟥

宪法：1.1.3

技能效果由 RON 配置驱动，禁止硬编码技能效果。

违反：新增技能需要修改代码，无法通过配置扩展。

---

## State Machine

本领域无状态机，为纯函数式计算。

冷却生命周期：
```
使用技能 → set(skill_id, cooldown) → 每回合 tick() → 归零移除
```

---

## Business Rules

### BR-SKL-01：技能定义

- 新增技能修改 RON 配置
- 技能标签作为 source_tags 传入效果管线
- 多效果按 effects 列表顺序处理
- 无 RON 文件时使用内置默认技能

### BR-SKL-02：条件校验

- 校验顺序：冷却 → conditions（按定义顺序，短路返回）
- TargetRequireTag 在 target_tags 为 None 时跳过
- 返回具体失败原因（SkillUseError）

### BR-SKL-03：冷却管理

- 使用技能后 set(skill_id, skill_data.cooldown)
- 每回合结束调用 tick()
- 归零后自动移除
- set(0) 不产生记录

### BR-SKL-04：有效射程

- range > 0 使用技能自身射程
- range == 0 使用单位基础攻击范围

---

## Pipelines

### 条件校验管线

冷却检查 → conditions 逐条检查 → 全部通过

| 步骤 | 输入 | 输出 | 约束 |
|------|------|------|------|
| 冷却检查 | skill_id + SkillCooldowns | OnCooldown 或继续 | 禁止修改冷却值 |
| conditions 检查 | conditions 列表 + 属性和标签 | SkillUseError 或 Ok | 禁止修改任何状态 |

### 预览管线

构建上下文 → 遍历效果 → Handler 生成预览 → 合并结果

| 步骤 | 输入 | 输出 | 约束 |
|------|------|------|------|
| 构建上下文 | source/target 实体 | SkillExecutionContext | 禁止修改 ECS 状态（INV-SKL-03） |
| 遍历效果 | SkillData.effects + Context | EffectPreview 列表 | 禁止修改 ECS 状态 |

---

## Data Model

### SkillData（Definition）

技能的静态定义，不可变。

- 标识：id / name / description
- 参数：cost_mp / range / targeting / cooldown / priority
- 效果：effects 列表
- 标签：tags 列表
- 条件：conditions 列表
- 配置来源：RON（assets/skills/）

### SkillSlots（Instance）

单位拥有的技能 ID 列表。

- skill_ids：字符串列表
- 第一个为默认攻击

### SkillCooldowns（Instance）

冷却追踪。

- cooldowns：skill_id → 剩余回合数映射
- get() 未记录返回 0
- tick() 归零后移除

### SkillCondition（值对象）

技能使用条件。

- MpCost(i32)：MP 不足时不可用
- RequireTag(GameplayTag)：自身缺少标签时不可用
- TargetRequireTag(GameplayTag)：目标缺少标签时不可用
- HpBelow(f32) / HpAbove(f32)：HP 阈值检查

### SkillPreview（值对象）

技能效果预览结果。

- skill_id / skill_name
- predictions：EffectPreview 列表
- 伤害预览最低 1，治疗预览不超过 MaxHp

---

## Cross Domain Contracts

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 技能使用 | CombatIntent 设置 | battle |
| 冷却设置 | 直接函数调用 | battle |
| 技能标签 | source_tags 传递 | battle（Modify 阶段） |

---

## Change Rules

### 新增技能

- 允许：新增 RON 配置文件 + 新增 EffectDef 变体（需配套 Handler）
- 禁止：修改 SkillData 结构、修改 can_use() 逻辑、修改 SkillRegistry 加载流程
- 检查：EffectHandlerRegistry 是否有对应 Handler、SkillCondition 是否覆盖新条件、tags 是否在 GameplayTag 枚举中

### 新增技能条件

- 允许：新增 SkillCondition 变体 + 在 can_use() 中添加检查分支
- 禁止：修改现有条件的逻辑、改变条件检查顺序
- 检查：SkillConditionDef RON 反序列化、can_use() 短路逻辑

### 新增效果类型

- 允许：新增 EffectDef 变体 + 新增 EffectHandler 实现并注册
- 禁止：修改 preview_skill_effects 流程、修改 generate_combat_effects 流程
- 检查：EffectHandlerRegistry 注册、预览 Handler 是否配套

---

## Architecture Violations

发现架构违规时统一输出：

```
ARCHITECTURE VIOLATION:
Rule: <RuleID>
Reason: <Why>
Fix: <How>
```

| RuleID | 违规行为 | Reason | Fix |
|--------|----------|--------|-----|
| INV-SKL-03 | 预览修改 ECS 状态 | 预览是纯函数 | 将状态修改移到执行阶段 |
| INV-SKL-04 | 运行时修改 SkillData | Definition/Instance 分离 | 改为修改运行时实例 |
| INV-SKL-05 | 硬编码技能效果 | Rule/Content 分离 | 改为 RON 配置 |

---

## Test Requirements

宪法：13.0.1-13.0.3

- 单元测试：验证 can_use() 条件校验
- 集成测试：验证完整技能使用流程
- Bug 修复必须先编写重现测试

排查顺序：
1. can_use() 条件是否全部通过
2. 冷却是否正确递减
3. effective_skill_range 计算
4. RON 配置是否合法
5. EffectHandler 是否正确注册
