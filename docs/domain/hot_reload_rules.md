# 热重载安全领域

Version: 1.1
Status: Proposed

热重载安全领域定义热重载的安全边界——什么可以/不可以热重载，什么时机可以/不可以热重载。只允许热重载 Definition 数据，绝不触碰 Instance 数据。

核心原则：
- 只热重载 Definition（Registry），绝不触碰 Instance（运行时 Component）
- 战斗进行中禁止热重载（必须等到 TurnEnd 或 BattleEnd）
- 热重载前必须验证数据合法性
- 热重载失败必须回退到上次有效状态

## 宪法合规矩子

| 条款 | 级别 | 落地规则 |
|------|------|----------|
| 12.1.5 热重载优先 | 🟥 | 所有配置必须优先支持热重载 |
| 12.1.6 自动校验 | 🟥 | 配置之间的引用关系必须实现自动校验 |
| 14.0.5 资源热重载 | 🟥 | 高频修改的资源必须优先支持热重载 |
| 11.5.1 命令层统一 | 🟩 | 热重载操作标准化为命令，支持回放和测试 |
| 22.2 配置与运行时分离 | 🟥 | 只热重载 Definition，绝不触碰 Instance |

---

# 术语定义

## 热重载（Hot Reload）

运行时替换 Definition 配置数据而不重启游戏。在文件变更检测到后，自动或手动触发 RON 文件重新加载。

不是重启。不是冷加载。不是修改 Instance。不是重新编译。

关键属性：
- 热重载只更新 Registry 中的 Definition 数据
- 热重载不修改任何运行时 Instance 状态
- 热重载在安全时间窗口内执行（TurnEnd 或 BattleEnd）
- 热重载失败时回退到上次有效状态
- 热重载后所有引用该 Definition 的新实例使用新数据

---

## 重载安全边界（Reload Safety Boundary）

热重载允许修改的数据范围。定义什么数据可以热重载，什么数据禁止热重载。

不是文件系统。不是模块边界。不是 ECS 边界。

关键属性：
- Definition（RON 配置）可热重载 ✅
- Instance（运行时状态）禁止热重载 🟥
- 战斗中任何数据禁止热重载 🟥
- UI 主题可热重载 ✅（无业务影响）
- 边界由游戏规则定义，不由文件位置决定

---

## 重载时机（Reload Window）

允许热重载生效的时间窗口。只有在安全的时间点才允许执行热重载。

不是任意时刻。不是战斗中途。不是回合中间。不是动画播放中。

关键属性：
- TurnEnd 阶段后允许热重载
- BattleEnd 阶段后允许热重载
- MainMenu 阶段允许热重载
- LevelSelect 阶段允许热重载
- Running 战斗状态禁止热重载

---

## 定义数据（Definition Data）

可热重载的不可变配置。存储在 Registry 中，通过 RON 文件加载。

不是 Instance 数据。不是 ECS Component 运行值。不是游戏状态。

关键属性：
- Definition 在 RON 文件中定义（assets/ 和 content/ 目录）
- Definition 不可变（加载后不修改）
- Definition 通过 Strong ID 引用
- 热重载后 Definition 更新，但不影响已生成的 Instance
- 参见 `content_system_rules.md#Definition`

---

## 实例数据（Instance Data）

不可热重载的运行时状态。存储在 ECS Component 中，由游戏逻辑修改。

不是 Definition。不是配置。不是静态数据。

关键属性：
- Instance 在运行时创建和修改
- Instance 通过 Strong ID 引用 Definition
- Instance 是可变的（游戏逻辑修改）
- 热重载绝不触碰 Instance 数据
- 参见 `persistence_rules.md#InstanceData`

---

## 重载通知（Reload Notification）

热重载完成后通知所有相关系统的消息。告知其他系统 Definition 已更新。

不是 Event。不是 Command。不是函数调用。

关键属性：
- 重载通知在热重载成功完成后发送
- 重载通知携带更新的 Definition ID 列表
- 重载通知通过 Message 系统广播
- 接收通知的系统可以决定是否响应（如刷新 UI）
- 重载通知不触发游戏逻辑执行

---

# 领域边界

## 本领域负责

- 文件变更检测（RON 文件监控）
- 重载时机判断（Reload Window 检查）
- RON 文件重新加载和校验
- 引用完整性校验
- Registry 更新
- 重载通知发送
- 热重载失败回退
- 热重载边界定义

## 本领域不负责

- Definition 的初始加载（由 Content 层负责）
- Registry 的维护（由 Core 层负责）
- Instance 数据的修改（由 Core 层游戏逻辑负责）
- 文件系统监控的具体实现（由 Infrastructure 资源加载模块负责）
- UI 界面（由 UI 层负责）
- 存档格式兼容性（由 Persistence 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 文件变更检测 | Observer | Infrastructure 资源加载模块 |
| 重载时机查询 | Resource 读取 | Core 层（查询游戏状态） |
| Registry 更新 | Resource 写入 | Core 层 Registry |
| 重载完成通知 | Message | 所有相关系统 |
| 重载失败通知 | Message | UI 层 |

---

# 生命周期

## 状态列表

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Idle | 无重载请求 | Detecting |
| Detecting | 检测文件变更 | Idle, ReloadPending |
| ReloadPending | 等待安全时间窗口 | Reloading, Idle |
| Reloading | 正在执行重载 | Idle, ReloadError |
| ReloadError | 重载失败 | Idle |

## 状态转换图

```
Idle → Detecting → ReloadPending → Reloading → Idle
  ↑         ↓            ↓            ↓
  ↑       Idle         Idle      ReloadError
  ↑                                  ↓
  └──────────────────────────────────┘
```

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| Idle | Detecting | 文件监控检测到 RON 文件变更 |
| Detecting | ReloadPending | 检测到有效文件变更 |
| Detecting | Idle | 文件变更是无效的（非 RON 文件或未修改） |
| ReloadPending | Reloading | 进入安全时间窗口（TurnEnd/BattleEnd/MainMenu） |
| ReloadPending | Idle | 超时未进入安全时间窗口（取消重载） |
| Reloading | Idle | 重载成功，Registry 已更新 |
| Reloading | ReloadError | 重载失败（校验失败或引用不完整） |
| ReloadError | Idle | 回退到上次有效状态 |

---

# 不变量

## 不变量1：热重载只更新 Definition [宪法 12.1.5 🟥]

任意时刻：

🟥 热重载只更新 Registry 中的 Definition 数据。禁止热重载修改任何运行时 Instance 数据（ECS Component）。这是宪法最高优先级条款（22.2 配置与运行时分离）。

违反表现：

已生成单位的属性值被覆盖。战斗状态不一致。游戏逻辑行为异常。

---

## 不变量2：Running 中的战斗不可热重载 [宪法 12.1.5 🟥]

任意时刻：

🟥 当游戏处于 Running 战斗状态（回合进行中、战斗阶段中间）时，禁止执行热重载。必须等到 TurnEnd 或 BattleEnd 阶段。破坏热重载时机保证回放确定性。

违反表现：

战斗中途 Definition 更新，导致伤害计算不一致。回放确定性被破坏。

---

## 不变量3：热重载不改变已生成 Instance 的属性

任意时刻：

热重载后的 Definition 只影响新生成的 Instance。已存在的 Instance 保持原有 Definition 引用不变。

违反表现：

已生成单位的属性被覆盖。战斗中途单位能力变化。游戏平衡性被破坏。

---

## 不变量4：热重载后引用完整性必须通过校验

任意时刻：

热重载完成后，所有 Definition 的引用完整性必须通过校验。任何缺失的引用必须被检测并阻止重载。

违反表现：

运行时引用缺失导致 panic。游戏状态不完整。数据错乱。

---

# 业务规则

## 规则1：热重载安全边界 [宪法 12.1.5 / 14.0.5 🟥]

🟥 **所有配置必须优先支持热重载。高频修改的资源必须优先支持热重载。**

允许：
- 🟥 热重载 Definition（RON 配置文件）
- 🟥 热重载 UI 主题配置
- 🟥 热重载关卡配置（非战斗中）
- 🟩 高频修改的资源（技能、Buff、装备定义）优先支持热重载

禁止：
- 🟥 热重载 Instance 数据（违反 22.2 配置与运行时分离）
- 🟥 热重载战斗中任何数据
- 热重载代码模块（需要重启）

必须：
- 热重载前验证文件格式合法性
- 🟥 热重载后发送重载通知
- 热重载失败时回退到上次有效状态
- 🟥 配置之间的引用关系必须实现自动校验（12.1.6）

---

## 规则2：热重载时机控制

允许：
- 在 TurnEnd 阶段后执行热重载
- 在 BattleEnd 阶段后执行热重载
- 在 MainMenu 阶段执行热重载
- 在 LevelSelect 阶段执行热重载

禁止：
- 在 Running 战斗状态执行热重载
- 在回合选择阶段执行热重载
- 在动画播放中执行热重载
- 在伤害结算中间执行热重载

必须：
- 检测到文件变更后等待安全时间窗口
- 超时未进入安全时间窗口时取消重载
- 重载完成后通知所有相关系统

---

## 规则3：热重载校验要求

允许：
- 校验 RON 文件格式合法性
- 校验 Definition 引用完整性
- 校验字段类型正确性

禁止：
- 跳过格式校验直接加载
- 跳过引用完整性校验
- 校验失败后继续重载

必须：
- 校验失败时返回错误并回退
- 校验通过后才更新 Registry
- 校验结果记录到日志

---

## 规则4：热重载失败回退

允许：
- 热重载失败时恢复到上次有效状态
- 热重载失败时发送错误通知
- 热重载失败时记录错误日志

禁止：
- 热重载失败后保留部分更新
- 热重载失败后静默忽略
- 热重载失败后重试而不回退

必须：
- 回退到上次有效的 Definition 状态
- 通知所有相关系统重载失败
- 记录失败原因到日志

---

## 规则5：热重载通知机制

允许：
- 热重载成功后发送重载通知
- 重载通知携带更新的 Definition ID 列表
- 接收通知的系统自主决定是否响应

禁止：
- 重载通知触发游戏逻辑执行
- 重载通知修改 Instance 数据
- 重载通知绕过安全边界

必须：
- 重载通知通过 Message 系统广播
- 重载通知在 Registry 更新后发送
- 重载通知携带完整的更新信息

---

## 规则6：热重载事件风暴缓解 [NEW since v2.2]

> **优化来源**: docs/architecture/config_system_design.md §4.1

必须：
- 热重载通知只传递 config_type（如 `SkillConfig`），不传递 changes 列表
- 下游 System 收到通知后标记 `NeedsRebuild<T>` Marker
- 在下一帧的特定 Phase（如 PreparePhase）统一执行重建
- 避免一帧内触发大量变更事件导致下游 System 卡顿

禁止：
- 热重载通知携带完整的变更列表（大数据量导致性能问题）
- 下游 System 收到通知后立即同步重建（阻塞主线程）
- 策划保存大量配置时在一帧内触发所有下游重建

允许：
- 合并同一配置类型的多次变更为一次重建
- 延迟到下一帧的特定 Phase 统一执行重建
- 使用 Marker 标记"需要重建"的系统，避免重复重建

---

## 规则7：配置拆分为细粒度 Resource [NEW since v2.2]

> **优化来源**: docs/architecture/config_system_design.md §2.2

必须：
- GameRulesConfig 按领域拆分为独立 Resource（`BattleConfig`、`SkillConfig`、`BuffConfig`）
- 每个 Resource 可独立热重载
- System 只声明所需 `Res<T>`，其他 System 可并行

禁止：
- 将所有游戏规则塞入单一的 `GameRulesConfig` Resource（全局读锁竞争）
- 修改 SkillConfig 时触发 BattleConfig 的 `Changed` 检测

允许：
- 使用 Bevy Asset（`Handle<T>`）天然支持热重载
- MOD 精确替换单个配置文件而不影响其他

---

# 流程管线

## 管线1：热重载管线

```
文件变更检测 → Reload Window 检查 → 重新加载 RON → 校验引用完整性 → 更新 Registry → 发送 ReloadNotification
```

### Step1：文件变更检测

输入：文件系统监控事件
处理：检测 RON 文件变更（创建、修改），过滤非 RON 文件
输出：变更的文件路径列表
禁止：处理非 RON 文件变更、处理未修改的文件

### Step2：Reload Window 检查

输入：变更的文件路径列表
处理：查询当前游戏状态，判断是否在安全时间窗口内
输出：允许重载/需要等待/拒绝重载
禁止：在 Running 战斗状态执行重载、跳过时机检查

### Step3：重新加载 RON

输入：允许重载的文件路径
处理：重新解析 RON 文件，校验格式合法性
输出：Definition 数据结构
禁止：加载格式不合法的文件、跳过格式校验

### Step4：校验引用完整性

输入：Definition 数据结构
处理：检查所有 Definition 引用是否完整（Strong ID 在 Registry 中存在）
输出：引用完整性校验结果
禁止：跳过引用完整性校验、校验失败后继续

### Step5：更新 Registry

输入：校验通过的 Definition 数据
处理：原子替换 Registry 中的 Definition 条目
输出：Registry 更新完成
禁止：非原子替换（必须整体替换）、更新过程中读取 Registry

### Step6：发送 ReloadNotification

输入：更新的 Definition ID 列表
处理：通过 Message 系统广播重载通知
输出：通知发送完成
禁止：通知触发游戏逻辑、通知修改 Instance 数据

---

# 数据结构

## ReloadRequest（重载请求）

职责：记录一次热重载请求的信息

结构：
- file_path：String — 变更的文件路径
- timestamp：u64 — 检测到变更的时间
- requested_by：String — 请求来源（文件监控/手动）

要求：
- file_path 必须指向有效的 RON 文件
- timestamp 用于超时判断

---

## ReloadWindow（重载窗口）

职责：判断当前是否在安全时间窗口内

结构：
- current_state：GameState — 当前游戏状态
- is_safe：bool — 是否在安全时间窗口
- reason：String — 不安全的原因（如果 is_safe 为 false）

要求：
- current_state 必须准确反映当前游戏状态
- is_safe 为 false 时 reason 必须有明确说明

---

## ReloadResult（重载结果）

职责：记录热重载的执行结果

结构：
- success：bool — 是否成功
- updated_definitions：Vec<String> — 更新的 Definition ID 列表
- error：Option<HotReloadError> — 错误信息（如果失败）
- rolled_back：bool — 是否已回退到上次有效状态

要求：
- success 为 false 时 error 必须有值
- rolled_back 为 true 时表示已恢复到上次有效状态
- updated_definitions 只在成功时有值

---

## ReloadNotification（重载通知）

职责：热重载完成后通知所有相关系统

结构：
- updated_ids：Vec<String> — 更新的 Definition ID 列表
- timestamp：u64 — 通知发送时间
- source：String — 重载来源标识

要求：
- 通过 Message 系统广播
- 携带完整的更新信息
- 不触发游戏逻辑执行

---

# 禁止事项

禁止：战斗中热重载（Running 战斗状态）

原因：战斗中途 Definition 更新会导致伤害计算不一致、Buff 结算异常。破坏游戏确定性和回放保证。

违反后果：战斗中途单位能力变化。回放确定性被破坏。游戏平衡性被破坏。

---

禁止：热重载修改 Instance 数据

原因：Instance 是运行时状态，热重载修改会导致状态不一致。Instance 的生命周期由游戏逻辑管理。

违反后果：已生成单位的属性被覆盖。战斗状态不一致。游戏逻辑行为异常。

---

禁止：热重载跳过引用完整性校验

原因：引用完整性是 Definition 之间关系的正确性保证。跳过校验可能导致运行时引用缺失。

违反后果：运行时引用缺失导致 panic。游戏状态不完整。数据错乱。

---

禁止：热重载失败时不回退

原因：部分更新的 Definition 可能处于不一致状态。不回退会导致游戏基于损坏的 Definition 运行。

违反后果：游戏基于部分更新的 Definition 运行。状态不一致。数据错乱。

---

禁止：热重载未验证的数据

原因：未验证的 RON 文件可能格式错误、类型不匹配或引用缺失。直接加载会导致运行时错误。

违反后果：反序列化失败。运行时 panic。游戏崩溃。

---

禁止：热重载通知触发游戏逻辑

原因：重载通知只告知 Definition 已更新，不应触发游戏逻辑执行。触发逻辑会导致状态不一致。

违反后果：热重载后意外执行游戏逻辑。状态不一致。回合流程被打断。

---

禁止：热重载超时不取消

原因：长时间等待安全时间窗口会影响用户体验。超时应该取消重载并通知用户。

违反后果：用户困惑（为什么重载没有生效）。资源浪费（持续监控文件变更）。

---

# AI 修改规则

## 如果新增可热重载的 Definition 类型

允许：
- 在 RON 文件中定义新的 Definition 结构
- 在热重载管线中添加新的校验逻辑
- 在热重载通知中携带新的 Definition ID

禁止：
- 新增的 Definition 类型引用 Instance 数据
- 跳过引用完整性校验
- 热重载失败时不回退

优先检查：
- 新 Definition 类型是否只包含静态配置
- 新 Definition 类型的引用是否完整
- 热重载后是否需要通知其他系统

---

## 如果修改热重载时机

允许：
- 扩展安全时间窗口（如添加新的安全阶段）
- 缩小安全时间窗口（如禁止某些阶段重载）
- 优化时机判断逻辑

禁止：
- 在 Running 战斗状态允许重载
- 跳过时机检查
- 超时不取消

优先检查：
- 新的安全阶段是否真的不影响游戏确定性
- 时机判断是否覆盖所有游戏状态
- 超时机制是否正常工作

---

## 如果修改热重载校验逻辑

允许：
- 新增校验规则
- 优化校验性能
- 改善校验错误信息

禁止：
- 跳过任何校验步骤
- 校验失败后继续重载
- 校验不检查引用完整性

优先检查：
- 校验规则是否覆盖所有可能的错误
- 校验失败时是否正确回退
- 校验错误信息是否清晰可读

---

## 如果修改热重载通知机制

允许：
- 新增通知携带的信息
- 优化通知发送时机
- 改善通知格式

禁止：
- 通知触发游戏逻辑
- 通知修改 Instance 数据
- 通知绕过安全边界

优先检查：
- 通知是否在 Registry 更新后发送
- 通知是否携带完整的更新信息
- 通知接收者是否正确处理

---

## 如果测试失败

排查顺序：
1. 检查热重载是否在安全时间窗口内执行（Running 状态禁止）
2. 检查热重载是否只更新 Definition（不修改 Instance）
3. 检查引用完整性校验是否通过
4. 检查热重载失败时是否正确回退
5. 检查重载通知是否正确发送
