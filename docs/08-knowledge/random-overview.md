---
id: 08-knowledge.random
title: 随机数系统深度解析
status: draft
owner: architect
created: 2026-06-21
updated: 2026-06-21
tags:
  - knowledge
  - random
  - rng
  - determinism
  - replay
  - seeding
---

# 随机数系统深度解析

> 三个 RNG、「DeterministicRng」撞名三次、为什么需要四个随机数流。本文从需求出发，自底向上讲清楚 Fre 项目的随机数设计。

---

## 0. 先讲一个 Bug 故事

假设你做了一个回合制战棋游戏。某天测试报告：「我的弓箭手 60% 暴击率，打了 20 箭一箭都没暴击。」

**第一次尝试调试：**

你打开代码看一眼，暴击判定用的是 `rand::random::<f32>() < 0.6`。你觉得逻辑没问题，改了几行日志，重新编译，跑了一次——这次暴击率又正常了。

**问题在哪？** 因为 `rand::random()` 每次运行种子不同，第一次运行恰好一直没暴击，第二次运行又看上去正常。你无法复现第一次的场景。

**第二次尝试调试：**

你手动设了一个固定种子 `rand::seed_from_u64(42)`，这次暴击率正常了。你加了大量日志，跑到一半程序崩溃了——因为日志里夹了一个 `println!`，破坏了帧时序，把下一帧的命令提前消费了。

**根源是什么？** 

看起来是三个独立的问题：
1. **种子不可控**——每次运行结果不同，无法复现 Bug
2. **随机数流混杂**——AI 决策消耗了战斗的随机数，战斗判定偏移了
3. **非确定性干扰**——时间/文件系统/网络等外部状态影响了判定逻辑

这三个问题，就是 Fre 随机数系统要解决的三个核心挑战。

---

## 1. 问题全景

### 1.1 游戏中的随机数在哪里？

一个 SRPG 游戏的随机数消费点散落在各处：

| 场景 | 随机用途 | 当前状态 |
|------|---------|---------|
| 命中判定 | 攻击是否命中（d20 投骰 vs AC） | ❌ 尚未接入统一 RNG |
| 暴击判定 | 攻击是否暴击 | ❌ 尚未接入统一 RNG |
| 伤害浮动 | 伤害在 ±10% 范围内波动 | ❌ 尚未接入统一 RNG |
| AI 决策 | AI 选择哪个技能、哪个目标 | ❌ 尚未接入统一 RNG |
| 掉落物品 | 杀死敌人掉什么装备 | ❌ 尚未接入统一 RNG |
| 世界事件 | 随机遭遇、随机天气 | ❌ 尚未接入统一 RNG |

**关键发现：游戏目前还没有真正使用任何统一 RNG 系统。** DeterministicRng 和 SeededRng 虽然都已实现，但业务代码（combat、AI、drop 等）尚未接入。

### 1.2 三个设计要求

| 要求 | 为什么 | 怎么做 |
|------|--------|--------|
| **确定性（Deterministic）** | 同一种子 + 相同输入 = 完全相同的结果。这是 Replay 的基石。 | 使用种子驱动的 PRNG，禁止系统时间/硬件随机 |
| **流隔离（Stream Isolation）** | AI 决策消耗的随机数不能影响战斗判定。否则 AI 多思考一步，暴击率就变了。 | 按用途拆分为 4 个独立的 RNG 流 |
| **可迁移（Migratable）** | 未来可以升级 PRNG 算法而不破坏已有的 Replay 文件 | 版本化种子格式，算法变更时提供迁移路径 |

---

## 2. 三套 RNG 系统，各司其职

Fre 项目目前有三套 RNG 系统共存，各自服务于不同的场景。外加一个仅供测试的第四套。理解「为什么有三套」是理解整个设计的关键。

```
┌────────────────────────────────────────────────────┐
│                    业务代码                         │
│  (combat / AI / drop / world)                      │
│                                                    │
│  ❌ 当前尚未接入任何统一 RNG                        │
│  📋 目标：全部走 DeterministicRng（四流版本）       │
└──────────────┬──────────────────────────┬──────────┘
               │                          │
               ▼                          ▼
┌──────────────────────┐    ┌──────────────────────────┐
│  GameRng             │    │  DeterministicRng         │
│  (shared/random/)    │    │  (replay/foundation/)     │
│                      │    │                          │
│  🔴 旧系统           │    │  🟢 新系统（Replay 目标）│
│  算法: ChaCha12      │    │  算法: MurmurHash3 混合   │
│  流: 单流（无隔离）   │    │  流: 4 流（Combat/Drop   │
│  用途: 所有旧代码     │    │       /AI/World）         │
│  状态: 不推荐使用     │    │  用途: 新代码 + Replay   │
│                      │    │  状态: ✅ 已实现但未接入   │
└──────────────────────┘    └──────────────────────────┘
         │                            │
         ▼                            ▼
┌──────────────────────────────────────────────────────┐
│                SeededRng (shared/random/)             │
│                                                      │
│  「门面」包装 ChaCha12Rng，提供统一 API               │
│  GameRng 的内部实现就是 SeededRng                     │
│  DeterministicRng（回放版）不依赖 SeededRng          │
│  有自己的独立算法（MurmurHash3）                     │
└──────────────────────────────────────────────────────┘

                单独有一只：
┌──────────────────────────────────────────────────────┐
│  DeterministicRng (shared/testing/deterministic.rs)  │
│                                                      │
│  仅供测试使用！包装 StdRng，种子固定 42               │
│  与上面两个 DeterministicRng 完全不同的类型            │
│  命名撞车，但职责清晰：只在 `#[cfg(test)]` 下出现     │
└──────────────────────────────────────────────────────┘
```

### 2.1 为什么有三套？

这不是设计失误，而是**渐进式迁移的自然产物**：

| 阶段 | 发生什么 | 引入的 RNG |
|------|---------|-----------|
| 初期 | 游戏需要随机数，最简单的方案 | `GameRng` / `SeededRng`（ChaCha12） |
| 中期 | 架构升级，需要 Replay 确定性和流隔离 | `DeterministicRng`（MurmurHash3，4 流） |
| 测试 | 测试需要确定性的随机数，独立于游戏逻辑 | `TestRng`（StdRng，shared/testing） |
| 未来 | 旧代码迁移到新系统，最终只剩一个 | 迁移完成后淘汰 `GameRng` + `SeededRng` |

**当前状态：迁移中。** 新旧系统并存，旧代码尚未接入新系统。

---

## 3. 第一套系统：SeededRng / GameRng（ChaCha12）

### 3.1 SeededRng — 种子驱动的确定性 PRNG

**位置**: `src/shared/random/mod.rs`

```rust
/// 种子驱动的确定性 PRNG。
/// 包装 ChaCha12Rng，从 u64 种子初始化。
/// 同一种子总是产生完全相同的随机数序列。
pub struct SeededRng(ChaCha12Rng);
```

**关键特性：**

| 特性 | 值 |
|------|-----|
| 底层算法 | ChaCha12（对流密码 ChaCha 的 12 轮变体） |
| 安全性 | 密码学安全（CSPRNG） |
| 种子类型 | `u64`（内部扩展为 32 字节） |
| 确定性 | ✅ 同一种子 → 完全相同的序列 |
| 可移植性 | 跨平台一致（ChaCha 是标准算法） |

**为什么用 ChaCha12？** 它的速度大约是 ChaCha20 的 1.6 倍，安全性对于游戏场景足够（我们不需要防攻击，只需要确定性）。这是旧系统——设计时只考虑了「确定性的随机数」这个基本要求，还没有想到流隔离。

### 3.2 GameRng — 全局 RNG Resource

```rust
#[derive(Resource)]
pub struct GameRng {
    inner: SeededRng,
}

impl GameRng {
    pub fn gen_range(&mut self, min: u64, max: u64) -> u64 { ... }
    pub fn gen_range_f32(&mut self, min: f32, max: f32) -> f32 { ... }
    pub fn gen_bool(&mut self, probability: f32) -> bool { ... }
    pub fn reseed(&mut self, seed: u64) { ... }
}

impl Default for GameRng {
    fn default() -> Self {
        Self::new(42)  // 默认种子 42
    }
}
```

**设计思路：**

- `GameRng` 是一个 Bevy Resource，任何 System 都可以通过 `ResMut<GameRng>` 访问
- 默认种子 42，确保开发环境的一致性
- `reseed()` 允许在加载存档或开始战斗时重新设定种子
- **只有一个内部 RNG 实例**，所有随机数消费都走同一个流

> **⚠️ 当前状态：`GameRng` 已标记为 `#[deprecated]`。**
>
> `GameRng` 是单流 RNG，AI 和战斗随机互相扰动时回放会断裂。新代码应使用 `DeterministicRng`（四流版本，`core::capabilities::runtime::replay::foundation::DeterministicRng`），其提供了 Combat/AI/Drop/World 四个独立流，互不干扰。

**最大局限：** 单流设计。AI 决策、战斗判定、掉落生成全部共用同一个 ChaCha12 序列。只要有一个系统多消费了一个随机数，其他所有系统的结果都会偏移。

### 3.3 已处理的遗留问题

`SeededRng` 曾经有一组被注释掉的 `RngCore` / `CryptoRng` trait 实现，原因是 `rand` 0.10 API 变更（`RngCore` 移至 `rand_core`，`rand::Error` 被移除）。

**结论：不再修复。** 既然 `GameRng` 已标记为 `#[deprecated]`，且新代码应使用四流 `DeterministicRng`，这些 trait 实现没有保留价值。对应的注释代码已被清理。

---

## 4. 第二套系统：DeterministicRng（MurmurHash3，四流）

### 4.1 流隔离：为什么需要四个 RNG？

想象一下没有流隔离的世界：

```
之前（单流 RNG）：
  AI 决策开始 → 消耗 3 个随机数（AI.stream）
  战斗判定开始 → 消耗 1 个随机数（Combat.stream）
  
  如果 AI 多思考了一步（消耗 4 个随机数而不是 3 个）：
  AI 决策开始 → 消耗 4 个随机数
  战斗判定开始 → 消耗的随机数已经偏移了 1 位 ← Bug！
  
  结果是：AI 的思考深度改变，战斗的暴击率也跟着变了。
```

流隔离之后：

```
之后（四流 RNG）：
  AI 决策开始 → 消耗 3 个随机数（AI.stream，独立计数器）
  战斗判定开始 → 消耗 1 个随机数（Combat.stream，独立计数器）
  
  AI 多思考了一步（消耗 4 个随机数）：
  AI 决策开始 → 消耗 4 个随机数（AI.stream，独立计数器）
  战斗判定开始 → 消耗 1 个随机数（Combat.stream，完全不受影响）✅
```

**四个流各管各的，互不干扰。**

### 4.2 RngStream 枚举

```rust
// src/core/capabilities/runtime/replay/foundation/types.rs
pub enum RngStream {
    Combat,  // 命中/暴击/伤害浮动
    Drop,    // 掉落/制造随机
    AI,      // AI 决策随机
    World,   // 世界事件随机
}
```

| 流 | 用途 | 为什么独立 |
|-----|------|-----------|
| Combat | 命中判定、暴击判定、伤害浮动 | 这是 Replay 最核心的验证目标 |
| Drop | 掉落生成、制造结果 | 不影响战斗结果，可独立调试 |
| AI | AI 决策 | AI 消耗随机数的模式不可预测，必须隔离 |
| World | 世界事件、天气、随机遭遇 | 完全不相关，隔离后互不影响 |

### 4.3 RngSeeds — 种子集合

```rust
pub struct RngSeeds {
    pub combat_seed: u64,
    pub drop_seed: u64,
    pub ai_seed: u64,
    pub world_seed: u64,
}
```

提供了两个构造方式：

- `RngSeeds::uniform(seed)` — 所有流使用同一种子（最简单，适合初始状态）
- `RngSeeds::new(combat, drop, ai, world)` — 每个流独立种子（用于高级场景）

### 4.4 DeterministicRng — 四流 PRNG

**位置**: `src/core/capabilities/runtime/replay/foundation/values.rs`

```rust
pub struct DeterministicRng {
    seeds: RngSeeds,                          // 4 个流的当前种子
    counters: HashMap<RngStream, u64>,        // 每个流的调用计数器
}
```

**核心算法（MurmurHash3 风格混合器）：**

```rust
pub fn next_u64(&mut self, stream: RngStream) -> u64 {
    let counter = self.counters.get(&stream).copied().unwrap_or(0);
    self.counters.insert(stream, counter + 1);

    // 不同流有不同的偏置常量（避免流间碰撞）
    let stream_offset = match stream {
        RngStream::Combat => 0x9E37_79B9_7F4A_7C15u64,
        RngStream::Drop   => 0xBF58_4766_71CE_4E5Bu64,
        RngStream::AI     => 0x3C6E_F372_FE94_7A9Bu64,
        RngStream::World  => 0x6A09_E667_F3BC_C4C9u64,
    };

    // 种子 + 流偏置 + 计数器 → 状态
    let state = self.seeds.get(stream)
        .wrapping_add(stream_offset)
        .wrapping_add(counter);

    // MurmurHash3 风格的混合器
    let mut x = state;
    x = x.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 27;
    x = x.wrapping_mul(0xBF58_4766_71CE_4E5B);
    x ^= x >> 31;
    x
}
```

**为什么用自实现的哈希混合器而不是 ChaCha12？**

| 考虑 | ChaCha12 | MurmurHash3 混合器 |
|------|----------|-------------------|
| 速度 | 慢（密码学安全级别） | 快（3 次乘 + 2 次移位 + 2 次 XOR） |
| 流隔离 | 需要 4 个独立实例 | 一个实例 + 4 个计数器 |
| 种子管理 | 每个实例有自己的状态 | 种子是纯数据，可快照/恢复 |
| 确定性 | ✅ | ✅ |
| 安全性 | 密码学安全 | 非加密（不需要） |

**结论：** 游戏不需要密码学安全的随机数（不存在攻击者通过预测随机数获利的场景）。我们需要的是**快速、可快照、可恢复**的确定性 PRNG。MurmurHash3 风格的混合器完美满足这些要求。

**提供的 API：**

| 方法 | 用途 |
|------|------|
| `next_u64(stream)` | 生成 0..u64::MAX 范围内的整数 |
| `next_f32(stream)` | 生成 0.0..1.0 范围内的浮点数 |
| `gen_bool(stream, prob)` | 以给定概率生成布尔值 |
| `gen_range(stream, min, max)` | 生成 min..max 范围内的整数 |

### 4.5 种子偏移机制：initial_seed + rng_seed_offset

这是理解 RNG 如何在帧间流动的关键。每一帧的 RNG 种子由两部分决定：

```
frame_N 的 RNG 种子 = initial_seed + frame_N.rng_seed_offset
```

**为什么不是直接递增种子？**

```
不好方案：种子逐帧递增 initial_seed + frame_number
  问题：如果跳过了第 3 帧（fast_forward），你就不知道第 4 帧的种子是什么
       因为中间帧的随机数消费可能影响了种子的内部状态
  结论：有状态 RNG 不能跳过帧

好方案：种子每帧固定为 initial_seed + frame.rng_seed_offset
  结论：无状态 RNG，每帧种子只由帧号决定，可以任意跳过帧
```

**录制时如何生成 offset：**

```rust
// 录制时，recording_frame_bookend_system 每帧：
fn recording_frame_bookend_system(mut recording: ResMut<RecordingSession>) {
    let Some(ref mut session) = recording.0 else { return };

    let current_frame = session.recorder.current_frame_number();

    // 计算本帧的 rng_seed_offset = 当前帧号
    // 简单设计：offset = frame_number
    let offset = current_frame;

    // 完成当前帧（含校验和）
    session.finalize_frame(Some(session.calculate_checksum()));

    // 开始下一帧（指定 offset）
    session.start_frame(current_frame + 1, offset + 1);
}
```

**回放时如何重置种子：**

```rust
// 每次 advance_frame 重新计算种子：
pub fn advance_frame(&mut self) -> bool {
    if !self.player.advance_frame() {
        return false;
    }
    // 取当前帧的 rng_seed_offset，加到 initial_seed 上
    if let Some(frame) = self.player.current_frame() {
        let seeds = RngSeeds::uniform(
            self.initial_seed.wrapping_add(frame.rng_seed_offset)
        );
        self.rng.set_all_seeds(seeds);
    }
    true
}
```

这套设计的核心优势：**任意帧的 RNG 状态只取决于帧号，不取决于之前帧的执行历史。** 这使得：

- `fast_forward` 可以直接跳到最后一帧
- `StepByStep` 可以倒退到之前的帧（只需要重新加载对应的种子）
- 每帧的校验和独立于其他帧

### 4.6 帧校验和与 RNG 的关系

帧校验和用于验证「录制时的 RNG 消费模式」是否与「回放时的 RNG 消费模式」一致：

```rust
// recorder.rs — calculate_frame_checksum
pub fn calculate_frame_checksum(frame: &ReplayFrame) -> u64 {
    let mut checksum = frame.frame_number.wrapping_mul(0x9E37_79B9);

    for cmd in &frame.commands {
        let cmd_hash = command_hash(cmd);  // 命令的 String 字段加权 hash
        checksum ^= cmd_hash;
    }

    checksum
}
```

校验和包含了两方面信息：
1. **帧号** —— 确保帧顺序正确
2. **命令哈希** —— 确保本帧的命令与录制时完全一致（包括命令类型和参数）

命令的消费模式决定了 RNG 的消费模式。如果回放时命令数量或顺序不同，校验和会不匹配，从而捕获到 RNG 不一致。

### 4.7 DeterministicRng 的帧级生命周期（完整时序）

```
录制时：
  帧 N 开始 → RNG 种子 = initial_seed + frame_N.rng_seed_offset
            → 业务系统通过 DeterministicRng 消费随机数
  帧 N 结束 → 帧校验和包含 RNG 种子状态
            → 下一帧种子偏移被记录

回放时：
  帧 N 开始 → RNG 种子 = initial_seed + frame_N.rng_seed_offset
            → 业务系统通过 DeterministicRng 消费随机数
            → 消费序列与录制时必须完全一致
  帧 N 结束 → 校验和比对：实际计算的 checksum == 录制时的 checksum
```

关键机制：**每一帧开始时将 RNG 种子重置为该帧的种子偏移。** 这意味着：

- 每一帧的随机数序列是**由帧号完整决定的**
- 帧之间的 RNG 状态**不累积**
- 回放时可以跳过任意帧（`fast_forward`），不需要知道中间帧的 RNG 状态
- 调试时可以直接跳到第 47 帧，RNG 状态仍然是正确的

---

## 5. 第三套系统：针对测试的 TestRng（StdRng）

**位置**: `src/shared/testing/deterministic.rs`

```rust
/// 测试用确定性 RNG，固定种子保证跨平台一致。
pub struct TestRng {
    rng: StdRng,
}

impl TestRng {
    pub fn new() -> Self { Self::with_seed(42) }        // 默认种子 42
    pub fn with_seed(seed: u64) -> Self { ... }
    pub fn gen_range(&mut self, low: u32, high: u32) -> u32 { ... }
    pub fn gen_f32(&mut self) -> f32 { ... }
    pub fn gen_bool(&mut self, probability: f64) -> bool { ... }
    pub fn fill_bytes(&mut self, buf: &mut [u8]) { ... }
}
```

**为什么测试版不叫 `DeterministicRng`？**

早期版本两个类型同名，造成严重混淆——写业务代码时根本看不清到底用的是哪个。重构后测试版改名 `TestRng`，与四流生产版彻底区分。规则很简单：

- 生产代码 → `DeterministicRng`（四流，`core/.../replay/foundation/`）
- 单元测试 → `TestRng`（单流，`shared/testing/`）

**与回放版 DeterministicRng 的异同：**

| 维度 | 回放版 `DeterministicRng` | 测试版 `TestRng` |
|------|---------------------------|-----------------|
| 路径 | `replay/foundation/values.rs` | `shared/testing/deterministic.rs` |
| 算法 | MurmurHash3 风格 | StdRng（ChaCha12） |
| 流隔离 | 4 流（RngStream） | 单流 |
| 种子 | 手动指定 | 默认 42 |
| 用途 | 生产环境的确定性随机 | 单元测试的确定性随机 |

测试版 `TestRng` 让单元测试不需要依赖回放系统的完整基础设施。测试一个函数需要随机数时，直接用 `TestRng` 即可。

---

## 6. 回放时 RNG 如何同步

### 6.1 rng_sync_system — 连接回放和全局 RNG 的桥梁

**位置**: `src/infra/replay/systems.rs`

```rust
pub fn rng_sync_system(
    session: Res<PlaybackSession>,
    mut rng: ResMut<DeterministicRng>,
) {
    // 只有在活跃回放时才同步
    let Some(ref session) = session.0 else { return };
    if !session.player.is_playing || session.is_finished() {
        return;
    }

    // 把 PlaybackSession 内部的 RNG 种子同步到全局 Resource
    let seeds = session.rng().get_all_seeds();
    rng.0.set_all_seeds(seeds);
}
```

**为什么需要这个？**

回放时，有两个 `DeterministicRng` 实例：

1. **`PlaybackSession` 内部的 RNG** —— 由回放日志的种子驱动，每帧更新
2. **全局 `Res<DeterministicRng>`** —— 业务系统通过它获取随机数

`rng_sync_system` 的作用就是让全局 RNG 与回放 RNG 保持同步。它在 `playback_frame_bookend_system` 之后执行，确保帧推进后种子立即同步。

**执行顺序（`PostUpdate` 中 `.chain()` 保证）：**

```
recording_bookend → playback_bookend → rng_sync
```

### 6.2 录制时的种子初始化

```rust
// recording.rs
pub(crate) fn start_recording_on_battle_begin(..., rng: Option<Res<DeterministicRng>>) {
    let initial_seed = rng.map(|r| r.0.get_seed(RngStream::Combat)).unwrap_or(42);
    let header = ReplayHeader::new(1, game_version, scene_id, initial_seed);
    session.start(header, 0);
}
```

录制开始时，读取当前全局 DeterministicRng 的 Combat 流种子作为初始种子。后续每一帧的 `rng_seed_offset` 从 0 开始递增，记录在 `ReplayFrame` 中。

---

## 7. 当前代码库中的使用现状

### 7.1 接入状态总览

| 模块 | 应该用哪个 RNG | 实际用的哪个 | 状态 |
|------|--------------|-------------|------|
| Replay 录制/回放 | DeterministicRng（四流） | ✅ DeterministicRng | 已实现 |
| Replay rng_sync | DeterministicRng（四流） | ✅ DeterministicRng | 已实现 |
| Combat（战斗判定） | DeterministicRng (Combat) | ❌ 未接入 | 待迁移 |
| AI 决策 | DeterministicRng (AI) | ❌ 未接入 | 待迁移 |
| 掉落系统 | DeterministicRng (Drop) | ❌ 未接入 | 待迁移 |
| 世界事件 | DeterministicRng (World) | ❌ 未接入 | 待迁移 |
| 单元测试 | `TestRng`（shared/testing） | ✅ `TestRng` | 可用 |

**现状：RNG 的基础设施已全部完成，但业务代码还没有接入。** DeterministicRng（四流版本）在 Replay 系统内部自用，但 combat、AI、drop 等模块尚未从 `rand::random()` 或 `GameRng` 迁移过来。

### 7.2 GameRng 和 DeterministicRng 的关系

```
                 注册为 Bevy Resource
SeededRng ──→ GameRng ──────────────────→ 旧代码访问点（尚未被使用）
                │
                │ 封装 ChaCha12
                │
                ▼
          ChaCha12Rng（rand_chacha crate）

                 注册为 Bevy Resource
DeterministicRng ──────────────────────→ 新代码访问点（Replay 系统已使用）
(replay/foundation/)
                │
                │ MurmurHash3 风格混合器
                │
                ▼
          自实现的 next_u64/next_f32/gen_bool/gen_range
```

### 7.3 RNG 在 Combat Pipeline 中的预定插入点

当业务代码接入 RNG 时，Combat Pipeline 的 Generate 阶段是 RNG 的主要消费点。以下是 Combat Pipeline 流程中需要随机数的位置：

```
CombatIntent 入场
  │
  ├── Generate 阶段（随机数在此集中消费）
  │     ├── 命中判定（d20 + 命中修正 vs AC）
  │     │     └── rng.gen_range(Combat, 1, 21)  → d20 投骰
  │     │
  │     ├── 暴击判定（仅当 d20 在暴击范围内）
  │     │     └── rng.gen_bool(Combat, crit_chance)  → 是否暴击
  │     │
  │     ├── 伤害浮动（暴击/非暴击的伤害倍率）
  │     │     └── rng.gen_range(Combat, min, max)  → 伤害波动
  │     │
  │     └── 特殊效果触发（如 25% 概率附加中毒）
  │           └── rng.gen_bool(Combat, proc_chance)  → 特效触发
  │
  ├── Modify 阶段（Modifier 影响数值）
  │     └── 此处不消费随机数，只做数值叠加
  │
  ├── Execute 阶段（执行最终伤害/治疗）
  │     └── 此处不消费随机数，只应用最终结果
  │
  └── Resolve 阶段（后效处理）
        └── 此处不消费随机数，只触发 Observer 链

重要约束：所有 RNG 调用必须集中在 Generate 阶段，且调用顺序固定。
          同一帧内 RNG 调用顺序变化 → 校验和不一致 → 回放断裂。
```

### 7.4 迁移路线图

```
当前状态：
  ┌──────────────┐    ┌──────────────────┐
  │  GameRng     │    │ DeterministicRng │
  │  (旧, 单流)  │    │  (新, 四流)      │
  │  已实现      │    │  已实现          │
  │  未接入业务  │    │  仅 Replay 自用  │
  └──────────────┘    └──────────────────┘

第一步（简单）：
  把 GameRng 垫在 DeterministicRng 下面
  → GameRng 内部包装 DeterministicRng 而非 SeededRng

第二步（核心）：
  combat 域的命中/暴击/伤害浮动接入 DeterministicRng::Combat 流
  → 这是 Replay 最关心的验证目标

第三步（扩展）：
  AI 决策接入 DeterministicRng::AI 流
  掉落系统接入 DeterministicRng::Drop 流

第四步（收尾）：
  淘汰 GameRng / SeededRng
  全部随机数消费走 DeterministicRng
```

---

## 8. 执行现状：代码的真实状态

> 前面的 7 节描述了全部 RNG 设计。本节告诉你实际代码里哪些已经实现、哪些还是蓝图。

### 8.1 三套 RNG 各自的状态

| RNG 系统 | 实现 | 单元测试 | 业务接入 | 总状态 |
|---------|------|---------|---------|--------|
| `SeededRng` + `GameRng`（shared/random/） | ✅ 全部实现（104 行） | ✅ 有测试文件 | ❌ 无业务代码使用 | 🟡 已实现、未使用、待淘汰 |
| `DeterministicRng` 四流（replay/foundation/） | ✅ 全部实现（470 行 values.rs） | ✅ 6 个不变量测试 | ⏳ 仅 Replay 系统自用 | 🟢 已实现、部分使用 |
| `TestRng`（shared/testing/） | ✅ 全部实现（52 行） | ❌ 无专门测试 | ✅ 被测试代码直接使用 | 🟢 已实现、可使用 |

### 8.2 功能维度状态

| 功能 | 状态 | 证据 |
|------|------|------|
| `SeededRng` 构造（new/from_seed） | ✅ 已完成 | `shared/random/mod.rs` |
| `GameRng` Resource（gen_range/gen_bool/reseed） | ✅ 已完成 | `shared/random/mod.rs` |
| `RngStream` 4 个变体 + all() + name() | ✅ 已完成 | `types.rs` |
| `RngSeeds`（uniform/new/get/set） | ✅ 已完成 | `types.rs` |
| `DeterministicRng` next_u64/next_f32/gen_bool/gen_range | ✅ 已完成 | `values.rs` 170 行 |
| `DeterministicRng.set_all_seeds` 计数器重置 | ✅ 已完成 | `values.rs` |
| 回放时 `rng_sync_system` | ✅ 已完成 | `systems.rs` |
| 录制时初始种子读取 | ✅ 已完成 | `recording.rs` |
| `PlaybackSession.advance_frame` RNG 种子更新 | ✅ 已完成 | `player.rs` |
| `RngSeeds::uniform(initial_seed + offset)` 帧种子 | ✅ 已完成 | `player.rs` |
| `SeededRng` `RngCore`/`CryptoRng` impl | 🟡 P2 TODO | `shared/random/mod.rs:97` rand 0.10 兼容 |
| **Combat 命中判定接 RNG** | ❌ 未开始 | 业务代码尚未编写 |
| **Combat 暴击判定接 RNG** | ❌ 未开始 | 业务代码尚未编写 |
| **Combat 伤害浮动接 RNG** | ❌ 未开始 | 业务代码尚未编写 |
| **AI 决策接 RNG** | ❌ 未开始 | AI 系统尚未实现 |
| **掉落系统接 RNG** | ❌ 未开始 | 掉落系统尚未实现 |
| **世界事件接 RNG** | ❌ 未开始 | 世界事件系统尚未实现 |
| `GameRng` → `DeterministicRng` 迁移 | ❌ 未开始 | 需要等业务代码先接入 |

### 8.3 执行现状判断

```
RNG 基础设施（~726 行代码）:
  SeededRng + GameRng:           ████████████████████████████████  100%
  DeterministicRng（四流）:      ████████████████████████████████  100%
  `TestRng`（测试版）:              ████████████████████████████████  100%
  不变量测试（6 个）:            ████████████████████████████████  100%

业务代码接入:
  Replay 系统自用:               ████████████████████████████████  100%
  Combat 命中/暴击/伤害:         ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░   0%
  AI 决策:                       ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░   0%
  掉落/制造:                     ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░   0%
  世界事件:                      ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░   0%

总体进度: ████████░░░░░░░░░░░░░░░░░░░░░░   ~25%
（投入产出倒挂：80% 的代码完成了基础设施，但 0% 的业务逻辑接入了）
```

### 8.4 RNG 不变量测试（6 个）

全部 6 个 RNG 不变量测试位于 `src/infra/replay/tests/invariant/rng_determinism_test.rs`：

| # | 测试名 | 验证内容 |
|---|--------|---------|
| 1 | `rng_output_is_deterministic_across_instances` | 相同种子 + 相同调用次数 → 不同实例产生完全相同的输出（4 个流各验证 100 次） |
| 2 | `rng_streams_produce_different_sequences` | 不同 RNG 流（Combat/Drop/AI/World）使用同一种子时，产生的输出不同 |
| 3 | `rng_gen_range_within_bounds` | `gen_range(5,10)` 的 1000 次调用全部返回 `[5,10)` 范围内的值 |
| 4 | `rng_gen_bool_deterministic` | `gen_bool(0.5)` 在相同种子 + 相同调用次数下结果一致 |
| 5 | `rng_different_seed_different_output` | 不同种子（1 vs 2）产生完全不同的输出序列 |
| 6 | `rng_set_all_seeds_resets_to_initial_state` | `set_all_seeds()` 重置计数器后，下一次输出与新建实例一致 |

## 9. 使用指南

### 8.1 新代码应该怎么写

```rust
// ✅ 正确的做法：通过 ResMut<DeterministicRng> 获取随机数
fn check_crit(
    mut rng: ResMut<DeterministicRng>,
    crit_chance: f32,
) -> bool {
    rng.gen_bool(RngStream::Combat, crit_chance)
}

fn roll_damage(
    mut rng: ResMut<DeterministicRng>,
    base_damage: u64,
    variance: u64,
) -> u64 {
    let min = base_damage.saturating_sub(variance);
    let max = base_damage + variance;
    rng.gen_range(RngStream::Combat, min, max)
}

fn ai_choose_target(
    mut rng: ResMut<DeterministicRng>,
    count: u64,
) -> usize {
    rng.gen_range(RngStream::AI, 0, count) as usize
}
```

### 8.2 反模式清单

| 反模式 | 为什么不行 | 正确做法 |
|--------|-----------|---------|
| `rand::random::<f32>()` | 非确定性，Replay 断裂 | `rng.gen_bool(stream, prob)` |
| `rand::thread_rng()` | 线程局部变量，不可控 | `ResMut<DeterministicRng>` |
| `GameRng.gen_range(...)` | 单流，隔离性差 | 使用 DeterministicRng（四流） |
| 直接使用 `SeededRng` | 太底层，没有 Resource 管理 | 通过 `ResMut<DeterministicRng>` |
| 在回放模式中使用系统时间作为种子 | 破坏确定性 | 种子由 ReplayFrame 提供 |
| 多个系统间共享非 RNG 的随机源 | 无法录制和回放 | 所有随机通过 DeterministicRng |

### 8.3 如何调试随机数问题

1. **检查使用的 RNG 流是否正确**——Combat 判定应该用 `RngStream::Combat`，不是 `RngStream::AI`
2. **检查 RNG 调用次数是否与录制一致**——在关键帧对比录制时的 RNG 计数器和回放时的计数器
3. **使用 StepByStep 模式逐帧回放**——在每一帧暂停，检查 `DeterministicRng` 的四个流种子
4. **对比 checksum**——帧校验和不一致说明该帧的 RNG 消费模式与录制时不同

---

## 10. 相关设计决策

| ADR / 文档 | 内容 |
|-----------|------|
| ADR-041 §3 | 确定性 RNG 管理——四流分离设计 |
| Data Law 010 | Replay 优先于便利——确定性 RNG 是强制要求 |
| `.trae/rules/SRPG专项规则.md` §八 | 随机数分层管理 |
| `.trae/rules/SRPG专项规则.md` §十 | 确定性强制要求 |
| `docs/04-data/foundation/replay_architecture.md` | Replay 架构详述 |
| `docs/04-data/infrastructure/replay_schema.md` | Replay Schema（种子格式） |

---

## 11. 总结：一张图记住所有 RNG

```
                    ┌──────────────────────────────┐
                    │    业务代码需要随机数时        │
                    │                              │
                    │  ✅ 新代码：                   │
                    │     ResMut<DeterministicRng>  │
                    │     .gen_bool(Combat, 0.6)   │
                    │                              │
                    │  ❌ 旧代码（待迁移）：         │
                    │     rand::random()            │
                    │     GameRng.gen_range(...)    │
                    └──────────────┬───────────────┘
                                   │
                    ┌──────────────▼───────────────┐
                    │    DeterministicRng           │
                    │    (replay/foundation/)       │
                    │                              │
                    │  ┌──────┬──────┬──────┬────┐ │
                    │  │Combat│ Drop │  AI  │World│ │
                    │  └──────┴──────┴──────┴────┘ │
                    │      MurmurHash3 混合器       │
                    └──────────────┬───────────────┘
                                   │
                    ┌──────────────▼───────────────┐
                    │  rng_sync_system             │
                    │  (infra/replay/systems.rs)   │
                    │  回放时同步种子到全局 RNG     │
                    └──────────────────────────────┘
```

**一句话总结：** `DeterministicRng`（四流 MurmurHash3）是目标，`GameRng` / `SeededRng`（单流 ChaCha12）已弃用，`TestRng`（StdRng）仅供单元测试。生产代码通过 `ResMut<DeterministicRng>` 获取随机数并指定正确的 `RngStream`，测试代码使用 `TestRng`。

---

## 附录：代码文件索引

| 想看什么 | 读哪个文件 |
|---------|-----------|
| SeededRng + GameRng 实现 | `src/shared/random/mod.rs` |
| RngStream 枚举定义 | `src/core/capabilities/runtime/replay/foundation/types.rs` |
| RngSeeds 种子集合 | `src/core/capabilities/runtime/replay/foundation/types.rs` |
| DeterministicRng 四流实现 | `src/core/capabilities/runtime/replay/foundation/values.rs` |
| rng_sync_system | `src/infra/replay/systems.rs` |
| 录制时种子初始化 | `src/core/domains/combat/integration/replay/recording.rs` |
| 测试版 TestRng | `src/shared/testing/deterministic.rs` |
| ADR-041 设计决策 | `docs/01-architecture/40-cross-cutting/ADR-041-replay-determinism.md` |
| RNG 确定性测试 | `src/infra/replay/tests/invariant/rng_determinism_test.rs` |
