---
id: 02-domain.formula.formula-rules
title: Formula Rules
status: draft
owner: domain-designer
created: 2026-06-14
updated: 2026-06-14
tags:
  - domain
  - formula
---

# 公式系统领域

Version: 1.0
Status: Proposed

公式系统领域管理游戏数值的计算方法，Effect 只负责调用公式，不负责怎么算。

核心原则：
- 🟩 8.0.5 Effect 只负责指定 FormulaId，不负责计算公式（公式集中管理在领域层）
- 🟩 1.4.1 Formula 计算是纯函数：相同输入 → 相同输出（核心领域与引擎解耦）
- 🟩 11.9 Formula 不包含随机逻辑（随机由独立的 Random 系统处理，RNG 多流分离）

---

# 术语定义

## 公式（Formula）

🟩 8.0.5 游戏数值的计算方法，封装伤害、治疗、护盾等效果的计算逻辑。公式集中在领域层管理。

不是 Effect。不是 Modifier。不是属性值。

关键属性：
- 实现 Formula trait：calculate(input) → output
- 纯函数：相同输入产生相同输出
- 不包含随机逻辑（随机由 Random 系统注入）
- 通过 FormulaId 注册到 FormulaRegistry

---

## 公式标识（FormulaId）

公式的唯一标识符，用于在运行时查找对应的 Formula 实现。

不是公式本身。不是 Effect 类型。不是公式名称。

关键属性：
- 枚举类型，每个变体对应一个公式
- 10 种内置：PhysicalDamage / MagicDamage / TrueDamage / HealFormula / PoisonFormula / BurnFormula / SummonFormula / ShieldFormula / ExperienceFormula / LevelUpFormula
- 通过 FormulaRegistry 按 ID 查找
- 新增公式需添加枚举变体

---

## 公式注册表（FormulaRegistry）

所有公式的注册集合，运行时通过 FormulaId 查找 Formula 实现。

不是计算函数集合。不是 HashMap。不是全局状态。

关键属性：
- 存储 FormulaId → Formula trait object 的映射
- 通过 register() 方法注册新公式
- 通过 get(id) 方法查找公式
- O(1) HashMap 查找

---

## 公式输入（FormulaInput）

公式计算所需的参数集合。

不是全局状态。不是随机数。不是 ECS World。

关键属性：
- source_attrs：攻击者属性
- target_attrs：目标属性
- base_value：基础值（如技能伤害系数）
- element：元素类型
- terrain_bonus：地形加成
- stack_count：Buff 层数（用于层数缩放）
- 纯数据传递，不存储持久状态

---

## 公式输出（FormulaOutput）

公式计算的结果值。

不是最终伤害。不是修饰后值。不是属性变化量。

关键属性：
- value：计算结果（i32 或 f32）
- breakdown：计算过程记录（用于 Debug）
- 纯计算结果，不包含执行逻辑
- 由 Effect Handler 在 Execute 阶段使用

---

## 公式计算（Formula Calculation）

从 FormulaInput 到 FormulaOutput 的纯函数计算过程。

不是 Modifier 修饰。不是效果执行。不是属性修改。

关键属性：
- 输入：FormulaInput
- 处理：Formula.calculate(input)
- 输出：FormulaOutput
- 纯函数，无副作用

---

# 领域边界

## 本领域负责

- FormulaId 的 10 种枚举定义
- Formula trait 的接口定义
- FormulaRegistry 的注册和查找
- FormulaInput / FormulaOutput 的数据结构
- 公式计算的纯函数执行

## 本领域不负责

- Effect 的调用逻辑（由 Effect Pipeline 领域负责）
- Modifier 的修饰链（由 Attribute Modifier 领域负责）
- 随机数生成（由 Random 系统负责）
- 属性值查询（由 Attribute 领域负责）
- Buff 层数管理（由 StackPolicy 领域负责）
- 效果执行和状态变更（由 Effect Pipeline 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 公式查找请求 | 函数调用（get） | Effect Pipeline 领域（Generate 阶段） |
| 公式计算结果 | 返回值（FormulaOutput） | Effect Pipeline 领域（Modify 阶段） |
| 公式注册 | 函数调用（register） | 初始化阶段（App 启动） |
| 属性值查询 | 函数调用 | Attribute 领域（构建 FormulaInput） |

---

# 生命周期

本领域无状态机，为纯函数式计算。

FormulaRegistry 在 App 启动时注册所有内置公式，运行时不变。Formula 计算是纯函数，无状态。

---

# 不变量

## 不变量1：Formula 计算是纯函数 🟥 1.4.1

任意时刻：

🟩 1.4.1 Formula.calculate(input) 必须是纯函数：相同 FormulaInput 产生相同 FormulaOutput，无副作用。

违反表现：

Formula 内部修改全局状态、读取 ECS World、产生随机数。

---

## 不变量2：Formula 不包含随机逻辑 🟥 11.9

任意时刻：

🟩 11.9.2 Formula 计算不包含随机数生成。随机性由独立的 Random 系统注入到 FormulaInput 中。

违反表现：

Formula 内部调用 rand() 生成随机数，导致相同输入产生不同输出。

---

## 不变量3：FormulaRegistry 注册在启动时完成

任意时刻：

FormulaRegistry 的注册操作仅在 App 启动时执行，运行时不修改注册表。

违反表现：

运行时动态注册新 Formula，导致注册表不一致。

---

## 不变量4：未注册的 FormulaId 返回错误

任意时刻：

通过未注册的 FormulaId 查找 Formula 时，必须返回错误或 None，禁止 panic。

违反表现：

未注册的 FormulaId 导致程序崩溃。

---

# 业务规则

## 规则1：Effect 调用 Formula 🟩 8.0.5

禁止：
- Effect 内部硬编码计算逻辑
- Effect 直接读取属性值进行计算
- Effect 跳过 FormulaRegistry 直接调用 Formula

必须：
- Effect 通过 FormulaId 查找 Formula
- Effect 构建 FormulaInput 传递给 Formula
- Effect 使用 FormulaOutput 作为计算结果

允许：
- Effect 根据 FormulaOutput 决定后续执行逻辑

---

## 规则2：Formula 纯函数约束

禁止：
- Formula 内部修改全局状态
- Formula 读取 ECS World
- Formula 产生随机数
- Formula 调用其他 Formula（无递归）

必须：
- Formula.calculate(input) 仅依赖 FormulaInput
- Formula 输出仅通过 FormulaOutput 返回
- Formula 不持有运行时状态

允许：
- Formula 调用辅助计算函数（纯函数）

---

## 规则3：FormulaRegistry 管理

禁止：
- 运行时动态注册 Formula
- 注册重复的 FormulaId
- 注册后修改已注册的 Formula

必须：
- App 启动时通过 register_defaults() 注册所有内置 Formula
- 新增 Formula 需添加 FormulaId 枚举变体和对应实现
- 注册时检查 key 是否存在（防止重复）

允许：
- 测试时临时注册额外 Formula

---

## 规则4：随机性注入 🟩 11.9

禁止：
- Formula 内部生成随机数
- FormulaInput 包含随机数（应在 Modifier 阶段注入）
- Formula 输出包含随机性

必须：
- 随机数由 Random 系统生成
- 随机结果注入到 FormulaInput 的对应字段
- Formula 对随机输入产生确定性输出

允许：
- FormulaInput 包含随机种子（由 Random 系统提供）

---

# 流程管线

## 公式调用管线

```
Effect 执行 → 查找 FormulaId → 构建 FormulaInput → formula.calculate() → 返回 FormulaOutput
```

### Step1：Effect 执行

输入：EffectDef + GenerateContext
处理：从 EffectDef 中提取 FormulaId
输出：FormulaId
禁止：EffectDef 中未指定 FormulaId 时使用默认公式

### Step2：查找 FormulaId

输入：FormulaId + FormulaRegistry
处理：通过 FormulaRegistry.get(id) 查找 Formula 实现
输出：Formula trait object
禁止：未注册的 FormulaId 直接调用 calculate

### Step3：构建 FormulaInput

输入：GenerateContext + 属性值 + 元素类型 + 地形加成
处理：构建 FormulaInput 结构
输出：FormulaInput
禁止：FormulaInput 缺少必需字段

### Step4：formula.calculate()

输入：FormulaInput
处理：调用 Formula.calculate(input) 纯函数计算
输出：FormulaOutput
禁止：在 calculate 内部修改状态或生成随机数

### Step5：返回 FormulaOutput

输入：FormulaOutput
处理：传递给 Effect Handler 的 Execute 阶段
输出：计算结果
禁止：修改 FormulaOutput 的值

---

## 公式注册管线

```
定义新 FormulaId → 实现 Formula trait → 注册到 FormulaRegistry → EffectHandler 通过 FormulaId 调用
```

### Step1：定义新 FormulaId

输入：新的公式需求
处理：在 FormulaId 枚举中添加新变体
输出：新 FormulaId
禁止：复用已有 FormulaId

### Step2：实现 Formula trait

输入：新 FormulaId + 计算逻辑
处理：实现 Formula trait 的 calculate 方法
输出：Formula struct
禁止：calculate 内部包含随机逻辑

### Step3：注册到 FormulaRegistry

输入：新 FormulaId + Formula struct
处理：调用 FormulaRegistry.register(id, formula)
输出：注册完成
禁止：注册重复的 FormulaId

### Step4：EffectHandler 通过 FormulaId 调用

输入：EffectDef 中的 FormulaId
处理：EffectHandler 在 Generate 阶段查找并调用 Formula
输出：FormulaOutput
禁止：EffectHandler 硬编码计算逻辑

---

# 数据结构

## FormulaId（公式标识枚举）

职责：唯一标识一个公式

结构：
- PhysicalDamage — 物理伤害公式
- MagicDamage — 魔法伤害公式
- TrueDamage — 真实伤害公式
- HealFormula — 治疗公式
- PoisonFormula — 中毒公式（基于攻击者属性）
- BurnFormula — 燃烧公式（固定值 + 属性缩放）
- SummonFormula — 召唤物属性公式
- ShieldFormula — 护盾吸收公式
- ExperienceFormula — 经验值公式
- LevelUpFormula — 升级公式

要求：
- 是枚举类型，不可变
- 每个变体对应一个 Formula trait 实现
- 通过 FormulaRegistry 注册

---

## Formula（公式 trait）

职责：定义公式的计算接口

结构：
- calculate(input: FormulaInput) → FormulaOutput

要求：
- 纯函数：无副作用、无随机、无状态
- 相同 FormulaInput 产生相同 FormulaOutput
- 不修改 FormulaInput
- 不访问 ECS World

---

## FormulaRegistry（公式注册表）

职责：存储所有 Formula 实现，通过 FormulaId 查找

结构：
- formulas：FormulaId → Formula trait object 映射

要求：
- App 启动时通过 register_defaults() 注册内置公式
- get(id) 返回 Option<&Formula>
- 未注册的 FormulaId 返回 None（不 panic）
- 运行时不修改

---

## FormulaInput（公式输入）

职责：封装公式计算所需的全部参数

结构：
- source_attrs：攻击者属性（攻击力、属性加成等）
- target_attrs：目标属性（防御力、属性抗性等）
- base_value：基础值（技能伤害系数）
- element：元素类型
- terrain_bonus：地形加成
- stack_count：Buff 层数（用于层数缩放）
- is_critical：是否暴击

要求：
- 纯数据传递，不存储持久状态
- 所有字段在构建时确定
- 不包含随机数（随机由 Random 系统注入）

---

## FormulaOutput（公式输出）

职责：封装公式计算的结果

结构：
- value：计算结果（i32）
- breakdown：计算过程记录（Vec of StepRecord）

要求：
- 纯计算结果，不包含执行逻辑
- breakdown 用于 Debug 面板展示
- 不修改游戏状态

---

# 禁止事项

禁止：Effect 内部硬编码计算逻辑

原因：Effect 应只负责调用 Formula，计算逻辑应在 Formula 中实现

违反后果：同一效果类型在不同 Effect 中有不同计算逻辑，无法统一维护

---

禁止：Formula 内部包含随机逻辑

原因：Formula 必须是纯函数，随机性由 Random 系统注入

违反后果：相同输入产生不同输出，无法复现和调试

---

禁止：运行时动态注册 Formula

原因：FormulaRegistry 在启动时固定，运行时修改导致不一致

违反后果：注册表不一致，Formula 查找失败

---

禁止：Formula 修改 FormulaInput

原因：FormulaInput 是只读输入，修改会破坏纯函数约束

违反后果：调用方的数据被意外修改

---

禁止：未注册的 FormulaId panic

原因：未注册的 FormulaId 应优雅处理，不应崩溃

违反后果：程序崩溃，用户体验差

---

禁止：Formula 调用其他 Formula（递归）

原因：Formula 之间递归调用可能导致无限循环

违反后果：栈溢出，程序崩溃

---

禁止：Formula 访问 ECS World

原因：Formula 是纯函数，不应依赖游戏状态

违反后果：Formula 与游戏状态耦合，无法独立测试

---

# AI 修改规则

## 宪法合规检查清单

修改本领域代码前，必须逐项确认：
- 🟩 1.4.1 公式计算为纯函数，不依赖 ECS World
- 🟩 1.4.2 公式无副作用，不触发事件、不修改全局状态
- 🟩 8.0.5 公式集中在领域层管理，Effect 通过 FormulaId 调用
- 🟩 11.9 公式不包含随机逻辑，随机由 Random 系统注入
- 🟩 11.8 公式支持离线仿真，相同输入产生相同输出

## 领域事件清单

本领域产生的领域事件：公式系统为纯函数计算，不直接产生领域事件。公式计算结果由 Effect Pipeline 在执行阶段产生事件。

> 🟩 2.2.7 领域事件由效果执行阶段统一产生，公式计算阶段不产生事件
> 🟩 13.10.1 所有核心业务事实通过领域事件表达

## 如果新增 FormulaId

允许：
- 在 FormulaId 枚举中新增变体
- 实现 Formula trait 的 calculate 方法
- 在 FormulaRegistry 中注册

禁止：
- 复用已有 FormulaId
- 修改现有 Formula 的计算逻辑
- 在 calculate 内部包含随机逻辑

优先检查：
- FormulaId 是否在 FormulaRegistry 中注册
- calculate 是否为纯函数（无副作用）
- FormulaInput 是否包含所有必需字段

---

## 如果修改 Formula 计算逻辑

允许：
- 调整 calculate 方法内的计算公式
- 添加新的计算步骤（在 breakdown 中记录）

禁止：
- 修改 Formula trait 的接口签名
- 在 calculate 内部访问 ECS World
- 在 calculate 内部生成随机数

优先检查：
- 修改后是否保持纯函数性质
- 相同输入是否仍产生相同输出
- breakdown 是否正确记录计算过程

---

## 如果修改 FormulaRegistry

允许：
- 在 register_defaults() 中添加新 Formula 注册
- 调整注册顺序

禁止：
- 运行时动态注册
- 注册重复的 FormulaId
- 修改已注册 Formula 的实现

优先检查：
- 新 FormulaId 是否与现有 ID 冲突
- 注册后 get(id) 是否返回正确实现
- 未注册的 FormulaId 是否返回 None

---

## 如果修改 FormulaInput

允许：
- 添加新的输入字段
- 调整字段的类型

禁止：
- 移除 source_attrs 或 target_attrs
- 使 FormulaInput 可变
- 在 FormulaInput 中添加随机数字段

优先检查：
- 新字段是否在所有 Formula 中都有意义
- 现有 Formula 的 calculate 是否兼容新字段
- FormulaInput 构建代码是否更新

---

## 如果测试失败

排查顺序：
1. 检查 FormulaId 是否在 FormulaRegistry 中注册
2. 检查 Formula.calculate 是否为纯函数（无副作用、无随机）
3. 检查 FormulaInput 是否包含所有必需字段
4. 检查 FormulaOutput 是否被正确使用
5. 检查 Formula 是否访问了 ECS World

---

# 交叉引用

| 主题 | 详细文档 |
|------|----------|
| Effect Pipeline（Generate → Modify → Execute） | `docs/02-domain/attribute_modifier_rules.md#效果管线` |
| EffectHandler trait 分发 | `docs/02-domain/attribute_modifier_rules.md#效果处理器` |
| Modifier 修饰链 | `docs/02-domain/attribute_modifier_rules.md#修饰规则` |
| Buff 叠层策略 | `docs/02-domain/stack_policy_rules.md` |
| 触发器和上下文 | `docs/02-domain/trigger_rules.md` |
| 技能定义和验证 | `docs/02-domain/skill_rules.md` |
