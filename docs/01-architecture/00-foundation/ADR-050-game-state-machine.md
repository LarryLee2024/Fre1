---
id: 01-architecture.ADR-050
title: ADR-050 — Game State Machine & Scene Architecture
status: accepted
owner: architect
created: 2026-06-19
updated: 2026-06-19
supersedes: ADR-021 (partial — BattlePhase 从独立 State 转为 SubState)
---

# ADR-050: Game State Machine & Scene Architecture

## 状态

**Proposed**

## 背景

当前项目 15 个 Capabilities + 15 个业务域 + 全部基础设施已实现，但缺乏顶层的**游戏流程编排**。`AppPlugin` 仅做 Plugin 注册，所有域系统的 `Update` 系统全局运行，没有任何状态栅栏（state gating）区分"当前处于哪个游戏阶段"。

`BattlePhase` 是目前唯一的 Bevy State，由 `CombatPlugin` 以独立 `States` 注册。其他域（TacticalMap、CampRest、Narrative）均无条件运行。

**问题**：
1. 运行 `cargo run` 后显示空白窗口，无主菜单/标题画面
2. 所有 Update 系统同时运行，可能产生错误的跨域交互
3. 不同游戏阶段（主菜单 vs 战术地图 vs 战斗）的实体/资源共存，无场景隔离
4. 场景切换时需要手动 spawn/despawn，无标准化模式

## 引用的领域规则与架构

- `docs/01-architecture/README.md` §4.3 — Schedule 权责划分
- `docs/01-architecture/README.md` §6 — Plugin 组合与注册顺序
- `ADR-021` — 回合状态机，被本 ADR 部分取代
- `.trae/rules/ECS规则.md` §四 — 状态管理使用 Bevy States

## 决策

### 1. 两层状态架构：GameState + OverlayState

SRPG 的游戏流程包含两种性质不同的"状态变化"：

| 层级 | 语义 | 举例 | 生命周期影响 |
|------|------|------|------------|
| **GameState** | 世界模式切换 | MainMenu → TacticalMap → Combat | 卸载前一个场景的全部 Entity |
| **OverlayState** | 临时覆盖层 | Dialogue / Shop / Cutscene | 世界保持挂载，覆盖层叠加 |

#### GameState — 顶层游戏状态

```rust
/// 顶层游戏状态 — 驱动全局游戏流程。
///
/// 切换 GameState 意味着进入一个不同的"世界模式"：
/// ECS 系统集变化、UI 集合变化、输入规则变化、摄像机规则变化。
/// 上一个场景的实体在 OnExit 时全部卸载。
#[derive(States, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum GameState {
    /// 主菜单/标题画面。默认启动状态。
    #[default]
    MainMenu,
    /// 队伍编成/战前准备。
    PartySetup,
    /// 战术地图（网格探索、遭遇、商店入口、对话入口）。
    TacticalMap,
    /// 战斗进行中。
    Combat,
    /// 战斗结算（胜利/失败奖励展示）。
    Result,
    /// 营地界面（短休/长休/队伍管理）。
    CampRest,
    /// 游戏结束画面。
    GameOver,
}
```

**状态转移图：**

```
                  ┌──────────────────────────────────┐
                  │                                  │
                  ▼                                  │
            ┌──────────┐    ┌────────────┐           │
  启动 ──→  │ MainMenu │──→ │ PartySetup │           │
            └──────────┘    └────────────┘           │
                  │              │                    │
                  │              ▼                    │
                  │        ┌──────────────┐           │
                  ├──────→ │ TacticalMap  │ ◄─────────┘
                  │        └──────┬───────┘
                  │               │
                  │               ▼
                  │         ┌──────────┐
                  │         │  Combat   │
                  │         └────┬─────┘
                  │              │
                  │         ┌──────────┐
                  │         │  Result   │
                  │         └────┬─────┘
                  │              │
                  │    ┌─────────┴──────────┐
                  │    ▼                    ▼
                  │ ┌──────────┐    ┌──────────┐
                  │ │CampRest  │    │ GameOver │
                  │ └──────────┘    └──────────┘
                  │       │
                  └───────┘
```

#### OverlayState — 临时覆盖层

```rust
/// 临时覆盖层 — 叠加在当前 GameState 之上。
///
/// 与 GameState 的核心区别：覆盖层不卸载当前场景。
/// 例如 TacticalMap 中触发 Dialogue，地图保持挂载，
/// PopOverlay 后直接回到地图，无需重建。
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum OverlayState {
    /// 无覆盖层。
    #[default]
    None,
    /// 对话界面（可叠加在任何 GameState 之上）。
    Dialogue,
    /// 商店界面（通常叠加在 TacticalMap 或 CampRest 之上）。
    Shop,
    /// 过场演出（可叠加在任何 GameState 之上）。
    Cutscene,
    /// 新手指引。
    Tutorial,
}
```

覆盖层不单独注册为 Bevy State，其生命周期通过 `StateTransitionQueue` 的 `PushOverlay` / `PopOverlay` 请求管理。

### 2. BattlePhase 转为 SubState

将 `BattlePhase` 从独立 `States` 改为 `SubStates`，绑定到 `GameState::Combat`：

```rust
#[derive(SubStates, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[source(GameState = GameState::Combat)]
pub enum BattlePhase {
    #[default]
    Preparation,
    Battle,
    Victory,
    Defeat,
}
```

**变更影响：**
- `app.init_state::<BattlePhase>()` → `app.add_sub_state::<BattlePhase>()`
- `OnEnter(BattlePhase::X)` 自动获得 `GameState::Combat` 的前置条件
- 当 `GameState ≠ Combat` 时 BattlePhase 不可访问

### 3. StateTransitionQueue + TransitionRequest

禁止域系统直接调用 `NextState<GameState>`。所有状态转移请求通过队列统一中转：

```rust
/// 状态转移请求。
pub enum TransitionRequest {
    /// 切换 GameState（触发场景卸载/加载）。
    Change(GameState),
    /// 推送覆盖层（当前场景保持挂载）。
    PushOverlay(OverlayState),
    /// 弹出覆盖层（回到上一个覆盖层或无覆盖状态）。
    PopOverlay,
}

/// 状态转移请求队列 — 唯一的 NextState 调用入口。
#[derive(Resource, Default)]
pub struct StateTransitionQueue {
    pending: Vec<TransitionRequest>,
}
```

**为什么要队列**：项目规模扩大后，十几个系统都可能调用 `next_state.set()`。队列将所有请求收敛到一个执行点，保证本帧只有一个转移生效。执行策略：仅处理最后一个请求（忽略中间跳转）。

### 4. 场景生命周期

每个 GameState 对应一个场景，拥有标准化的生命周期：

- **OnEnter**：spawn 场景根实体（带 `SceneRoot` 标记组件），初始化 Resource
- **OnUpdate**：通过 `run_if(in_state(GameState::X))` 栅栏运行的域系统
- **OnExit**：`cleanup_scene` 通用系统，despawn 所有 `SceneRoot` 实体

```rust
#[derive(Component)]
pub struct SceneRoot;
```

场景间数据传递通过 ECS Resource 通道完成（发送方 OnExit 时设置，接收方 OnEnter 时消费），具体数据通道的 Schema 见 `docs/04-data/domains/scene_data_schema.md`。

### 5. 域系统状态栅栏原则

| 系统类型 | 栅栏策略 | 理由 |
|---------|---------|------|
| Update 核心业务系统 | `run_if(in_state(GameState::X))` | 只在特定场景运行 |
| Observer 系统 | **无条件**运行 | 事件驱动；事件在错误状态发射时应优雅忽略 |
| 全局系统（progression/party/inventory/faction） | 无 | 跨场景有效 |
| save/load 系统 | 无 | 任何状态都可存档/读档 |

### 6. 与 ADR-021 的关系

本 ADR **部分取代** ADR-021。ADR-021 定义的 `BattlePhase` 枚举保留，注册方式从独立 `States` 转为 `SubStates`。ADR-021 中的回合内流程（TurnStart → PhaseCheck → UnitAction → TurnSettlement → TurnEnd）已由 `CombatPipelineDriver` 取代，本 ADR 不影响此部分。

## Module Design

### 新增模块

```
src/app/scenes/           # 场景管理模块（App 层）
├── mod.rs                # pub mod 声明
├── plugin.rs             # ScenePlugin — 注册 GameState + 场景生命周期
├── components.rs         # SceneRoot 标记组件
├── state.rs              # GameState + OverlayState + TransitionRequest 枚举
└── queue.rs              # StateTransitionQueue + 执行系统
```

### 修改文件

| 文件 | 变更 |
|------|------|
| `src/app/app_plugin.rs` | 新增 `ScenePlugin` 注册 |
| `src/core/domains/combat/plugin.rs` | `init_state<BattlePhase>` → `add_sub_state<BattlePhase>` |
| `src/core/domains/combat/components.rs` | `derive(States)` → `derive(SubStates)` + source 绑定 |
| 多个 domain `plugin.rs` | Update 系统加 `run_if` 栅栏（详见实施计划） |

**不修改的文件**：所有 Observer、所有 capability Plugin、content/ infra/ shared/ modding/ tools/。

## Communication Design

| 通信 | 机制 | 方向 |
|------|------|------|
| GameState 转移 | `StateTransitionQueue` → `NextState<GameState>` | 任意系统 → 场景管理 |
| Overlay 推/弹 | `TransitionRequest::PushOverlay/PopOverlay` | 任意系统 → 场景管理 |
| 战斗阶段转移 | `NextState<BattlePhase>` | CombatPlugin 内部 |
| 场景间数据 | Resource 通道 | OnExit 域 → OnEnter 域 |
| UI→Game 指令 | `UiCommand` Event | UI 系统 → 域系统 |

## 边界定义

### 允许
- 域系统通过 `StateTransitionQueue` 请求转移
- 覆盖层结束时通过 `PopOverlay` 返回
- 场景 OnEnter 读取 Resource 通道消费前一个场景的数据
- 场景 OnExit 写入 Resource 通道供下一个场景读取
- Observer 无条件运行

### 🟥 禁止
- 域系统直接调用 `NextState<GameState>`
- 场景实体跨越 GameState 边界存活
- OnEnter 执行重型业务逻辑
- 场景间通过全局变量/单例传递数据
- BattlePhase 在 `GameState ≠ Combat` 时访问
- 覆盖层修改 GameState
- PopOverlay 时空栈

## Forbidden

| 行为 | 理由 |
|------|------|
| 直接 NextState 绕过队列 | 违反"单一转移入口"原则，导致多系统竞争 |
| 场景实体跨状态存活 | 状态泄漏，造成资源泄漏 |
| OnEnter 执行耗时业务 | 阻塞主线程 |
| 覆盖层修改 GameState | Overlay 不改变世界模式 |
| PopOverlay 时空栈 | 必须确保栈内有覆盖层 |

## Definition / Instance Design

- **Definition**: 无（GameState/OverlayState 是运行时概念）
- **Instance**: `GameState` (States), `BattlePhase` (SubStates)
- **Runtime Resources**: `StateTransitionQueue`, 场景数据通道
- **Component**: `SceneRoot` — 场景根实体标记
- **数据通道 Schema**: 详见 `docs/04-data/domains/scene_data_schema.md`

## 后果

### 正面
- GameState/OverlayState 两层分离，Dialogue/Shop 不触发场景重建
- BattlePhase SubState 自动跟随 Combat 生命周期
- 转移队列消除多系统状态竞争
- 场景生命周期标准化

### 负面
- 实现比单层 GameState 略复杂
- 约 10 个 Update 系统需要添加 `run_if`
- 场景数据通道需维护读写配对

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| Dialogue/Shop 作为 GameState | 触发不必要场景重建，FE/BG3 都不这么做 |
| 单层全量 States + if 判断 | 不如 SubState 声明式清晰 |
| 不设转移队列 | 大项目必然出现状态竞争 |
| Overlay 作为 Bevy SubState | 状态数 = len(GameState) × len(OverlayState)，组合爆炸 |
