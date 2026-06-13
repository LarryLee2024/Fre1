# 全局校验与合法性守卫领域

Version: 1.0
Status: Proposed

全局校验与合法性守卫领域管理游戏状态的全局校验框架、不变量约定和违规处理策略，是防止数据腐败的最后一道防线。

核心原则：
- 校验是只读的，禁止修改游戏状态
- 违规处理三级策略：Reject / Clamp / Panic
- Release 构建中关键不变量校验必须保留

---

# 术语定义

## 全局校验（Global Validation）

状态变更后的合法性检查，确保游戏状态始终满足不变量条件。

不是业务规则。不是输入校验。

关键属性：
- 在特定检查点触发（TurnEnd、BattleEnd、状态转换等）
- 检查内容：属性范围、伤害非负、Buff 数量限制、单位位置合法
- 校验函数为纯函数，以 validate_ 为前缀
- 校验失败时根据违规类型选择处理策略

---

## 校验检查点（Validation Checkpoint）

执行全局校验的时机点，在状态变更到达时触发。

不是任意时刻。不是每帧。

关键属性：
- 回合结束（OnExit(TurnPhase::TurnEnd)）：所有单位状态合法性
- 战斗结束（OnExit(AppState::InGame)）：全局不变量
- 状态转换（任何 OnExit(AppState::*)）：状态一致性
- 关卡加载后：配置数据合法性
- 存档加载后：数据完整性
- MOD 加载后：MOD 数据合法性

---

## 违规处理策略（Violation Handling）

校验失败时的三种处理方式：Reject（拒绝变更）/ Clamp（修正到合法值）/ Panic（立即中断）。

不是静默忽略。

关键属性：
- Reject：阻止状态变更，记录 ERROR 日志，保持游戏状态不变
- Clamp：自动修正到合法值，记录 WARN 日志，继续执行
- Panic：立即终止游戏，生成崩溃报告，记录 error! + 堆栈
- 策略选择取决于违规类型和严重程度

---

## 全局不变量（Global Invariant）

任何时刻必须成立的跨领域条件，违反时触发校验。

不是单领域规则。不是建议。

关键属性：
- HP 范围：0 ≤ current_hp ≤ max_hp（TurnEnd 检查）
- 伤害非负：damage ≥ 0（每次伤害计算，Panic）
- Buff 数量：buff_count ≤ MAX_BUFFS_PER_UNIT（每次施加 Buff，Reject）
- Modifier 来源：每个 Modifier 必须有来源（Panic）
- 单位位置：在地图边界内（每次移动，Clamp）

---

## MOD 校验（MOD Validation）

MOD 内容加载时的安全校验，确保 MOD 数据合法且不破坏游戏状态。

不是游戏内校验。不是代码审查。

关键属性：
- ID 唯一性：MOD 的 ID 不能与已有 ID 冲突（Reject）
- 无循环引用：技能引用的 Buff 不能反向引用技能（Reject）
- 无权限升级：MOD 不能访问非 MOD API 的能力（Reject + 卸载 MOD）
- 数值合法：伤害、冷却等数值在合理范围内（Clamp）
- 格式合规：RON 文件符合 schema 定义（Reject）

---

# 领域边界

## 本领域负责

- 全局校验框架（Validator trait、ValidationResult）
- 校验检查点定义和触发时机
- 违规处理策略（Reject / Clamp / Panic）
- 全局不变量的定义和维护
- MOD 内容加载校验
- Debug 构建增强校验
- 校验失败复现支持（审计日志、状态快照）

## 本领域不负责

- 业务规则的校验（由各功能领域负责）
- 输入校验（由 Input / UI 领域负责）
- Effect Pipeline 的校验（由 Attribute Modifier 领域负责）
- 战斗数值的具体计算（由 Attribute Modifier 领域负责）
- 回合状态机的校验（由 Turn 领域负责）
- 审计日志的存储（由 Infrastructure 层负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 校验失败通知 | Message | UI / Debug 领域 |
| Clamp 修正记录 | Message | Debug 领域 |
| Panic 崩溃报告 | 函数调用 | Infrastructure 层 |
| 校验结果 | 函数调用返回 | 调用方领域 |

---

# 生命周期

本领域无状态机，为纯函数式校验。

校验的生命周期为：检查点触发 → 执行校验 → 返回结果 → 处理违规。

校验结果的生命周期为：Valid / Clamp / Reject / Panic → 记录日志 → 通知 UI。

---

# 不变量

## 不变量1：0 ≤ current_hp ≤ max_hp

回合结算阶段：

所有存活单位的 current_hp 必须在 [0, max_hp] 范围内。

违反表现：

current_hp 为负数（单位已死但未标记 Dead）、current_hp 超过 max_hp（治疗溢出）。

---

## 不变量2：damage ≥ 0

每次伤害计算完成后：

伤害值必须 ≥ 0。

违反表现：

伤害值为负数，导致治疗而非伤害。

---

## 不变量3：buff_count ≤ MAX_BUFFS_PER_UNIT

每次施加 Buff 时：

目标单位的 Buff 数量不能超过上限。

违反表现：

Buff 数量无限增长，内存泄漏，战斗逻辑异常。

---

## 不变量4：每个 Modifier 必须有来源

回合结算阶段：

所有活跃的 Modifier 必须携带有效的 ModifierSource。

违反表现：

孤立 Modifier 无法被清理，属性值异常。

---

## 不变量5：单位位置在地图边界内

每次移动后：

单位的 GridPosition 必须在地图边界内。

违反表现：

单位移动到地图外，寻路失败，渲染异常。

---

# 业务规则

## 规则1：校验检查点

必须：
- 回合结束时执行所有单位状态合法性检查
- 战斗结束时执行全局不变量检查
- 状态转换时执行状态一致性检查
- 关卡加载后执行配置数据合法性检查
- 存档加载后执行数据完整性检查
- MOD 加载后执行 MOD 数据合法性检查

禁止：
- 在非检查点时刻执行全局校验（影响性能）
- 跳过任何检查点的校验
- 在校验中修改游戏状态

允许：
- Debug 构建中在每次属性修改后增强校验

---

## 规则2：违规处理策略

必须：
- Reject 时记录 ERROR 级日志（违规类型、当前值、期望值、Entity ID）
- Clamp 时记录 WARN 级日志
- Panic 时使用 error! 宏 + backtrace
- 每种违规类型有明确的处理策略

禁止：
- 校验失败时静默忽略
- Panic 时但不生成崩溃报告
- Clamp 修正破坏游戏逻辑

允许：
- 根据违规类型选择 Reject / Clamp / Panic

---

## 规则3：校验函数规范

必须：
- 校验函数以 validate_ 为前缀
- 校验函数为纯函数（只读不写）
- 校验函数返回 Result<(), ValidationError>

禁止：
- 校验函数修改游戏状态
- 校验逻辑包含业务规则
- 手动检查代替自动校验

允许：
- 校验函数调用其他纯函数
- 校验函数读取 World 状态

---

## 规则4：MOD 校验

必须：
- MOD 内容加载时执行 Schema 校验
- 校验 ID 唯一性
- 校验引用完整性
- 校验数值合法性
- 校验权限范围

禁止：
- MOD 内容跳过校验
- MOD 访问非 MOD API 的能力
- MOD 引用循环

允许：
- MOD 数值超出默认范围时 Clamp 到合理值

---

## 规则5：Release / Debug 构建差异

必须：
- Release 构建：关键不变量（damage ≥ 0、HP 范围）始终强制执行
- Debug 构建：每次属性修改和状态转换后增强校验
- Debug 构建记录所有 Clamp/Reject 操作

禁止：
- Release 构建中跳过所有校验
- Debug 构建中不记录校验失败

允许：
- Release 构建中可选择性关闭非关键校验（性能优化）
- Debug 构建中启用额外校验

---

# 流程管线

## 校验执行管线（Validation Execution Pipeline）

```
系统执行 → 状态变更 → 到达检查点 → 执行全局校验 → 处理结果
```

### Step1：状态变更

输入：系统执行产生的状态变更
处理：状态变更正常执行
输出：新状态
禁止：在校验完成前阻止状态变更（除 Reject 策略）

### Step2：到达检查点

输入：状态变更完成
处理：检测是否到达校验检查点
输出：触发校验 或 继续执行
禁止：在非检查点时刻触发全局校验

### Step3：执行全局校验

输入：当前游戏状态
处理：遍历所有不变量检查
输出：ValidationResult（Valid / Clamp / Reject / Panic）
禁止：在校验中修改游戏状态

### Step4：处理结果

输入：ValidationResult
处理：
- Valid → 继续
- Clamp → 修正到合法值 + WARN 日志
- Reject → 拒绝变更 + ERROR 日志
- Panic → 崩溃报告 + 终止游戏
禁止：静默忽略校验失败

---

## MOD 校验管线（MOD Validation Pipeline）

```
MOD 文件加载 → Schema 校验 → ID 唯一性 → 引用完整性 → 数值合法性 → 权限范围 → 注册/拒绝
```

### Step1：Schema 校验

输入：MOD RON 文件
处理：检查格式是否符合 schema 定义
输出：格式合法 或 Reject
禁止：跳过 Schema 校验直接加载

### Step2：ID 唯一性

输入：MOD 内容中的 ID 列表
处理：检查是否与已有 ID 冲突
输出：ID 唯一 或 Reject
禁止：允许 ID 冲突

### Step3：引用完整性

输入：MOD 内容中的引用关系
处理：检查是否有循环引用
输出：引用完整 或 Reject
禁止：允许循环引用

### Step4：数值合法性

输入：MOD 内容中的数值字段
处理：检查是否在合理范围内
输出：数值合法 或 Clamp 到默认范围
禁止：允许极端数值

### Step5：权限范围

输入：MOD 内容的 API 访问列表
处理：检查是否访问非 MOD API 的能力
输出：权限合法 或 Reject + 卸载 MOD
禁止：允许权限升级

---

# 数据结构

## ValidationResult（校验结果）

职责：标识校验的最终状态

结构：
- Valid：校验通过
- Clamp { field, from, to }：修正到合法值
- Reject { field, reason }：拒绝变更
- Panic { field, reason }：立即崩溃

要求：
- 每种结果携带足够的调试信息
- Clamp 必须记录原始值和修正值
- Reject 必须记录拒绝原因
- Panic 必须记录 field 和 reason

---

## ValidationError（校验错误类型）

职责：描述校验失败的具体原因

结构：
- HpOutOfBounds：HP 超出范围
- NegativeDamage：伤害为负
- BuffLimitExceeded：Buff 数量超限
- OrphanModifier：Modifier 无来源
- PositionOutOfBounds：单位位置越界
- StateMachineViolation：状态机非法转换
- ConfigInvalid：配置数据非法

要求：
- 每个错误类型携带相关参数
- 错误信息足够定位问题

---

## Validator（校验器 trait）

职责：定义校验接口

结构：
- validate()：执行校验，返回 ValidationResult
- validate_ 前缀命名

要求：
- 纯函数，只读不写
- 返回 Result<(), ValidationError>
- 不修改游戏状态

---

## ValidationCheckpoint（校验检查点）

职责：定义校验触发时机

结构：
- TurnEnd：回合结束
- BattleEnd：战斗结束
- StateTransition：状态转换
- LevelLoad：关卡加载后
- SaveLoad：存档加载后
- ModLoad：MOD 加载后

要求：
- 每个检查点有明确的触发条件
- 检查点之间无重叠
- 检查点覆盖所有关键状态边界

---

# 禁止事项

禁止：Release 构建中跳过所有校验

原因：关键不变量校验（damage ≥ 0、HP 范围）在任何构建中都必须执行，跳过会导致数据损坏。

违反后果：伤害值为负、HP 超出范围等问题在 Release 中隐藏，玩家遇到严重 Bug。

---

禁止：校验失败时静默忽略

原因：静默忽略导致校验失去意义，违规行为不被记录。

违反后果：游戏状态持续损坏，问题无法定位。

---

禁止：校验函数修改游戏状态

原因：校验是只读的，修改状态会导致非确定性行为和校验结果不一致。

违反后果：校验结果不可复现，回放系统失效。

---

禁止：校验失败时 crash 但不生成报告

原因：崩溃但不生成报告导致问题无法定位。

违反后果：玩家遇到崩溃但无法提供有效信息，开发者无法复现问题。

---

禁止：MOD 内容跳过校验

原因：MOD 是最高风险的数据源，跳过校验可能导致游戏状态损坏。

违反后果：恶意或错误的 MOD 导致游戏崩溃、数据损坏、安全问题。

---

禁止：手动检查代替自动校验

原因：手动检查不可靠，关键不变量必须通过自动校验保证。

违反后果：关键不变量被遗漏，游戏状态损坏。

---

禁止：校验逻辑包含业务规则

原因：校验只检查数值合法性，不执行游戏逻辑。

违反后果：校验与业务规则耦合，维护困难，校验逻辑膨胀。

---

禁止：Clamp 修正破坏游戏逻辑

原因：Clamp 修正必须在合法范围内，修正后不能引入新的违规。

违反后果：修正一个违规导致另一个违规，连锁反应。

---

# AI 修改规则

## 如果新增全局不变量

允许：
- 定义新的不变量条件
- 选择违规处理策略（Reject / Clamp / Panic）
- 在对应检查点注册校验

禁止：
- 不定义违规处理策略
- 在非检查点时刻触发校验
- 校验中修改游戏状态

优先检查：
- 不变量是否可验证（能写断言）
- 违规处理策略是否合适
- 检查点时机是否正确
- 是否影响性能

---

## 如果新增校验检查点

允许：
- 定义新的检查点触发时机
- 在对应系统中注册校验
- 调整校验范围

禁止：
- 跳过任何检查点的校验
- 在检查点中修改游戏状态
- 检查点之间有重叠

优先检查：
- 新检查点的触发条件是否明确
- 校验范围是否完整
- 是否与其他检查点冲突
- 性能影响是否可接受

---

## 如果修改违规处理策略

允许：
- 调整违规类型的处理方式（Reject → Clamp 等）
- 新增违规类型的处理策略

禁止：
- 将 Panic 降级为 Reject（数据损坏不可恢复）
- 将 Reject 降级为静默忽略
- Clamp 修正破坏游戏逻辑

优先检查：
- 新策略是否安全
- 是否影响其他不变量
- 日志级别是否正确
- 是否需要调整 Debug 构建的增强校验

---

## 如果测试失败

排查顺序：
1. 检查校验检查点是否在正确时机触发
2. 检查不变量条件是否正确定义
3. 检查违规处理策略是否合适（Reject / Clamp / Panic）
4. 检查校验函数是否只读（未修改游戏状态）
5. 检查 Clamp 修正是否引入新的违规
6. 检查 MOD 校验是否被跳过
