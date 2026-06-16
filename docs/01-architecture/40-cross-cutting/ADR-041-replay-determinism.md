---
id: 01-architecture.ADR-041
title: ADR-041 — Replay Determinism Architecture
status: proposed
owner: architect
created: 2026-06-16
updated: 2026-06-16
supersedes: none
---

# ADR-041: 回放确定性与架构

## 状态

**Proposed** — 依赖 ADR-002（ECS Communication）、内部管线 ADR（010/011/020）和 `docs/04-data/infrastructure/replay_schema.md`。

## 背景

回放（Replay）是 Fre 项目的核心质量保证手段（Data Law 010：Replay 优先于便利）。所有战斗必须可重现——相同初始状态 + 相同输入 + 相同种子，必须得到完全一致的战斗结果。回放系统横跨所有 Feature，是最重要的横切关注点。

## 引用的领域规则与数据架构

- `docs/04-data/foundation/replay_architecture.md` — Replay 架构详述
- `docs/04-data/infrastructure/replay_schema.md` — Replay Schema
- `docs/04-data/README.md` — Data Law 010（Replay 优先）
- `.trae/rules/SRPG专项规则.md` §八（随机数分层管理）、§十（确定性强制要求）
- `docs/05-testing/test-spec.md` — 回放测试要求

## 决策

### 1. 回放架构总览

```
┌──────────────────────────────────────────────────────────────────┐
│                         Replay System                            │
│                                                                  │
│  ┌──────────────┐          ┌──────────────┐                     │
│  │   Recorder   │          │    Player    │                     │
│  │  (录制模式)   │          │  (回放模式)   │                     │
│  └──────┬───────┘          └──────┬───────┘                     │
│         │                         │                             │
│         │ records                │ replays                     │
│         ▼                         ▼                             │
│  ┌──────────────────────────────────────────────────────┐       │
│  │              ReplayFrame Sequence                     │       │
│  │  [Frame 1] [Frame 2] [Frame 3] ... [Frame N]         │       │
│  └──────────────────────────────────────────────────────┘       │
│                                                                  │
│  ┌──────────────────────────────────────────────────────┐       │
│  │              Determinism Engine                       │       │
│  │  • Seeded PRNG (per RNG stream)                      │       │
│  │  • GameTime (frame count)                            │       │
│  │  • SyncCheckpoint (periodic state hash)              │       │
│  └──────────────────────────────────────────────────────┘       │
└──────────────────────────────────────────────────────────────────┘
```

### 2. ReplayFrame 格式

```rust
/// ReplayFrame — 每帧记录的增量数据
pub struct ReplayFrame {
    pub frame_number: u64,
    pub commands: Vec<RecordedCommand>,
    pub rng_seeds: RngSeeds,
}

/// 录制命令 — 录制的玩家/AI 输入
pub struct RecordedCommand {
    pub command_type: CommandType,
    pub payload: Vec<u8>,       // 序列化后的命令数据
    pub timestamp: GameTime,
}

/// RNG 种子 — 每个 RNG 流独立种子
pub struct RngSeeds {
    pub combat_seed: u64,
    pub drop_seed: u64,
    pub ai_seed: u64,
    pub world_seed: u64,
}

/// 同步检查点 — 每 N 帧记录一次状态 Hash 用于验证
pub struct SyncCheckpoint {
    pub frame_number: u64,
    pub world_hash: [u8; 32],   // 关键状态的 SHA-256
}
```

### 3. 确定性 RNG 管理

#### 3.1 RNG 流分离

根据 SRPG §8.3，随机数按用途拆分独立 RNG 流：

```rust
/// DeterministicRng — 每流一个独立实例
#[derive(Resource)]
pub struct DeterministicRng {
    streams: EnumMap<RngStream, SeededRng>,
}

pub enum RngStream {
    Combat,  // 命中/暴击/伤害浮动
    Drop,    // 掉落/制造随机
    AI,      // AI 决策随机
    World,   // 世界事件随机
}

impl DeterministicRng {
    /// 获取指定流的可变引用
    pub fn stream(&mut self, stream: RngStream) -> &mut SeededRng;

    /// 同步设置所有流种子（回放模式）
    pub fn set_seeds(&mut self, seeds: RngSeeds);

    /// 获取当前所有流种子（用于录制）
    pub fn get_seeds(&self) -> RngSeeds;
}
```

#### 3.2 业务代码使用 RNG

```rust
/// ✅ 正确：通过统一 RNG 服务
fn roll_crit(rng: ResMut<DeterministicRng>, crit_chance: f32) -> bool {
    rng.stream(RngStream::Combat).gen_bool(crit_chance as f64)
}

/// ❌ 错误：直接调用 rand
fn roll_crit_bad() -> bool {
    rand::random::<f32>() < 0.3  // 禁止! 非确定性
}
```

### 4. 录制模式

```rust
/// ReplayRecorder — 录制资源
#[derive(Resource)]
pub struct ReplayRecorder {
    pub is_recording: bool,
    pub frames: Vec<ReplayFrame>,
    pub current_frame: ReplayFrame,
    pub checkpoint_interval: u32,    // 每多少帧一个检查点
}

/// 录制流程：
/// 1. 每帧开始：`RecorderSystem` 创建新的 `ReplayFrame`
/// 2. 帧期间：Input/AI System 产生 Command → 通过 `RecordedCommandSink` 录制
/// 3. 帧结束：录制种子状态
/// 4. 达到 checkpoint_interval：录制 `SyncCheckpoint`
impl Plugin for ReplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ReplayRecorder>()
           .add_systems(PreUpdate, (
               start_frame_recording,
               capture_rng_seeds,
           ))
           .add_systems(PostUpdate, (
               finalize_frame_recording,
               maybe_write_checkpoint,
           ));
    }
}
```

### 5. 回放模式

```rust
/// ReplayPlayer — 回放资源
#[derive(Resource)]
pub struct ReplayPlayer {
    pub is_playing: bool,
    pub frames: Vec<ReplayFrame>,
    pub current_index: usize,
    pub mode: ReplayMode,
}

pub enum ReplayMode {
    /// 完整回放（逐帧执行，检查 SyncCheckpoint）
    Full,
    /// 快速回放（跳过非关键帧，仅验证 Checkpoint）
    FastForward,
    /// 单步调试
    StepByStep,
}
```

### 6. 回放模式下禁止的操作

回放模式通过一个全局 Resource 标记：

```rust
#[derive(Resource)]
pub struct ReplayModeGuard {
    pub is_replay: bool,
}

/// 在回放模式下被禁止的操作
fn guard_external_state(
    guard: Res<ReplayModeGuard>,
) {
    if guard.is_replay {
        // 🟥 禁止读取系统时间
        // 🟥 禁止访问文件系统
        // 🟥 禁止使用非确定 RNG
    }
}
```

### 7. 每个 Feature 的回放责任

| Feature | 回放责任 | 必须录制 |
|---------|---------|---------|
| 全部 | 在 public API 中使用 `Res<DeterministicRng>` 而非 `rand::random()` | 所有随机决策 |
| combat | 管线阶段通过种子确定 | CombatIntent |
| reaction | Reaction 触发条件确定性 | 无额外要求（由 Combat 触发） |
| movement | 寻路算法确定性（使用固定种子） | MoveCommand |
| turn_phase | TurnQueue 确定性 | EndTurnCommand |
| ability | Spec 快照包含种子 | AbilityCastCommand |
| modifier | Modifier 不包含随机逻辑 | 无 |
| cue | Cue 信号非确定性（纯表现） | 不录制 |

## Module Design

```
src/infra/replay/
  ├── plugin.rs              — ReplayPlugin
  ├── resources.rs           — ReplayRecorder, ReplayPlayer, ReplayModeGuard, DeterministicRng
  ├── systems.rs             — start_recording, finalize_frame, playback_step
  ├── events.rs              — ReplayEvent
  └── api.rs                 — ReplayFrame, RecordedCommand, SyncCheckpoint

// 确定性 RNG 在 replay Feature 中实现，但被所有 Feature 使用
```

## Communication Design

| 通信 | 机制 | 方向 |
|------|------|------|
| 录制命令 | `RecordedCommandSink` (非 ECS 接口，直接 push) | 任何 System → replay |
| 回放命令分发 | `ReplayCommandReader` → 直接执行 | replay → 模拟输入 |
| 回放模式切换 | `ReplayModeGuard` Resource 查询 | replay → 所有 System |
| RNG 使用 | `ResMut<DeterministicRng>` | 所有 System → replay |

## 边界定义

### 允许
- 录制模式记录所有 Command 和 RNG 种子
- 回放模式设置确定性种子并重放 Command
- SyncCheckpoint 比较 World 状态 Hash
- 快速回放跳过非关键帧

### 🟥 禁止
- 回放模式下读取外部状态（文件系统、网络、系统时间）
- 非确定性随机数（`rand::random()`）出现在业务代码中
- 回放模式下修改录制数据
- 表现层（Cue/VFX）影响回放确定性
- 回放跨越不同的游戏版本（必须版本匹配或迁移）

## Forbidden

| 禁止行为 | 理由 |
|---------|------|
| 业务代码直接调用 `rand::random()` | 非确定性 |
| 回放时读取系统时钟 | 时间不匹配 |
| 录制后修改 ReplayFrame | 破坏完整性 |
| 版本不匹配的回放加载 | 逻辑可能不一致 |
| 表现层影响回放结果 | 违反逻辑/表现分离 |

## Definition / Instance Design

- **Definition**: `ReplaySettings` (config: checkpoint_interval, max_frames)
- **Instance**: `ReplayRecorder` (Resource), `ReplayPlayer` (Resource), `DeterministicRng` (Resource)
- **Persistence**: `ReplayFrame` 序列化文件（.replay 格式）

## 后果

### 正面
- 所有随机数通过统一路径，完全可控
- 回放录制/播放对称，测试可直接使用
- SyncCheckpoint 提供回放完整性验证
- 各 Feature 回放责任清晰

### 负面
- 每个使用随机数的 Feature 需要接入 `DeterministicRng`
- 回放模式下需要额外的保护代码（禁止外部状态读取）
- ReplayFrame 文件可能很大（每帧录制 Command）

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 仅录制输入，不录种子 | 种子不确定性导致回放不一致 |
| 全局单一 RNG | 各系统随机数互相干扰 |
| 无回放系统 | 违反 Data Law 010，无法保证可测试性 |

## 评审要点

- [ ] `RngSeeds` 的分流是否覆盖所有随机场景？是否需要更多流？
- [ ] SyncCheckpoint 的间隔——每 60 帧一个检查点是否太密？
- [ ] 回放版本不匹配时的行为——拒绝加载还是自动迁移？
- [ ] Command 录制是否覆盖了所有玩家操作？（移动、攻击、使用物品、结束回合等）
