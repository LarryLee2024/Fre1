---
id: 01-architecture.ADR-043
title: ADR-043 — Command Layer & Input Abstraction
status: proposed
owner: architect
created: 2026-06-16
updated: 2026-06-16
supersedes: none
---

# ADR-043: 命令层与输入抽象

## 状态

**Proposed** — 依赖 ADR-002（ECS Communication）和 ADR-041（Replay Determinism）。

## 背景

根据 SRPG 规则（§七），所有操作入口为标准化业务命令。玩家输入、AI 决策、回放执行三种入口最终必须转换为同一套命令。执行系统不区分命令来源，只处理命令本身。

## 引用的领域规则与数据架构

- `.trae/rules/SRPG专项规则.md` §七 — 命令层与输入抽象
- `.trae/rules/ECS规则.md` — PreUpdate 用于输入处理

## 决策

### 1. 命令层三层架构

```
┌─────────────────────────────────────────────────────────────┐
│  Layer 1: Raw Input (硬件原始输入)                            │
│  ─────────────────────────────────────                       │
│  键盘/鼠标/手柄/触摸 → 统一 Input 抽象                        │
│  位置: src/infra/input/                                            │
├─────────────────────────────────────────────────────────────┤
│  Layer 2: Game Command (业务命令)                            │
│  ─────────────────────────────────────                       │
│  标准化命令类型：MoveUnit / CastSkill / EndTurn / UseItem    │
│  不区分命令来源（玩家/AI/Replay 统一）                       │
│  位置: src/infra/command/ 或分散在各 Feature 的 events.rs          │
├─────────────────────────────────────────────────────────────┤
│  Layer 3: Execution (命令执行)                               │
│  ─────────────────────────────────────                       │
│  实际执行业务逻辑的系统                                      │
│  位置: 各 Feature 的 systems.rs                              │
└─────────────────────────────────────────────────────────────┘
```

### 2. 命令枚举

```rust
/// GameCommand — 所有业务命令的统一枚举
/// 这是 Layer 2 的中心类型
#[derive(Event, Serialize, Deserialize, Clone)]
pub enum GameCommand {
    // Tactical
    MoveUnit {
        entity: Entity,
        path: Vec<GridPos>,
    },
    Wait {
        entity: Entity,
    },

    // Combat
    Attack {
        attacker: Entity,
        target: Entity,
        ability: Option<AbilitySlot>,
    },
    CastSpell {
        caster: Entity,
        spell: SpellDefId,
        target: TargetingData,
    },
    UseItem {
        user: Entity,
        item_slot: u32,
        target: TargetingData,
    },

    // Turn
    EndTurn {
        entity: Entity,
    },

    // Meta
    OpenMenu,
    SaveGame,
    LoadGame,
}
```

### 3. 命令来源统一

所有命令来源经过同一个入口：

```rust
/// CommandQueue — 统一命令入口
/// 玩家、AI、Replay 都通过此 Resource 提交命令
#[derive(Resource, Default)]
pub struct CommandQueue {
    /// 待处理命令队列（当前帧）
    pending: Vec<GameCommand>,
    /// 历史记录（用于录制）
    history: Vec<RecordedCommand>,
}

impl CommandQueue {
    /// 提交一个命令（不限来源）
    pub fn push(&mut self, command: GameCommand) {
        self.pending.push(command);
    }

    /// 处理所有待处理命令（在 PreUpdate 中执行）
    pub fn drain(&mut self) -> impl Iterator<Item = GameCommand> {
        self.pending.drain(..)
    }
}
```

### 4. 输入处理流程

```
┌─ PreUpdate Schedule ───────────────────────────────────────┐
│                                                             │
│  PlayerInputSystem (读取原始输入 → GameCommand)               │
│       │                                                     │
│       ├── 鼠标点击格子 → grid_map 转换 → MoveUnit           │
│       ├── 键盘快捷键 → EndTurn                              │
│       └── UI 按钮点击 → UseItem                             │
│       │                                                     │
│       ▼                                                     │
│  CommandQueue.push(command)                                  │
│       │                                                     │
│       ▼                                                     │
│  ReplayInputSystem (仅在回放模式)                             │
│       └── 从 ReplayFrame 读取 → GameCommand                 │
│              │                                              │
│              ▼                                              │
│       CommandQueue.push(command)                             │
│                                                             │
│── AIInputSystem (仅在 AI 回合) ──────────────────────────│
│       └── AI 决策 → GameCommand                             │
│              │                                              │
│              ▼                                              │
│       CommandQueue.push(command)                             │
│                                                             │
│  CommandExecutionSystem                                      │
│       └── CommandQueue.drain() → dispatch 到具体 System      │
│              │                                              │
│              ▼                                              │
│       MoveCommand → movement::execute_move                 │
│       Attack → combat::execute_attack                      │
│       CastSpell → spell::execute_cast                      │
│       EndTurn → turn_phase::end_turn                       │
│       ...                                                   │
└─────────────────────────────────────────────────────────────┘
```

### 5. AI 输入集成

```rust
/// AIInputSystem — 在 PreUpdate 中运行
/// 仅在当前单位由 AI 控制时激活
fn ai_input_system(
    ai_query: Query<&AIControlled>,
    turn_queue: Res<TurnQueue>,
    mut command_queue: ResMut<CommandQueue>,
    ai_plugin: Res<AIPlugin>,
) {
    let current = turn_queue.current();
    if ai_query.contains(current.entity) {
        // AI 决策（可能是同步或异步）
        if let Some(command) = ai_plugin.decide(current.entity) {
            command_queue.push(command);
        }
    }
}
```

### 6. 原始输入抽象

```rust
/// InputAction — 语义化的输入动作
/// 键盘/鼠标/手柄的按键绑定映射到此枚举
pub enum InputAction {
    Select,
    Confirm,
    Cancel,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    CameraMove(Vec2),
    CameraZoom(f32),
    QuickSave,
    QuickLoad,
    OpenMenu,
    EndTurn,
    SkillSlot1,
    SkillSlot2,
    SkillSlot3,
    SkillSlot4,
}

/// InputMap — 按键到 InputAction 的映射（可配置）
#[derive(Resource)]
pub struct InputMap {
    mappings: HashMap<KeyCode, InputAction>,
    mouse_mappings: HashMap<MouseButton, InputAction>,
}
```

### 7. 命令的可录制性

由于 `GameCommand` 实现了 `Serialize + Deserialize`，它天然支持录制：

```rust
fn record_command_system(
    mut command_queue: ResMut<CommandQueue>,
    mut recorder: Option<ResMut<ReplayRecorder>>,
) {
    if let Some(ref mut recorder) = recorder {
        if recorder.is_recording {
            for cmd in &command_queue.pending {
                recorder.record_command(cmd.clone());
            }
        }
    }
}
```

## Module Design

```
src/infra/input/
  ├── plugin.rs              — InputPlugin
  ├── resources.rs           — InputMap, InputState
  ├── systems.rs             — keyboard_mouse_input, camera_control
  └── api.rs                 — InputAction, InputMap

src/infra/command/  (或在各 Feature 的 events.rs 中定义)
  ├── plugin.rs              — CommandPlugin (可选)
  ├── resources.rs           — CommandQueue
  ├── systems.rs             — command_dispatcher
  └── api.rs                 — GameCommand 枚举
```

## Communication Design

| 通信 | 机制 | 说明 |
|------|------|------|
| 原始输入 → InputAction | InputMap 查找 | input Feature 内部 |
| InputAction → GameCommand | 状态机 + 当前上下文转换 | input Feature 内部 |
| GameCommand 提交 | `CommandQueue.push()` | 任何来源（玩家/AI/Replay） |
| GameCommand 分发 | `CommandQueue.drain()` → 各 Feature Event | command → 具体 Feature |
| Replay → GameCommand | `ReplayFrame.commands` 反序列化 | replay → command |

## 边界定义

### 允许
- 任何来源通过 `CommandQueue.push()` 提交命令
- InputAction 配置化（支持按键重映射）
- AI 通过 `ai_input_system` 提交命令
- Replay 模式通过 `ReplayInputSystem` 提交命令

### 🟥 禁止
- 业务代码直接读取原始按键状态（必须经过 InputAction → GameCommand）
- 命令在执行前被丢弃（Queue 是可靠的）
- 同一命令在玩家和 Replay 模式下执行不同逻辑
- Input System 修改业务状态（只产生 Command）

## Forbidden

| 禁止行为 | 理由 |
|---------|------|
| 业务代码直接读取 `Input<KeyCode>` | 违反输入抽象层，不可回放 |
| AI 直接调用业务函数而非提交 Command | 不可录制，不可回放 |
| 命令来源不同导致执行逻辑不同 | 违反来源无关性原则 |
| CommandQueue 无限增长 | 需要帧末清空或阈值限制 |

## Definition / Instance Design

- **Definition**: `InputMap` (config: 按键绑定配置)
- **Instance**: `CommandQueue` (Resource), `InputMap` (Resource), `InputState` (Resource)
- **Persistence**: InputMap（玩家自定义按键配置可持久化，但不需要在存档中）

## 后果

### 正面
- 玩家/AI/Replay 三种来源统一执行路径
- 所有命令天然可录制、可回放
- 输入抽象层屏蔽硬件差异
- 按键映射可配置化

### 负面
- 每新增一个命令需要更新 `GameCommand` 枚举
- 所有输入经过排队 - 分发流程，增加一步间接层
- AI 决策可能需要在 PreUpdate 中同步完成（压力在 AI Plugin）

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 玩家和 AI 走不同的执行路径 | 不可回放，逻辑不一致 |
| 命令直接调用 System 函数 | 无法录制，无法 undo |
| 不使用 CommandQueue，直接 Event 分发 | Event 不能保证顺序和可靠性 |

## 评审要点

- [ ] `GameCommand` 枚举是否需要支持 undo（撤销操作）？
- [ ] 命令来源标记是否需要（用于调试/回放标注"此命令来自 AI"）？
- [ ] 网络多人模式如何扩展——CommandQueue 是否天然支持？
- [ ] AI 决策异步化——如果 AI 决策需要多帧计算，CommandQueue 如何处理？
