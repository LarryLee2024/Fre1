---
id: 09-planning.phase-e
title: Phase E — Game Flow Integration Execution Plan
status: done
owner: feature-developer
created: 2026-06-19
completed: 2026-06-25
based-on: ADR-050 (Game State Machine & Scene Architecture)
---

# Phase E — Game Flow Integration Execution Plan

> **架构依据**: `docs/01-architecture/00-foundation/ADR-050-game-state-machine.md`
> **数据 Schema 参考**: `docs/04-data/domains/scene_data_schema.md`

---

## 概览

Phase E 将项目从"域系统独立运行"升级为"GameState 驱动的游戏流程编排"。共 4 个子阶段：

| 阶段 | 内容 | 文件数 | 涉及编译器 |
|------|------|--------|-----------|
| E-1 | scenes/ 基础框架（新增 ~5 文件） | +5 | rustc + test ✅ |
| E-2 | BattlePhase SubState 迁移 | 改 2 文件 | rustc + test ✅ |
| E-3 | 域系统 run_if 栅栏 + 场景桩 | 改 ~8 文件 | rustc + test ✅ |
| E-4 | 流程验证 + 测试 | 0 改 | test ✅ |

---

## E-1: scenes/ 基础框架

### 目标

创建场景管理模块，包含 GameState/OverlayState/TransitionRequest 定义、StateTransitionQueue、SceneRoot 标记组件、ScenePlugin。

### 步骤

#### E-1.1: `src/app/scenes/state.rs`

```rust
use bevy::prelude::*;

/// 顶层游戏状态。
#[derive(States, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    PartySetup,
    TacticalMap,
    Combat,
    Result,
    CampRest,
    GameOver,
}

/// 临时覆盖层 — 叠加在当前 GameState 之上，不触发场景重建。
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum OverlayState {
    #[default]
    None,
    Dialogue,
    Shop,
    Cutscene,
    Tutorial,
}

/// 状态转移请求。
pub enum TransitionRequest {
    Change(GameState),
    PushOverlay(OverlayState),
    PopOverlay,
}
```

#### E-1.2: `src/app/scenes/components.rs`

```rust
use bevy::prelude::*;

/// 场景根实体标记 — OnExit 时通过此组件 despawn 整个场景。
#[derive(Component)]
pub struct SceneRoot;
```

#### E-1.3: `src/app/scenes/queue.rs`

```rust
use bevy::prelude::*;
use super::state::{GameState, OverlayState, TransitionRequest};

/// 状态转移请求队列 — 唯一的 NextState 调用入口。
#[derive(Resource, Default)]
pub struct StateTransitionQueue {
    pub pending: Vec<TransitionRequest>,
}

/// 在 Last 调度中执行队列，保证每帧最多一次转移。
pub fn process_transition_queue(
    mut queue: ResMut<StateTransitionQueue>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let Some(request) = queue.pending.pop() {
        queue.pending.clear();
        match request {
            TransitionRequest::Change(state) => next_state.set(state),
            TransitionRequest::PushOverlay(_) | TransitionRequest::PopOverlay => {
                // Overlay 生命周期由 UI/输入层管理
                // 此处仅记录、不涉及 NextState
            }
        }
    }
}

/// 通用场景清理 — OnExit 时 despawn 所有 SceneRoot 实体。
pub fn cleanup_scene(mut commands: Commands, scene_roots: Query<Entity, With<SceneRoot>>) {
    for entity in &scene_roots {
        commands.entity(entity).despawn_recursive();
    }
}
```

#### E-1.4: `src/app/scenes/plugin.rs`

```rust
use bevy::prelude::*;
use super::state::GameState;
use super::queue::{StateTransitionQueue, process_transition_queue, cleanup_scene};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .insert_resource(StateTransitionQueue::default())
            .add_systems(Last, process_transition_queue);

        // 各场景 OnEnter/OnExit 由后续阶段添加
    }
}
```

#### E-1.5: `src/app/scenes/mod.rs`

```rust
pub mod plugin;
pub mod components;
pub mod state;
pub mod queue;

pub use plugin::ScenePlugin;
pub use components::SceneRoot;
pub use state::{GameState, OverlayState, TransitionRequest};
pub use queue::StateTransitionQueue;
```

#### E-1.6: 更新 `src/app/mod.rs`

在现有 `pub mod app_plugin` 后添加 `pub mod scenes;`

#### E-1.7: 更新 `src/app/app_plugin.rs`

在 Phase 0（DefaultPlugins + SharedPlugin 之后）添加 `.add_plugins(scenes::ScenePlugin)`

### 验证

```bash
cargo build
cargo nextest run
```

---

## E-2: BattlePhase SubState 迁移

### 目标

将 `BattlePhase` 从独立 State 转为 GameState::Combat 的 SubState。

### 步骤

#### E-2.1: `src/core/domains/combat/components.rs`

```rust
// 改前:
#[derive(States, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum BattlePhase { ... }

// 改后:
#[derive(SubStates, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[source(GameState = crate::app::scenes::GameState::Combat)]
pub enum BattlePhase { ... }
```

#### E-2.2: `src/core/domains/combat/plugin.rs`

```rust
// 改前:
app.init_state::<BattlePhase>();

// 改后:
// BattlePhase 作为 SubState 自动由 GameState::Combat 激活
// Bevy 0.18 不需要显式注册 SubState，derive(SubStates) 自动处理
```

### 验证

```bash
cargo build
cargo nextest run
```

注意：现有测试可能直接初始化 `App::new().init_state::<BattlePhase>()`，需要测试中改为依赖 `GameState::Combat` 的上下文。

---

## E-3: 域系统 run_if 栅栏 + 场景桩

### 目标

为所有 Update/PostUpdate 域系统添加状态栅栏，为每个场景注册 OnEnter/OnExit 桩系统。

### 步骤

#### E-3.1: 系统栅栏配置

| 域 | 系统 | 调度 | run_if | 文件 |
|----|------|------|--------|------|
| combat | `combat_pipeline_driver` | Update | `in_state(GameState::Combat)` | `combat/plugin.rs` |
| combat | `combat_input_system` | Update | `in_state(GameState::Combat)` | `combat/plugin.rs` |
| tactical | `tactical_input_system` | Update | `in_state(GameState::TacticalMap)` | `tactical/plugin.rs` |
| tactical | `initialize_default_grid` | 从 Startup 移到 OnEnter(TacticalMap) | 场景生命周期 | `tactical/plugin.rs` |
| camp_rest | `process_camp_events` | Update | `in_state(GameState::CampRest)` | `camp_rest/plugin.rs` |
| narrative | `cutscene_progress_system` | Update | `in_state(GameState::TacticalMap)` | `narrative/plugin.rs` |
| reaction | `reset_reactions_on_turn_start` | First | `in_state(GameState::Combat)` | `reaction/plugin.rs` |
| reaction | `process_reaction_queue` | Update | `in_state(GameState::Combat)` | `reaction/plugin.rs` |
| reaction | `cleanup_reaction_queue` | Last | `in_state(GameState::Combat)` | `reaction/plugin.rs` |
| spell | `tick_concentration_duration` | Update | `in_state(GameState::Combat)` | `spell/plugin.rs` |
| terrain | `TileEntityMap::update` | PostUpdate | `in_state(GameState::TacticalMap)` | `terrain/plugin.rs` |

#### E-3.2: 示例改动（tactical/plugin.rs）

```rust
// 改前:
app.add_systems(Startup, initialize_default_grid);
app.add_systems(Update, tactical_input_system);

// 改后:
app.add_systems(OnEnter(GameState::TacticalMap), initialize_default_grid);
app.add_systems(Update, tactical_input_system.run_if(in_state(GameState::TacticalMap)));
```

#### E-3.3: 场景生命周期注册（在 ScenePlugin 中补充）

```rust
// 桩系统 — 后续由 feature-developer 填充具体逻辑
fn setup_main_menu(mut commands: Commands) {
    // TODO[P2][Scene]: 生成主菜单 UI
}

// 注册：
app.add_systems(OnEnter(GameState::MainMenu), setup_main_menu);
app.add_systems(OnExit(GameState::MainMenu), cleanup_scene);
// ... 每个场景相似模式
```

### 验证

```bash
cargo build
cargo nextest run
```

---

## E-4: 流程验证 + 测试

### 目标

验证 GameState 生命周期正确性，确保：
1. State 转移不 panic
2. 场景清理不泄漏 Entity
3. BattlePhase SubState 绑定正确
4. run_if 生效（非激活状态系统不运行）

### 测试清单

- [ ] `GameState` 从默认值 `MainMenu` 开始
- [ ] 切换 `GameState::Combat` 后 `BattlePhase` 可访问
- [ ] 切出 `GameState::Combat` 后 `BattlePhase` 自动失效
- [ ] `StateTransitionQueue` 批量请求只执行最后一个
- [ ] `cleanup_scene` despawn 所有 `SceneRoot` 实体
- [ ] `OverlayState` 默认值为 `None`

### 验证

```bash
cargo nextest run
```
