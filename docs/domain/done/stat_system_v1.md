# Stat System 领域

Version: 1.1

Stat System 领域管理角色的所有数值计算，采用三层架构（核心属性 → 衍生属性 → 生命资源），通过统一修饰符管线实现所有数值变化。

核心原则：
- 🟥 Primary Stat 与 Derived Stat 分离
- 🟩 Derived Stat 优先实时计算
- 🟥 统一 Modifier 管线（宪法 2.2.1）
- 🟩 属性公式集中管理

---

# 术语定义

## Core Stat

8 维核心属性，有基础值，可通过修饰符修改。

不是 Derived Stat。Core Stat 有 base 值，Derived Stat 没有。

关键属性：
- Might / Dexterity / Agility / Vitality / Intelligence / Willpower / Presence / Luck

---

## Derived Stat

13 维衍生属性，从 Core Stat 实时推导，不存储基础值。

不是 Core Stat。Derived Stat 无 base 值，set_base() 对其无效。

关键属性：
- MaxHp / MaxMp / MaxStamina / Attack / Defense / MagicAttack / MagicDefense / Accuracy / Evasion / CritRate / MoveRange / Initiative / AttackRange

---

## Vital Resource

3 维生命资源，存储当前值。

不是 Derived Stat。Vital Resource 有 current 值而非 base 值。

关键属性：
- HP / MP / Stamina

---

## Modifier

属性修饰符，通过 Add 或 Multiply 修改属性值。

不是直接属性修改。Modifier 走管线，直接修改绕过管线。

关键属性：
- ModifierOp：Add（加法）或 Multiply（乘法）
- value：修饰值
- source：来源标识（ModifierSource）

---

## ModifierSource

修饰符来源标识，u64 类型，按区间隔离。

不是 GameplayTag。ModifierSource 标识"谁加的"，GameplayTag 标识"有什么标签"。

关键属性：
- Trait 区间：u64::MAX ~ u64::MAX - 999
- Equipment 区间：u64::MAX - 1000 ~ u64::MAX - 1999
- Buff 区间：1 ~ 999999

---

## ModifierRule

数据驱动的效果修饰规则，通过标签匹配触发。

不是硬编码 if-else。ModifierRule 是配置，不是代码。

关键属性：
- source_tag：攻击方技能需要的标签
- target_tag：目标需要的标签
- effect：修饰效果（DamageMultiplier / DamageBonus / HealMultiplier / HealBonus）

---

## ModifierCalculator

修饰计算器 trait，执行具体数值计算。

不是 ModifierRule。Calculator 是执行者，Rule 是配置。

关键属性：
- type_name()：分发键
- applies_to_damage / applies_to_heal：适用范围
- calculate()：计算逻辑

---

## ModifierEntry

修饰记录，记录每步修饰的前后值和规则名。

不是 Modifier。Entry 是记录，Modifier 是操作。

关键属性：
- before / after：修饰前后值
- rule_name：触发规则名称

---

# 领域边界

## 本领域负责

- 三层属性架构（Core / Derived / Vital）
- 衍生属性公式计算
- 修饰符管线的添加、移除、计算
- ModifierSource 区间管理
- ModifierRule 的标签匹配和计算
- 伤害/治疗的修饰计算和下限保护

## 本领域不负责

- 属性变化后的 UI 刷新（由 ui_rules 领域负责）
- Buff 的生命周期管理（由 buff_rules 领域负责）
- 装备的穿脱触发（由 equipment_rules 领域负责）
- 效果管线的生成和执行（由 effect_pipeline 领域负责）
- Trait 效果分发（由 trait_rules 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 属性修饰符变化 | 通过 ModifierSource 添加/移除 | character（重建 Trait） |
| 伤害/治疗修饰结果 | 返回值 | battle（效果执行） |
| HP 变化 | 通过 set_vital() 修改 | battle（死亡判定） |

---

# 生命周期

本领域无状态机，为纯函数式计算。

属性值 = f(base, modifiers)，任意时刻可重新计算。

---

# 不变量

## 不变量1：衍生属性无基础值 🟥

任意时刻：

Derived Stat 不存储 base 值，set_base() 对其无效。

违反表现：

衍生属性与核心属性脱钩，公式一致性被破坏。

---

## 不变量2：先加后乘 🟥

任意时刻：

修饰符计算顺序固定：先 Add 后 Multiply。

最终值 = (base + Σ Add) × Π Multiply

违反表现：

修饰符顺序影响结果，同一组修饰符不同顺序得到不同值。

---

## 不变量3：乘法零值保护 🟥

任意时刻：

乘法修饰符乘积为 0 时视为 1.0。

违反表现：

单个 Multiply ×0 修饰符将属性归零，角色无法行动。

---

## 不变量4：伤害/治疗下限 🟥

修饰计算完成后：

伤害 ≥ 1，治疗 ≥ 0。

违反表现：

伤害为 0 或负数，治疗为负数（变成伤害）。

---

## 不变量5：Source 区间隔离 🟥

任意时刻：

Trait / Equipment / Buff 修饰符的 Source 值在各自区间内，互不重叠。

违反表现：

按来源移除修饰符时误删其他来源的修饰符。

---

## 不变量6：三类属性互斥 🟥

任意时刻：

每个 AttributeKind 恰好属于 Core / Derived / Vital 三类之一。

违反表现：

is_core() / is_derived() / is_vital() 判定矛盾。

---

## 不变量7：禁止直接修改最终属性值 🟥

宪法依据：2.2.1（禁止直接修改最终属性值）

任意时刻：

所有属性修改必须通过 Modifier 管线，禁止直接修改 get() 返回的最终值。

违反表现：

代码中存在 `attributes.attack = 50` 或 `hp -= damage` 等直接赋值。

架构违规检测：

发现直接修改最终属性值时，必须停止。必须输出：

```
ARCHITECTURE VIOLATION: 直接修改最终属性值，违反"属性修改必须通过 Modifier 管线"原则。
```

---

# 业务规则

## 规则1：属性访问 🟥

宪法依据：2.2.1（禁止直接修改最终属性值）

禁止：
- 🟥 直接修改最终属性值
- 🟥 对衍生属性调用 set_base()
- 🟥 对非 Vital 调用 set_vital()

必须：
- 通过 get(kind) 统一访问
- Core Stat 通过 set_base() 设置基础值
- Vital Resource 通过 set_vital() 设置当前值
- Derived Stat 通过公式实时计算

允许：
- 🟩 Modifier 作用于 Core Stat 和 Derived Stat
- 🟩 Modifier 作用于 Core Stat 时级联影响 Derived Stat

---

## 规则2：修饰符管理 🟥

禁止：
- 🟥 绕过 Modifier 管线直接改属性
- 🟥 修饰符无 Source 标识
- 🟥 修饰符无过期条件

必须：
- 添加修饰符时指定 ModifierSource
- 按来源移除时使用 remove_modifiers_from(source)
- 减益判定：Add < 0 或 Multiply < 1.0

允许：
- 🟩 叠加多个修饰符
- 🟩 remove_debuff_modifiers() 批量清理减益

---

## 规则3：ModifierRule 标签匹配 🟥

宪法依据：1.1.3（Rule/Content 分离）、6.0.2（Trait 替代 match）

禁止：
- 🟥 硬编码 if-else 替代 ModifierRule
- 🟥 match 分发效果类型

必须：
- source_tag ∈ 攻击方标签 AND target_tag ∈ 目标标签时触发
- 多条规则按顺序叠加
- 伤害 ≥ 1，治疗 ≥ 0

允许：
- 🟩 自定义 Calculator 注册到 CalculatorRegistry
- 🟩 RON 配置新增规则

---

## 规则4：fill_vital_resources 🟩

禁止：
- 战斗中调用 fill_vital_resources()

必须：
- 仅在单位生成时调用一次
- 将 HP/MP/Stamina 设为对应 Max 值

允许：
- 🟩 战斗中通过 set_vital() 修改当前值

---

# 流程管线

## 修饰符计算管线

基础值 → Add 修饰符求和 → Multiply 修饰符求积 → 下限保护

### Step1：基础值

输入：AttributeKind
处理：Core Stat 返回 base + 修饰符；Derived Stat 从 Core Stat 实时计算
输出：基础值（含 Core 修饰符）
🟥 禁止：对 Derived Stat 返回 base 值

### Step2：Add 求和

输入：所有 ModifierOp::Add 修饰符
处理：求和
输出：加法总和
🟥 禁止：跳过任何 Add 修饰符

### Step3：Multiply 求积

输入：所有 ModifierOp::Multiply 修饰符
处理：求积，乘积为 0 时视为 1.0
输出：乘法乘积
🟥 禁止：乘积为 0 时返回 0

### Step4：下限保护

输入：计算结果
处理：伤害 max(1, result)，治疗 max(0, result)
输出：最终值
🟥 禁止：跳过下限保护

---

## ModifierRule 应用管线

输入 amount → 遍历规则 → 标签匹配 → Calculator 计算 → 记录 Entry → 下限保护

### Step1：遍历规则

输入：所有 ModifierRule
处理：逐条检查标签匹配
输出：匹配的规则列表
🟥 禁止：跳过任何规则

### Step2：Calculator 计算

输入：匹配的 ModifierEffect + 当前值
处理：通过 CalculatorRegistry 查找计算器，调用 calculate()
输出：修饰后值
🟥 禁止：绕过 Calculator 直接计算

### Step3：记录 Entry

输入：修饰前后值 + 规则名
处理：创建 ModifierEntry
输出：ModifierEntry（with_breakdown 模式）
🟩 禁止：在非 with_breakdown 模式下记录

### Step4：下限保护

输入：最终值
处理：伤害 max(1, result)，治疗 max(0, result)
输出：最终值
🟥 禁止：跳过下限保护

---

# 数据结构

## Attributes（Instance）

职责：角色的所有数值容器

结构：
- base：Core Stat 基础值映射（8 维）
- current_hp / current_mp / current_stamina：Vital Resource 当前值
- base_attack_range：基础攻击范围
- modifiers：修饰符实例列表

要求：
- 🟥 get(kind) 统一访问接口
- 🟥 衍生属性通过公式实时计算
- 🟥 修饰符按先加后乘顺序计算

---

## AttributeModifierDef（Definition）

职责：修饰符的数据定义，用于 RON 配置

结构：
- attribute：目标属性种类
- op：Add 或 Multiply
- value：修饰值

要求：
- 🟩 无 Source，由运行时添加时指定
- 🟩 用于 Trait/Buff/装备的属性修饰配置

---

## AttributeModifierInstance（Instance）

职责：运行时修饰符实例

结构：
- def：AttributeModifierDef
- source：ModifierSource

要求：
- 🟥 Source 必须在对应区间内
- 🟥 按来源精确移除

---

## ModifierRule（Definition）

职责：数据驱动的效果修饰规则

结构：
- name：规则名称
- source_tag：攻击方技能标签
- target_tag：目标标签
- effect：修饰效果

要求：
- 🟥 标签匹配使用 AND 逻辑
- 🟥 RON 配置路径：assets/rules/*.ron（宪法 1.1.5）

---

## ModifierEntry（记录）

职责：修饰步骤记录

结构：
- before：修饰前值
- after：修饰后值
- rule_name：触发规则名称

要求：
- 🟩 仅在 with_breakdown 模式下生成
- 🟩 写入 BattleRecord

---

# 禁止事项

🟥 禁止：直接修改最终属性值

原因：所有属性修改必须走 Modifier 管线，保证来源可追踪（宪法 2.2.1）

违反后果：属性变化无法追踪、无法回滚、无法调试

架构违规检测：

```
ARCHITECTURE VIOLATION: 直接修改最终属性值，违反"属性修改必须通过 Modifier 管线"原则。
```

---

🟥 禁止：对衍生属性调用 set_base()

原因：Derived Stat 由公式推导，不允许设置基础值

违反后果：衍生属性与核心属性脱钩，公式一致性被破坏

---

🟥 禁止：绕过 Modifier 管线直接改属性

原因：管线保证先加后乘、下限保护、来源追踪（宪法 2.2.1）

违反后果：修饰符计算顺序错误、伤害为 0、来源丢失

---

🟥 禁止：修饰符无 Source 标识

原因：Source 是按来源移除修饰符的唯一依据

违反后果：无法精确移除过期修饰符（如 Buff 过期、装备卸下）

---

🟥 禁止：乘法修饰符乘积归零

原因：单个 ×0 修饰符会将属性归零

违反后果：角色无法行动，游戏逻辑崩溃

---

🟥 禁止：硬编码 if-else 替代 ModifierRule

原因：ModifierRule 是数据驱动，新增规则不修改代码（宪法 1.1.3）

违反后果：新增元素克制关系需要修改代码，违反 Rule/Content 分离

---

🟥 禁止：match 分发效果类型

原因：Calculator trait 通过 type_name() 分发，无需 match（宪法 6.0.2）

违反后果：新增效果类型需要修改分发代码，违反开放封闭原则

架构违规检测：

```
ARCHITECTURE VIOLATION: match 分发效果类型，违反"Calculator 通过 type_name 分发"原则。
```

---

# AI 修改规则

## 如果新增属性种类

允许：
- 在 AttributeKind 枚举中新增变体
- 在 is_core / is_derived / is_vital 中分类
- 新增衍生公式

禁止：
- 🟥 修改三类互斥规则
- 🟥 修改先加后乘计算顺序
- 🟥 跳过下限保护

优先检查：
- 新属性属于哪一类（Core / Derived / Vital）
- 衍生公式是否依赖正确的 Core Stat
- ModifierRule 是否需要适配

---

## 如果新增修饰效果类型

允许：
- 新增 ModifierEffect 变体
- 新增 ModifierCalculator 实现并注册

禁止：
- 🟥 修改现有 Calculator 的计算逻辑
- 🟥 修改 ModifierRule 应用管线流程

优先检查：
- CalculatorRegistry 注册
- applies_to_damage / applies_to_heal 判定
- 下限保护是否覆盖新类型

---

## 如果新增 ModifierRule

允许：
- 新增 RON 配置文件

禁止：
- 🟥 修改 ModifierRule 应用管线代码
- 🟥 修改 Calculator 分发逻辑

优先检查：
- source_tag 和 target_tag 是否在 GameplayTag 枚举中
- effect 类型是否有对应 Calculator
- 规则叠加顺序

---

## 如果测试失败

排查顺序：
1. 检查属性分类是否正确（Core / Derived / Vital）
2. 检查衍生公式是否依赖正确的 Core Stat
3. 检查修饰符 Source 区间是否冲突
4. 检查先加后乘计算顺序
5. 检查下限保护是否生效

测试要求（宪法 13.0.1-13.0.3）：
- 🟩 单元测试：验证属性计算公式正确性
- 🟩 集成测试：验证完整修饰符管线
- 🟩 Bug 修复必须先编写重现测试（宪法 13.0.2）

---

# 宪法条款映射

| 宪法条款 | 本领域对应 |
|----------|-----------|
| 1.1.3 Rule/Content 分离 | ModifierRule 是配置，Calculator 是规则 |
| 1.1.5 数据驱动 | ModifierRule 从 RON 加载 |
| 2.2.1 禁止直接修改最终属性 | 所有属性修改通过 Modifier 管线 |
| 6.0.2 Trait 替代 match | ModifierCalculator trait |
| 2.1.2 数据与行为分离 | Attributes 是数据，公式是行为 |

---

# 架构违规检测

| 违规行为 | 检测方式 | 输出 |
|----------|----------|------|
| 直接修改最终属性值 | 代码审查 | ARCHITECTURE VIOLATION: 直接修改最终属性值，违反"属性修改必须通过 Modifier 管线"原则。 |
| match 分发效果类型 | 代码审查 | ARCHITECTURE VIOLATION: match 分发效果类型，违反"Calculator 通过 type_name 分发"原则。 |
