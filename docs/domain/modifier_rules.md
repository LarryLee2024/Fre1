# Modifier Rules 领域

Version: 1.0

Modifier Rules 领域管理数据驱动的效果修饰管线，通过标签匹配 + 计算器 trait 分发，实现伤害/治疗的多步修饰链。

核心原则：
- Rule / Content 分离
- Trait 替代 match
- 标签驱动匹配
- 修饰链可追踪

---

# 术语定义

## ModifierRule

修饰规则，定义"在什么条件下对效果做什么修改"。

不是 ModifierEffect。Rule 是条件+效果的组合，Effect 是纯效果类型。

关键属性：
- name：规则名称
- source_tag：攻击方技能需包含的标签
- target_tag：目标需包含的标签
- effect：修饰效果

---

## ModifierEffect

修饰效果类型，定义"怎么修改数值"。

不是 ModifierRule。Effect 是纯效果，Rule 包含匹配条件。

关键属性：
- DamageMultiplier / DamageBonus / HealMultiplier / HealBonus

---

## ModifierCalculator

计算器 trait，执行具体的数值计算。

不是 ModifierEffect。Calculator 是执行者，Effect 是配置。

关键属性：
- type_name()：分发键
- applies_to_damage() / applies_to_heal()：适用范围
- calculate()：计算修饰后的值

---

## ModifierEntry

修饰记录，追踪每步修饰的 before/after。

不是 ModifierRule。Entry 是记录，Rule 是规则。

关键属性：
- before / after：修饰前后值
- rule_name：规则名称

---

# 领域边界

## 本领域负责

- ModifierRule 定义和注册表（ModifierRuleRegistry）
- ModifierEffect 定义
- ModifierCalculator trait 和注册表（ModifierCalculatorRegistry）
- ModifierEntry 修饰记录
- apply_damage_modifiers / apply_heal_modifiers 方法

## 本领域不负责

- 效果管线的 Generate 和 Execute（由 effect_pipeline 领域负责）
- 属性计算和修饰符管线（由 stat_system 领域负责）
- 标签系统（由 character 领域负责）
- UI 展示（由 ui_rules 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 伤害修饰 | apply_damage_modifiers_with_breakdown | effect_pipeline |
| 治疗修饰 | apply_heal_modifiers_with_breakdown | effect_pipeline |
| 修饰记录 | ModifierEntry | battle_record |

---

# 生命周期

本领域无状态机，为纯函数式计算。

修饰规则生命周期由外部驱动：
- 加载时：从 RON 配置注册到 ModifierRuleRegistry
- Modify 阶段：apply_damage/heal_modifiers 被调用
- 每次调用：遍历规则 → 匹配标签 → 链式计算 → 返回结果

---

# 不变量

## 不变量1：双标签同时匹配

规则生效时：

source_tag 在攻击方技能标签中 AND target_tag 在目标标签集合中。

违反表现：

单标签匹配导致规则误触发。

---

## 不变量2：链式修饰顺序固定

修饰完成后：

规则按注册顺序依次应用，结果累积。

违反表现：

修饰顺序影响结果，同一组规则不同顺序得到不同值。

---

## 不变量3：伤害下限为 1

修饰完成后：

伤害值 ≥ 1。

违反表现：

伤害为 0 或负数。

---

## 不变量4：治疗下限为 0

修饰完成后：

治疗值 ≥ 0。

违反表现：

治疗为负数。

---

# 业务规则

## 规则1：标签匹配

禁止：
- 单标签匹配就触发规则
- 跳过标签检查

必须：
- source_tag 在攻击方技能标签中
- target_tag 在目标标签集合中
- 两个条件同时满足时规则生效

---

## 规则2：链式修饰

禁止：
- 只应用第一条匹配规则
- 修改规则应用顺序

必须：
- 遍历所有规则
- 匹配的规则按注册顺序依次应用
- 结果累积

允许：
- 无规则匹配时原值不变

---

## 规则3：Calculator 分发

禁止：
- match 分发效果类型

必须：
- 通过 type_name() 查找 Calculator
- 新增效果类型只需实现 trait 并注册
- applies_to_damage / applies_to_heal 过滤

---

## 规则4：修饰记录

禁止：
- 丢弃修饰步骤信息

必须：
- 每步记录 before / after / rule_name
- ModifierEntry 写入 BattleRecord

---

# 流程管线

## 修饰管线

遍历规则 → 标签匹配 → 查找计算器 → 计算 → 记录 → 下一条

### Step1：遍历规则

输入：ModifierRuleRegistry.rules
处理：逐条检查
输出：匹配的规则
禁止：跳过任何规则

### Step2：标签匹配

输入：source_tags + target_tags + rule.source_tag + rule.target_tag
处理：双标签匹配
输出：是否匹配
禁止：单标签匹配

### Step3：查找计算器

输入：rule.effect + ModifierCalculatorRegistry
处理：按 type_name 查找 + applies_to 过滤
输出：Calculator
禁止：找不到计算器时 panic

### Step4：计算和记录

输入：current_value + Calculator + rule.effect
处理：calculate() → 记录 ModifierEntry
输出：修饰后的值
禁止：跳过记录

---

# 数据结构

## ModifierRule（Definition）

职责：修饰规则定义

结构：
- name：规则名称
- source_tag：攻击方标签条件
- target_tag：目标标签条件
- effect：修饰效果

要求：
- RON 配置路径：assets/rules/
- 双标签同时匹配

---

## ModifierEffect（值对象）

职责：修饰效果类型

结构：
- DamageMultiplier(f32)：伤害 × 倍率
- DamageBonus(i32)：伤害 + 固定值
- HealMultiplier(f32)：治疗 × 倍率
- HealBonus(i32)：治疗 + 固定值

要求：
- 与 Calculator 一一对应

---

## ModifierCalculator（Trait）

职责：修饰计算器

结构：
- type_name()：分发键
- applies_to_damage() / applies_to_heal()：适用范围
- calculate()：计算修饰后的值

要求：
- 内置四种计算器
- 新增效果类型只需实现并注册

---

## ModifierEntry（值对象）

职责：修饰记录

结构：
- before / after：修饰前后值
- rule_name：规则名称

要求：
- 每步修饰必须记录
- 写入 BattleRecord

---

# 禁止事项

禁止：单标签匹配触发规则

原因：双标签匹配防止规则误触发

违反后果：不相关的修饰被错误应用

---

禁止：match 分发效果类型

原因：Calculator 通过 type_name() 分发

违反后果：新增效果类型需要修改分发代码

---

禁止：丢弃修饰步骤信息

原因：ModifierEntry 用于战斗记录和调试

违反后果：无法追踪伤害/治疗的修饰过程

---

禁止：修改规则应用顺序

原因：链式修饰的顺序影响最终结果

违反后果：同一组规则不同顺序得到不同值

---

# AI 修改规则

## 如果新增修饰效果类型

允许：
- 新增 ModifierEffect 变体
- 新增 ModifierCalculator 实现并注册

禁止：
- 修改修饰管线流程
- 修改标签匹配逻辑

优先检查：
- ModifierCalculatorRegistry 注册
- type_name() 是否与变体名一致
- applies_to_damage / applies_to_heal 是否正确

---

## 如果新增修饰规则

允许：
- 新增 ModifierRule RON 配置

禁止：
- 硬编码规则逻辑

优先检查：
- source_tag 和 target_tag 是否在 GameplayTag 枚举中
- 效果数值是否平衡
- 与现有规则的叠加效果

---

## 如果测试失败

排查顺序：
1. 检查双标签是否同时匹配
2. 检查 Calculator 是否正确注册
3. 检查链式修饰顺序
4. 检查伤害下限 ≥ 1
5. 检查治疗下限 ≥ 0
