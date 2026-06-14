# 消耗系统领域

Version: 1.0
Status: Proposed

消耗系统管理技能释放的资源消耗。统一管理所有类型的消耗，不要把 MP 消耗写进技能逻辑。

核心原则：
- 🟩 消耗是技能的组成部分，不是独立的系统
- 🟩 消耗校验在释放前执行，消耗扣除在释放后执行
- 🟩 所有消耗类型通过 CostValidator trait 统一接口
- 🟩 消耗校验为纯函数，不修改游戏状态（宪法 11.7.1 读路径无副作用）
- 🟥 禁止先扣除后校验
- 🟥 禁止消耗校验失败时输出错误日志（这是 RuleFailure，不是 DomainError）（宪法 13.9.2）

---

# 宪法合规声明

本领域遵循以下宪法条款：

| 条款编号 | 条款名称 | 合规状态 | 说明 |
|----------|----------|----------|------|
| 11.2.1 | 技能执行五阶段管线 | 🟩 已合规 | 消耗在目标校验之后、效果执行之前 |
| 11.5.1 | 所有操作入口为命令 | 🟩 已合规 | 消耗通过标准化命令触发 |
| 11.7.1 | 读路径无副作用 | 🟩 已合规 | 消耗校验为纯函数 |
| 11.7.2 | 写路径收口 | 🟩 已合规 | 消耗扣除通过统一流程处理 |
| 13.9.2 | 失败分类学 | 🟩 已合规 | 资源不足为 RuleFailure，配置缺失为 DomainError |
| 1.1.3 | 规则与内容分离 | 🟩 已合规 | CostDef 通过 RON 配置，Validator 实现规则 |
| 1.1.2 | 定义与实例分离 | 🟩 已合规 | CostDef（定义态）与 CostContext（运行态）分离 |
| 2.2.6 | 领域事件是唯一事实源 | 🟩 已合规 | 消耗失败通过 CostError 返回值传递 |
| 18.4.1 | 战斗完全可重现 | 🟩 已合规 | 消耗校验结果确定，支持回放 |

---

# 失败分类学（宪法 13.9.2）

消耗系统严格区分两类失败：

| 失败类型 | 定义 | 消耗系统示例 | 处理方式 |
|----------|------|-------------|----------|
| RuleFailure | 业务规则正常不满足 | MP 不足、HP 不足、弹药不足 | 返回 CostError，不输出错误日志 |
| DomainError | 领域内预期内异常 | CostDef 配置缺失、CostValidator 未注册 | 返回 Result::Err，输出 WARN 日志 |

禁止：
- 🟥 禁止将 RuleFailure（资源不足）作为 DomainError 处理（宪法 13.9.2）
- 🟥 禁止将 DomainError（配置缺失）作为 RuleFailure 处理
- 🟥 禁止使用全局统一的 AppError（宪法 13.9.1）

---

# 四级通信机制（宪法 2.2）

消耗领域在四级通信机制中的定位：

| 通信层级 | 用途 | 消耗领域应用 |
|----------|------|-------------|
| Hook（2.2.1） | 组件生命周期 | 无（纯函数领域，无组件副作用） |
| Trigger（2.2.2） | Feature 内事件链 | 无（消耗检查不产生事件链） |
| Observer（2.2.3） | 局部状态变化响应 | 无（消耗检查不响应状态变化） |
| Message（2.2.4） | 跨域广播 | 无（消耗结果通过函数返回值传递） |

禁止事项（宪法 2.2.5）：
- 🟥 禁止将消耗检查逻辑事件化（纯函数直接调用即可）
- 🟥 禁止为消耗失败单独创建领域事件（通过 CostError 返回值传递）

---

# 术语定义

## 消耗（Cost）

技能释放需要支付的资源，如 MP、HP、怒气等。

不是 Requirement。不是冷却。不是 Effect。

关键属性：
- 定义态为 CostDef（RON 反序列化用），运行态为 CostDef 实例
- 每种 Cost 类型有对应的 CostValidator 进行校验
- 通过 CostRegistry 按类型名查找分发
- 消耗校验和扣除分离：先校验后扣除

---

## 消耗校验（Cost Validation）

释放前检查施法者是否有足够资源支付消耗。

不是消耗扣除。不是 UI 展示。不是 Requirement 检查。

关键属性：
- 纯函数，不修改资源值
- 校验所有 CostDef 列表中的消耗项
- 全部通过时返回 Ok，任一不通过时返回 Err（含失败原因）
- 校验失败时技能不可释放

---

## 消耗扣除（Cost Deduction）

释放后实际扣减施法者资源的操作。

不是校验。不是冷却设置。不是效果执行。

关键属性：
- 必须在消耗校验通过后执行
- 扣除所有 CostDef 列表中的消耗项
- 扣除后资源值不得为负数
- 扣除操作通过 set_vital / modify_resource 接口

---

## 资源池（Resource Pool）

存储消耗资源的实体属性（MP、HP、怒气等）。

不是单个值。不是 Cost。不是修饰器。

关键属性：
- 存储在 Attributes 组件中（current_mp / current_hp 等）
- 通过 set_vital() 设置当前值
- 通过 get() 查询当前值
- 资源池与消耗系统通过接口交互，不直接操作

---

# 领域边界

## 本领域负责

- Cost 类型定义和校验逻辑（CostValidator trait）
- 消耗校验和扣除的分离流程
- Cost 注册表（CostRegistry）
- 资源池的查询接口
- 消耗失败的原因反馈

## 本领域不负责

- 技能释放前的前置检查（由 Requirement 领域负责：HasWeapon / NotSilenced 等）
- 目标选择和范围计算（由 Selector 领域负责）
- 效果的实际执行（由 Effect Pipeline 领域负责：Generate → Modify → Execute）
- 条件效果的判断（由 Condition 领域负责）
- 技能冷却管理（由 Skill 领域负责）
- 属性修饰和计算（由 Attribute Modifier 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 消耗校验结果 | 函数调用（validate） | Skill/Battle 领域 |
| 消耗扣除执行 | 函数调用（deduct） | Battle 领域 |
| 资源池查询 | 函数调用（get / set_vital） | Attribute 领域 |
| 消耗失败原因 | 函数调用（CostError） | UI 领域 |

---

# 生命周期

本领域无状态机，为纯函数式计算。

消耗校验是无状态的纯函数调用：输入 CostDef 列表 + 资源池状态，输出 Ok/Err。消耗扣除是状态修改操作，但不涉及状态机转换。

唯一有状态的是 CostRegistry（Resource），其生命周期为：
- 系统启动时注册所有内置 CostValidator
- 运行时只读查找，不修改

---

# 不变量

## 不变量1：先校验后扣除

任意时刻：

消耗扣除必须在消耗校验全部通过后执行，禁止先扣除后校验或跳过校验直接扣除。

违反表现：

资源不足时仍扣除资源（导致负数）。校验失败后仍执行扣除。

---

## 不变量2：扣除后资源不为负

任意时刻：

消耗扣除操作完成后，施法者的资源值（MP / HP / 怒气等）必须 >= 0。

违反表现：

扣除后 current_mp 为负数。扣除后 current_hp 为负数。

---

## 不变量3：消耗校验为纯函数

任意时刻：

CostValidator::validate() 不修改任何游戏状态，仅读取资源池状态并返回结果。

违反表现：

validate() 内部修改了资源值、添加/移除 Buff、改变了游戏状态。

---

## 不变量4：消耗类型通过注册表分发

任意时刻：

新增消耗类型必须通过实现 CostValidator trait 并注册到 CostRegistry，禁止使用 match 硬编码分发。

违反表现：

在消耗管线中使用 match CostDef::X 来分发校验逻辑。

---

# 业务规则

## 规则1：消耗校验与扣除分离

禁止：
- 校验和扣除在同一函数中执行
- 校验失败后仍执行扣除
- 扣除后重新校验

必须：
- 校验阶段纯函数，不修改状态
- 扣除阶段在确认校验通过后执行
- 校验失败时返回 CostError，不执行扣除

允许：
- 校验和扣除在同一个系统调用中（但逻辑分离）

---

## 规则2：消耗类型注册

禁止：
- 使用 match 硬编码消耗校验分发
- 未注册的消耗类型在校验时静默跳过（应输出 warn 日志）

必须：
- 每种 CostDef 变体对应一个 CostValidator 实现
- 评估器通过 CostRegistry 注册
- type_name 与 CostDef 的类型标识一致

允许：
- 新增消耗类型只需实现 trait + 注册，不修改管线代码

---

## 规则3：多消耗项全部通过

禁止：
- 部分消耗项通过就执行扣除
- 某个消耗项失败时忽略其他项

必须：
- 所有消耗项全部校验通过才执行扣除
- 任一消耗项失败时返回第一个失败原因
- 扣除时按顺序扣除所有消耗项

允许：
- 扣除顺序由 CostDef 列表顺序决定

---

## 规则4：消耗定义数据驱动

禁止：
- 在 Rust 代码中硬编码消耗逻辑
- 技能释放逻辑中直接写 set_vital(current_mp - cost)

必须：
- 消耗定义通过 RON 文件配置（assets/skills/*.ron 的 costs 字段）
- 消耗校验和扣除通过 CostValidator trait 统一处理

允许：
- 内置消耗类型（MpCost / HpCost 等）在验证器中实现

---

# 流程管线

## 消耗管线

```
Skill 释放 → CostValidation（检查所有 costs）→ 全部通过 → CostDeduction（扣除每个 cost）→ 资源池更新
```

### Step1：Skill 释放

输入：施法者 Entity + SkillData（含 costs 列表）
处理：获取技能的消耗定义列表
输出：CostDef 列表
禁止：跳过消耗检查直接释放

### Step2：CostValidation

输入：CostDef 列表 + 施法者资源池状态
处理：逐条调用 CostValidator::validate()，检查资源是否足够
输出：Ok(()) 或 Err(CostError)
禁止：校验失败后继续执行扣除

### Step3：全部通过

输入：所有消耗项的校验结果
处理：AND 逻辑判断——全部为 Ok 时通过
输出：通过 → 进入扣除阶段；不通过 → 返回 CostError
禁止：任一不通过时仍执行扣除

### Step4：CostDeduction

输入：CostDef 列表 + 施法者资源池
处理：逐条调用 CostValidator::deduct()，扣除资源
输出：更新后的资源池状态
禁止：扣除后资源值为负数

### Step5：资源池更新

输入：扣除后的资源池状态
处理：通过 set_vital() 更新施法者属性
输出：资源池更新完成
禁止：跳过资源池更新（扣除不生效）

---

# 数据结构

## CostDef（消耗定义）

职责：定义一个消耗类型的资源和数量（RON 反序列化用）

结构：
- 类型标识：CostDef 枚举变体（MpCost / HpCost / RageCost / ActionPointCost / AmmoCost / DurabilityCost / CurrencyCost / SacrificeCost）
- 参数：根据变体不同而不同（如 MpCost 的 amount 值）

要求：
- 每个变体通过 type_name() 返回消耗类型名
- type_name 与 CostValidator::type_name 一致
- 不包含运行时状态

---

## CostValidator（消耗校验器 trait）

职责：描述如何校验和扣除一种消耗类型的 trait 实现

结构：
- type_name()：返回消耗类型名（与 CostDef::type_name 对应）
- validate(&self, ctx: &CostContext) -> Result<(), CostError>：校验资源是否足够
- deduct(&self, ctx: &mut CostContext)：扣除资源

要求：
- 每种 CostDef 变体实现一个 CostValidator
- 通过 CostRegistry 注册
- validate 为纯函数，deduct 修改资源池

---

## CostRegistry（消耗注册表）

职责：存储所有 CostValidator 实现，通过类型名查找分发

结构：
- validators：类型名到校验器的映射

要求：
- 注册所有内置消耗校验器（8 种）
- O(1) HashMap 查找
- 不重复注册（register 时检查 key 是否存在）

---

## CostContext（消耗上下文）

职责：封装消耗校验和扣除所需的全部输入数据

结构：
- source_entity：施法者 Entity
- source_attrs：施法者属性快照（含资源池）
- cost_def：CostDef（当前消耗定义）

要求：
- 纯数据传递，不存储持久状态
- 校验时只读，扣除时可修改（&mut）

---

## CostError（消耗失败）

职责：标识消耗校验失败的原因

结构：
- InsufficientMp { required, current } — MP 不足
- InsufficientHp { required, current } — HP 不足
- InsufficientRage { required, current } — 怒气不足
- InsufficientActionPoint { required, current } — 行动点不足
- InsufficientAmmo { required, current } — 弹药不足
- InsufficientDurability { required, current } — 耐久不足
- InsufficientCurrency { required, current } — 金币不足
- SacrificeTargetMissing — 献祭目标不存在

要求：
- 携带足够的上下文信息用于 UI 展示
- 由 CostValidator::validate() 返回

---

# 禁止事项

禁止：先扣除后校验

原因：校验是安全检查，先扣除会导致资源不足时仍执行扣除

违反后果：资源值为负数，游戏状态异常

---

禁止：校验失败后仍执行扣除

原因：校验失败意味着资源不足，扣除会破坏资源一致性

违反后果：MP 不足时仍扣除 MP，HP 不足时仍扣除 HP

---

禁止：扣除后资源值为负

原因：资源值不为负是游戏状态的基本不变量

违反后果：负数资源值导致后续校验异常，UI 显示异常

---

禁止：使用 match 硬编码消耗校验分发

原因：match 分发违反 CostValidator trait 扩展原则，新增消耗类型需修改分发代码

违反后果：每次新增消耗类型都要修改核心校验管线，违反开闭原则

---

禁止：消耗校验时修改游戏状态

原因：校验是纯函数，修改状态会导致不可预测的副作用

违反后果：校验操作产生副作用，游戏状态不一致

---

禁止：修改 CostDef 定义态

原因：CostDef 是不可变配置，运行时通过校验器判断

违反后果：全局消耗配置被污染，多场战斗数据不一致

---

禁止：SacrificeCost 不验证目标存在

原因：献祭消耗需要目标存在才能执行，否则语义不完整

违反后果：献祭技能在目标死亡后仍可释放，产生无意义的效果

---

# AI 修改规则

## 如果新增消耗类型

允许：
- 在 CostDef 枚举中添加新变体
- 实现对应的 CostValidator trait
- 注册到 CostRegistry

禁止：
- 修改消耗管线的调度逻辑
- 在 validate / deduct 方法中使用 match 硬编码分发

优先检查：
- CostDef::type_name 与 CostValidator::type_name 是否一致
- CostContext 是否包含新消耗所需的上下文字段
- 新消耗的 RON 反序列化是否兼容旧配置

---

## 如果修改消耗校验逻辑

允许：
- 调整现有 CostValidator 的 validate 实现
- 修改消耗数量的计算逻辑

禁止：
- 改变先校验后扣除的顺序
- 修改 CostValidator trait 的 validate / deduct 方法签名
- 移除现有消耗类型的校验器

优先检查：
- 修改后的校验逻辑是否影响现有技能的消耗结果
- 边界情况处理（如消耗量 = 0 时的行为）
- 校验失败时是否返回正确的 CostError

---

## 如果修改 CostContext

允许：
- 添加新的上下文字段（如新的消耗类型需要的数据）
- 调整 from_query() 的构建逻辑

禁止：
- 移除现有上下文字段（会影响现有校验器）
- 改变 from_query() 的返回类型

优先检查：
- 新字段是否为 Option 类型（向后兼容）
- 所有现有校验器是否兼容新 Context 结构
- 缺失新字段时校验器是否正确处理

---

## 如果测试失败

排查顺序：
1. 检查 CostDef::type_name 与 CostValidator::type_name 是否匹配
2. 检查 CostContext 是否包含校验器所需的全部字段
3. 检查先校验后扣除的顺序是否正确
4. 检查扣除后资源值是否 >= 0
5. 检查多消耗项是否全部校验通过

---

# 宪法禁止事项

以下禁止事项源自 AI 开发宪法，消耗领域必须严格遵守：

## 禁止：将 RuleFailure 作为 DomainError 处理（宪法 13.9.2）

原因：资源不足（MP/HP/弹药不足）是业务规则正常不满足，不是程序错误。

违反后果：日志被大量正常逻辑淹没，真正的错误被掩盖。

---

## 禁止：使用全局统一 AppError（宪法 13.9.1）

原因：消耗领域使用 CostError，不使用全局 AppError 或 anyhow::Error。

违反后果：错误失去领域上下文，无法快速定位问题。

---

## 禁止：消耗检查逻辑事件化（宪法 2.2.5）

原因：消耗检查是纯函数直接调用，无需事件化。结果通过 CostError 返回值传递。

违反后果：过度事件化导致调试困难、性能下降。

---

## 禁止：读路径产生副作用（宪法 11.7.1）

原因：消耗校验为纯读操作，不修改游戏状态。

违反后果：校验过程改变游戏状态、仿真结果不准确。

---

## 禁止：跳过命令层直接执行消耗扣除（宪法 11.5.1）

原因：所有操作入口必须为标准化命令，消耗扣除通过统一管线执行。

违反后果：业务逻辑与消耗实现耦合、无法支持回放。

---

## 禁止：核心领域逻辑直接依赖 Bevy ECS 类型（宪法 1.4.1）

原因：消耗校验应实现为纯函数，参数为 CostContext 数据结构体而非 ECS Query/Entity。

违反后果：无法离线仿真、无法独立测试。

---

## 禁止：为未来需求过度设计消耗系统（宪法 1.1.7）

原因：当前 8 种消耗类型已覆盖所有已知场景，禁止为未明确需求提前设计更多类型。

违反后果：架构复杂度上升、维护成本增加。
