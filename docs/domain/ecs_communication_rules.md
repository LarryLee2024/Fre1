# ECS 通信领域

Version: 1.0
Status: Proposed

ECS 通信领域管理模块间在 ECS 架构下的通信方式选择、注册、使用和约束。

核心原则：
- Hook = 固有行为，Observer = 局部响应，Message = 跨 Feature 广播
- 通信方式的选择由模块边界决定，不由实现便利性决定
- 同模块内优先函数调用，跨模块才使用事件系统

---

# 术语定义

## Message（消息）

模块间跨 Feature 广播的通信载体。使用 `#[derive(Message)]` 定义，通过 `MessageWriter<T>` 发送，通过 `MessageReader<T>` 消费。

不是 Observer。不是 Hook。不是函数调用。不是 Command。

关键属性：
- 定义在发送方或接收方所在模块，双方都能访问的公共类型
- 必须在 App 中通过 `add_message::<T>()` 注册
- 同一帧内发送的消息在该帧的 `Update` 阶段末尾统一消费
- 携带完整上下文数据，接收方无需反向查询发送方

---

## Observer（观察者）

对特定事件触发的响应式处理单元。使用 `add_observer()` 注册，由 Bevy 的事件触发机制调用。

不是 Message。不是 Hook。不是轮询。不是 System。

关键属性：
- 用于同一 Feature 内的局部响应（如 Pointer 点击响应、死亡动画触发）
- 触发条件是明确的事件类型（如 `Pointer<Click>`、`Pointer<Over>`）
- 不同于 Message 的广播机制，Observer 是点对点响应
- 不能替代 Message 的跨模块通知功能

---

## Hook（钩子）

组件生命周期的固有副作用声明。通过 `#[component(on_add=..., on_remove=...)]` 属性绑定到组件定义上。

不是 Observer。不是 Message。不是 System。不是游戏逻辑。

关键属性：
- 副作用与组件定义绑定，不分散在各个系统中
- 触发时机为组件的 on_add、on_insert、on_remove
- 必须保持轻量：仅做组件级固有行为（如 Dead 组件移除 MoveableRange）
- 禁止包含业务逻辑或跨模块协调

---

## Command（命令）

延迟执行的 ECS 变更操作。通过 `Commands` 参数在系统中写入，在当前系统执行完毕后统一应用。

不是 Message。不是立即执行的函数调用。不是状态查询。

关键属性：
- 用于 Entity 的 spawn/despawn、组件的 insert/remove
- 延迟执行：系统写入 Commands 后不立即生效，当前 SystemSet 结束后统一应用
- 不携带跨模块通知语义，仅做 ECS 世界变更
- 与 Message 的区别：Command 是变更操作，Message 是信息广播

---

## System（系统）

Bevy ECS 中处理数据的逻辑单元。每个 System 在每帧按调度顺序执行一次。

不是通信机制。不是跨模块协调的好场所。

关键属性：
- 只能包含纯逻辑，禁止存储任何状态
- 通过 System 参数声明对 Resource、Component、Message 的读写依赖
- 系统间的数据交换通过 Message、Resource、Component 完成
- 禁止 System 之间直接函数调用（同模块内除外）

---

## EventBus（消息总线）

Bevy 内部管理 Message 缓冲和分发的机制。每种 Message 类型有独立的缓冲区。

不是全局可变状态。不是直接函数调用链。不是消息存储。

关键属性：
- 每种 Message 类型在 `add_message::<T>()` 注册时创建独立缓冲
- 发送方通过 `MessageWriter<T>` 写入，接收方通过 `MessageReader<T>` 读取
- 同一帧内发送的消息在该帧结束前可被多个接收方消费
- 帧结束后未消费的消息不会自动清除（需注意避免消息堆积）

---

## 跨层通信

不同架构层之间的通信。受层间依赖规则严格约束。

不是同层内通信。不是自由的函数调用。

关键属性：
- Core→Shared：允许直接引用（同层内通信）
- Core↔Infra：必须通过 Message，禁止直接读写 ECS 组件
- UI→Logic：只通过 UiCommand Message
- Logic→UI：只通过 ViewModel Resource
- 禁止跨层直接 Query 对方模块的 ECS 组件

---

## 同层通信

同一架构层内模块之间的通信。规则比跨层通信宽松。

不是跨层通信。不是无约束的直接访问。

关键属性：
- Core 内部跨模块：必须通过 Message 广播
- Core 内部同模块：允许函数调用
- Infrastructure 内部：允许直接函数调用
- Shared 内部：允许直接引用
- 禁止 Core 内部模块直接访问其他模块的内部组件

---

## 消息注册表

项目中所有已注册 Message 类型的清单。每次新增 Message 必须同步更新。

不是可选文档。不是实现细节。

关键属性：
- 记录 Message 名称、发送方模块、接收方模块
- 定义在 `docs/architecture.md` 的 Message 注册表中
- 新增 Message 必须在注册表中登记
- 删除 Message 必须先移除所有注册和引用

---

# 领域边界

## 本领域负责

- 定义 Hook/Observer/Message/Command 四种通信方式的使用规范
- 定义通信方式的选择标准（何时用哪种）
- 定义 Message 的注册、命名和上下文携带规范
- 定义 Observer 的触发条件和使用范围
- 定义 Hook 的组件绑定规范
- 定义跨层和同层通信的约束
- 维护 Message 注册表

## 本领域不负责

- 具体 Message 类型的业务语义（由各业务模块定义）
- 具体 Observer 的触发逻辑实现（由各 Feature 模块实现）
- 具体 Hook 的组件副作用实现（由各 Component 定义方实现）
- 状态机的阶段转换逻辑（由 Turn 领域负责）
- 错误在消息中的传播（由 Error System 领域负责）
- 层间依赖方向（由 Layer Architecture 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 战斗结果 | DamageApplied / HealApplied Message | Battle / UI |
| 回合阶段切换 | NextState\<TurnPhase\> | Turn |
| 角色死亡 | CharacterDied Message | Battle / UI / Turn |
| 用户意图 | UiCommand Message | UI / Input |
| 组件生命周期 | Hook（on_add/on_remove） | Component 定义方 |
| 胜负判定 | LevelCompleted Message | Turn / UI |

---

# 生命周期

## 消息类型生命周期

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Defined | 消息类型已定义 | Registered |
| Registered | 已在 App 中 add_message 注册 | Active |
| Active | 消息在系统中正常发送和消费 | Deprecated |
| Deprecated | 消息不再发送但接收方仍兼容 | Removed |
| Removed | 消息类型和所有引用已删除 | — |

## 状态转换图

```
Defined → Registered → Active → Deprecated → Removed
```

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| Defined | Registered | `add_message::<T>()` 注册完成 |
| Registered | Active | 至少一个系统发送 + 至少一个系统消费 |
| Active | Deprecated | 业务变更导致消息不再需要发送 |
| Deprecated | Removed | 所有发送方和消费方已移除引用 |

---

# 不变量

## 不变量1：跨模块通信必须通过 Message

任意时刻：

Core 层内部跨模块通信只通过 Message 广播，禁止直接访问其他模块的内部组件。

违反表现：

`core/battle/` 中直接查询 `core/character/` 的 Unit 组件。`core/skill/` 中直接修改 `core/buff/` 的 ActiveBuffs。

---

## 不变量2：UI 不直接访问 Core 组件

任意时刻：

UI 层只读取 ViewModel Resource，不直接 Query Core 层的 ECS 组件。UI→Logic 只通过 UiCommand Message。

违反表现：

UI 系统中出现 `Query<&Attributes, With<Unit>>`。UI 系统直接修改 `unit.acted = true`。

---

## 不变量3：Hook 不包含业务逻辑

任意时刻：

Hook 回调只执行组件级固有行为（如 Dead 组件移除 MoveableRange），不包含跨模块协调或游戏规则判断。

违反表现：

`on_add` Hook 中执行伤害计算。`on_remove` Hook 中触发 UI 刷新。

---

## 不变量4：每种 Message 必须有唯一注册

任意时刻：

每个 Message 类型在 App 中只注册一次（`add_message::<T>()`），重复注册会导致 panic。

违反表现：

两个 Plugin 分别注册同一 Message 类型导致 panic。

---

## 不变量5：Message 携带完整上下文

任意时刻：

Message 必须携带接收方处理所需的全部上下文信息（Entity ID、名称、阵营、数值等），禁止接收方反向查询发送方获取缺失数据。

违反表现：

`DamageApplied` 不携带 `target_name`，接收方需要额外 Query UnitName 组件。

---

## 不变量6：Observer 不替代 Message 的跨模块功能

任意时刻：

Observer 仅用于同一 Feature 内的局部响应，不用于跨模块命令分发或跨层通信。

违反表现：

Observer 中执行跨模块的状态修改。用 Observer 替代 Message 实现跨 Feature 通知。

---

# 业务规则

## 规则1：通信方式选择

允许：
- 同 Feature 内局部响应使用 Observer
- 组件固有行为使用 Hook
- 跨 Feature 广播使用 Message
- ECS 世界变更使用 Command
- 同模块内直接函数调用

禁止：
- 跨模块直接访问内部组件
- 用 Observer 模拟函数调用（同模块内应直接调用）
- 用 Message 模拟函数调用（同 Feature 内应直接调用）
- 高频逻辑（每帧 10 次以上）走 Observer 造成风暴

必须：
- 跨模块通信只通过 Message / Observer / Command
- Hook 只用于组件固有行为

---

## 规则2：Message 定义规范

允许：
- 消息类型使用 `#[derive(Message, Debug, Clone)]`
- 消息字段使用命名参数（提高可读性）
- 消息携带 Option 类型字段（可选上下文）

禁止：
- 消息字段使用元组结构体（降低可读性）
- 消息不携带上下文信息（如 `DamageApplied` 不带 `target_name`）
- 在消息中嵌套 Entity 查询（消息应携带查询结果）

必须：
- 每个消息类型在 `add_message::<T>()` 注册一次
- 新增消息在 `docs/architecture.md` 注册表中登记

---

## 规则3：Message 注册与消费

允许：
- 多个接收方消费同一消息（广播语义）
- 接收方通过 `MessageReader<T>` 消费
- 发送方通过 `MessageWriter<T>` 发送

禁止：
- 发送方在同一帧内多次 `add_message` 注册同一类型
- 接收方使用 `Single<T>` 读取消息（应遍历 `MessageReader`）
- 在 OnEnter 阶段发送消息后立即消费（消息在 Update 阶段统一消费）

必须：
- 发送方在发送前确认消息已注册
- 接收方遍历 `MessageReader` 消费所有待处理消息

---

## 规则4：Observer 使用规范

允许：
- 同 Feature 内响应特定事件（如 `Pointer<Click>`、`Pointer<Over>`）
- Observer 读取触发事件的 Entity 信息
- Observer 更新 Resource 状态

禁止：
- 跨 Feature 的 Observer（应改用 Message）
- Observer 中执行耗时操作
- Observer 中修改非触发 Entity 的组件

必须：
- Observer 注册在 Plugin 的 `build()` 方法中
- Observer 函数签名包含触发事件类型参数

---

## 规则5：Hook 使用规范

允许：
- 在 `#[component(on_add=..., on_remove=...)]` 中声明固有行为
- Hook 移除关联组件（如 Dead 移除 MoveableRange）
- Hook 执行轻量级标签操作

禁止：
- Hook 中执行跨模块通信
- Hook 中包含游戏规则判断
- Hook 中发送 Message（Hook 执行时机可能早于消息系统就绪）

必须：
- Hook 函数为纯函数（无副作用、无资源访问）
- Hook 只操作当前 Entity 的组件

---

## 规则6：Command 使用规范

允许：
- 使用 `commands.entity(e).try_despawn()` 销毁实体
- 使用 `commands.entity(e).insert(Component)` 添加组件
- 使用 `commands.entity(e).remove::<Component>()` 移除组件
- 使用 `commands.insert_resource(R)` 插入资源
- 使用 `commands.write_message(T)` 发送消息（Command 级别的消息发送）

禁止：
- 在 Command 中查询其他 Entity 的状态（Command 是延迟执行，查询可能不一致）
- 用 Command 替代 Message 的跨模块通知功能
- 在同一系统中混合使用立即查询和延迟 Command 修改同一 Entity

必须：
- Entity 的 spawn/despawn 统一通过 Command
- Command 批量操作时注意 Entity 存活状态检查

---

## 规则7：State 驱动通信

允许：
- 通过 `NextState<TurnPhase>` 驱动阶段转换
- 通过 `OnEnter(State)` 和 `OnExit(State)` 触发阶段初始化和清理
- 通过 `run_if(in_state(...))` 条件调度系统

禁止：
- 手动设置 State 而不经过 `NextState`
- 在 `OnEnter` 中执行跨阶段跳转
- 在 `OnEnter` / `OnExit` 中包含重逻辑

必须：
- 阶段转换只通过 `NextState<TurnPhase>` 驱动
- OnEnter / OnExit 系统保持轻量

---

# 管线

## Message 发送管线

```
System 生成 Message → MessageWriter 写入 → EventBus 缓冲 → 目标 System 读取 → Message 消费
```

### Step1：System 生成 Message

输入：业务逻辑产生的事件（伤害结算完成、回合结束等）
处理：构造 Message 结构体，携带完整上下文
输出：Message 实例
禁止：不携带上下文信息就发送消息

### Step2：MessageWriter 写入

输入：Message 实例
处理：通过 `MessageWriter<T>.write(msg)` 写入 EventBus
输出：消息进入缓冲区
禁止：在非系统函数中直接写入消息

### Step3：EventBus 缓冲

输入：已写入的消息
处理：按 Message 类型分发到对应缓冲区
输出：缓冲区中的待消费消息
禁止：清空缓冲区中未消费的消息

### Step4：目标 System 读取

输入：EventBus 缓冲区
处理：目标系统通过 `MessageReader<T>.read()` 遍历读取
输出：处理后的状态变更
禁止：假设消息只有 1 条（必须遍历 reader）

### Step5：Message 消费

输入：已读取的消息
处理：执行业务逻辑（更新组件、发送新消息、触发 UI 刷新）
输出：状态变更和可能的新消息
禁止：在消费消息时发送同类型的消息（避免递归）

---

## Observer 触发管线

```
事件发生 → Observer 注册表匹配 → Observer 函数执行 → 状态变更
```

### Step1：事件发生

输入：用户交互（Pointer 点击、悬停）或 ECS 事件
处理：Bevy 内部事件系统分发
输出：事件实例
禁止：在事件中嵌套触发其他事件

### Step2：Observer 注册表匹配

输入：事件类型
处理：查找所有注册的 Observer 函数
输出：匹配的 Observer 列表
禁止：无注册的 Observer 响应未注册的事件类型

### Step3：Observer 函数执行

输入：触发事件和 ECS 查询参数
处理：执行 Observer 逻辑（通常是状态更新或消息发送）
输出：状态变更
禁止：在 Observer 中执行耗时操作或跨模块协调

---

## Hook 生命周期管线

```
组件生命周期事件 → Hook 函数执行 → 组件级固有行为
```

### Step1：组件生命周期事件

输入：Entity 的组件添加/移除操作
处理：Bevy 检测到组件变更
输出：on_add / on_insert / on_remove 事件
禁止：在组件操作后绕过 Hook 直接执行固有行为

### Step2：Hook 函数执行

输入：触发事件
处理：执行 Hook 函数（纯函数，无资源访问）
输出：组件级副作用（如移除关联组件）
禁止：Hook 中查询其他 Entity、访问 Resource、发送 Message

---

# 数据结构

## DamageApplied（伤害消息示例）

职责：通知伤害已应用，供 UI 飘字、战斗日志、战斗记录消费

结构：
- target：Entity — 被攻击的实体
- target_name：String — 被攻击者名称
- target_faction：Faction — 被攻击者阵营
- attacker：Entity — 攻击者实体
- attacker_name：String — 攻击者名称
- attacker_faction：Faction — 攻击者阵营
- amount：i32 — 伤害量
- is_skill：bool — 是否技能攻击
- terrain_label：String — 地形标签
- target_coord：IVec2 — 目标格子坐标
- breakdown：Option\<DamageBreakdown\> — 伤害分解（可选）

要求：
- 每个字段都携带完整上下文，接收方无需反向查询
- 使用 `#[derive(Message, Debug, Clone)]`
- 在 BattlePlugin 中通过 `add_message::<DamageApplied>()` 注册

---

## UiCommand（用户意图消息）

参见 `ui_architecture_rules.md#UI 命令`。

UiCommand 是 UI→Core 的 Message 通道。其完整变体定义、结构和使用规范由 UI Architecture 领域负责。本领域仅定义其在 ECS 通信中的角色：跨层 Message，由 UI 系统发送、command_handler 消费。

---

## CombatIntent（战斗意图资源）

参见 `turn_battle_rules.md#战斗意图`。

CombatIntent 是 Turn/Battle 领域的 Resource，用于传递玩家或 AI 的攻击意图到 Effect Pipeline。本领域仅定义其在 ECS 通信中的角色：跨模块 Resource 共享，由 SelectTarget/ExecuteAction 阶段读写。

---

## MovementIntent（移动意图消息）

职责：统一玩家和 AI 的移动请求，实现意图与执行分离

结构：
- entity：Entity — 要移动的单位
- target_coord：IVec2 — 目标坐标
- source：IntentSource — 意图来源（Player / Ai）

要求：
- 使用 `#[derive(Message, Debug, Clone)]`
- 由 command_handler 或 AI 决策系统发送
- 由移动执行系统消费
- 参见 `turn_battle_rules.md` 中的行动顺序编排规则

---

# 禁止事项

禁止：跨模块直接访问内部组件或状态

原因：跨模块直接访问破坏模块边界，导致模块间强耦合，任何组件变更都影响所有依赖方。

违反后果：修改一个模块的组件导致全项目编译失败，无法独立测试。

---

禁止：用 Observer 替代 Message 的跨模块通知功能

原因：Observer 是点对点响应，不适合一对多广播。跨模块通知使用 Message 才能解耦发送方和接收方。

违反后果：发送方必须知道所有接收方，违反开闭原则。

---

禁止：用 Message 模拟同模块内的函数调用

原因：同模块内的逻辑直接函数调用更高效、更清晰。事件化同模块逻辑增加不必要的复杂度。

违反后果：简单函数调用变成消息发送+接收，代码可读性下降、调试困难。

---

禁止：高频逻辑走 Observer 造成风暴

原因：每帧执行 10 次以上的逻辑如果走 Observer，会产生大量 Observer 实例，性能严重下降。

违反后果：帧率骤降、内存膨胀、Observer 注册表膨胀。

---

禁止：Hook 中包含业务逻辑或跨模块通信

原因：Hook 是组件的固有行为，执行时机特殊（可能在 ECS 世界状态不一致时触发），不适合执行业务逻辑。

违反后果：Hook 中的业务逻辑可能在错误的时机执行，导致状态不一致。

---

禁止：UI 直接 Query Core 层 ECS 组件

原因：UI 只读 ViewModel，直接 Query 会破坏 Logic/Presentation 分离，导致 UI 和业务逻辑耦合。

违反后果：UI 和业务逻辑无法独立替换，UI 修改导致业务逻辑测试失败。

---

禁止：消息不携带完整上下文

原因：接收方需要额外 Query 获取缺失数据，增加了模块间的隐式依赖，且在 ECS 世界状态变化时可能查询到不一致的数据。

违反后果：接收方依赖发送方的组件结构，发送方组件变更导致接收方编译失败。

---

禁止：在同一帧内发送同类型消息后立即消费

原因：Bevy 的 Message 缓冲机制保证同一帧内所有发送的消息都可被消费，但递归发送同类型消息可能导致消息堆积。

违反后果：消息处理无限循环，系统阻塞。

---

禁止：在 OnEnter 阶段发送消息后假设立即可用

原因：OnEnter 阶段的消息写入可能在 Update 阶段才被消费，依赖立即可用会导致逻辑错误。

违反后果：OnEnter 中发送的消息在 OnEnter 内部消费不到，导致初始化逻辑不完整。

---

禁止：Command 中查询其他 Entity 状态

原因：Command 是延迟执行的，在系统执行时 World 状态可能已变更，查询结果不可靠。

违反后果：基于过时查询结果执行的 Command 操作导致状态不一致。

---

# AI 修改规则

## 如果新增 Message 类型

允许：
- 使用 `#[derive(Message, Debug, Clone)]` 定义
- 在 Plugin 的 `build()` 中调用 `add_message::<T>()`
- 消息字段携带完整上下文

禁止：
- 不注册就使用（忘记 `add_message`）
- 消息字段使用元组结构体
- 在消息中嵌套 Entity 查询

优先检查：
- 消息类型是否已在 `docs/architecture.md` 注册表中登记
- 消息字段是否携带接收方所需的完整上下文
- 是否与现有消息类型重复（命名、语义）

---

## 如果修改现有 Message 类型

允许：
- 新增字段（Option 类型，保持向后兼容）
- 修改字段名称（需更新所有引用）

禁止：
- 删除已有字段（破坏接收方）
- 修改字段类型（破坏接收方）
- 不更新发送方和接收方就修改消息结构

优先检查：
- 所有使用该消息的 System 是否同步更新
- `docs/architecture.md` 注册表是否同步更新
- 测试是否需要更新

---

## 如果新增 Observer

允许：
- 在 Plugin 的 `build()` 中注册
- Observer 响应同一 Feature 内的事件
- Observer 更新 Resource 或发送 Message

禁止：
- Observer 跨 Feature 执行业务逻辑
- Observer 中执行耗时操作
- Observer 替代 Message 的跨模块通知

优先检查：
- Observer 的触发事件类型是否正确
- Observer 是否只处理同一 Feature 内的逻辑
- 是否有更合适的方式（Message 或直接 System）

---

## 如果修改 Hook

允许：
- 新增组件的 on_add / on_remove 声明
- Hook 执行轻量级标签操作

禁止：
- Hook 中包含业务逻辑
- Hook 中查询其他 Entity 或访问 Resource
- Hook 中发送 Message

优先检查：
- Hook 是否保持纯函数（无副作用）
- Hook 是否只操作当前 Entity 的组件
- Hook 执行时机是否与组件生命周期一致

---

## 如果测试失败（通信相关）

排查顺序：
1. 检查 Message 是否已注册（`add_message::<T>()` 是否调用）
2. 检查 Message 字段是否完整（接收方是否需要额外查询）
3. 检查通信方式选择是否正确（Hook/Observer/Message/Command）
4. 检查 Observer 触发事件类型是否正确
5. 检查 Hook 是否在正确的组件生命周期触发
