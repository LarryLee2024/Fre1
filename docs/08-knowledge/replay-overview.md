---
id: 08-knowledge.replay
title: Replay 回放系统深度解析
status: draft
owner: architect
created: 2026-06-21
updated: 2026-06-21
tags:
  - knowledge
  - replay
  - determinism
  - combat
---

# Replay 回放系统深度解析

> 从开发者噩梦到确定性乐园：回放系统的设计、实现和全部代码。

---

## 0. 先讲个故事：没有回放的噩梦

想象你是一个 SRPG 开发者，游戏已经做了一年。你发现了一个 Bug：某些时候战斗的伤害计算不对，暴击率莫名其妙翻倍了。

没有回放系统，你的调试流程是这样的：

1. **复现**：手动进入战斗，走完特定步骤，祈祷 Bug 再现
2. **定位**：在可能出问题的地方插满 `println!` 和 `tracing::info!`
3. **猜测**：可能是 RNG 顺序错了？可能是 Modifier 多了？可能是 Effect 没按正确顺序应用？
4. **改代码**：凭感觉改一个地方
5. **重来**：再打一遍战斗
6. **发现新 Bug**：刚才的改动修好了暴击率，但伤害浮动又不对了

一天改 50 次配置、重开 50 次战斗，一整天过去了，Bug 还没修好。

现在想象一下有回放系统的世界：

1. **玩家遇到了 Bug**，系统自动录下了整场战斗的 Replay 文件
2. 开发者收到反馈，**加载 Replay 文件**，点一下"步进回放"
3. 在第 47 帧，看到一个 Effect 被错误地应用了两次
4. **修复代码**，重新运行同一个 Replay
5. Replay 在第 142 帧校验不一致——**自动检测到新的 Bug**
6. 修好所有问题，Replay 从头到尾完整通过

这就是 Replay 系统（Data Law 010：Replay 优先于便利）存在的原因——**它把调试从玄学变成科学**。

---

## 1. 什么是 Replay？

**Replay（回放）** 是一个"动作录像机"——它录制所有玩家和 AI 的操作（不是录像），然后在回放时用一个**确定性引擎**重现整个游戏过程。

### 类比

| 现实场景 | Replay 类比 |
|---------|-------------|
| 考试的答题卡 | 记录了你写的每一个字，完全不由你之后怎么改 |
| 钢琴卷帘（MIDI） | 记录了按下哪个键、什么时候按、按多大力——不是录音 |
| 棋谱 | 记录了每一步走法，任何人都可以照着走一遍得到同样的终局 |
| 飞机的黑匣子 | 记录了所有操作和仪表数据，用于事故分析 |

### 核心理念：命令录制，而非状态快照

Replay 不录"帧画面"，而录"帧输入"——每一帧的玩家/AI 操作 + RNG 种子。因为：

```
录画面（视频）：一场 30 分钟战斗 ≈ 几百 MB
录命令（Replay）：一场 30 分钟战斗 ≈ 几十 KB

录画面：只能看，不能调试
录命令：可以逐帧步进、修改输入、自动校验
```

---

## 2. 架构总览：三层各司其职

Fre 项目的 Replay 系统采用**彻底的 DDD 三层分离**：

```
┌─────────────────────────────────────────────────────────────────┐
│  App Layer (app_plugin.rs)                                       │
│  装配顺序：ReplayPlugin → CombatReplayBridgePlugin               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  L2 Infra — 技术实现层（桥接到 ECS）                              │
│                                                                  │
│  src/infra/replay/                                               │
│  ├── plugin.rs    → ReplayPlugin（注册 5 Resource + 4 System）   │
│  ├── resources.rs → 5 个 Bevy Resource 包装（None = 未激活）      │
│  ├── systems.rs   → 4 个帧生命周期系统（frame_counter /          │
│  │                    recording_bookend / playback_bookend /     │
│  │                    rng_sync）                                 │
│  └── events.rs    → 事件重导出（core 层事件的 infra 别名）        │
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  L1 Core / Capabilities — 纯逻辑层（零 Bevy 依赖）                │
│                                                                  │
│  src/core/capabilities/runtime/replay/                           │
│  ├── foundation/          → 纯数据结构                            │
│  │   ├── types.rs         → ReplayFrame, ReplayCommand(8种),     │
│  │   │                      RngStream(4), RngSeeds, ReplayHeader │
│  │   ├── values.rs        → DeterministicRng, ReplayRecorder,    │
│  │   │                      ReplayPlayer, ReplayValidator,       │
│  │   │                      ReplayLog, ReplayMode(3),           │
│  │   │                      ReplayModeGuard                      │
│  │   ├── error.rs         → ReplayError(6 种)                    │
│  │   └── traits.rs        → Replayable trait + ReplayAction      │
│  │                                                                  │
│  ├── mechanism/           → 逻辑实现                              │
│  │   ├── recorder.rs      → RecordingSession（录制会话封装 +      │
│  │   │                      calculate_frame_checksum 校验和函数） │
│  │   └── player.rs        → PlaybackSession（回放会话封装 +      │
│  │                          fast_forward 快速回放）               │
│  ├── events.rs            → 6 个领域事件                           │
│  └── tests/               → 单元测试（61 个）                      │
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  L1 Core / Domains — Combat 桥接层                                │
│                                                                  │
│  src/core/domains/combat/integration/replay/                     │
│  ├── mod.rs              → CombatReplayBridgePlugin              │
│  ├── registry.rs         → build_battle_unit_registry()          │
│  │                          (EntityMapper<BattleUnitId> 构建)     │
│  ├── recording.rs        → 3 个 Observer：                        │
│  │                          start/record_action/stop             │
│  ├── playback.rs         → 2 个 System：                          │
│  │                          dispatch_commands/block_input        │
│  └── tests/              → 集成测试（25 个）                      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 为什么分三层？

| 层 | 为什么存在 | 如果有修改，改哪层 |
|------|----------|-----------------|
| **Core foundation** | 纯数据，不依赖 Bevy，可独立测试 | 新增命令类型、修改 RNG 算法 |
| **Core mechanism** | 录制/回放逻辑，不依赖 ECS，纯函数 | 修改校验算法、新增回放模式 |
| **Infra** | 把纯逻辑接到 Bevy ECS 世界 | 修改 System 执行时机、新增 Resource |
| **Combat bridge** | 把 Replay 接到战斗流程 | 修改录制/回放与战斗的交互方式 |

每一层都可以独立修改，不会溢出到其他层。

---

## 3. 核心数据结构：八种命令、四种 RNG 流

### 3.1 ReplayCommand — 可录制的八种操作

```rust
pub enum ReplayCommand {
    UnitMove { unit: String, path: Vec<String> },
    UseAbility { caster: String, ability_def_id: String, target: AbilityTarget },
    UseItem { user: String, item_instance_id: String, target: Option<String> },
    SkipTurn { unit: String },
    DialogueChoice { speaker: String, choice_id: String },
    ReactionConfirm { reactor: String, trigger_def_id: String, accepted: bool },
    ConfirmTargets { caster: String, ability_def_id: String, selected_targets: Vec<String> },
    Custom { domain: String, command_type: String, params: Vec<(String, String)> },
}
```

每个变体都用 `String`（而非 `Entity`）标识实体，这是关键设计——**回放时 Entity 句柄可能不同**，所以用稳定的 String ID 桥接。

**`Custom` 变体**是跨域扩展点：任何业务域都可以用它录制本域特有的命令，不需要修改 ReplayCommand 本身。

### 3.2 ReplayFrame — 单帧的命令集合

```rust
pub struct ReplayFrame {
    pub frame_number: u64,         // 从 0 开始递增
    pub commands: Vec<ReplayCommand>,  // 本帧的命令
    pub rng_seed_offset: u64,      // RNG 种子偏移
    pub checksum: Option<u64>,     // 校验和（可选，每 N 帧记录一次）
}
```

### 3.3 ReplayLog — 完整回放日志

```rust
pub struct ReplayLog {
    pub header: ReplayHeader,
    pub frames: Vec<ReplayFrame>,
    pub final_checksum: Option<u64>,
}

pub struct ReplayHeader {
    pub schema_version: u32,
    pub game_version: String,
    pub scene_id: String,
    pub participants: Vec<String>,
    pub initial_seed: u64,
    pub total_frames: u64,
}
```

### 3.4 RngSeeds — 四流 RNG 种子

```rust
pub struct RngSeeds {
    pub combat_seed: u64,   // 战斗（命中/暴击/伤害浮动）
    pub drop_seed: u64,     // 掉落/制造随机
    pub ai_seed: u64,      // AI 决策随机
    pub world_seed: u64,   // 世界事件随机
}
```

**为什么分成四个流？** 因为如果只有一个 RNG：
1. AI 决策多消耗了一个随机数 → 战斗随机偏移了一位 → 伤害判定全错了
2. 你没法单独重放 AI 逻辑，因为每次随机调用都影响后续全局状态

四流隔离之后，**AI 决策和战斗判定不互相干扰**。

### 3.5 DeterministicRng — 确定性 PRNG

```rust
pub struct DeterministicRng {
    seeds: RngSeeds,
    counters: HashMap<RngStream, u64>,
}
```

这是全局唯一的 RNG 入口。所有业务系统的随机操作必须通过它：

```rust
// ✅ 正确：通过 DeterministicRng
fn check_crit(rng: ResMut<DeterministicRng>, chance: f32) -> bool {
    rng.0.gen_bool(RngStream::Combat, chance)
}

// ❌ 错误：直接调用 rand
fn check_crit_bad() -> bool {
    rand::random::<f32>() < 0.3  // 回放会断裂！
}
```

RNG 算法使用 MurmurHash3 风格的混合函数，不是加密安全的，但是快速且确定性的。

---

## 4. 录制全流程：从战斗开始到战斗结束

### 4.1 简要序列

```
OnBattleStart─────→UnitActionComplete─────→OnBattleEnd
       │                    │                      │
       ▼                    ▼                      ▼
  build_registry       record_command          session.stop()
  create_session                                -> ReplayLog
```

### 4.2 详细流程

#### 第一步：战斗开始（start_recording_on_battle_begin）

```
OnBattleStart 事件触发
  │
  ├── 枚举所有 CombatParticipant 实体
  ├── 为每个实体分配 BattleUnitId（格式 "bu:{team}:{index}"）
  ├── 建立 EntityMapper<BattleUnitId> 双向映射
  ├── 创建 ReplayHeader（version=1, seed=当前RNG种子）
  ├── 创建 CoreRecordingSession
  │     ├── recorder → ReplayRecorder(checkpoint_interval=60)
  │     └── validator → ReplayValidator
  ├── session.start(header, initial_seed_offset=0)
  └── 插入 Resource: EntityMapper + RecordingSession
```

#### 第二步：战斗进行中 — 每帧

```
PreUpdate:
    frame_counter_system
        counter.0 += 1      ← 帧计数器递增

PostUpdate:
    recording_frame_bookend_system（如果正在录制）
        ├── calculate_frame_checksum(current_frame)  ← 计算校验和
        ├── session.finalize_frame(checksum)         ← 完成当前帧
        ├── session.start_frame(next_number, next)   ← 创建新帧
        └── ...
```

#### 第三步：单位行动（record_unit_action）

```
UnitActionComplete 事件触发
  │
  ├── 从 EntityMapper 获取 BattleUnitId
  ├── 创建 ReplayCommand::SkipTurn { unit: id_str }
  │     （当前是简略版，只录 SkipTurn）
  └── session.record_command(command)
```

#### 第四步：战斗结束（stop_recording_on_battle_end）

```
OnBattleEnd 事件触发
  │
  ├── session.stop(final_checksum)
  │     ├── 完成当前帧
  │     ├── 构建完整 ReplayLog
  │     └── 返回 Result<ReplayLog, ReplayError>
  ├── 清理: recording.0 = None
  └── 清理: mapper.clear()
```

### 4.3 RecordingSession 的生命周期状态机

```
         session.start()
              │
              ▼
        ┌─────────────┐    每帧:
        │  Recording   │────→ recording_frame_bookend_system
        │  Active      │←──── (计算 checksum → 新帧)
        └──────┬──────┘
               │
         session.stop()
               │
               ▼
        ┌─────────────┐
        │  Stopped     │
        │  (ReplayLog) │
        └─────────────┘
```

### 4.4 帧级别的生命周期

```
Frame N 开始
  │
  ├── PreUpdate: frame_counter_system (counter++)
  │
  ├── Update: 业务系统执行、产生 UnitActionComplete 事件
  │            → record_unit_action 拦截 → session.record_command()
  │
  └── PostUpdate: recording_frame_bookend_system
        ├── calculate_frame_checksum(frame)  → 计算本帧校验和
        ├── session.finalize_frame(checksum) → 完成本帧
        └── session.start_frame(N+1, offset) → 准备下一帧
```

---

## 5. 回放流程：从 ReplayLog 到战斗重现

### 5.1 简要序列

```
加载 ReplayLog
  → PlaybackSession.load()
  → PlaybackSession.start()
  → dispatch_combat_replay_commands (每帧)
  → playback_frame_bookend_system (每帧)
  → 全部完成 → ReplayCompleted
```

### 5.2 详细流程

#### 第一步：加载回放日志

```rust
PlaybackSession::new(mode, initial_seed)
  │
  ├── player = ReplayPlayer::new(mode)
  ├── rng = DeterministicRng::with_seed(initial_seed)
  └── validator = ReplayValidator::new()

PlaybackSession::load(log)
  ├── 校验: schema_version <= 1
  ├── 校验: 帧号连续（从 0 递增）
  ├── 校验: 日志非空
  ├── 设置种子: RngSeeds::uniform(initial_seed)
  ├── player.load(log)
  └── validator.start_verification()
```

#### 第二步：逐帧回放

```
PreUpdate:
    block_player_input_during_replay（回放模式）
        input.just_pressed_actions.clear()      ← 阻止真实玩家输入

Update:
    dispatch_combat_replay_commands（回放模式 + 管线暂停）
        ├── 读取 PlaybackSession.current_commands()
        ├── 用 EntityMapper 将 String ID 转回 Entity
        ├── 匹配 TurnQueue.current() 的实体
        ├── 找到匹配 → advance_frame() + trigger(UnitActionComplete)
        └── → 管线恢复

PostUpdate:
    playback_frame_bookend_system
        ├── verify_current_frame()           ← 校验当前帧
        ├── trigger(ReplayFrameProcessed)    ← 发送帧处理事件
        ├── advance_frame() → 更新 RNG 种子
        ├── 如果最后一帧:
        │     ├── mode_guard.is_replay = false  ← 切回正常模式
        │     └── trigger(ReplayCompleted)
        └── session_wrapper.0 = None         ← 清理回放会话

    rng_sync_system（在 playback_bookend 之后）
        └── rng.set_all_seeds(session.rng().get_all_seeds())
            ← 确保全局 DeterministicRng 与回放同步
```

### 5.3 回放命令分发策略

```
管线在 unit_action 阶段暂停
  │
  ├── TurnQueue.current() 获得当前单位 Entity
  ├── dispatch_combat_replay_commands:
  │     ├── 读取 PlaybackSession 当前帧所有命令
  │     ├── 遍历命令，提取 unit/caster/user/reactor/speaker
  │     ├── BattleUnitId::new(cmd_unit_id) → EntityMapper.get_entity()
  │     └── 如果匹配 → trigger(UnitActionComplete)
  │
  └── on_unit_action_complete（原有 Observer）
        └── pipeline.resume()
```

### 5.4 回放模式守卫

```rust
pub struct ReplayModeGuard {
    pub is_replay: bool,
}
```

这个守卫控制整个游戏的行为模式。回放模式下：

- **禁止**读取系统时间
- **禁止**访问文件系统
- **禁止**使用非确定性 RNG
- **禁止**玩家输入影响游戏（通过 `block_player_input_during_replay`）

---

## 6. 校验体系：四道防线确保确定性

### 6.1 帧校验和（calculate_frame_checksum）

每帧结束时计算校验和：

```
checksum = frame_number * 0x9E37_79B9
  XOR (command_hashes...)

command_hash:
  对每个命令变体的 String 字段做 hash = bytes * 31 加权
```

### 6.2 ReplayValidator

```rust
ReplayValidator {
    recording: bool,
    current_frame: u64,
    accumulated_checksum: u64,    // 累计校验和（XOR 所有帧）
    mismatches: Vec<ReplayMismatch>,
}

ReplayMismatch {
    frame: u64,
    expected_checksum: u64,
    actual_checksum: u64,
}
```

### 6.3 版本校验

加载回放日志时检查 `schema_version`：

```
V1 规则:
  - schema_version > 当前版本 → 拒绝
  - 帧号不连续 → 拒绝
  - 日志为空 → 拒绝
```

### 6.4 同步检查点（未来）

每 N 帧记录一次世界状态的哈希值（`SyncCheckpoint`），在回放时逐帧比对。当前框架中的 checksum 是为这一机制预留的基础设施。

---

## 7. 桥接层：Entity ↔ String ID 的桥梁

### 7.1 为什么需要桥接？

回放系统中最大的技术问题是：**Entity 句柄在每次运行都不同**。

```
录制时: Entity(0x7f) = "单位A"
下次启动: Entity(0x7f) ≠ "单位A"（可能有别的 Entity 占了这个地址）
回放时: Entity(0x7f) 根本不存在
```

### 7.2 BattleUnitId

解决方案是使用稳定的 String ID：

```rust
define_string_id! {
    pub BattleUnitId,
    prefix: "bu",
}
```

格式：`"bu:{team_id}:{index}"`，如 `"bu:Player:0"`, `"bu:Enemy:3"`。

### 7.3 EntityMapper<BattleUnitId>

使用 `EntityMapper` 泛型 Resource 做双向映射：

```rust
EntityMapper<BattleUnitId> {
    entity_to_id: HashMap<Entity, BattleUnitId>,
    id_to_entity: HashMap<BattleUnitId, Entity>,
}
```

- **录制时**：`Entity → BattleUnitId`（提取 String 写入 ReplayCommand）
- **回放时**：`BattleUnitId → Entity`（找到当前运行的 Entity，触发命令）

### 7.4 桥接层插件注册

```
CombatReplayBridgePlugin

Resources:
  EntityMapper<BattleUnitId> (空，战斗开始时填充)

Observers:
  OnBattleStart → start_recording_on_battle_begin
  UnitActionComplete → record_unit_action
  OnBattleEnd → stop_recording_on_battle_end

Systems:
  Update: dispatch_combat_replay_commands
  PreUpdate: block_player_input_during_replay
```

---

## 8. infra 层的四个 System

| System | Schedule | 有什么作用 |
|--------|----------|----------|
| `frame_counter_system` | PreUpdate | 帧计数器递增，给录制/回放提供帧号 |
| `recording_frame_bookend_system` | PostUpdate | 完成当前录制帧，计算校验和，创建新帧 |
| `playback_frame_bookend_system` | PostUpdate | 验证回放帧，推进到下一帧，清理完成会话 |
| `rng_sync_system` | PostUpdate | 回放模式：把会话的 RNG 种子同步到全局资源 |

执行顺序（`.chain()` 保证）：

```
PostUpdate:
  recording_bookend → playback_bookend → rng_sync
```

---

## 9. 六种领域事件

所有 Replay 事件都实现了 `ReplayEvent` marker trait：

| 事件 | 触发时机 | 包含什么 |
|------|---------|---------|
| `ReplayStarted` | 回放开始时 | scene_id, total_frames |
| `ReplayFrameProcessed` | 每帧处理完成 | frame_number, total_frames, commands_in_frame |
| `ReplayCompleted` | 回放完成 | total_frames, total_commands, has_mismatches |
| `RecordingStarted` | 录制开始时 | scene_id, initial_seed |
| `RecordingCompleted` | 录制完成 | frames_recorded, commands_recorded |
| `ReplayMismatchDetected` | 校验不一致 | frame, expected_checksum, actual_checksum |

这些事件让其他系统可以监听回放进度——比如 UI 显示回放进度条、日志系统记录回放结果。

---

## 10. 六种错误类型

```rust
pub enum ReplayError {
    VersionMismatch { expected: u32, actual: u32 },
    FrameNumberGap { expected: u64, got: u64 },
    ChecksumMismatch { frame: u64, expected: u64, actual: u64 },
    NotRecording,
    NotPlaying,
    EmptyLog,
}
```

---

## 11. 三种回放模式

```rust
pub enum ReplayMode {
    Full,          // 逐帧执行 + 校验所有 SyncCheckpoint
    FastForward,   // 跳过非关键帧，仅验证 Checkpoint
    StepByStep,    // 单步调试（每帧暂停等外部输入）
}
```

- **Full**：最严格的模式，适合 CI 自动化测试
- **FastForward**：快速验证模式，适合回归测试
- **StepByStep**：开发者调试模式，逐帧分析

---

## 12. Replay 与 Event History 的关系

项目中有两个易混淆的系统，它们的职责严格分离：

| 维度 | Replay | Event History（ADR-059） |
|------|--------|------------------------|
| 录制什么 | 输入命令（玩家的操作、AI 决策、RNG） | 输出事件（领域事件的结构化快照） |
| 为什么录 | 确定性验证、Bug 复现 | 事后分析、调试、QA |
| 完整性 | 必须完整，缺一不可 | 环形缓冲区，溢出丢弃 |
| 因果关系 | 原因 | 结果 |

**互补工作流**：

```
Replay 还原输入序列 → 重现 Bug 场景
    ↓
EventStore 查询输出事件 → 找到异常事件（如错误的伤害值）
    ↓
结合因果链分析 → 确定根因
```

**关键禁止**：

- ❌ 禁止将 Event History 作为 Replay 的替代（Event History 不保证确定性）
- ❌ 禁止 Replay 文件包含 EventStore 数据（单一职责）

---

## 13. 测试覆盖

Replay 系统有 **~86 个测试**，覆盖三层：

### Core 层（~61 个）

```
src/core/capabilities/runtime/replay/tests/
├── unit/
│   ├── types_test.rs         → ReplayCommand, ReplayFrame 构造
│   ├── values_test.rs        → DeterministicRng, ReplayRecorder, ReplayPlayer, ReplayValidator
│   ├── recorder_test.rs      → RecordingSession 生命周期
│   └── player_test.rs        → PlaybackSession 生命周期
```

### Infra 层（~25 个）

```
src/infra/replay/tests/
├── unit/
│   └── resources_test.rs     → Resource 默认值
├── integration/
│   ├── replay_plugin_test.rs      → Plugin 注册
│   ├── recording_lifecycle_test.rs → 录制帧边界
│   ├── playback_lifecycle_test.rs → 回放帧推进
│   └── record_replay_roundtrip_test.rs → 录制→回放完整往返
└── invariant/
    └── rng_determinism_test.rs → RNG 确定性验证
```

### Combat 桥接层（根据审查报告，已包含）

```
src/core/domains/combat/integration/replay/tests/
├── fixtures/
├── integration/
│   ├── recording_test.rs
│   └── playback_test.rs
└── invariant/
    └── replay_invariant_test.rs
```

**审查结果**：✅ PASS（939 passed, 0 failed，其中 86 个 replay 测试）

---

## 14. ADR 设计决策索引

| ADR | 标题 | 核心决策 |
|-----|------|---------|
| ADR-041 | 回放确定性与架构 | RNG 四流隔离、命令录制而非状态快照、SyncCheckpoint、禁止操作 |
| ADR-048 | Replay→Combat 桥接层 | EntityMapper<BattleUnitId> 桥接、最小侵入原则、录制/回放分离 |
| ADR-059 | Event History 架构 | EventStore 是 Replay 的互补系统，不替代 Replay |

---

## 15. 代码阅读指引

| 你想了解什么 | 读哪个文件 |
|------------|-----------|
| ReplayCommand 的 8 种变体 | `core/.../runtime/replay/foundation/types.rs` |
| DeterministicRng 算法 | `core/.../runtime/replay/foundation/values.rs` |
| RecordingSession 封装 | `core/.../runtime/replay/mechanism/recorder.rs` |
| PlaybackSession 封装 | `core/.../runtime/replay/mechanism/player.rs` |
| 校验和计算 | `core/.../runtime/replay/mechanism/recorder.rs`（calculate_frame_checksum） |
| 6 种错误类型 | `core/.../runtime/replay/foundation/error.rs` |
| 6 个领域事件 | `core/.../runtime/replay/events.rs` |
| ReplayPlugin 注册 | `infra/replay/plugin.rs` |
| 4 个帧生命周期 System | `infra/replay/systems.rs` |
| 5 个 Resource 包装 | `infra/replay/resources.rs` |
| BattleUnitId 定义 | `shared/ids/types/battle_unit_id.rs` |
| EntityMapper 泛型 | `shared/ids/mapping/entity_mapper.rs` |
| 桥接层 Plugin | `core/domains/combat/integration/replay/mod.rs` |
| 桥接层注册表 | `core/domains/combat/integration/replay/registry.rs` |
| 桥接层录制 | `core/domains/combat/integration/replay/recording.rs` |
| 桥接层回放 | `core/domains/combat/integration/replay/playback.rs` |
| ReplayEvent marker | `shared/diagnostics/observable.rs` |
| 装配顺序 | `app/app_plugin.rs`（Phase 8） |
| 核心 ADR | `docs/01-architecture/40-cross-cutting/ADR-041-replay-determinism.md` |
| 桥接 ADR | `docs/01-architecture/40-cross-cutting/ADR-048-replay-combat-bridge.md` |
| Event History ADR | `docs/01-architecture/40-cross-cutting/ADR-059-event-history.md` |
| Replay Schema | `docs/04-data/infrastructure/replay_schema.md` |
| Replay 架构详述 | `docs/04-data/foundation/replay_architecture.md` |
| Data Laws | `docs/04-data/README.md` §5（Data Law 010） |
| 审查报告 | `docs/10-reviews/done/replay-bridge-review.md` |

---

## 16. 宪法对照

| 宪法 / Data Law | Replay 如何满足 |
|----------------|----------------|
| Data Law 010: Replay 优先于便利 | 所有随机操作通过 DeterministicRng，禁止时间/文件系统依赖 |
| Data Law 001: Def-Instance 分离 | ReplayLog 是 Persistence 层，不干预 Def/Spec/Instance |
| ADR-041: RNG 四流分离 | Combat/Drop/AI/World 四个独立种子流 |
| ADR-048: 最小侵入桥接 | 通过 Observer 接入事件流，不修改 CombatPipelineDriver |
| 编码规则: 禁止绕过 Registry | ReplayCommand 使用 String ID，通过 EntityMapper 转 Entity |

---

## 17. Future Extension

| 方向 | 目前状态 | 需要什么 |
|------|---------|---------|
| `SyncCheckpoint` 世界状态哈希 | 框架预留（checksum 字段） | 实现 compute_sync_hash（遍历所有 #[replay_key] Component） |
| `.freplay` 二进制文件格式 | 设计文档已完成（replay_architecture.md） | 实现序列化/反序列化 |
| 多种 Command 录制 | 当前只录 SkipTurn | 桥接层扩展 record_unit_action 匹配 action 类型 |
| 回放加载 UI | 未实现 | 文件选择器 + ReplayPlayer 初始化 |
| `fast_forward` 跳过非关键帧 | 函数已实现 | System 层接入 |
| 回放时的 AI 决策模式 | 设计文档已完成 | AI 模块实现"读取录制的命令"模式 |
| EventStore 集成 | ADR-059 已通过 | 实现 EventStore 写入 + 查询接口 |
| 桥接层不变量测试 | 测试脚手架已就绪 | 添加更多场景（如边界 Frame、多单位命令） |
