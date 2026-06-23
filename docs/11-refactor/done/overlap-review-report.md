# Shared / Infra vs Core 重复评审报告

> 评审时间: 2026-06-21
> 评审范围: `src/shared/`, `src/infra/`, `src/core/`
> 评审目标: 识别三层之间的代码重复、职责越界、抽象泄漏

---

## 1. 总体结论

**架构分层基本正确，无严重代码重复。** 三层职责边界清晰：

| 层 | 定位 | 职责 |
|----|------|------|
| Shared (L0) | 原子工具层 | 零业务语义的通用编程工具 |
| Core (L1) | 领域规则层 | 业务能力骨架 + 玩法复杂度 |
| Infra (L2) | 技术实现层 | ECS 桥接、持久化、渲染等"脏活" |

发现 **0 处代码级重复**，但有 **3 处职责越界**、**2 处可改进的抽象未利用** 和 **1 处缺失桥接**。

---

## 2. 逐模块分析

### 2.1 shared/localization_key.rs vs infra/localization/ — ✅ 无重复

| 模块 | 类型 | 职责 |
|------|------|------|
| `shared::localization_key::LocalizationKey` | `Cow<'static, str>` 包装器 | Def 配置中 name_key/desc_key 的强类型载体 |
| `infra::localization::foundation::locale_id::LocaleId` | BCP-47 枚举 | 运行时语言标识（en-US, zh-CN…） |

**结论**: 不同抽象层的不同概念，不存在重复。`LocalizationKey` 是"查什么"，`LocaleId` 是"用什么语言查"。

### 2.2 shared/error vs core error types — ✅ 无重复

| 模块 | 类型 | 职责 |
|------|------|------|
| `shared::error::ErrorContext<E>` | 泛型错误包装 + `.domain()` / `.with_context()` | 跨领域错误传播的上下文附加 |
| `core::capabilities::runtime::replay::foundation::error::ReplayError` | 领域错误枚举 | 回放操作的具体错误类型 |

**结论**: `ErrorContext<E>` 是基础设施，`ReplayError` 是具体错误。`infra/save/events.rs` 中 `SaveError` 正确使用了 `ErrorContext<String>`。无重复。

### 2.3 shared/hashing — ✅ 无重复

`FastHasher` / `fast_hash` 被以下模块消费：
- `shared::ids` — ID 哈希
- `infra::localization::storage::cache` — 缓存键哈希

**结论**: 纯工具函数，多处消费是正确用法。

### 2.4 shared/ids vs infra/registry — ✅ 无重复

| 模块 | 类型 | 职责 |
|------|------|------|
| `shared::ids::DefinitionId` | 通用 Def 标识符 | Registry 查询的 key 类型 |
| `infra::registry::RegistryBucket<T>` | `HashMap<DefinitionId, T>` 包装 | 版本化 Def 存储桶 |

**结论**: `RegistryBucket` 构建在 `DefinitionId` 之上，是正确的层依赖。

### 2.5 infra/replay vs core/capabilities/runtime/replay — ✅ 无重复（正确的 re-export 模式）

```
core/capabilities/runtime/replay/
  ├── foundation/     ← 纯类型定义（ReplayLog, ReplayFrame, ReplayError...）
  └── mechanism/      ← 纯逻辑（RecordingSession, PlaybackSession, calculate_frame_checksum）

infra/replay/
  ├── resources.rs    ← re-export core 类型 + Bevy Resource 包装
  ├── systems.rs      ← 帧管理 System（recording_frame_bookend_system...）
  └── plugin.rs       ← Plugin 注册入口
```

`infra/replay/resources.rs` 第 13-15 行明确 re-export：
```rust
pub use crate::core::capabilities::runtime::replay::mechanism::{
    FrameCounter, PlaybackSession, RecordingSession, ReplayModeGuard,
};
pub use crate::shared::random::DeterministicRng;
```

**结论**: Core 定义逻辑，Infra 桥接到 ECS。这是 ADR-041 设计的正确实现，不是重复。

### 2.6 shared/validation vs infra validation — ⚠️ 抽象未利用

| 模块 | 类型 | 职责 |
|------|------|------|
| `shared::validation::Validator<T>` trait | 泛型校验抽象 | `.validate()` + `ValidationChain` |
| `infra::registry::resolver.rs` | 自实现校验逻辑 | ID 格式校验、唯一性检测 |
| `infra::localization::validation/` | 自实现校验逻辑 | Key 完整性验证、覆盖率审计 |

**问题**: `infra::registry::resolver` 和 `infra::localization::validation` 各自实现了校验逻辑，但**没有使用** `shared::validation::Validator` trait。

**影响**:
- `Validator` trait 的 `.validate()` / `ValidationChain` / `NotEmpty` / `MinLength` 未被 infra 层消费
- Infra 层的校验逻辑与 shared 层的校验抽象是**平行实现**，不是代码重复，但属于**抽象泄漏**

**建议**: 中优先级。考虑让 infra 校验实现 `Validator` trait，统一校验模式。

---

## 3. 职责越界问题

### 3.1 shared/constants 包含游戏特定常量 — ⚠️ 越界

```rust
// src/shared/constants/mod.rs
pub const MAX_PARTY_SIZE: usize = 6;       // ← 队伍管理领域规则
pub const MAX_INVENTORY_SIZE: usize = 100; // ← 背包系统领域规则
pub const MAX_BUFF_STACK: usize = 5;       // ← Buff 叠加层数领域规则
pub const MAX_OBSERVER_DEPTH: u32 = 10;    // ← 这个是纯技术限制，合理
```

**问题**: `MAX_PARTY_SIZE`、`MAX_INVENTORY_SIZE`、`MAX_BUFF_STACK` 是**业务规则常量**，应属于 Core 层的对应领域模块（`core/domains/party/`、`core/domains/inventory/`、`core/domains/effect/`）。

**建议**: 高优先级。将业务常量迁移到各自的领域模块，shared/constants 仅保留 `MAX_OBSERVER_DEPTH` 等纯技术常量。

### 3.2 shared/game_state.rs 包含游戏流程状态机 — ⚠️ 越界

```rust
// src/shared/game_state.rs
pub enum GameState {
    MainMenu, PartySetup, TacticalMap, Combat, Result, CampRest, GameOver,
}
pub enum OverlayState { None, Dialogue, Shop, Cutscene, Tutorial }
pub enum TransitionRequest { Change(GameState), PushOverlay(OverlayState), PopOverlay }
```

**问题**: `GameState` / `OverlayState` / `TransitionRequest` 定义了**完整的游戏流程状态机**，这是领域规则，不是通用工具。放在 shared 是为了"Avoid Core → App 的层依赖违规"（ADR-050），但这本质上是**用错误的分层解决了依赖方向问题**。

**影响**:
- shared 层本应零业务语义，但 GameState 包含了全部游戏流程
- 每次新增/修改游戏状态都需要改 shared，违反 L0 稳定性原则

**建议**: 中优先级。根据 ADR-050 的意图，GameState 应放在一个所有层都能引用的"共享类型"位置，但不应该是 L0 shared。考虑：
1. 将 GameState 提升为独立的 `crate`（如 `fre-types`），或
2. 将 GameState 放在 Core 层，通过 trait 抽象解除 Infra 对 Core 的直接依赖

### 3.3 shared/diagnostics/domain.rs 包含 infra 级别的 Domain — ⚠️ 轻微越界

```rust
Self::Save => "infra.save",
Self::Replay => "infra.replay",
```

**问题**: `Domain` 枚举中 `Save` 和 `Replay` 的 tracing target 使用了 `infra.` 前缀，暗示这两个 Domain 属于 Infra 层，但定义却在 Shared 层。

**建议**: 低优先级。这只是 tracing target 命名问题，不影响运行时行为。可在后续 logging 重构时统一。

---

## 4. Core/Capabilities/Runtime 深度分析

### 4.1 runtime/command vs infra/input — ✅ 无重复（正确的三层流水线）

```
Raw Input (Bevy ButtonInput)
    ↓ infra/input/systems.rs: collect_input_actions()
InputAction (infra/input/action.rs)          ← 硬件抽象层
    ↓ infra/input/systems.rs: process_meta_commands() (仅元命令)
    ↓ 各 Domain System (core/domains/)       (业务命令)
GameCommand (core/capabilities/runtime/command/foundation/types.rs)  ← 业务命令层
    ↓ CommandQueue.push()
CommandQueue (core/capabilities/runtime/command/foundation/values.rs) ← 统一入口
```

| 模块 | 枚举 | 职责 |
|------|------|------|
| `infra::input::action::InputAction` | `Select, Cancel, MoveUp, CameraZoomIn, QuickSave...` | 硬件按键 → 语义化动作 |
| `core::capabilities::runtime::command::foundation::types::GameCommand` | `MoveUnit, Attack, CastSpell, SaveGame...` | 业务操作意图 |

**结论**: 不同抽象层的不同概念。`InputAction` 是"玩家按了什么"，`GameCommand` 是"玩家想做什么"。`infra/input/systems.rs:74` 的 `process_meta_commands` 正确地将 `InputAction::QuickSave` 转换为 `GameCommand::SaveGame`。

### 4.2 runtime/command vs infra/save — ✅ 无重复

| 模块 | 类型 | 职责 |
|------|------|------|
| `core::capabilities::runtime::command::foundation::types::GameCommand::SaveGame` | 命令变体 | 用户意图（"我要保存"） |
| `infra::save::events::SaveRequest` | 事件 | 存档系统实现（路径、标签） |

**结论**: `GameCommand::SaveGame` 是用户意图，`SaveRequest` 是存档系统的具体实现参数。命令处理系统将 `SaveGame` 转换为 `SaveRequest` 事件。无重复。

### 4.3 runtime/replay/foundation vs infra/replay — ✅ 无重复（re-export 模式）

```
core/capabilities/runtime/replay/
  ├── foundation/
  │   ├── types.rs      ← ReplayFrame, ReplayCommand, ReplayHeader（纯数据类型）
  │   ├── values.rs     ← ReplayLog, ReplayRecorder, ReplayPlayer, ReplayValidator, ReplayModeGuard
  │   ├── traits.rs     ← Replayable trait + blanket impl
  │   └── error.rs      ← ReplayError 枚举
  └── mechanism/
      ├── recorder.rs   ← RecordingSession（录制会话封装）+ calculate_frame_checksum
      ├── player.rs     ← PlaybackSession（回放会话封装）+ fast_forward
      └── resources.rs  ← ECS Resource 包装（ReplayModeGuard, RecordingSession, PlaybackSession, FrameCounter）

infra/replay/
  ├── resources.rs      ← re-export core 类型（第 13-15 行）
  ├── systems.rs        ← 帧管理 System（recording_frame_bookend_system, playback_frame_bookend_system, rng_sync_system）
  └── plugin.rs         ← ReplayPlugin 注册入口
```

**关键证据**: `infra/replay/resources.rs` 第 1-16 行：
```rust
pub use crate::core::capabilities::runtime::replay::mechanism::resources::{
    FrameCounter, PlaybackSession, RecordingSession, ReplayModeGuard,
};
pub use crate::shared::random::DeterministicRng;
```

**结论**: Core 定义纯逻辑，Infra 仅做 re-export + ECS System 桥接。教科书级的分层。

### 4.4 runtime/replay/mechanism/resources.rs vs infra/replay/resources.rs — ✅ 无重复（同一定义的两层可见性）

| 模块 | 定义 | 可见性 |
|------|------|--------|
| `core::capabilities::runtime::replay::mechanism::resources` | `ReplayModeGuard`, `RecordingSession`, `PlaybackSession`, `FrameCounter` | `pub(crate)` |
| `infra::replay::resources` | re-export 同类型 | `pub` |

**结论**: Core 层定义 Resource 类型（`pub(crate)`），Infra 层 re-export 使其对外可见。这是 ADR-045 可见性控制的正确实现。

### 4.5 runtime/replay vs runtime/command 中的命令类型 — ⚠️ 缺失转换桥接

| 模块 | 枚举 | 职责 |
|------|------|------|
| `core::capabilities::runtime::command::foundation::types::GameCommand` | `MoveUnit, Attack, CastSpell...` | 完整业务命令（Rich Parameters） |
| `core::capabilities::runtime::replay::foundation::types::ReplayCommand` | `UnitMove, UseAbility, SkipTurn...` | 最小化回放命令（String 标识符） |

**问题**: 两个枚举覆盖相似的业务语义，但参数粒度不同：
- `GameCommand::Attack { attacker_id, target_id, ability_slot }` — 丰富参数
- `ReplayCommand::UseAbility { caster, ability_def_id, target: AbilityTarget }` — 最小化参数

**缺失**: 没有 `From<GameCommand> for ReplayCommand` 或 `fn to_replay_command(&self) -> ReplayCommand` 的转换实现。录制系统需要手动构造 `ReplayCommand`，而非自动从 `GameCommand` 转换。

**建议**: 低优先级。如果录制系统确实需要从 `GameCommand` 构造 `ReplayCommand`，应提供显式转换函数。当前设计可能是有意为之（录制系统可能直接构造 `ReplayCommand` 而不经过 `GameCommand`）。

### 4.6 calculate_frame_checksum vs shared/hashing — ✅ 无重复（不同确定性保证）

| 模块 | 算法 | 用途 |
|------|------|------|
| `core::capabilities::runtime::replay::mechanism::recorder::calculate_frame_checksum` | 手写 `wrapping_mul(31)` + XOR | 回放帧校验和（跨平台确定性） |
| `shared::hashing::fast_hash` | aHash（AES-NI 优化） | 通用高速哈希（HashMap 键） |

**结论**: `calculate_frame_checksum` 使用手写哈希是为了**跨平台确定性保证**（不依赖 CPU 指令集），而 `fast_hash` 使用 aHash 是为了**性能**。两者用途不同，不是重复。

### 4.7 core/capabilities/runtime/plugin.rs vs command/plugin.rs — ✅ 无重复（插件层级）

```rust
// core/capabilities/runtime/plugin.rs
RuntimePlugin {
    fn build() {
        app.add_plugins(CommandPlugin);  // 注册命令处理子插件
    }
}

// core/capabilities/runtime/command/plugin.rs
CommandPlugin {
    fn build() {
        app.init_resource::<CommandQueue>();
        app.add_systems(PreUpdate, command_processing_system);
    }
}
```

**结论**: `RuntimePlugin` 是父插件，`CommandPlugin` 是子插件。标准的插件层级模式。

---

## 5. 其他发现

### 5.1 shared/time vs infra/replay FrameCounter — ✅ 无重复

| 模块 | 类型 | 职责 |
|------|------|------|
| `shared::time::GameTime` | `(frame, turn)` 二元组 | 游戏逻辑时间（Resource） |
| `infra::replay::resources::FrameCounter` | 帧计数器 | 回放帧计数（Resource） |

`GameTime` 是领域时间概念，`FrameCounter` 是回放帧计数。无重复。

### 5.2 shared/random — ✅ 无重复

`DeterministicRng` / `RngSeeds` 在 shared 层定义，被 core（回放）和 infra（ECS 包装）消费。正确用法。

---

## 6. 改进建议优先级排序

| 优先级 | 问题 | 建议 | 影响范围 |
|--------|------|------|---------|
| **P1** | shared/constants 含业务常量 | 迁移到各自领域模块 | shared, core/domains |
| **P2** | shared/game_state 含游戏流程 | 考虑提升为独立 crate 或放入 Core | shared, core, infra, app |
| **P2** | infra validation 未使用 shared Validator trait | 统一校验模式 | infra/registry, infra/localization |
| **P3** | GameCommand → ReplayCommand 缺失转换桥接 | 提供显式转换函数 | core/capabilities/runtime |
| **P3** | Domain 枚举含 infra 前缀 | 后续 logging 重构时统一 | shared/diagnostics |

---

## 7. 总结

**代码重复: 0 处。** 三层之间的类型引用都是正确的层依赖（Infra 引用 Core 类型，Core 引用 Shared 类型）。

**职责越界: 3 处。** shared 层混入了业务规则常量和游戏状态机，这是主要的技术债。

**抽象未利用: 1 处。** infra 层的校验逻辑没有使用 shared 层提供的 Validator 抽象。

**缺失桥接: 1 处。** `GameCommand` → `ReplayCommand` 缺少显式转换函数（低优先级）。

**整体评价**: 架构分层设计合理，`core/capabilities/runtime/` 作为领域规则层正确定义了命令系统和回放系统的核心类型，`infra/replay` 和 `infra/input` 作为技术实现层正确地桥接到 ECS。主要改进方向是清理 shared 层中的业务语义残留。
