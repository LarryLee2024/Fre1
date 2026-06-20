# 系统间通信深度解析 — 四级通信 + 命令系统 + 事件链

> 一个 50 万行的 Bevy ECS 项目，代码分布在 15 个能力领域 + 15 个业务领域 + 6 个基础设施模块 + UI 四层。无数个 System、Observer、Component、Resource 需要相互通信——但又不能直接「import 对方的类型然后调用方法」，因为那会毁掉模块独立性。
>
> 读完本文，你会知道：为什么选这四种机制（Hook/Trigger/Observer/Message）而不是其他方案、它们各自解决什么问题、在实际代码中长什么样、以及一条消息从产生到你看到它的完整路径。

---

## 目录

1. [通信问题与设计原则](#1-通信问题与设计原则)
2. [整体架构图](#2-整体架构图)
3. [第一层：Hook — 组件生命周期](#3-第一层hook--组件生命周期)
4. [第二层：Trigger — 同 Feature 事件链](#4-第二层trigger--同-feature-事件链)
5. [第三层：Observer — 跨领域通信首选](#5-第三层observer--跨领域通信首选)
6. [第四层：Message — 全局广播（备选）](#6-第四层message--全局广播备选)
7. [命令系统：CommandQueue + GameCommand](#7-命令系统commandqueue--gamecommand)
8. [UI Action 模式](#8-ui-action-模式)
9. [Integration 模块 — Domain ↔ Capability 的桥梁](#9-integration-模块--domain--capability-的桥梁)
10. [日志 Observer 系统 — 通信的下游消费者](#10-日志-observer-系统--通信的下游消费者)
11. [完整数据流：三个端到端示例](#11-完整数据流三个端到端示例)
12. [通信规则速查](#12-通信规则速查)

---

## 1. 通信问题与设计原则

### 1.1 在 ECS 项目中，「通信」到底指什么？

传统面向对象项目里，模块 A 调用模块 B 的方法就行了。ECS 不同——你不能写 `some_domain.do_something()`，因为：

1. **ECS 没有方法调用** — Component 只有数据，System 只有逻辑，System 不「属于」任何模块
2. **模块间耦合红线** — Capabilities 与 Domains 之间、Domain 与 Domain 之间都不能直接 import 内部类型
3. **数据流不是调用链** — 一个效果（Effect）可能同时触发伤害计算、护盾吸收、Buff 刷新、UI 更新、日志记录——如果每个响应都写成一个显式调用，系统间依赖会像蜘蛛网一样

所以 Fre 把一个「模块 A 想告诉模块 B 某事」的需求，分解成两种不同的机制：

| 需求 | 用哪种通信？ |
|------|-------------|
| "加上了 Dead 标签后自动移除 Movement 组件" | Hook（组件生命周期） |
| "伤害 → 计算减伤 → 触发护盾 → 触发吸血 → 判定死亡" | Trigger（同一 Feature 内的链式响应） |
| "战斗结束了，Quest 需要检查任务进度" | Observer（跨 Domain 的通知） |
| "按了快捷键 F5，立刻存档" | Command（命令入队 + 执行） |

### 1.2 四条总体原则

这些原则是所有通信机制的上层约束，写在了宪法 §6.3 和 ADR-002 里：

```
原则一：最小粒度优先
  用最简单的机制解决问题。在四种通信方式中，从上到下选第一个能满足需求的：
  Hook > Trigger > Observer > Message

原则二：领域事件是唯一业务事实源
  一个「角色升级」事件发生后，日志要记录、成就要检查、任务追踪要更新、
  回放系统要存档——这些下游消费者都订阅同一个事件，而不是各自轮询。

原则三：写操作用事件，读操作用 Query API
  Domain A 想修改 Domain B 的状态？ ➔ 发事件
  Domain A 想查询 Domain B 的状态？ ➔ 调用公开的 Query API（如 is_quest_completed()）

原则四：系统互调禁令
  禁止 System A 内部直接调用 System B 的函数。系统间通信必须通过
  Hook→Trigger→Observer→Message 进行。
```

---

## 2. 整体架构图

```
                                ┌──────────────────────────────────────┐
                                │         用户输入 / AI / Replay        │
                                │                │                      │
                                │                ▼                      │
                                │     InputState + InputMap             │
                                │         ↓ (翻译)                       │
                                │     GameCommand                       │
                                │         ↓ (入队)                       │
                                │     CommandQueue (Resource)           │
                                └──────────────────┬───────────────────┘
                                                   │
                                                   ▼
          ┌───────────────────────────────────────────────────────────┐
          │                  四层通信系统                              │
          │                                                           │
          │  ┌────────────┐  ┌───────────┐  ┌───────────┐  ┌────────┐│
          │  │  Level 1   │  │  Level 2  │  │  Level 3  │  │Level 4 ││
          │  │   Hook     │  │  Trigger  │  │  Observer │  │Message ││
          │  │ (生命周)    │  │ (链式)     │  │ (跨域)    │  │(广播)  ││
          │  └─────┬──────┘  └─────┬─────┘  └─────┬─────┘  └───┬────┘│
          │        │               │               │             │     │
          │        ▼               ▼               ▼             ▼     │
          │  Component         Effect Chain    Domain Events   Global  │
          │  Lifecycle         dmg→shield→                          │
          │  (on_add)          lifeleech→die    跨域事件           │
          │                                                           │
          └───────────────────────────────────────────────────────────┘
                            │                    │
                            ▼                    ▼
              ┌─────────────────────┐  ┌──────────────────┐
              │   业务逻辑 System    │  │  下游消费者       │
              │   (combat/quest/    │  │  LogObserver × 56 │
              │    inventory/...)   │  │  Replay           │
              │                    │  │  MetricsCollector  │
              └─────────────────────┘  │  UI (ViewModels)  │
                                        └──────────────────┘
```

关键路径总结：

```
输入路径：
  键盘/鼠标 → ButtonInput → InputMap → InputState → GameCommand → CommandQueue

域内通信路径：
  Capability System → trigger(Event) → Observer(同 Capability 内) → Effect

跨域通信路径：
  Domain System → trigger(Event) → Observer(infra/logging/*) → MetricsCollector + tracing
  Domain System → trigger(DomainEvent) → Observer(跨 Domain) → 业务逻辑

UI 路径：
  按钮点击 → ButtonClicked(Observer) → Screen.on_battle_button_clicked → BattleAction
```

---

## 3. 第一层：Hook — 组件生命周期

### 3.1 它解决什么问题

当你往 Entity 上加了一个 `Dead` 组件，你希望这个 Entity 的 `Movement` 组件自动被移除。这种「A 组件出现时自动处理 B 组件」的场景，不适合用完整的 Observer 去监听——太重量级了。Hook 就是为此设计的。

### 3.2 在代码中的样子

Hook 是通过 `#[component(on_add, on_remove)]` 声明的，在组件定义时直接绑定：

```rust
// Bevy 0.19 原生支持
#[derive(Component)]
#[component(on_add = Dead::on_add_dead)]
struct Dead;

impl Dead {
    fn on_add_dead(mut commands: Commands, entity: Entity) {
        // Dead 添加时自动移除移动相关组件
        commands.entity(entity).remove::<Movement>();
    }
}
```

### 3.3 规则

- ✅ **只能用于轻量副作用** — 几个命令调用就结束
- ❌ **禁止承载复杂业务逻辑** — 做不了条件判断、事件链、跨域通知
- ❌ **禁止在 Hook 中调用 `trigger()`** — 那不是生命周期行为

### 3.4 项目中的使用

当前 Fre 代码库中，Hook 的使用较少——因为很多「A 出现时触发 B」的逻辑被实现了 Observer。Hook 更多是一个「未来应该用但还没大规模迁移」的模式。在旧代码（`ai_ignore_this_dir`）中有 `Dead::on_add_dead` 用例。

---

## 4. 第二层：Trigger — 同 Feature 事件链

### 4.1 它解决什么问题

假设你在实现战斗伤害系统：攻击命中 → 计算减伤 → 触发护盾 → 触发吸血 → 判定死亡。这些步骤都在一个 Feature（combat）内部，但需要链式触发——上一步的结果影响下一步的输入。

Trigger 就是用在同一 Feature 内部的链式响应。它绑定了一个触发 Entity，Observer 可以在同一个 World 中按序响应。

### 4.2 在代码中的样子

Capability 领域的内部事件定义，全部使用 `trigger()` + Observer 模式：

```rust
// ─── 事件定义（capability 内）───
// src/core/capabilities/modifier/events.rs
#[derive(Event, Debug, Clone, Reflect)]
pub struct ModifierApplied {
    pub target: Entity,
    pub modifier_id: String,
}

// ─── 触发（capability system 内）───
// src/core/capabilities/modifier/mechanism/lifecycle.rs
fn apply_modifier(mut commands: Commands, modifier: ModifierComponent) {
    // ... 应用 modifier 的逻辑 ...
    commands.trigger(ModifierApplied {
        target: entity,
        modifier_id: modifier.id.clone(),
    });
}

// ─── 响应（同一 capability 内）───
// src/core/capabilities/modifier/mechanism/systems/modifier_system.rs
fn on_modifier_applied(trigger: On<ModifierApplied>, mut query: Query<&mut SomeComponent>) {
    let evt = trigger.event();
    // 响应 modifier 应用后的逻辑
}
```

这个模式在代码库里非常一致。几乎每个 Capability 和 Domain 都有自己的 `events.rs`，定义内部事件，然后在 `plugin.rs` 中用 `app.add_observer(...)` 注册。

### 4.3 与 Observer 的区别

| | Trigger（域内） | Observer（跨域） |
|---|---|---|
| 作用范围 | 同一 Feature 内部 | 跨 Feature、跨 Domain |
| 携带 Entity | `trigger(entity, event)` 可选 | 与 `commands.trigger()` 相同 |
| 性能消耗 | 轻量 | 比直接 System 调用重 |
| 不可滥用？ | 可正常使用 | 高频场景禁止 |

### 4.4 主要 Trigger 使用场景（代码库现状）

| 领域（Capability） | 事件 | 触发时机 |
|---|---|---|
| `tag` | `TagAdded`, `TagRemoved`, `TagHierarchyChanged` | 标签增删改 |
| `modifier` | `ModifierApplied`, `ModifierRemoved` | 数值修改器增删 |
| `attribute` | `AttributeInitialized`, `AttributeChanged`, `AttributeClamped` | 属性变更 |
| `event` (能力) | `EventPublished`, `EventCycleDetected`, `EventDelivered` | 事件总线循环 |
| `ability` | `AbilityActivated`, `AbilityCancelled`, `AbilityCompleted` | 技能生命周期 |
| `cue` | `CueTriggered`, `CueSuppressed` | 表现信号触发/抑制 |
| `targeting` | `TargetSelected`, `NoValidTarget` | 目标选择结果 |
| `gameplay_context` | `ContextCreated`, `ContextValidationFailed` | 上下文创建/校验 |
| `condition` | 各 Observer 监听 `TagAdded`/`TagRemoved`/`AttributeChanged` 做条件重算 | 条件依赖变更 |
| `effect` | `EffectApplied`, `EffectRemoved`, `EffectTicked` | 效果生命周期 |

---

## 5. 第三层：Observer — 跨领域通信首选

### 5.1 它解决什么问题

这是 Fre 项目中最核心的通信机制。当 Combat 领域说"战斗结束了"，Quest 领域需要知道（检查任务进度），Progression 领域需要知道（结算战斗经验），Logger 需要知道（记录日志）——这就是 Observer 的场景。

Observer 是 Bevy 0.19 原生引入的机制，`EventWriter`/`EventReader` 的进化版。它的核心区别是：Observer 不需要手动 `app.add_event::<T>()` + 每帧 `EventReader<T>::read()`，而是通过 `commands.trigger(T)` 触发后自动调度。

### 5.2 跨领域事件白名单

所有跨 Domain 的事件，需要在 `src/core/events.rs` 中声明——这是**唯一**的跨 Domain 事件白名单：

```rust
// src/core/events.rs — 跨域共享事件白名单
// 每个事件都注释了"谁发射"、"谁消费"

/// 全局回合结束事件
/// 发射方: combat pipeline (TurnSettlement 阶段)
/// 消费方: terrain (表面恢复), effect (DOT tick), 其他回合感知 Domain
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
pub struct TurnEnded {
    pub unit: Entity,
}

/// 全局回合开始事件
/// 发射方: combat pipeline (Initiative 阶段)
/// 消费方: 任何需要回合开始时触发逻辑的 Domain
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
pub struct TurnStarted {
    pub unit: Entity,
}

/// 全局战斗开始事件
/// 发射方: combat battle_start_system
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
pub struct BattleStarted;

/// 全局战斗结束事件
/// 发射方: combat victory_system
/// 消费方: 任何需要战斗结束时结算的 Domain
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
pub struct BattleEnded {
    pub victory: bool,
}
```

### 5.3 Domain 内部事件的 Observer 注册

每个 Domain 在自己的 `plugin.rs` 中注册 Observer。例如 inventory domain：

```rust
// src/core/domains/inventory/plugin.rs
impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_item_acquired)
           .add_observer(on_equip_item)
           .add_observer(on_item_used);
    }
}
```

对应的 Observer 函数：

```rust
// src/core/domains/inventory/systems/inventory_system.rs
pub(crate) fn on_item_acquired(
    trigger: On<ItemAcquired>,
    mut query: Query<&mut Inventory>,
) {
    let evt = trigger.event();
    // 检查库存是否有空间
    // 如果空间够：添加到背包
    // 如果空间不够：发射 InventoryFull 事件
}
```

### 5.4 Capability 领域的 Observer 注册

Capability 领域同样按照这个模式。例如 tag capability：

```rust
// src/core/capabilities/tag/plugin.rs
impl Plugin for TagPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_tag_added)
           .add_observer(on_tag_removed);
    }
}
```

### 5.5 Observer 的条件守卫

Observer 支持 `run_if()`，可以避免在函数内部写大段 if 判断：

```rust
app.add_observer(on_something_expensive)
   .run_if(not_in_debug_mode);  // 只在非调试模式触发
```

### 5.6 Observer 的使用禁忌

来自宪法 §6.4 和 ADR-002：

- ❌ **禁止高频使用**：每帧执行 10 次以上的逻辑，直接使用 System
- ❌ **禁止嵌套过深**：无递归深度保护可能导致无限循环，应设置 `MAX_OBSERVER_DEPTH`
- ❌ **禁止旧 EventWriter/EventReader**：新代码一律使用 `trigger()` + Observer
- ❌ **禁止 Observer 中写业务逻辑**（这里是日志 Observer 专用的规则）

---

## 6. 第四层：Message — 全局广播（备选）

### 6.1 它解决什么问题

Message 是四级通信的最底层——当同 Feature 内的 Trigger 和跨域的 Observer 都不适用时，才考虑 Message。Message 使用 Bevy 的 `Event<T>` + `trigger()` 模式。

在 Bevy 0.19 之后，Message 实际上已经被 Observer 取代了——`EventWriter<T>`/`EventReader<T>` 在新代码中被禁止，使用 `commands.trigger(T)` + `On<T>` Observer 即可覆盖所有需求。

### 6.2 在代码中的样子

本质上 `commands.trigger(T)` + `app.add_observer(on_t)` 既充当了 Trigger 也充当了 Message。区别在于使用意图：

- **Trigger 用法**：`trigger(CombatDamage { ... })` — 我知道谁会响应，这就是同一 Feature 内的链
- **Message 用法**：`trigger(TurnEnded { ... })` — 我不知道谁会响应，可能有很多人订阅

```rust
// 一个 Message 风格的跨域事件（在核心 events.rs 中）
commands.trigger(BattleEnded { victory: true });

// 多个 Observer 各自响应（完全解耦）
// 在 combat 领域：有的 Observer 做结算
// 在 quest 领域：有的 Observer 检查任务
// 在 progression 领域：有的 Observer 给经验
// 在 logging 领域：有的 Observer 写日志
```

---

## 7. 命令系统：CommandQueue + GameCommand

除了四层通信机制之外，Fre 还有一套独立的**命令系统**。它不是用于模块间通信的，而是用于从外部（玩家输入/AI决策/Replay）向内部（游戏逻辑）传递动作。

### 7.1 为什么要单独搞一个 Command？

因为玩家的每次操作都需要被录制到 Replay 里，而 ECS 的事件（Trigger/Observer）是 Bevy 内部调度的，不直接暴露为可序列化的命令。所以 Fre 定义了一个统一的 `GameCommand` 枚举：

```rust
// src/core/capabilities/runtime/command/foundation/types.rs
/// 业务命令——所有玩家/AI/Replay 操作的统一枚举。
#[derive(Debug, Clone, PartialEq)]
pub enum GameCommand {
    // ── Tactical ──
    MoveUnit { unit_id: String, path: Vec<String> },
    Wait { unit_id: String },
    // ── Combat ──
    Attack { attacker_id: String, target_id: String, ability_slot: Option<u32> },
    CastSpell { caster_id: String, spell_def_id: String, target_id: String },
    UseItem { user_id: String, item_instance_id: String, target_id: Option<String> },
    // ── Turn ──
    EndTurn { unit_id: String },
    // ── Meta ──
    OpenMenu, SaveGame, LoadGame,
}
```

### 7.2 CommandQueue：统一命令入口

命令不是直接执行的，而是统一入队到一个 `CommandQueue` Resource——这是玩家、AI 和 Replay 提交命令的唯一入口：

```rust
// src/core/capabilities/runtime/command/foundation/values.rs
#[derive(Resource, Debug, Clone)]
pub struct CommandQueue {
    pending: Vec<GameCommand>,    // 待处理命令
    history: Vec<RecordedCommand>, // 历史记录（用于录制）
    frame_number: u64,
}

impl CommandQueue {
    pub fn push(&mut self, command: GameCommand) -> Result<(), CommandError>;
    pub fn push_recorded(&mut self, command: GameCommand, source: CommandSource) -> Result<(), CommandError>;
    pub fn drain(&mut self) -> Vec<GameCommand>;
}
```

### 7.3 命令来源追踪

每条命令都标记了来源：

```rust
pub enum CommandSource {
    Player,  // 玩家键盘/鼠标操作
    AI,      // AI 决策
    Replay,  // 回放系统
    System,  // 系统内部
}
```

`push_recorded()` 会自动把命令和来源一起记录到历史中，供 Replay 和调试使用。

### 7.4 完整的输入→命令流程

```
Step 1: 原始输入采集
────────────────────────────────
  collect_input_system (PreUpdate)
  采集 ButtonInput<KeyCode>, ButtonInput<MouseButton>
  通过 InputMap 翻译为 InputAction

Step 2: 语义化输入状态
────────────────────────────────
  InputState Resource (更新中)
  • pressed_actions: Vec<InputAction>
  • just_pressed_actions: Vec<InputAction>
  • just_released_actions: Vec<InputAction>

Step 3: 元命令直接入队
────────────────────────────────
  process_meta_commands system
  QuickSave → GameCommand::SaveGame → CommandQueue.push()
  OpenMenu → GameCommand::OpenMenu → CommandQueue.push()

Step 4: 业务命令由 Domain 构造入队
────────────────────────────────
  Domain 系统读取 InputState + 当前游戏状态
  → 构造 GameCommand（如 MoveUnit { unit_id, path }）
  → CommandQueue.push()

Step 5: 命令执行
────────────────────────────────
  后续 System 读取 CommandQueue 并 drain() 执行
```

### 7.5 命令系统 vs 四层通信

```
                    命令系统                        四层通信
              ┌──────────────────┐         ┌──────────────────┐
  外部来源    │  玩家/AI/Replay  │         │  System ↔ System │
  方向        │  外部 → 内部     │         │  内部 ↔ 内部     │
  目的地      │  CommandQueue    │         │  Observer 调度   │
  是否录制    │  是（Replay）     │         │  不直接录制      │
  使用场景    │  回合操作、存档   │         │  域内事件链、跨域 │
```

---

## 8. UI Action 模式

### 8.1 它解决什么问题

UI 按钮需要向游戏逻辑传递操作意图——比如「我点了'结束回合'按钮，游戏应该结束当前回合」。但 UI 是表现层，不能直接调用 Domain 的 System。所以需要一种机制：UI 给按钮打上「动作标签」，Observer 识别标签后执行对应逻辑。

### 8.2 在代码中的样子

每个 widget 或 screen 定义一个 Action 枚举：

```rust
// src/ui/screens/battle/systems.rs
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub enum BattleAction {
    EndTurn,
}

// src/ui/widgets/skill_slot/components.rs
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub enum SkillSlotAction {
    Use,
}
```

然后在 Observer 中监听 UI 原语的 `ButtonClicked` 事件，通过 Query 获取 Action 组件来识别意图：

```rust
// src/ui/screens/battle/systems.rs
/// Observer: handles battle button clicks
pub fn on_battle_button_clicked(on: On<ButtonClicked>, query: Query<&BattleAction>) {
    let entity = on.event().entity;  // 被点击的按钮实体
    let Ok(action) = query.get(entity) else {
        return;  // 不是战斗按钮，忽略
    };
    match action {
        BattleAction::EndTurn => {
            // 这里可以触发 GameCommand::EndTurn 或 trigger(DomainEvent)
        }
    }
}
```

### 8.3 UI 动作的完整路径

```
按钮点击（玩家鼠标/触摸）
    │
    ▼
ButtonClicked 事件（UI 原语发射）
    │  (commands.trigger(ButtonClicked { entity, interaction_type }))
    ▼
Observer 调度（screen/widget 注册的 Observer）
    │  (On<ButtonClicked> + Query<&Action>)
    ▼
Action 匹配
    │  BattleAction::EndTurn
    ▼
后续处理（两种路径之一）
    ├─ 直接 trigger(DomainEvent) — 如果已经在 Core 层
    └─ GameCommand → CommandQueue.push() — 如果需要录制
```

---

## 9. Integration 模块 — Domain ↔ Capability 的桥梁

### 9.1 它解决什么问题

Domain 需要调用 Capabilities 的能力——比如 combat domain 需要读取 tag domain 中的标签、使用 modifier domain 来施加修改器。但宪法规定：Domain 不能直接 import Capabilities 的内部 Component 来访问字段。

所以每个 Domain 都有一个 `integration/` 子模块，作为唯一的跨层通信入口。

### 9.2 在代码中的样子

```
domains/combat/
├── integration/
│   ├── mod.rs              # re-export 所有子模块
│   ├── movement/
│   │   ├── facade.rs       # 业务语义 API
│   │   ├── types.rs        # View Types（Domain 自己的类型，包装 Capabilities 原始类型）
│   │   └── system_param.rs # Bevy SystemParam
│   ├── terrain/
│   │   └── ...
│   └── targeting/
│       └── ...
```

### 9.3 Facade + View Types 模式

```rust
// Domain 自己的类型，不直接暴露 Capabilities 的内部结构
pub struct MovementView {
    pub tiles: Vec<TilePosition>,
    pub mp_cost: u32,
}

// SystemParam 封装 Capabilities 查询依赖
#[derive(SystemParam)]
pub struct MovementSystemParam<'w, 's> {
    tag_query: Query<'w, 's, &'static TagSet>,
    position_query: Query<'w, 's, &'static TilePosition>,
}

impl<'w, 's> MovementSystemParam<'w, 's> {
    pub fn get_movement_for(&self, entity: Entity) -> Option<MovementView> {
        // 只读访问 Capabilities 组件，返回 Domain 自己的类型
    }
}
```

### 9.4 这个模式的意义

```
     ╔══════════════════════════════╗
     ║     Domain (仅依赖本人类型)   ║
     ║  Systems 只看到 MovementView  ║
     ║  + MovementSystemParam       ║
     ╚══════════╦═══════════════════╝
                │ Facade 方法调用
                ▼
     ╔══════════════════════════════╗
     ║   Integration (抗腐化层)     ║
     ║  View Types + SystemParam    ║
     ╚══════════╦═══════════════════╝
                │ 内部访问
                ▼
     ╔══════════════════════════════╗
     ║     Capabilities (原始组件)   ║
     ║  TagSet, TilePosition ...     ║
     ╚══════════════════════════════╝
```

这就是**数据访问**的通信路径（读操作）。写操作（修改别人状态）走事件（Observer/Message）。

---

## 10. 日志 Observer 系统 — 通信的下游消费者

### 10.1 架构分层

日志系统是通信的最重要消费者之一。它的核心设计是：**领域层发事件，基础设施层记录日志**。

```
领域事件             →   Observer 订阅   →   三路输出
(TurnStarted)              │
                            │
                            ├─ tracing::info!()    → 控制台
                            ├─ telemetry::record() → MetricsCollector (每60帧聚合)
                            └─ span 嵌套          → 调用链追踪
```

### 10.2 20 个 Observer 模块，56 个监听器

每个业务领域都有一个对应的 Logger 模块，放在 `src/infra/logging/observers/`：

| 模块 | 事件数 | 覆盖领域 |
|------|--------|---------|
| `battle_logger.rs` | 2 | 战斗开始/结束 |
| `turn_logger.rs` | 2 | 回合开始/结束 |
| `tactical_logger.rs` | 2 | 移动/位置变更 |
| `terrain_logger.rs` | 4 | 地形进入/表面变化/陷阱/地形效果 |
| `spell_logger.rs` | 1 | 法术结果 |
| `reaction_logger.rs` | 5 | 反应触发/执行/取消/机会攻击/反制 |
| `ability_logger.rs` | 4 | 技能激活/完成/取消/冷却 |
| `effect_logger.rs` | 4 | 效果施加/移除/Tick/免疫 |
| `quest_logger.rs` | 5 | 接任务/目标完成/交任务/失败/进度 |
| `progression_logger.rs` | 6 | 经验/升级/天赋/子职业/ASI/职业 |
| `inventory_logger.rs` | 5 | 获得/使用/装备/移除/战利品 |
| `economy_logger.rs` | 3 | 交易/价格/货币 |
| `crafting_logger.rs` | 2 | 铸造成功/失败 |
| `faction_logger.rs` | 4 | 声望/关系/等级/判定 |
| `party_logger.rs` | 5 | 加入/移除/换人/羁绊 |
| `camp_rest_logger.rs` | 5 | 短休/长休/中断/营地事件 |
| `narrative_logger.rs` | 5 | 对话/选择/标记/过场 |
| `summon_logger.rs` | 4 | 创建/消失/命令/槽位 |
| `content_logger.rs` | 1 | 配置热重载 |

### 10.3 一个 Observer 的解剖

```rust
// src/infra/logging/observers/progression_logger.rs
#[tracing::instrument(skip_all, target = "domain.progression", fields(
    code = ?LogCode::PRG002,
    event = "level_up",
))]
pub(crate) fn on_level_up(trigger: On<LevelUp>) {
    let event = trigger.event();
    emit_info!(
        LogCode::PRG002,
        entity = ?event.entity,
        old = event.old_level,
        new = event.new_level,
        "角色升级",
    );
}
```

这里体现了三条设计决策：
1. `#[instrument]` span 放不变量（`code`, `event`）——只要函数被调用就记录
2. `info!()` 只放变量（`entity`, `old`, `new`）——每行日志动态的部分
3. `emit_info!()` 宏内部自动完成 `telemetry::record(LogCode)` + `tracing::info!()`——保证两条路径不会分离

### 10.4 日志 Observer 的注册

所有日志 Observer 一次性在 `LoggingPlugin`（Phase 8, Infra 层）中注册：

```rust
// src/infra/logging/plugin.rs
impl Plugin for LoggingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MetricsCollector>();
        app.add_systems(Update, metrics::metrics_flush_system);

        // 注册 56 个日志 Observer
        app.add_observer(battle_logger::on_battle_started);
        app.add_observer(turn_logger::on_turn_started);
        app.add_observer(ability_logger::on_ability_activated);
        // ... 共 56 个
    }
}
```

---

## 11. 完整数据流：三个端到端示例

### 示例一：玩家操作「攻击」

```
1. 键盘输入
   玩家按下 A 键 (Attack 快捷键)
   collect_input_actions: ButtonInput → InputAction::Attack
   InputState.just_pressed_actions 增加 Attack

2. 领域系统构造命令
   CombatDomain 的系统读取 InputState + 当前选中目标
   → 构造 GameCommand::Attack { attacker_id, target_id, ability_slot: None }
   → CommandQueue.push_recorded(..., CommandSource::Player)

3. CommandQueue 被 drain
   下一帧 PreUpdate 中，命令执行器取出命令
   → 匹配 GameCommand::Attack
   → 调用 execute_attack 逻辑

4. 伤害计算（Trigger 链）
   execute_attack 中:
   commands.trigger(CombatDamage { attacker, target, base_damage })
     ↓
   Observer(CombatDamage) → 护盾减伤计算
     ↓ trigger(ShieldAbsorbed)
   Observer(ShieldAbsorbed) → 伤害修正
     ↓ trigger(DamageFinalized)
   Observer(DamageFinalized) → 生命值修改
     ↓ trigger(UnitDamaged)

5. 跨域通知（Observer）
   UnitDamaged → Quest 域检查任务进度
   UnitDamaged → Progression 域可能给经验
   UnitDamaged → Logger 域记录 BAT007 (受伤)
   UnitDamaged → UI 域刷新血条

6. 日志输出
   battle_logger::on_unit_damaged 被 Bevy 调度
   emit_info!(..., code=BAT007, entity, damage, "单位受伤")
     ├─ tracing::info! → 控制台输出
     ├─ telemetry::record(BAT007) → MetricsCollector 计数
     └─ #[instrument] span → 调用链追踪
```

### 示例二：角色升级

```
1. 领域事件发射
   Progression 域系统：
   commands.trigger(LevelUp {
       entity: player,
       old_level: 4,
       new_level: 5,
       class_id: ...,
       is_asi_level: false,
   });

2. Bevy 调度 Observer
   ┌─ logs/progression_logger.rs:
   │  on_level_up(On<LevelUp>) — 记录日志
   │
   ├─ core/domains/progression/:
   │  on_level_up(On<LevelUp>) — 检查 ASI 是否需要授予
   │
   ├─ core/domains/quest/:
   │  on_character_level_up(On<LevelUp>) — 检查"达到5级"任务
   │
   ├─ core/domains/combat/:
   │  on_level_up_reroll_initiative(On<LevelUp>) — 更新先攻
   │
   └─ ui/screens/character/:
      on_level_up_show_animation(On<LevelUp>) — 播放升级特效

3. 日志输出
   progression_logger 输出:
   PRG002 | entity=Entity(3) old=4 new=5 | "角色升级"
```

### 示例三：UI 按钮点击「使用技能」

```
1. 玩家点击技能按钮
   → UI 原语 Button 检测交互
   → commands.trigger(ButtonClicked { entity: skill_button_entity })

2. 技能槽 Observer 响应
   obs: On<ButtonClicked>
   query: Query<&SkillSlotAction> on the button entity
   → 找到 SkillSlotAction::Use
   → dispatching to use skill...

3. 后续路径（两种选择）
   如果技能使用需要录制 Replay：
     → 构造 GameCommand::CastSpell { caster_id, spell_def_id, target_id }
     → CommandQueue.push_recorded(..., CommandSource::Player)

   如果技能使用是纯粹的域内逻辑：
     → commands.trigger(AbilityActivated { entity, spec_id })
     → Ability 链式处理
```

---

## 12. 通信规则速查

### ✅ 允许的

| 场景 | 怎么做 |
|------|--------|
| 同一 Feature 内的链式响应 | Trigger（`commands.trigger()` + Observer） |
| 跨 Domain 的业务通知 | Observer（`On<T>` + `app.add_observer()`） |
| 组件生命周期副作用 | Hook（`#[component(on_add, on_remove)]`） |
| 全局事件广播（备选） | `commands.trigger()`（本质也是 Observer） |
| 玩家/AI 操作 | Command → GameCommand → CommandQueue |
| UI 按钮意图传递 | Action enum + `On<ButtonClicked>` Observer |
| 只读查询跨域 | Domain 的 `integration/` facade + Query API |
| 跨域写操作 | 领域事件（白名单登记） |
| 写入日志 | 领域代码发事件 → LogObserver 自动处理 |
| 模块内部函数调用 | 直接调用（不要事件化） |

### ❌ 禁止的

| 场景 | 为什么 |
|------|--------|
| 高频循环中用 Observer | 每帧 10 次以上必须直接用 System |
| `EventWriter<T>` / `EventReader<T>` | Bevy 0.19 后已废弃，新代码禁止使用 |
| 将同一模块内普通函数调用事件化 | 为了"统一"而用事件模拟函数调用 |
| 系统函数直接互调 | System A 内部调用 System B 的函数 |
| 无递归深度保护的 Observer | 可能无限循环 |
| 跨 Domain 直接 import 数据结构 | 违反 Data Law 012 |
| 在 Observer 中写业务逻辑 | 日志 Observer 只能输出日志，不能做逻辑判断 |
| 在 Hook 中写复杂业务逻辑 | Hook 只能做轻量副作用 |
| 为临时副作用新建领域事件 | 必须走白名单流程 |
| 命令系统绕过 CommandQueue | 所有玩家/AI/Replay 操作必须入队 |

---

## 参考文档

| 文档 | 内容 |
|------|------|
| `docs/00-governance/ai-constitution-complete.md` §6.3 | 四级通信机制宪法 |
| `docs/00-governance/coding-rules.md` §4-5 | Message / Observer / Hook 编码规范 |
| `docs/01-architecture/00-foundation/ADR-002-ecs-communication.md` | ECS 四级通信机制选型（完整决策矩阵） |
| `docs/01-architecture/40-cross-cutting/ADR-043-command-input.md` | 命令层与输入抽象架构 |
| `docs/01-architecture/README.md` §4.2 | 架构总纲中的通信概述 |
| `docs/08-knowledge/logging-overview.md` | 日志系统深度解析（Observer + Metrics + FileSink） |
| `src/core/events.rs` | 跨域共享事件白名单 |
| `src/core/capabilities/runtime/command/foundation/types.rs` | GameCommand 枚举定义 |
| `src/core/capabilities/runtime/command/foundation/values.rs` | CommandQueue + CommandHistory |
| `src/infra/logging/plugin.rs` | LoggingPlugin（56 Observers 注册） |
| `src/infra/input/systems.rs` | 输入→命令流程 |
| `src/ui/screens/battle/systems.rs` | UI Action + Observer 示例 |
