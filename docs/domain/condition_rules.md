# 条件系统领域

Version: 1.0
Status: Proposed

条件系统管理效果的条件触发判断。ConditionalEffect = condition + effect，控制效果是否生效。

核心原则：
- Condition 判断"效果是否生效"，不是判断"技能能不能放"（那是 Requirement 的职责）
- 每个 Condition 通过 ConditionEvaluator 纯函数评估，不修改状态
- 条件全部通过时效果执行，任一不通过时效果静默跳过

---

# 术语定义

## 条件（Condition）

效果执行时的触发条件判断，决定效果是否生效。

不是 Requirement。不是 Effect。不是 Selector。

关键属性：
- 定义态为 ConditionDef（RON 反序列化用），运行态为 ConditionDef 实例
- 每个 Condition 有对应的 ConditionEvaluator 进行评估
- 所有条件全部通过（AND 逻辑）时效果执行
- 评估时需要 ConditionContext 提供上下文数据

---

## 条件效果（ConditionalEffect）

condition + effect 的组合结构，将条件判断与效果绑定。

不是独立 Effect 类型。不是 Modifier。不是 SkillCondition。

关键属性：
- 包含一个 ConditionDef（触发条件）和一个 EffectDef（效果）
- ConditionDef 为 true 时执行 EffectDef，为 false 时静默跳过
- 是 Effect 列表中的包装结构，不是独立的效果执行器
- 技能的 effects 列表中可包含 ConditionalEffect

---

## 条件评估器（ConditionEvaluator）

每个条件类型的纯函数评估单元，接收 ConditionContext 返回 bool。

不是函数指针。不是 enum match。不是系统（System）。

关键属性：
- 实现 ConditionEvaluator trait：evaluate(&self, ctx: &ConditionContext) -> bool
- 通过 ConditionEvaluatorRegistry 按类型名查找分发
- 新增条件类型只需实现 trait 并注册，不修改评估管线
- 纯函数，不修改任何游戏状态

---

## 条件上下文（ConditionContext）

条件评估时所需的全部输入数据，封装施法者和目标的快照信息。

不是 ECS World。不是技能定义。不是全局状态。

关键属性：
- source_entity：施法者 Entity
- target_entity：目标 Entity
- source_attrs / target_attrs：双方属性快照
- source_tags / target_tags：双方标签快照
- damage_dealt：造成的伤害量（用于 IsKill 等后置条件）
- is_critical：是否暴击
- terrain_tags：当前地形标签
- adjacent_ally_count：相邻友军数量
- has_moved：本回合是否已移动

---

# 领域边界

## 本领域负责

- 条件类型的定义和评估逻辑（ConditionEvaluator trait）
- ConditionalEffect 的条件判断流程
- 条件评估器注册表（ConditionEvaluatorRegistry）
- 条件上下文的构建和传递
- 条件评估结果的判定（全部通过 → 执行，任一不通过 → 跳过）

## 本领域不负责

- 技能释放前的前置检查（由 Requirement 领域负责：HasWeapon / NotSilenced 等）
- 技能消耗的校验和扣除（由 Cost 领域负责）
- 目标选择和范围计算（由 Selector 领域负责）
- 效果的实际执行（由 Effect Pipeline 领域负责：Generate → Modify → Execute）
- Buff 的触发时机管理（由 Buff 领域负责）
- 效果管线的修饰和计算（由 Attribute Modifier 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 条件评估结果 | 函数调用（evaluate） | Effect Pipeline 领域 |
| 条件上下文构建 | 函数调用（from_query） | Battle 领域 |
| ConditionalEffect 执行 | 函数调用（Effect Pipeline） | Effect Pipeline 领域 |
| 条件评估器注册 | 注册表操作 | 无（内部管理） |

---

# 生命周期

本领域无状态机，为纯函数式计算。

条件评估是无状态的纯函数调用：输入 ConditionContext，输出 bool。不涉及状态转换、不产生副作用。

唯一有状态的是 ConditionEvaluatorRegistry（Resource），其生命周期为：
- 系统启动时注册所有内置 ConditionEvaluator
- 运行时只读查找，不修改

---

# 不变量

## 不变量1：条件评估为纯函数

任意时刻：

ConditionEvaluator::evaluate() 不修改任何游戏状态，仅读取 ConditionContext 并返回 bool。

违反表现：

evaluate() 内部修改了 Entity 属性、添加/移除 Buff、改变了游戏状态。

---

## 不变量2：条件全部通过才执行效果

任意时刻：

ConditionalEffect 中的所有 ConditionDef 必须全部评估为 true（AND 逻辑），效果才执行。任一条件为 false 时，效果静默跳过，不输出任何日志或错误。

违反表现：

任一条件不满足时仍执行效果。条件为空列表时跳过执行（空条件应视为全部通过）。

---

## 不变量3：条件评估器通过注册表分发

任意时刻：

新增条件类型必须通过实现 ConditionEvaluator trait 并注册到 ConditionEvaluatorRegistry，禁止使用 match 硬编码分发。

违反表现：

在评估管线中使用 match ConditionDef::X 来分发评估逻辑。

---

## 不变量4：ConditionContext 必须完整

任意时刻：

ConditionContext 必须包含当前条件评估所需的全部数据。缺失上下文字段时，评估器应返回 false（保守策略），不 panic。

违反表现：

ConditionContext 缺少 terrain_tags 时 TerrainIs 条件 panic。缺少 damage_dealt 时 IsKill 条件返回错误值。

---

# 业务规则

## 规则1：条件与需求语义分离

禁止：
- 将 Requirement 类型（HasWeapon / NotSilenced）放入 Condition 系统
- 将 Condition 类型（HpBelow / BehindTarget）放入 Requirement 系统

必须：
- Requirement 判断"技能能不能放"（释放前）
- Condition 判断"效果是否生效"（执行时）

允许：
- 同一技能同时包含 Requirement 和 Condition（不同阶段检查）

---

## 规则2：条件评估器注册

禁止：
- 使用 match 硬编码条件评估分发
- 未注册的条件类型在评估管线中静默跳过（应输出 warn 日志）

必须：
- 每种 ConditionDef 变体对应一个 ConditionEvaluator 实现
- 评估器通过 ConditionEvaluatorRegistry 注册
- type_name 与 ConditionDef 的类型标识一致

允许：
- 新增条件类型只需实现 trait + 注册，不修改管线代码

---

## 规则3：条件结果处理

禁止：
- 条件不通过时输出错误日志（这是正常的游戏逻辑，不是错误）
- 条件不通过时中断整个效果列表的遍历

必须：
- 条件不通过时静默跳过该 ConditionalEffect
- 继续遍历效果列表中的下一个 Effect
- 空条件列表视为全部通过（等价于无条件效果）

允许：
- 条件评估结果用于日志记录（Debug 模式下）

---

## 规则4：条件定义数据驱动

禁止：
- 在 Rust 代码中硬编码条件判断逻辑
- 使用 skill_id == "fireball" 等字符串匹配判断条件

必须：
- 条件定义通过 RON 文件配置（assets/skills/*.ron 的 conditions 字段）
- 条件判断基于 GameplayTag 位掩码，不基于字符串

允许：
- 内置条件类型（HpBelow / BehindTarget 等）在评估器中实现

---

# 流程管线

## 条件检查管线

```
Effect 生成 → 遍历 conditions → 逐条 ConditionEvaluator 评估 → 全部通过 → 效果执行
```

### Step1：Effect 生成

输入：SkillData.effects 中的 ConditionalEffect
处理：识别效果列表中的 ConditionalEffect（含 condition 字段）
输出：待评估的 ConditionalEffect 列表
禁止：在 Generate 阶段修改目标属性

### Step2：遍历 conditions

输入：ConditionalEffect 的 conditions 列表
处理：逐条取出 ConditionDef
输出：单个 ConditionDef
禁止：跳过条件评估直接执行效果

### Step3：ConditionEvaluator 评估

输入：ConditionDef + ConditionContext
处理：通过 ConditionEvaluatorRegistry 查找评估器，调用 evaluate()
输出：bool（true = 通过，false = 不通过）
禁止：评估器修改游戏状态（纯函数）

### Step4：全部通过

输入：所有条件的评估结果
处理：AND 逻辑判断——全部为 true 时通过
输出：通过 → 进入效果执行；不通过 → 静默跳过
禁止：任一不通过时仍执行效果

### Step5：效果执行

输入：通过条件的 ConditionalEffect.effect
处理：路由到 Effect Pipeline（Generate → Modify → Execute）
输出：EffectResult
禁止：在条件检查阶段执行效果

---

# 数据结构

## ConditionDef（条件定义）

职责：定义一个条件判断的类型和参数（RON 反序列化用）

结构：
- 类型标识：ConditionDef 枚举变体（HpBelow / BehindTarget / HasBuff / NoBuff / IsCritical / IsKill / TerrainIs / AdjacentAlly / NotMoved / HpAbove）
- 参数：根据变体不同而不同（如 HpBelow 的百分比阈值）

要求：
- 每个变体通过 type_name() 返回条件类型名
- type_name 与 ConditionEvaluator::type_name 一致
- 不包含运行时状态

---

## ConditionalEffect（条件效果）

职责：将条件判断与效果绑定的组合结构

结构：
- conditions：条件列表（Vec）— 所有条件必须全部满足（AND 逻辑）
- effect：EffectDef — 条件满足时执行的效果

要求：
- 是技能 effects 列表中的包装结构
- conditions 为空时视为无条件（等价于直接放置 EffectDef）
- 效果执行必须通过 Effect Pipeline

---

## ConditionEvaluator（条件评估器 trait）

职责：描述如何评估一种条件类型的 trait 实现

结构：
- type_name()：返回条件类型名（与 ConditionDef::type_name 对应）
- evaluate(&self, ctx: &ConditionContext) -> bool：评估条件是否满足

要求：
- 每种 ConditionDef 变体实现一个 ConditionEvaluator
- 通过 ConditionEvaluatorRegistry 注册
- 纯函数，不修改任何游戏状态

---

## ConditionEvaluatorRegistry（条件评估器注册表）

职责：存储所有 ConditionEvaluator 实现，通过类型名查找分发

结构：
- evaluators：类型名到评估器的映射

要求：
- 注册所有内置条件评估器（10 种）
- O(1) HashMap 查找
- 不重复注册（register 时检查 key 是否存在）

---

## ConditionContext（条件上下文）

职责：封装条件评估所需的全部输入数据

结构：
- source_entity：施法者 Entity
- target_entity：目标 Entity
- source_attrs / target_attrs：双方属性快照
- source_tags / target_tags：双方标签快照
- damage_dealt：造成的伤害量（可选，用于 IsKill 等后置条件）
- is_critical：是否暴击
- terrain_tags：当前地形标签
- adjacent_ally_count：相邻友军数量
- has_moved：本回合是否已移动

要求：
- 纯数据传递，不存储持久状态
- 通过 from_query() 从 ECS 查询构建
- 克隆属性和标签数据，避免借用冲突
- 缺失字段时评估器应返回 false（保守策略）

---

# 禁止事项

禁止：ConditionEvaluator 评估时修改游戏状态

原因：条件评估必须是纯函数，修改状态会导致不可预测的副作用

违反后果：条件评估产生副作用，游戏状态不一致，调试困难

---

禁止：将 Requirement 类型放入 Condition 系统

原因：Requirement 和 Condition 有明确的语义边界——"能不能放" vs "是否生效"

违反后果：条件检查时机混乱，释放前/执行时的边界模糊

---

禁止：使用 match 硬编码条件评估分发

原因：match 分发违反 ConditionEvaluator trait 扩展原则，新增条件类型需修改分发代码

违反后果：每次新增条件类型都要修改核心评估管线，违反开闭原则

---

禁止：条件不通过时输出错误日志

原因：条件不通过是正常的游戏逻辑（如处决需要目标 HP<30%），不是错误

违反后果：日志被大量正常逻辑淹没，真正的错误被掩盖

---

禁止：条件不通过时中断效果列表遍历

原因：一个 ConditionalEffect 不通过不影响后续效果的执行

违反后果：后续无条件效果或通过条件的效果被错误跳过

---

禁止：修改 ConditionDef 定义态

原因：ConditionDef 是不可变配置，运行时通过评估器判断

违反后果：全局条件配置被污染，多场战斗数据不一致

---

禁止：评估器未注册时静默跳过

原因：未注册的评估器意味着评估逻辑缺失，跳过会导致效果意外执行或不执行

违反后果：游戏行为不确定，难以调试

---

# AI 修改规则

## 如果新增条件类型

允许：
- 在 ConditionDef 枚举中添加新变体
- 实现对应的 ConditionEvaluator trait
- 注册到 ConditionEvaluatorRegistry

禁止：
- 修改评估管线的调度逻辑
- 在 evaluate 方法中使用 match 硬编码分发

优先检查：
- ConditionDef::type_name 与 ConditionEvaluator::type_name 是否一致
- ConditionContext 是否包含新条件所需的上下文字段
- 新条件的 RON 反序列化是否兼容旧配置

---

## 如果修改条件评估逻辑

允许：
- 调整现有 ConditionEvaluator 的 evaluate 实现
- 修改条件参数（如 HpBelow 的阈值范围）

禁止：
- 改变 AND 逻辑（全部通过才执行）
- 修改 ConditionEvaluator trait 的 evaluate 方法签名
- 移除现有条件类型的评估器

优先检查：
- 修改后的评估逻辑是否影响现有技能的条件判断结果
- 边界情况处理（如 HpBelow(0%) 和 HpBelow(100%) 的边界值）
- 条件评估结果是否与预期一致

---

## 如果修改 ConditionContext

允许：
- 添加新的上下文字段（如新的条件类型需要的数据）
- 调整 from_query() 的构建逻辑

禁止：
- 移除现有上下文字段（会影响现有评估器）
- 改变 from_query() 的返回类型

优先检查：
- 新字段是否为 Option 类型（向后兼容）
- 所有现有评估器是否兼容新 Context 结构
- 缺失新字段时评估器是否返回 false（保守策略）

---

## 如果测试失败

排查顺序：
1. 检查 ConditionDef::type_name 与 ConditionEvaluator::type_name 是否匹配
2. 检查 ConditionContext 是否包含评估器所需的全部字段
3. 检查 AND 逻辑是否正确（全部通过才执行）
4. 检查评估器是否修改了游戏状态（应为纯函数）
5. 检查未注册的条件类型是否输出 warn 日志
