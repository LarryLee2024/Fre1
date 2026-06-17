---
id: infrastructure.input.schema.v1
title: Input Schema — 输入与命令层数据架构
status: stable
owner: data-architect
created: 2026-06-17
updated: 2026-06-17
layer: runtime
replay-safe: true
---

# Input Schema — 输入与命令层数据架构

> **领域归属**: Infrastructure — Input | **依赖 Schema**: Tag, Ability, Targeting | **定义依据**: `docs/01-architecture/40-cross-cutting/ADR-043-command-input.md`

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `InputAction` | Definition | 语义化输入动作枚举 |
| `InputMap` | Definition | 按键到动作的映射配置 |
| `GameCommand` | Runtime | 业务命令统一枚举 |
| `CommandQueue` | Runtime | 命令队列（Resource） |
| `InputState` | Instance | 当前帧输入状态 |

---

## 2. Schema Design

### 2.1 InputAction（Definition 层）

```rust
/// 语义化输入动作 — 键盘/鼠标/手柄映射到此枚举
pub enum InputAction {
    // 选择
    Select,
    Confirm,
    Cancel,

    // 方向
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,

    // 摄像机
    CameraMove(Vec2),
    CameraZoom(f32),

    // 快捷操作
    QuickSave,
    QuickLoad,
    OpenMenu,
    EndTurn,

    // 技能槽
    SkillSlot1,
    SkillSlot2,
    SkillSlot3,
    SkillSlot4,
}
```

### 2.2 InputMap（Definition 层 — 配置）

```rust
/// 按键绑定配置 — 可通过 RON 文件自定义
#[derive(Resource, Serialize, Deserialize)]
pub struct InputMap {
    pub keyboard: HashMap<KeyCode, InputAction>,
    pub mouse: HashMap<MouseButton, InputAction>,
    pub gamepad: HashMap<GamepadButton, InputAction>,
}
```

```ron
// assets/config/input.ron
InputMap(
    keyboard: {
        Space: Confirm,
        Escape: Cancel,
        Return: EndTurn,
        S: QuickSave,
        L: QuickLoad,
        M: OpenMenu,
        Key1: SkillSlot1,
        Key2: SkillSlot2,
        Key3: SkillSlot3,
        Key4: SkillSlot4,
    },
    mouse: {
        Left: Select,
        Right: Cancel,
    },
)
```

### 2.3 GameCommand（Runtime 层）

```rust
/// 业务命令统一枚举 — 所有来源（玩家/AI/Replay）共用
#[derive(Event, Serialize, Deserialize, Clone, Debug)]
pub enum GameCommand {
    // ── Tactical ──
    /// 移动单位到目标路径
    MoveUnit {
        entity: Entity,
        path: Vec<GridPos>,
    },
    /// 等待（不执行行动）
    Wait {
        entity: Entity,
    },

    // ── Combat ──
    /// 普通攻击
    Attack {
        attacker: Entity,
        target: Entity,
        ability: Option<AbilitySlot>,
    },
    /// 施放法术
    CastSpell {
        caster: Entity,
        spell: SpellDefId,
        target: TargetingData,
    },
    /// 使用物品
    UseItem {
        user: Entity,
        item_slot: u32,
        target: TargetingData,
    },

    // ── Turn ──
    /// 结束当前回合
    EndTurn {
        entity: Entity,
    },

    // ── Meta ──
    OpenMenu,
    SaveGame,
    LoadGame,
}
```

### 2.4 CommandQueue（Runtime 层 — Resource）

```rust
/// 统一命令队列 — 玩家/AI/Replay 通过此提交命令
#[derive(Resource, Default)]
pub struct CommandQueue {
    /// 待处理命令（当前帧）
    pending: Vec<GameCommand>,
    /// 历史记录（用于 Replay 录制）
    history: Vec<RecordedCommand>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RecordedCommand {
    pub command: GameCommand,
    pub frame: u64,
    pub source: CommandSource,
}

pub enum CommandSource {
    Player,
    Ai,
    Replay,
}
```

### 2.5 InputState（Instance 层 — Resource）

```rust
/// 当前帧输入状态 — 由 InputSystem 更新
#[derive(Resource, Default)]
pub struct InputState {
    /// 当前帧的原始输入
    pub mouse_position: Vec2,
    pub mouse_grid_pos: Option<GridPos>,
    pub pressed_actions: Vec<InputAction>,
    pub just_pressed_actions: Vec<InputAction>,
}
```

### 2.6 TargetingData（Runtime 层 — 命令参数）

```rust
/// 目标选择数据 — CastSpell/UseItem 的目标参数
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TargetingData {
    /// 单体目标
    Single(Entity),
    /// 范围目标（中心 + 半径）
    Area {
        center: GridPos,
        radius: u32,
    },
    /// 直线目标（起点 + 方向 + 长度）
    Line {
        origin: GridPos,
        direction: Direction,
        length: u32,
    },
    /// 自身
    SelfCast,
}
```

---

## 3. 数据流

```
┌─ Definition 层 ─────────────────────────────────────────────┐
│  InputMap (RON 配置)                                        │
│  → 加载到 Resource                                          │
│  → InputSystem 读取映射                                     │
└─────────────────────────────────────────────────────────────┘
         │
         ▼
┌─ Runtime 层 ───────────────────────────────────────────────┐
│  InputState (当前帧输入)                                    │
│       │                                                     │
│       ├── PlayerInputSystem: InputState → GameCommand       │
│       ├── AIInputSystem: AI 决策 → GameCommand              │
│       └── ReplayInputSystem: ReplayFrame → GameCommand      │
│       │                                                     │
│       ▼                                                     │
│  CommandQueue.push(command)                                  │
│       │                                                     │
│       ▼                                                     │
│  CommandExecutionSystem: CommandQueue.drain()               │
│       └── dispatch → 具体 Feature 的 System                 │
└─────────────────────────────────────────────────────────────┘
         │
         ▼
┌─ Persistence 层 ──────────────────────────────────────────┐
│  RecordedCommand[] → ReplayFrame                           │
│  InputMap → 玩家自定义配置（可选持久化，不在存档中）          │
└─────────────────────────────────────────────────────────────┘
```

---

## 4. Replay Compatibility

| 场景 | 兼容性 | 说明 |
|------|--------|------|
| GameCommand 序列化 | 🟩 完全确定 | derive(Serialize, Deserialize) |
| 命令执行顺序 | 🟩 完全确定 | CommandQueue 按 push 顺序 drain |
| 命令来源标记 | 🟩 不影响执行 | CommandSource 仅用于日志/调试 |
| InputMap 配置 | ⚠️ 不影响回放 | 回放使用录制的 Command，不重放输入 |

---

## 5. Save Compatibility

| 数据 | 持久化 | 说明 |
|------|--------|------|
| GameCommand 历史 | ✅ 存入 ReplayFrame | 回放所需 |
| InputMap | ❌ 不存档 | 玩家偏好，非游戏状态 |
| InputState | ❌ 不存档 | 瞬时状态，每帧重建 |
| CommandQueue | ❌ 不存档 | 运行时队列，帧末清空 |

---

## 6. Constitution Check

| 条款 | 合规 | 说明 |
|------|------|------|
| Replay First | ✅ | GameCommand 可序列化，执行顺序确定 |
| 输入抽象层 | ✅ | 业务代码不直接读取原始按键 |
| 来源无关性 | ✅ | 玩家/AI/Replay 走同一路径 |
| CommandQueue 可靠性 | ✅ | 帧内保序，帧末清空 |
