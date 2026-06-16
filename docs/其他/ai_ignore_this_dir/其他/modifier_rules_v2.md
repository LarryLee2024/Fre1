# Modifier Rules 领域

Version: 2.0

## Purpose

Modifier Rules 领域管理数据驱动的效果修饰管线，通过标签匹配 + 计算器 trait 分发，实现伤害/治疗的多步修饰链。本领域为纯函数式计算，无状态机。

---

## Glossary

| 术语 | 定义 | 易混淆项 |
|------|------|----------|
| ModifierRule | 修饰规则，定义"在什么条件下对效果做什么修改" | ≠ ModifierEffect：Rule 是条件+效果的组合，Effect 是纯效果类型 |
| ModifierEffect | 修饰效果类型，定义"怎么修改数值" | ≠ ModifierRule：Effect 是纯效果，Rule 包含匹配条件 |
| ModifierCalculator | 计算器 trait，执行具体的数值计算 | ≠ ModifierEffect：Calculator 是执行者，Effect 是配置 |
| ModifierEntry | 修饰记录，追踪每步修饰的 before/after | ≠ ModifierRule：Entry 是记录，Rule 是规则 |

---

## Responsibilities

### Owns

- ModifierRule 定义和注册表
- ModifierEffect 定义
- ModifierCalculator trait 和注册表
- ModifierEntry 修饰记录
- apply_damage_modifiers / apply_heal_modifiers 方法

### Does Not Own

- 效果管线的 Generate 和 Execute → effect_pipeline
- 属性计算和修饰符管线 → stat_system
- 标签系统 → character
- UI 展示 → ui_rules

---

## Invariants

### INV-MOD-01：双标签同时匹配 🟥

规则生效时：source_tag 在攻击方技能标签中 AND target_tag 在目标标签集合中。

违反：单标签匹配导致规则误触发。

### INV-MOD-02：链式修饰顺序固定 🟥

规则按注册顺序依次应用，结果累积。

违反：修饰顺序影响结果，同一组规则不同顺序得到不同值。

### INV-MOD-03：伤害下限为 1 🟥

修饰完成后伤害值 ≥ 1。

违反：伤害为 0 或负数。

### INV-MOD-04：治疗下限为 0 🟥

修饰完成后治疗值 ≥ 0。

违反：治疗为负数。

### INV-MOD-05：Calculator 通过 type_name 分发 🟥

宪法：6.0.2

效果类型分发必须通过 ModifierCalculator.type_name() 查找对应计算器，禁止 match 分发效果类型。

违反：新增效果类型需要修改分发代码。

### INV-MOD-06：Rule/Content 分离 🟥

宪法：1.1.3

修饰管线是规则（代码实现），修饰规则是内容（RON 配置）。禁止硬编码规则逻辑。

违反：新增规则需修改代码。

---

## State Machine

本领域无状态机，为纯函数式计算。

修饰规则生命周期由外部驱动：
- 加载时：从 RON 配置注册到 ModifierRuleRegistry
- Modify 阶段：apply_damage/heal_modifiers 被调用
- 每次调用：遍历规则 → 匹配标签 → 链式计算 → 返回结果

---

## Business Rules

### BR-MOD-01：标签匹配

- source_tag 在攻击方技能标签中
- target_tag 在目标标签集合中
- 两个条件同时满足时规则生效
- 无规则匹配时原值不变

### BR-MOD-02：链式修饰

- 遍历所有规则
- 匹配的规则按注册顺序依次应用
- 结果累积

### BR-MOD-03：Calculator 分发

- 通过 type_name() 查找 Calculator
- applies_to_damage / applies_to_heal 过滤
- 新增效果类型只需实现 trait 并注册

### BR-MOD-04：修饰记录

- 每步记录 before / after / rule_name
- ModifierEntry 写入 BattleRecord

---

## Pipelines

### 修饰管线

遍历规则 → 标签匹配 → 查找计算器 → 计算 → 记录 → 下一条

| 步骤 | 输入 | 输出 | 约束 |
|------|------|------|------|
| 遍历规则 | ModifierRuleRegistry.rules | 匹配的规则 | 禁止跳过任何规则 |
| 标签匹配 | source_tags + target_tags + rule 条件 | 是否匹配 | 禁止单标签匹配（INV-MOD-01） |
| 查找计算器 | rule.effect + ModifierCalculatorRegistry | Calculator | 找不到时跳过该规则 |
| 计算和记录 | current_value + Calculator + rule.effect | 修饰后的值 + ModifierEntry | 禁止跳过记录 |

---

## Data Model

### ModifierRule（Definition）

修饰规则定义。

- name：规则名称
- source_tag：攻击方标签条件
- target_tag：目标标签条件
- effect：修饰效果
- 配置来源：RON（assets/rules/）

### ModifierEffect（值对象）

修饰效果类型。

- DamageMultiplier(f32)：伤害 × 倍率
- DamageBonus(i32)：伤害 + 固定值
- HealMultiplier(f32)：治疗 × 倍率
- HealBonus(i32)：治疗 + 固定值

### ModifierCalculator（Trait）

修饰计算器。

- type_name()：分发键
- applies_to_damage() / applies_to_heal()：适用范围
- calculate()：计算修饰后的值

### ModifierEntry（值对象）

修饰记录。

- before / after：修饰前后值
- rule_name：规则名称

---

## Cross Domain Contracts

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 伤害修饰 | apply_damage_modifiers_with_breakdown | effect_pipeline |
| 治疗修饰 | apply_heal_modifiers_with_breakdown | effect_pipeline |
| 修饰记录 | ModifierEntry | battle_record |

---

## Change Rules

### 新增修饰效果类型

- 允许：新增 ModifierEffect 变体 + 新增 ModifierCalculator 实现并注册
- 禁止：修改修饰管线流程、修改标签匹配逻辑
- 检查：ModifierCalculatorRegistry 注册、type_name() 是否与变体名一致、applies_to 过滤

### 新增修饰规则

- 允许：新增 ModifierRule RON 配置
- 禁止：硬编码规则逻辑（INV-MOD-06）
- 检查：source_tag 和 target_tag 是否在 GameplayTag 枚举中、效果数值平衡、与现有规则的叠加效果

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
| INV-MOD-05 | match 分发效果类型 | 应通过 type_name() 分发 | 改为注册表查找 |
| INV-MOD-06 | 硬编码规则逻辑 | Rule/Content 分离 | 改为 RON 配置 |

---

## Test Requirements

宪法：13.0.1-13.0.3

- 单元测试：验证每种 Calculator 计算正确性
- 集成测试：验证完整修饰管线
- Bug 修复必须先编写重现测试

排查顺序：
1. 双标签是否同时匹配
2. Calculator 是否正确注册
3. 链式修饰顺序
4. 伤害下限 ≥ 1
5. 治疗下限 ≥ 0
