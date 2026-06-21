---
id: 03-technical.ecs-communication-rules
title: ECS Communication Rules
status: draft
owner: feature-developer
created: 2026-06-14
updated: 2026-06-14
tags:
  - technical
---

# ECS 通信领域

Version: 2.0 [NEW since v2.0]
Status: Proposed

ECS 通信领域管理模块间在 ECS 架构下的通信方式选择、注册、使用和约束。

核心原则（对应宪法第二部分 2.2 四级通信机制）：
- 🟩 2.2.1 Hook = 组件生命周期固有行为（on_add/on_remove）
- 🟩 2.2.2 Trigger = Feature 内事件链载体（伤害→护盾→吸血→反击）
- 🟩 2.2.3 Observer = 局部状态变化响应（死亡动画、血量UI刷新）
- 🟩 2.2.4 Message = 跨 Feature 全局广播（回合结束、战斗胜利）
- 🟥 2.2.5 绝对禁止将同一模块内普通逻辑全部事件化
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

## Trigger（触发器）

> 🟩 对应宪法 2.2.2：Trigger = Feature 内事件链载体

同一 Feature 内的多段响应逻辑、战斗事件链，必须使用 `commands.trigger()` 机制实现。典型场景：伤害触发护盾、吸血、反击等连锁效果。比全局 Message 轻量，天然绑定实体，适合构建线性事件链。

不是 Observer。不是 Message。不是函数调用。不是 Command。

关键属性：
- 用于同一 Feature 内的事件链式传播（如伤害→护盾吸收→吸血结算→反击触发）
- 比 Message 轻量，天然绑定实体
- 触发的是确定性的线性事件链，不是全局广播
- 不能替代 Message 的跨模块通知功能（跨模块用 Message）
- 不同于 Observer 的点对点响应，Trigger 构建的是链式传播路径

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
- 定义在 `docs/01-architecture/README.md` 的 Message 注册表中
- 新增 Message 必须在注册表中登记
- 删除 Message 必须先移除所有注册和引用

---

## Definition Component

运行时只读的配置数据 Component，从 AssetServer 加载，命名以 Def 或 Config 结尾（如 UnitClassDef、SkillConfig）。

不是 Data Component。不是 Marker。

关键属性：
- 运行时绝对不可变，禁止获取 &mut 引用
- 由 Registry 加载，存储为 Resource 或挂载在预制体 Entity 上
- 命名规范：必须以 Def 或 Config 结尾
- 修改 Definition 必须通过热重载，禁止运行时修改

---

## 四类组件分类

ECS 组件分为四类：Marker（纯标记）、Data（持久状态）、Status（临时状态）、Definition（只读配置）。

不是三类。不是全 Data。

关键属性：
- Marker：零字段，仅用于 Query 过滤
- Data：持久运行时状态，允许纯读取辅助方法（is_alive/ratio），禁止业务逻辑方法
- Status：临时生命周期状态，必须有自动清理逻辑
- Definition：运行时只读配置，禁止 &mut 访问

---

## 组件命名规范

> **优化来源**: docs/01-architecture/component_design_rules.md

ECS 组件的命名遵循 `[Entity][Concept]` 模式，按类别使用不同的命名约定。

不是 OOP 风格（XxxManager/XxxData/XxxState）。不是任意命名。不是匈牙利命名法。

关键属性：
- Marker：`IsXxx` 前缀（阵营/属性相关）或直接命名（临时状态）— `IsAlly`、`IsEnemy`、`IsFlying`、`Dead`、`Selected`
- Data：`EntityConcept` — `GridPosition`、`UnitName`、`UnitId`、`PersistentTags`
- Status：`EntityAction` — `MovingUnit`、`CastingSkill`、`PendingEffect`
- Definition：`EntityDef` / `EntityConfig` 后缀 — `UnitClassDef`、`SkillConfig`、`BattleRulesDef`
- UI Marker：`XxxBg` / `XxxFg` — `HpBarBg`、`HpBarFg`
- 🟥 禁止使用 `XxxManager`、`XxxData`、`XxxState` 等 OOP 风格命名

---

## 组件单一职责与拆分阈值

> **优化来源**: docs/01-architecture/component_design_rules.md

一个 Component = 一个数据关注点。超过 8 个字段时必须评估是否需要拆分。

不是硬性规则。不是绝对禁止。不是代码审查标准。

关键属性：
- 8 字段是"参考阈值"，核心判断标准是"关注点分离"
- 字段分为明显不同的关注点 → 即使不足 8 字段也必须拆分
- 部分字段的生命周期不同 → 拆分（如逻辑坐标 vs 表现坐标）
- 字段的修改频率差异大 → 拆分（高频变更 vs 低频变更）
- 极端性能场景且经团队评审 → 可允许少量字段合并（需记录理由）

---

## SRPG 专项组件设计建议

> **优化来源**: docs/01-architecture/component_design_rules.md

| 组件类型 | SRPG 专项建议 | 理由 |
|---------|-------------|------|
| Status Component | 必须包含 `source: Entity`（施加者）和 `duration: u32`（剩余回合） | Buff/Debuff 和临时状态极多，记录来源用于仇恨/反击计算 |
| Data Component | 坐标类组件必须区分**逻辑坐标**（`IVec2`）和**表现坐标**（`Vec3`） | 逻辑坐标用于寻路/计算（低频变更），表现坐标用于动画插值（每帧变更） |
| Marker Component | 引入 `NeedsPathfinding`、`NeedsVisionUpdate` 等"脏标记" | 寻路和视线计算极重，通过 Marker 标记"需要重算"的单位 |
| Definition Component | 技能/职业/特质的静态配置统一使用 `XxxDef` 命名 | 大量从 RON 文件加载的配置必须与运行时 Instance 数据物理隔离 |

---

## 序列化规范与版本字段

> **优化来源**: docs/01-architecture/component_design_rules.md

所有可序列化组件必须携带 `version` 字段，支持数据迁移。

不是可选字段。不是调试字段。不是元数据。

关键属性：
- 所有可序列化 Component 必须携带 `version: u32` 字段
- 新增字段（有默认值）：小版本 +1（如 1.0 → 1.1）
- 删除字段：大版本 +1（如 1.x → 2.0）
- 字段类型变更：大版本 +1
- Definition（配置）组件禁止在运行时修改
- Instance（运行时）组件必须引用 Definition ID，不保存 Definition 本身
- 序列化使用 `#[reflect(Serialize, Deserialize)]` 属性

---

## 变更检测陷阱

Changed\<T\> 因防御性赋值（即使值未改变也对 &mut 赋值）误触发，导致下游 System 每帧执行。

不是过滤全部有效。不是一定能用。

关键属性：
- 任何对 Component 的 &mut 访问（即使值没变）都会触发 Changed
- 防御性赋值是根本原因：修改前必须先判断值是否真正改变
- System 中如果只需要读取，必须用 &T 而非 &mut T
- 高频路径中 Changed 误触发会导致性能雪崩

---

## Plugin（插件）

Bevy 的模块化单元，负责注册 System、Resource、Message 和组件。每个 Plugin 通过 `build()` 方法声明其依赖和暴露的公共 API。

不是 System。不是通信机制。不是业务逻辑容器。

关键属性：
- 每个 Plugin 必须显式声明其依赖的其他 Plugin，禁止隐式依赖
- Plugin 只能通过公共 Message、公共 Resource、公共 Component 对外暴露能力
- Plugin 的 `build()` 方法只负责声明，不执行业务逻辑
- Plugin 之间禁止循环依赖（A 依赖 B 且 B 依赖 A）

---

## Schedule（调度）

Bevy 控制系统执行顺序的核心机制。项目定义自定义 Schedule 组织系统执行阶段。

不是 SystemSet。不是 System。不是运行时逻辑。

关键属性：
- 自定义 Schedule 包括：InputSchedule、LogicSchedule、PresentationSchedule
- 每帧执行顺序：PreUpdate → InputSchedule → Update → LogicSchedule → PostUpdate → PresentationSchedule → Last
- Effect Pipeline 使用专属 EffectPipelineSchedule 实现严格串行
- Schedule 通过 `run_if(in_state(...))` 实现状态门控调度

---

## SystemSet（系统集）

对 System 进行分组和排序的层级结构。通过 `configure_sets` 声明 Set 间的依赖关系。

不是 System。不是通信机制。不是执行顺序的隐式依赖。

关键属性：
- 三层嵌套结构：顶层 Set（阶段级）→ 功能域 Set → 管线 Set
- Set 间排序使用 `after()`/`before()` 约束，禁止使用 `.chain()` 强制串行
- 同一层级内无 `after/before` 约束的系统可被 Bevy 自动并行化
- 每个 System 必须归属到某个 SystemSet，禁止无 Set 归属直接注册

---

## Plugin 公共 API 契约

Plugin 对外暴露能力的标准化方式，定义了 Plugin 之间的通信边界。

不是内部实现细节。不是私有系统。

关键属性：
- 公共 API 类型：Message（跨 Plugin 广播）、Resource（全局状态共享）、Component（Entity 数据）
- 私有 System 默认只在本 Plugin 内部执行，禁止调用其他 Plugin 的私有系统
- 公共 SystemSet 用于排序约束，State/SubState 用于状态机定义
- 绕过 Plugin 边界直接注册资源破坏 Plugin 封装性

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

## 不变量7：所有系统必须归属某个 SystemSet

任意时刻：

每个 System 必须通过 `configure_sets` 归属到某个 SystemSet。禁止系统无 Set 归属直接注册。

违反表现：

`app.add_systems(Update, my_orphan_system);` 无 Set 归属直接注册。

> **优化来源**: docs/01-architecture/schedules_design.md

---

## 不变量8：Plugin 间禁止循环依赖

任意时刻：

Plugin 依赖图必须是 DAG（有向无环图）。禁止 A 依赖 B 且 B 依赖 A 的循环依赖。

违反表现：

`PluginA` 中 `add_plugins(PluginB)`，`PluginB` 中 `add_plugins(PluginA)` 导致 Bevy panic。

> **优化来源**: docs/01-architecture/plugin_contract_rules.md

---

## 不变量9：Effect Pipeline 三步必须严格顺序

任意时刻：

EffectGenerate → EffectModify → EffectExecute 必须严格顺序执行。禁止并行执行管线三步，禁止跳过任何一步。

违反表现：

三步系统并行注册到 EffectSet，导致效果错误。

> **优化来源**: docs/01-architecture/schedules_design.md

---

## 不变量10：LogicSet 必须在 EffectSet 之前

任意时刻：

所有业务逻辑系统必须在效果管线系统之前执行。禁止 EffectSet 在 LogicSet 之前。

违反表现：

`app.configure_sets(Update, (EffectSet, LogicSet.after(EffectSet)))` 导致业务逻辑在效果之后执行。

> **优化来源**: docs/01-architecture/schedules_design.md

---

# 业务规则

## 规则1：通信方式选择

允许：
- 🟩 同 Feature 内事件链式传播使用 Trigger（伤害→护盾→吸血→反击）
- 🟩 同 Feature 内局部响应使用 Observer（死亡动画、血量UI刷新）
- 🟩 组件固有行为使用 Hook（on_add/on_remove）
- 🟩 跨 Feature 广播使用 Message（回合结束、战斗胜利）
- 🟩 ECS 世界变更使用 Command（Entity spawn/despawn、组件 insert/remove）
- 🟩 同模块内直接函数调用

禁止：
- 🟥 跨模块直接访问内部组件（违反宪法 3.0.4）
- 🟥 用 Observer 模拟函数调用（同模块内应直接调用）
- 🟥 用 Message 模拟同模块内的函数调用（同 Feature 内应直接调用）
- 🟥 高频逻辑（每帧 10 次以上）走 Observer 造成风暴（违反宪法 2.3.5）
- 🟥 将同一模块内的普通逻辑全部事件化（违反宪法 2.2.5）

必须：
- 🟩 跨模块通信只通过 Message / Observer / Command
- 🟩 同 Feature 内事件链使用 Trigger
- 🟩 Hook 只用于组件固有行为

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
- 新增消息在 `docs/01-architecture/README.md` 注册表中登记

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

## 规则8：Definition Component 规范 [NEW since v2.0]

必须：
- Definition Component 运行时只读，System 中禁止获取 &mut 引用
- Definition Component 由 Registry 加载，命名以 Def 或 Config 结尾
- Definition Component 的修改只通过热重载

禁止：
- System 中对 Definition Component 取 &mut 引用
- Definition Component 包含运行时可变状态
- Definition Component 的命名不遵循 Def/Config 后缀约定

允许：
- Definition Component 作为 Resource 或挂载在预制体 Entity 上
- 通过 Res\<T\> 读取 Definition Component

---

## 规则9：临时标记归 Status [NEW since v2.0]

必须：
- 生命周期有限的临时标记（如 IsCasting）归为 Status Component
- Status Component 必须有 OnExit 清理逻辑

禁止：
- 临时标记归为 Marker Component（Marker 是永久标记，无需清理）
- Status Component 在生命周期结束后残留

允许：
- 永久标记（无生命周期）归为 Marker Component
- Status Component 通过 State 的 OnExit 自动清理

---

## 规则10：变更检测防御性编程 [NEW since v2.0]

必须：
- 修改 Data Component 前先判断值是否真正改变
- System 中如果只需要读取，必须用 &T 而非 &mut T
- 高频路径避免不必要的 &mut 访问

禁止：
- 对 Component 无条件 &mut 赋值导致 Changed 误触发
- System 中不需要修改却获取 &mut T

允许：
- 确认值改变后再 &mut 修改（if health.current != new_hp { health.current = new_hp; }）
- Debug 构建中启用 Changed 触发次数监控

---

## 规则11：Hook 禁止跨 Component 修改 [NEW since v2.0]

必须：
- Hook 中只操作当前 Entity 的组件
- 跨 Component 联动通过 Observer 监听事件或 cleanup_system 处理

禁止：
- Hook 中通过 get_mut 修改同一 Entity 上的其他 Component
- Hook 中发送 Message（Hook 执行时机可能早于消息系统就绪）

允许：
- Hook 中触发事件（world.commands().trigger_targets(...)）
- Hook 执行轻量级标签操作（移除/添加当前 Entity 的组件）

---

## 规则12：Data Component 纯读取辅助方法 [NEW since v2.0]

必须：
- Data Component 允许纯读取辅助方法（is_alive/ratio/max_limit）
- 辅助方法不修改 &mut self，不访问外部 World/Assets

禁止：
- Data Component 包含业务逻辑方法（take_damage/apply_buff）
- 辅助方法访问外部状态（World、AssetServer、其他 Component）

允许：
- impl Default
- #[inline(always)] 纯计算方法
- 常量查询方法（const fn）

---

## 规则13：组件循环依赖检测 [NEW since v2.0]

必须：
- 新增 #[require(...)] 声明时检查是否形成循环
- 核心组件（如 Unit）的依赖通过 Plugin 批量注册

禁止：
- A require B 且 B require A 的循环依赖
- 未检测循环依赖就注册组件

允许：
- 单向依赖链（A require B require C）
- 通过编译器错误在开发期发现循环

---

## 规则14：Plugin 显式依赖声明

必须：
- 每个 Plugin 必须显式声明其依赖的其他 Plugin
- 依赖通过 `app.add_plugins((DepA, DepB))` 在 `build()` 中声明
- 或在 App 层统一按顺序注册所有 Plugin

禁止：
- 隐式依赖其他 Plugin 注册的 Resource（未声明依赖关系）
- Plugin 之间循环依赖（A 依赖 B 且 B 依赖 A）
- 跨层注册（UI Plugin 注册 Core 系统的 Message）

优先检查：
- Plugin 的 `build()` 中是否声明了所有依赖
- 依赖方向是否符合 Shared → Infra → Core → Content → UI 顺序
- 是否有隐式依赖（依赖其他 Plugin 的 Resource 但未声明）

> **优化来源**: docs/01-architecture/plugin_contract_rules.md

---

## 规则15：Plugin 初始化顺序

必须：
- Plugin 按 Shared → Infrastructure → Core → Content → UI → Debug → Modding 顺序注册
- UI Plugin 依赖 Core（只读 ViewModel），禁止反向依赖
- Infrastructure Plugin 禁止反向依赖领域 Plugin

禁止：
- UI Plugin 在 Core Plugin 之前注册
- 领域 Plugin 依赖 UI Plugin
- 基础设施 Plugin 反向依赖领域 Plugin

允许：
- 强依赖在 `build()` 中无条件注册
- 弱依赖通过 `#[cfg(feature = "...")]` 条件编译注册
- 弱依赖缺失时必须有兜底逻辑

> **优化来源**: docs/01-architecture/plugin_contract_rules.md

---

## 规则16：Plugin 间通信契约

允许：
- 通过注册的 Message 跨 Plugin 广播（如 DamageApplied）
- 通过共享 Resource 全局状态共享（如 BattleRecord）
- 同 Plugin 局部响应使用 Observer

禁止：
- 调用其他 Plugin 的私有系统
- 绕过 Plugin 边界直接注册资源
- 跨层注册（UI Plugin 注册 Core 系统的 Message）

必须：
- Plugin 只能通过公共 Message/Resource/Component 对外暴露能力
- System 默认私有，只在本 Plugin 内部执行

> **优化来源**: docs/01-architecture/plugin_contract_rules.md

---

## 规则17：SystemSet 层级与排序

必须：
- 三层嵌套 SystemSet 结构：顶层 Set（阶段级）→ 功能域 Set → 管线 Set
- Set 间排序使用 `after()`/`before()` 约束
- 每个 System 必须归属到某个 SystemSet

禁止：
- 系统无 Set 归属直接注册
- Set 间依赖不显式声明
- Set 间循环依赖

允许：
- 同一层级内无 `after/before` 约束的系统可被 Bevy 自动并行化
- 使用 `bevy_mod_debugdump` 导出依赖图进行审查

> **优化来源**: docs/01-architecture/schedules_design.md

---

## 规则18：System 粒度与单责任

必须：
- 单系统 Query/Resource 参数上限为 8 个，超过必须拆分
- 每个 System 遵循"一个系统，一个职责"原则
- System 只负责调度，业务规则放在领域模块中

禁止：
- 单系统参数超过 8 个（职责混杂，编译时间长）
- System 包含属于领域模块的业务规则（违反 Logic/Presentation 分离）
- System 存储任何状态（System 是纯逻辑单元）

优先检查：
- Query/Resource 参数是否超过 8 个
- System 是否包含多个不相关的职责
- 业务逻辑是否下沉到领域模块

> **优化来源**: docs/01-architecture/system_design_rules.md

---

## 规则19：System 读写分离

必须：
- 读操作与写操作系统必须分离，最大化 Bevy 调度器的并行空间
- 不同 Component 的读系统可被 Bevy 自动并行
- 使用 `Changed<T>` 过滤器只处理变更的 Entity

禁止：
- 读系统和写系统混在同一 System（缩小并行空间）
- 单个 `ResMut` 阻塞整个 Set

允许：
- 读系统之间读取不同 Component 时可并行
- 使用 `#[inline(always)]` 标注纯计算方法
- 拆分为多个细粒度 Resource 避免阻塞

> **优化来源**: docs/01-architecture/system_design_rules.md

---

## 规则20：System 命名规范

必须：
- System 命名遵循 `[schedule]_[verb]_[object]_system` 模式
- verb 使用明确的动作词（apply/check/calculate/spawn/cleanup）
- suffix 固定为 `_system`

禁止：
- 命名不含 `_system` 后缀
- 使用含糊不清的名称（如 handle_stuff、process）
- 使用过于冗长的名称

示例：
- `update_apply_buff_damage_system` — 正确命名
- `check_unit_death_system` — 正确命名
- `calculate_movement_range_system` — 正确命名

> **优化来源**: docs/01-architecture/system_design_rules.md

---

## 规则21：run_if 状态门控

必须：
- 使用 `run_if(in_state(...))` 而非手动状态检查
- 所有门控必须通过 `run_if` 在调度阶段完成
- Bevy 在图构建阶段直接裁剪不需要的系统，零运行时开销

禁止：
- 在 System 内部手动检查状态（如 `if *phase.get() != TurnPhase::SelectUnit { return; }`）
- 手动检查导致每帧执行 System 后 Query 遍历才 return

允许：
- 组合条件：`.run_if(in_state(AppState::InGame).and_then(in_state(TurnPhase::ExecuteAction)))`
- 谓词条件：`.run_if(|world: &World| { ... })`
- 资源条件：`.run_if(resource_exists::<BattleState>())`

> **优化来源**: docs/01-architecture/system_design_rules.md

---

## 规则22：并行与串行执行策略

必须：
- 确定性战斗结算放在 FixedUpdate 中（固定 10Hz tick）
- Update 只处理输入和 UI 表现
- 效果管线（Generate→Modify→Execute）使用 EffectPipelineSchedule 实现严格串行

禁止：
- 所有系统串行执行（浪费并行能力）
- 确定性逻辑放在 Update 而非 FixedUpdate（帧率波动影响数值）
- 确定性逻辑读取 `Time::delta_seconds()` 等浮点时间

允许：
- 不同 Entity 的 AI 系统可并行
- 输入处理系统互相独立可并行
- ViewModel 更新系统互不依赖可并行

> **优化来源**: docs/01-architecture/schedules_design.md

---

## 规则23：.chain() 使用限制

必须：
- 单系统管道（无其他系统可并行）可使用 `.chain()`
- 多系统管道必须使用 `after()`/`before()` 显式约束

禁止：
- 在多系统管道中使用 `.chain()`（强制串行，破坏 Bevy 多线程并行优势）
- `.chain()` 导致同屏 20+ 单位结算时帧率线性暴跌

允许：
- 单系统管道使用 `.chain()` 保证顺序
- 使用 EffectPipelineSchedule 替代 `.chain()` 实现严格串行

> **优化来源**: docs/01-architecture/schedules_design.md

---

## 规则24：组件命名规范 [NEW since v2.2]

> **优化来源**: docs/01-architecture/component_design_rules.md

必须：
- Marker 使用 `IsXxx` 前缀（阵营/属性相关）或直接命名（临时状态）
- Data 使用 `EntityConcept` 模式（`GridPosition`、`UnitName`）
- Status 使用 `EntityAction` 模式（`MovingUnit`、`CastingSkill`）
- Definition 使用 `EntityDef` / `EntityConfig` 后缀（`UnitClassDef`、`SkillConfig`）

禁止：
- 使用 `XxxManager`、`XxxData`、`XxxState` 等 OOP 风格命名
- Marker Component 携带任何字段
- Data Component 命名不遵循 `EntityConcept` 模式

允许：
- UI Marker 使用 `XxxBg` / `XxxFg` 模式（`HpBarBg`、`HpBarFg`）
- 临时状态 Marker 不加 `Is` 前缀（`Dead`、`Selected`、`Frozen`）

---

## 规则25：组件单一职责与拆分 [NEW since v2.2]

> **优化来源**: docs/01-architecture/component_design_rules.md

必须：
- 一个 Component = 一个数据关注点
- 超过 8 个字段时评估是否需要拆分（参考阈值，非硬性规则）
- 字段分为明显不同关注点时，即使不足 8 字段也必须拆分
- 字段生命周期不同时必须拆分（如逻辑坐标 vs 表现坐标）

禁止：
- 跨领域的"上帝组件"（破坏模块边界）
- 字段修改频率差异大的组件不拆分（高频变更 vs 低频变更）

允许：
- 极端性能场景且经团队评审后少量字段合并（需记录理由）
- 同一关注点的字段合并（即使超过 8 个）

---

## 规则26：序列化组件版本字段 [NEW since v2.2]

> **优化来源**: docs/01-architecture/component_design_rules.md

必须：
- 所有可序列化 Component 携带 `version: u32` 字段
- 新增字段（有默认值）：小版本 +1
- 删除字段或类型变更：大版本 +1
- Instance 组件引用 Definition ID，不保存 Definition 本身

禁止：
- 可序列化 Component 无 version 字段
- Definition 组件在运行时修改
- Instance 组件直接嵌入 Definition 数据

允许：
- 序列化使用 `#[reflect(Serialize, Deserialize)]` 属性
- 数据迁移时使用默认值填充新字段

---

## 规则27：SRPG 专项组件设计 [NEW since v2.2]

> **优化来源**: docs/01-architecture/component_design_rules.md

必须：
- Status Component 包含 `source: Entity`（施加者）和 `duration: u32`（剩余回合）
- 坐标类 Data Component 区分逻辑坐标（`IVec2`）和表现坐标（`Vec3`）
- 引入脏标记 Marker（`NeedsPathfinding`、`NeedsVisionUpdate`）优化重计算
- 技能/职业/特质的静态配置统一使用 `XxxDef` 命名

禁止：
- 逻辑坐标和表现坐标混用（导致 `Changed<GridPosition>` 误触发）
- 寻路/视线计算不使用脏标记（每帧全量计算）

允许：
- 逻辑坐标用于寻路/计算（低频变更），表现坐标用于动画插值（每帧变更）
- 脏标记 System 只处理带有该 Marker 的实体，计算完后移除

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

## Schedule 执行顺序管线

```
每帧执行：PreUpdate → InputSchedule → Update → LogicSchedule → PostUpdate → PresentationSchedule → Last
```

### Step1：PreUpdate（输入预处理）

输入：原始输入事件
处理：Bevy 内置输入预处理
输出：标准化的输入事件
禁止：在 PreUpdate 中执行游戏逻辑

### Step2：InputSchedule（游戏命令转换）

输入：标准化的输入事件
处理：读取原始输入，转换为游戏命令
输出：UiCommand Message
禁止：在 InputSchedule 中修改游戏状态

### Step3：Update（核心业务逻辑）

输入：游戏命令和当前状态
处理：按 TurnPhase 状态门控执行业务系统
输出：状态变更和 Effect 数据
禁止：在 Update 中执行 UI 渲染

### Step4：LogicSchedule（复杂逻辑编排）

输入：Effect 数据
处理：Effect Pipeline、战斗结算等需要严格顺序的逻辑
输出：最终状态变更
禁止：在 LogicSchedule 中执行输入处理

### Step5：PostUpdate → PresentationSchedule（表现层）

输入：最终状态变更
处理：UI 更新、动画、音效
输出：视觉呈现
禁止：在 PostUpdate 中修改游戏状态

> **优化来源**: docs/01-architecture/schedules_design.md

---

## SystemSet 层级管线

```
InputSet → CommandSet → LogicSet → EffectSet → ViewModelSet → UISet
```

### Step1：InputSet（输入处理）

输入：用户输入
处理：键盘、鼠标、触摸输入分别处理
输出：输入事件
禁止：InputSet 内部系统之间存在依赖

### Step2：CommandSet（命令分发）

输入：输入事件
处理：将输入事件转换为游戏命令
输出：游戏命令 Message
禁止：CommandSet 在 InputSet 之前执行

### Step3：LogicSet（业务逻辑）

输入：游戏命令
处理：TurnSet → MovementSet → CombatSet → BuffSet → AISet 顺序执行
输出：Effect 数据
禁止：LogicSet 在 CommandSet 之前执行

### Step4：EffectSet（效果管线）

输入：Effect 数据
处理：EffectGenerateSet → EffectModifySet → EffectExecuteSet 严格串行
输出：最终状态变更
禁止：EffectSet 在 LogicSet 之前执行

### Step5：ViewModelSet（视图模型更新）

输入：最终状态变更
处理：BattleViewModelSet、BuffViewModelSet、TurnViewModelSet 分别更新
输出：ViewModel 数据
禁止：ViewModelSet 在 EffectSet 之前执行

### Step6：UISet（UI 渲染）

输入：ViewModel 数据
处理：BattleUISet、BuffUISet、DebugUISet 分别渲染
输出：视觉呈现
禁止：UISet 在 ViewModelSet 之前执行

> **优化来源**: docs/01-architecture/schedules_design.md

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

禁止：Plugin 隐式依赖其他 Plugin

原因：Resource 未就绪时运行会 panic，隐式依赖导致模块间耦合不可见。

违反后果：Plugin 依赖的 Resource 未注册导致运行时 panic，无法独立测试。

> **优化来源**: docs/01-architecture/plugin_contract_rules.md

---

禁止：Plugin 间循环依赖

原因：循环依赖导致编译错误、难以维护，破坏模块化设计。

违反后果：Bevy panic，所有相关 Plugin 无法加载。

> **优化来源**: docs/01-architecture/plugin_contract_rules.md

---

禁止：系统无 Set 归属直接注册

原因：无法控制执行顺序，导致竞态条件和不确定性。

违反后果：系统执行顺序不确定，产生间歇性 Bug。

> **优化来源**: docs/01-architecture/schedules_design.md

---

禁止：在多系统管道中使用 .chain()

原因：`.chain()` 强制所有系统串行执行，彻底破坏调度器的多线程并行能力。

违反后果：同屏 20+ 单位结算时帧率线性暴跌。

> **优化来源**: docs/01-architecture/schedules_design.md

---

禁止：System 包含领域逻辑

原因：System 是 ECS 的"调度层"，业务规则放在领域模块中作为纯函数实现，可独立测试和复用。

违反后果：逻辑绑死在 System 中，无法被其他模块调用，需要启动 Bevy 运行时才能测试。

> **优化来源**: docs/01-architecture/system_design_rules.md

---

禁止：System 间直接函数调用

原因：直接函数调用让 Bevy 无法分析系统间依赖，两个本可并行的系统被迫串行。

违反后果：破坏 Bevy 并行调度，无法单独测试 System，重构困难。

> **优化来源**: docs/01-architecture/system_design_rules.md

---

禁止：在 System 内部手动检查状态

原因：每帧执行 System，Query 遍历后才 return，效率低于 run_if 在图构建阶段直接裁剪。

违反后果：每帧浪费一次 Query 遍历，状态逻辑隐藏在 System 内部难以追踪。

> **优化来源**: docs/01-architecture/system_design_rules.md

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
- 消息类型是否已在 `docs/01-architecture/README.md` 注册表中登记
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
- `docs/01-architecture/README.md` 注册表是否同步更新
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

---

## 如果新增 Definition Component [NEW since v2.0]

允许：
- 命名以 Def 或 Config 结尾（如 UnitClassDef、SkillConfig）
- 作为 Resource 或挂载在预制体 Entity 上
- 运行时只读（&T 引用）

禁止：
- Definition Component 包含运行时可变状态
- System 中对 Definition Component 取 &mut 引用
- 命名不遵循 Def/Config 后缀约定

优先检查：
- 命名是否以 Def 或 Config 结尾
- 是否只通过 Res\<T\> 读取
- 是否包含可变字段（如有则需拆分为 Definition + Instance）

---

## 如果修改 Data Component 辅助方法 [NEW since v2.0]

允许：
- 纯读取辅助方法（is_alive/ratio/max_limit）
- #[inline(always)] 标注

禁止：
- 业务逻辑方法（take_damage/apply_buff）
- 辅助方法访问外部状态

优先检查：
- 方法是否修改 &mut self
- 方法是否访问 World/AssetServer/其他 Component
- 方法是否为纯计算/纯读取

---

## 如果修改 Plugin 依赖 [NEW since v2.1]

允许：
- 新增 Plugin 的显式依赖声明
- 调整 Plugin 初始化顺序（遵循 Shared → Infra → Core → Content → UI）
- 将隐式依赖转换为显式依赖

禁止：
- 删除 Plugin 的依赖声明（导致运行时 panic）
- 引入循环依赖（A 依赖 B 且 B 依赖 A）
- 跨层注册（UI Plugin 注册 Core 系统的 Message）

优先检查：
- 依赖方向是否符合分层架构
- 是否有隐式依赖（依赖其他 Plugin 的 Resource 但未声明）
- 依赖变更后是否需要调整初始化顺序

> **优化来源**: docs/01-architecture/plugin_contract_rules.md

---

## 如果新增 SystemSet [NEW since v2.1]

允许：
- 在三层嵌套结构中新增 Set（阶段级 → 功能域级 → 管线级）
- 使用 `after()`/`before()` 声明 Set 间排序
- 为 Set 添加 `run_if` 状态门控

禁止：
- 新增 Set 但不声明与现有 Set 的排序关系
- 使用 `.chain()` 强制串行（多系统管道必须用 `after()`/`before()`）
- 新增 Set 导致循环依赖

优先检查：
- Set 是否归属到正确的父 Set
- Set 间排序是否使用 `after()`/`before()` 而非 `.chain()`
- Set 的 `run_if` 条件是否正确

> **优化来源**: docs/01-architecture/schedules_design.md

---

## 如果修改系统执行顺序 [NEW since v2.1]

允许：
- 调整 SystemSet 内部的 `after()`/`before()` 约束
- 新增系统到现有 SystemSet
- 为系统添加 `run_if` 条件

禁止：
- 移除 Effect Pipeline 三步的严格顺序
- 将 EffectSet 放在 LogicSet 之前
- 使用 `.chain()` 替代 `after()`/`before()`（多系统管道）

优先检查：
- 效果管线三步顺序是否保持（Generate → Modify → Execute）
- LogicSet 是否仍在 EffectSet 之前
- 是否有系统无 Set 归属直接注册

> **优化来源**: docs/01-architecture/schedules_design.md

---

## 如果修改 System 粒度或职责 [NEW since v2.1]

允许：
- 拆分参数超过 8 个的 System
- 将 System 中的领域逻辑下沉到领域模块
- 将读写混在一起的 System 拆分为读系统和写系统

禁止：
- 合并已经拆分好的细粒度 System
- 将领域逻辑上移到 System 中
- 在 System 中存储状态

优先检查：
- System 的 Query/Resource 参数是否超过 8 个
- System 是否包含领域逻辑（应下沉到领域模块）
- System 是否遵循单责任原则

> **优化来源**: docs/01-architecture/system_design_rules.md
