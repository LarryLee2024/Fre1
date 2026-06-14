# 释放前提系统领域

Version: 1.0
Status: Proposed

释放前提系统管理技能释放前必须满足的条件。Requirement 判断"技能能不能放"，Condition 判断"效果是否生效"。

核心原则：
- 🟩 Requirement 判断"技能能不能放"（释放前），不是判断"效果是否生效"（那是 Condition 的职责）
- 🟩 任一 Requirement 不满足时技能不可用，UI 显示灰显
- 🟩 Requirement 检查在 Cost 检查之前执行（宪法 11.2.1 五阶段管线）
- 🟩 Requirement 检查为纯函数，不修改游戏状态（宪法 11.7.1 读路径无副作用）
- 🟥 禁止将 Condition 类型放入 Requirement 系统
- 🟥 禁止使用 match 硬编码前提检查分发

---

# 宪法合规声明

本领域遵循以下宪法条款：

| 条款编号 | 条款名称 | 合规状态 | 说明 |
|----------|----------|----------|------|
| 11.2.1 | 技能执行五阶段管线 | 🟩 已合规 | Requirement 在目标校验之前执行 |
| 11.5.1 | 所有操作入口为命令 | 🟩 已合规 | 技能释放通过标准化命令触发 |
| 11.7.1 | 读路径无副作用 | 🟩 已合规 | 前提检查为纯函数，不修改状态 |
| 1.1.3 | 规则与内容分离 | 🟩 已合规 | RequirementDef 通过 RON 配置，Checker 实现规则 |
| 1.1.2 | 定义与实例分离 | 🟩 已合规 | RequirementDef（定义态）与检查结果（运行态）分离 |
| 2.2.6 | 领域事件是唯一事实源 | 🟩 已合规 | 前提失败通过 RequirementError 反馈 |
| 18.4.1 | 战斗完全可重现 | 🟩 已合规 | 前提检查结果确定，支持回放 |

---

# 四级通信机制（宪法 2.2）

释放前提领域在四级通信机制中的定位：

| 通信层级 | 用途 | 释放前提领域应用 |
|----------|------|-------------|
| Hook（2.2.1） | 组件生命周期 | 无（纯函数领域，无组件副作用） |
| Trigger（2.2.2） | Feature 内事件链 | 无（前提检查不产生事件链） |
| Observer（2.2.3） | 局部状态变化响应 | 无（前提检查不响应状态变化） |
| Message（2.2.4） | 跨域广播 | 无（前提结果通过函数返回值传递） |

禁止事项（宪法 2.2.5）：
- 🟥 禁止将前提检查逻辑事件化（纯函数直接调用即可）
- 🟥 禁止为前提失败单独创建领域事件（通过 RequirementError 返回值传递）

---

# 术语定义

## 释放前提（Requirement）

技能释放前必须满足的条件，决定技能是否可用。

不是 Condition。不是 Cost。不是 Skill。

关键属性：
- 定义态为 RequirementDef（RON 反序列化用），运行态为 RequirementDef 实例
- 每种 Requirement 类型有对应的 RequirementChecker 进行校验
- 通过 RequirementRegistry 按类型名查找分发
- 所有 Requirement 全部满足（AND 逻辑）时技能可用

---

## 前提检查（Requirement Check）

释放前逐条验证 Requirement 是否满足的过程。

不是消耗检查。不是效果检查。不是条件评估。

关键属性：
- 纯函数，不修改游戏状态
- 校验所有 RequirementDef 列表中的前提项
- 全部通过时返回 Ok，任一不通过时返回 Err（含失败原因）
- 检查失败时技能按钮灰显，不进入 Cost/Effect 阶段

---

## 前提失败（Requirement Failure）

Requirement 不满足时的处理方式，返回失败原因用于 UI 展示。

不是异常。不是 Error。不是 Panic。

关键属性：
- 返回 RequirementError 枚举（含失败原因）
- 携带足够的上下文信息用于 UI 展示
- 由 RequirementChecker::check() 返回
- 失败时技能按钮灰显，不中断游戏流程

---

# 领域边界

## 本领域负责

- Requirement 类型定义和校验逻辑（RequirementChecker trait）
- 前提检查流程（逐条验证 → 全部通过/任一失败）
- Requirement 注册表（RequirementRegistry）
- 前提失败的原因反馈（RequirementError）
- 与 Condition 系统的语义边界维护

## 本领域不负责

- 技能消耗的校验和扣除（由 Cost 领域负责）
- 目标选择和范围计算（由 Selector 领域负责）
- 效果的实际执行（由 Effect Pipeline 领域负责：Generate → Modify → Execute）
- 条件效果的判断（由 Condition 领域负责）
- 技能冷却管理（由 Skill 领域负责：CooldownReady 由 Skill 领域实现）
- 战场地图数据和寻路（由 Map 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 前提检查结果 | 函数调用（check） | Skill/Battle 领域 |
| 前提失败原因 | 函数调用（RequirementError） | UI 领域 |
| 资源池查询 | 函数调用（get） | Attribute 领域 |
| 标签查询 | 函数调用（has / has_all） | Character 领域 |
| 地图数据查询 | 函数调用（is_visible / line_of_sight） | Map 领域 |

---

# 生命周期

本领域无状态机，为纯函数式计算。

前提检查是无状态的纯函数调用：输入 RequirementDef 列表 + 上下文数据，输出 Ok/Err。不涉及状态转换。

唯一有状态的是 RequirementRegistry（Resource），其生命周期为：
- 系统启动时注册所有内置 RequirementChecker
- 运行时只读查找，不修改

---

# 不变量

## 不变量1：前提检查为纯函数

任意时刻：

RequirementChecker::check() 不修改任何游戏状态，仅读取上下文数据并返回结果。

违反表现：

check() 内部修改了 Entity 属性、添加/移除 Buff、改变了游戏状态。

---

## 不变量2：任一前提不满足则技能不可用

任意时刻：

RequirementDef 列表中的所有前提必须全部满足（AND 逻辑），技能才可用。任一前提不满足时，技能按钮灰显，不进入 Cost/Effect 阶段。

违反表现：

某个前提不满足时仍允许释放技能。跳过前提检查直接进入 Cost 阶段。

---

## 不变量3：前提检查在消耗检查之前

任意时刻：

技能释放管线中，Requirement 检查必须在 Cost 检查之前执行。Cost 检查在 Selector 之前执行。

违反表现：

先检查 Cost 再检查 Requirement，导致资源不足的错误信息掩盖了前提不满足的真实原因。

---

## 不变量4：前提类型通过注册表分发

任意时刻：

新增前提类型必须通过实现 RequirementChecker trait 并注册到 RequirementRegistry，禁止使用 match 硬编码分发。

违反表现：

在前提检查管线中使用 match RequirementDef::X 来分发检查逻辑。

---

# 业务规则

## 规则1：Requirement 与 Condition 语义分离

禁止：
- 将 Condition 类型（HpBelow / BehindTarget）放入 Requirement 系统
- 将 Requirement 类型（HasWeapon / NotSilenced）放入 Condition 系统

必须：
- Requirement 判断"技能能不能放"（释放前）
- Condition 判断"效果是否生效"（执行时）

允许：
- 同一技能同时包含 Requirement 和 Condition（不同阶段检查）

---

## 规则2：前提检查注册

禁止：
- 使用 match 硬编码前提检查分发
- 未注册的前提类型在检查时静默跳过（应输出 warn 日志）

必须：
- 每种 RequirementDef 变体对应一个 RequirementChecker 实现
- 检查器通过 RequirementRegistry 注册
- type_name 与 RequirementDef 的类型标识一致

允许：
- 新增前提类型只需实现 trait + 注册，不修改管线代码

---

## 规则3：前提失败处理

禁止：
- 前提失败时输出错误日志（这是正常的游戏逻辑，不是错误）
- 前提失败时中断游戏流程

必须：
- 前提失败时返回 RequirementError
- 技能按钮灰显（UI 状态更新）
- 不进入 Cost/Effect 阶段

允许：
- 前提检查结果用于日志记录（Debug 模式下）

---

## 规则4：前提定义数据驱动

禁止：
- 在 Rust 代码中硬编码前提判断逻辑
- 使用 skill_id == "fireball" 等字符串匹配判断前提

必须：
- 前提定义通过 RON 文件配置（assets/skills/*.ron 的 requirements 字段）
- 前提判断基于 GameplayTag 位掩码，不基于字符串

允许：
- 内置前提类型（HasWeapon / NotSilenced 等）在检查器中实现

---

# 流程管线

## 前提检查管线

```
技能选择 → Requirement check → 全部通过 → Cost check → Selector → Effect Pipeline
```

### Step1：技能选择

输入：用户选择的 SkillData
处理：获取技能的前提定义列表
输出：RequirementDef 列表
禁止：跳过前提检查直接进入 Cost 阶段

### Step2：Requirement check

输入：RequirementDef 列表 + 上下文数据
处理：逐条调用 RequirementChecker::check()，验证前提是否满足
输出：Ok(()) 或 Err(RequirementError)
禁止：校验失败后继续执行 Cost 检查

### Step3：全部通过

输入：所有前提的检查结果
处理：AND 逻辑判断——全部为 Ok 时通过
输出：通过 → 进入 Cost 检查；不通过 → 返回 RequirementError
禁止：任一不通过时仍进入 Cost 阶段

### Step4：Cost check

输入：通过前提检查的技能 + CostDef 列表
处理：调用 CostValidator::validate()，检查资源是否足够
输出：Ok(()) 或 Err(CostError)
禁止：在前提检查失败时执行 Cost 检查

### Step5：Selector

输入：通过 Cost 检查的技能 + SelectorDef
处理：调用 TargetSelector::resolve_targets()，解析目标实体列表
输出：目标实体列表
禁止：在 Cost 检查失败时执行目标解析

### Step6：Effect Pipeline

输入：通过所有检查的技能 + 目标实体列表
处理：路由到 Effect Pipeline（Generate → Modify → Execute）
输出：EffectResult
禁止：在任何检查失败时执行效果

---

## 失败处理管线

```
任一 Requirement 不满足 → 技能按钮灰显（UI）→ 不进入 Cost/Effect 阶段
```

### Step1：检测失败

输入：RequirementError
处理：识别失败原因类型
输出：失败原因 + 上下文信息
禁止：忽略失败原因

### Step2：UI 状态更新

输入：失败原因 + 技能 ID
处理：更新技能按钮状态为灰显
输出：UI 重绘技能面板
禁止：跳过 UI 更新（按钮仍可点击）

### Step3：中断管线

输入：失败状态
处理：不进入 Cost/Effect 阶段
输出：技能释放流程中断
禁止：在失败后继续执行后续阶段

---

# 数据结构

## RequirementDef（前提定义）

职责：定义一个前提检查的类型和参数（RON 反序列化用）

结构：
- 类型标识：RequirementDef 枚举变体（HasWeapon / NotSilenced / TargetExists / MpAbove / HasAmmo / IsStanding / NotStunned / HasLineOfSight / CooldownReady）
- 参数：根据变体不同而不同（如 MpAbove 的阈值）

要求：
- 每个变体通过 type_name() 返回前提类型名
- type_name 与 RequirementChecker::type_name 一致
- 不包含运行时状态

---

## RequirementChecker（前提检查器 trait）

职责：描述如何检查一种前提类型的 trait 实现

结构：
- type_name()：返回前提类型名（与 RequirementDef::type_name 对应）
- check(&self, ctx: &RequirementContext) -> Result<(), RequirementError>：检查前提是否满足

要求：
- 每种 RequirementDef 变体实现一个 RequirementChecker
- 通过 RequirementRegistry 注册
- 纯函数，不修改任何游戏状态

---

## RequirementRegistry（前提检查器注册表）

职责：存储所有 RequirementChecker 实现，通过类型名查找分发

结构：
- checkers：类型名到检查器的映射

要求：
- 注册所有内置前提检查器（9 种）
- O(1) HashMap 查找
- 不重复注册（register 时检查 key 是否存在）

---

## RequirementContext（前提检查上下文）

职责：封装前提检查所需的全部输入数据

结构：
- source_entity：施法者 Entity
- target_entity：目标 Entity（可选，TargetExists 需要）
- source_attrs：施法者属性快照
- source_tags：施法者标签快照
- target_tags：目标标签快照（可选）
- source_position：施法者坐标
- target_position：目标坐标（可选）
- skill_id：技能 ID（CooldownReady 需要）
- cooldowns：冷却状态快照（CooldownReady 需要）
- grid：战场网格数据（HasLineOfSight 需要）

要求：
- 纯数据传递，不存储持久状态
- 通过 from_query() 从 ECS 查询构建
- 克隆属性和标签数据，避免借用冲突

---

## RequirementError（前提失败）

职责：标识前提检查失败的原因

结构：
- MissingWeapon { required_tag } — 缺少指定武器类型
- Silenced — 被沉默
- TargetMissing — 目标不存在
- MpBelowThreshold { required, current } — MP 低于阈值
- NoAmmo — 无弹药
- NotStanding — 未站立（倒地状态）
- Stunned — 被眩晕
- NoLineOfSight — 无视线
- OnCooldown { remaining } — 冷却中

要求：
- 携带足够的上下文信息用于 UI 展示
- 由 RequirementChecker::check() 返回

---

# 禁止事项

禁止：Requirement 不满足时仍允许释放技能

原因：前提检查是技能释放的安全门，绕过会导致游戏逻辑异常

违反后果：沉默状态下释放技能、无武器时使用需要武器的技能

---

禁止：将 Condition 类型放入 Requirement 系统

原因：Requirement 和 Condition 有明确的语义边界——"能不能放" vs "是否生效"

违反后果：检查时机混乱，释放前/执行时的边界模糊

---

禁止：使用 match 硬编码前提检查分发

原因：match 分发违反 RequirementChecker trait 扩展原则，新增前提类型需修改分发代码

违反后果：每次新增前提类型都要修改核心检查管线，违反开闭原则

---

禁止：前提检查时修改游戏状态

原因：检查是纯函数，修改状态会导致不可预测的副作用

违反后果：检查操作产生副作用，游戏状态不一致

---

禁止：前提失败时输出错误日志

原因：前提失败是正常的游戏逻辑（如冷却中、被沉默），不是错误

违反后果：日志被大量正常逻辑淹没，真正的错误被掩盖

---

禁止：修改 RequirementDef 定义态

原因：RequirementDef 是不可变配置，运行时通过检查器判断

违反后果：全局前提配置被污染，多场战斗数据不一致

---

禁止：前提检查在 Cost 检查之后执行

原因：检查顺序影响错误信息的准确性，Requirement 必须最先检查

违反后果：资源不足的错误信息掩盖了前提不满足的真实原因

---

禁止：CooldownReady 检查绕过 Skill 领域

原因：冷却管理由 Skill 领域负责，Requirement 系统只调用查询接口

违反后果：冷却状态不一致，多个系统同时修改冷却数据

---

# AI 修改规则

## 如果新增前提类型

允许：
- 在 RequirementDef 枚举中添加新变体
- 实现对应的 RequirementChecker trait
- 注册到 RequirementRegistry

禁止：
- 修改前提检查管线的调度逻辑
- 在 check 方法中使用 match 硬编码分发

优先检查：
- RequirementDef::type_name 与 RequirementChecker::type_name 是否一致
- RequirementContext 是否包含新前提所需的上下文字段
- 新前提的 RON 反序列化是否兼容旧配置

---

## 如果修改前提检查逻辑

允许：
- 调整现有 RequirementChecker 的 check 实现
- 修改前提参数（如 MpAbove 的阈值范围）

禁止：
- 改变 AND 逻辑（全部通过才可用）
- 修改 RequirementChecker trait 的 check 方法签名
- 移除现有前提类型的检查器

优先检查：
- 修改后的检查逻辑是否影响现有技能的前提判断结果
- 边界情况处理（如 MpAbove(0) 的边界值）
- 检查结果是否与预期一致

---

## 如果修改 RequirementContext

允许：
- 添加新的上下文字段（如新的前提类型需要的数据）
- 调整 from_query() 的构建逻辑

禁止：
- 移除现有上下文字段（会影响现有检查器）
- 改变 from_query() 的返回类型

优先检查：
- 新字段是否为 Option 类型（向后兼容）
- 所有现有检查器是否兼容新 Context 结构
- 缺失新字段时检查器是否正确处理

---

## 如果测试失败

排查顺序：
1. 检查 RequirementDef::type_name 与 RequirementChecker::type_name 是否匹配
2. 检查 RequirementContext 是否包含检查器所需的全部字段
3. 检查 AND 逻辑是否正确（全部通过才可用）
4. 检查检查顺序是否正确（Requirement 在 Cost 之前）
5. 检查前提失败时是否返回正确的 RequirementError

---

# 宪法禁止事项

以下禁止事项源自 AI 开发宪法，释放前提领域必须严格遵守：

## 禁止：前提检查时修改游戏状态（宪法 11.7.1）

原因：检查是纯函数，修改状态会导致不可预测的副作用。

违反后果：检查操作产生副作用，游戏状态不一致、仿真失败。

---

## 禁止：将前提检查逻辑事件化（宪法 2.2.5）

原因：前提检查是纯函数直接调用，无需事件化。结果通过返回值传递。

违反后果：过度事件化导致调试困难、性能下降。

---

## 禁止：为前提失败单独创建领域事件（宪法 2.2.7）

原因：前提失败是正常的游戏逻辑（冷却中、被沉默），不是业务事实事件。通过 RequirementError 返回值传递。

违反后果：领域事件泛滥，正常逻辑被误认为业务事件。

---

## 禁止：读路径产生副作用（宪法 11.7.1）

原因：前提检查为纯读操作，不修改游戏状态。

违反后果：检查过程改变游戏状态、仿真结果不准确。

---

## 禁止：核心领域逻辑直接依赖 Bevy ECS 类型（宪法 1.4.1）

原因：前提检查应实现为纯函数，参数为 RequirementContext 数据结构体而非 ECS Query/Entity。

违反后果：无法离线仿真、无法独立测试。

---

## 禁止：为未来需求过度设计前提系统（宪法 1.1.7）

原因：当前 9 种前提类型已覆盖所有已知场景，禁止为未明确需求提前设计更多类型。

违反后果：架构复杂度上升、维护成本增加。
