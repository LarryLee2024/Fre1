---
id: foundation.replay-architecture.v1
title: Replay Architecture Deep Dive — 回放架构详述
status: draft
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: persistence
replay-safe: true
---

# Replay Architecture — 回放架构详述

> **总纲引用**: `docs/04-data/README.md` §7 — Replay 架构
> **本文档是回放架构的深度展开**，覆盖录制格式、确定性保证、命令类型、同步校验和调试工具。

---

## 1. Replay 文件格式

### 1.1 完整 Replay 布局

```
Replay File (.frreplay)
┌────────────────────────────────────────────────────────────┐
│ Magic Header (8 bytes)                                     │
│   magic: [0x46, 0x52, 0x52, 0x45, 0x50, 0x4C, 0x41, 0x59] │
│   → "FRREPLAY\0"                                           │
├────────────────────────────────────────────────────────────┤
│ Header (variable)                                          │
│   replay_format_version: u32                               │
│   game_version: String                                     │
│   timestamp: u64          # 录制时间（仅显示）              │
│   total_frames: u64      # 总帧数                         │
│   initial_seed: u64      # 确定性 PRNG 种子               │
│   checksum: [u8; 32]     # SHA-256 of entire body         │
├────────────────────────────────────────────────────────────┤
│ Body (zstd compressed)                                     │
│   ├── Metadata (JSON)                                      │
│   │   ├── scenario_name: String     # 战斗/场景名称        │
│   │   ├── player_party: Vec<String> # 参战队伍             │
│   │   ├── enemy_party: Vec<String>                         │
│   │   ├── map_id: String                                   │
│   │   ├── difficulty: String                               │
│   │   ├── total_rounds: u32                                │
│   │   └── result: String   # victory / defeat              │
│   │                                                         │
│   ├── SyncCheckpoints (MessagePack)                         │
│   │   ├── interval: u32    # 每 N 帧一个检查点             │
│   │   └── entries: Vec<SyncCheckpoint>                     │
│   │                                                         │
│   └── Frames (MessagePack, 帧序列)                          │
│       └── entries: Vec<ReplayFrame>                        │
└────────────────────────────────────────────────────────────┘
```

### 1.2 ReplayFrame 格式

```rust
/// 单帧数据。记录一个"时间片"内的所有命令和状态。
struct ReplayFrame {
    /// 帧序号（从 0 开始连续递增）
    frame_number: u64,

    /// 游戏内时间（帧计数）
    game_time: GameTime,

    /// 本帧内的所有命令
    commands: Vec<Command>,

    /// 本帧的 RNG 种子（用于确定性随机）
    rng_seed: u64,

    /// 本帧的 RNG 消耗计数（追踪已使用的随机数数量）
    rng_consumed: u64,
}
```

### 1.3 Command 类型

```rust
/// 所有可录制的玩家/AI 命令。
enum Command {
    // ── 移动 ──
    MoveUnit {
        entity_id: EntityId,
        path: Vec<GridPosition>,
    },
    MoveUnitInterrupted {
        entity_id: EntityId,
        path: Vec<GridPosition>,
        interrupted_at: GridPosition,
    },

    // ── 行动 ──
    UseAbility {
        caster: EntityId,
        ability_id: AbilityDefId,
        target: TargetingSnapshot,
        spec_overrides: Vec<SpecOverride>,
    },
    UseItem {
        user: EntityId,
        item_instance_id: ItemInstanceId,
        target: TargetingSnapshot,
    },

    // ── 回合管理 ──
    EndTurn {
        entity_id: EntityId,
    },
    SwapMember {
        outgoing: EntityId,
        incoming: EntityId,
    },

    // ── 反应 ──
    ReactionDecision {
        reactor: EntityId,
        queue_entry_id: ReactionEntryId,
        decision: ReactionDecision,
    },

    // ── 对话/叙事（非战斗回放） ──
    DialogueChoice {
        entity_id: EntityId,
        choice_id: DialogueChoiceId,
    },

    // ── AI 决策 ──
    AIDecision {
        entity_id: EntityId,
        decision_type: AIDecisionType,
        parameters: Vec<u8>,
    },
}

/// 目标快照——在录制时冻结目标数据，确保回放时不受实时状态影响
struct TargetingSnapshot {
    primary_target: Option<EntityId>,
    position: Option<GridPosition>,
    area: Option<Vec<GridPosition>>,
    secondary_targets: Vec<EntityId>,
}
```

---

## 2. 确定性保证

### 2.1 确定性 RNG

```rust
/// 确定性伪随机数生成器（PCG 或类似算法）
struct DeterministicRng {
    /// 当前种子（每帧从 ReplayFrame 获取）
    seed: u64,

    /// 已消耗的随机数计数
    consumed: u64,
}

impl DeterministicRng {
    /// 每帧开始时重置种子
    fn reset_for_frame(&mut self, frame: &ReplayFrame) {
        self.seed = frame.rng_seed;
        self.consumed = 0;
    }

    /// 在回放中，每次随机调用必须严格按照相同顺序
    fn next_u32(&mut self) -> u32 {
        self.consumed += 1;
        pcg_hash(self.seed, self.consumed)
    }
}
```

**关键规则**：录制和回放时，RNG 消费的**顺序和次数必须完全一致**。任何额外的随机调用都会导致后续结果偏移。

### 2.2 确定性要求清单

| 组件 | 录制时 | 回放时 | 确定性保证 |
|------|--------|--------|-----------|
| RNG | 使用 ReplayFrame.rng_seed | 使用相同的 seed + consumed | ✅ 严格 |
| GameTime | 使用帧计数 | 使用帧计数 | ✅ 严格 |
| 随机事件（暴击/命中） | 结果由 RNG 决定 | 相同的 RNG 顺序得到相同结果 | ✅ 严格 |
| AI 决策 | 录制为 AIDecision Command | 直接使用录制的决策 | ✅ 严格 |
| 浮点计算 | f32 受平台影响 | 同一平台回放 | ⚠️ 同一平台保证 |
| 外部输入（文件系统） | 禁止在回放关键路径读取 | 同左 | ✅ 禁止 |
| 系统时钟 | 禁止使用 | 同左 | ✅ 禁止 |

### 2.3 浮点一致性

`f32` 在不同平台（x86 vs ARM）上可能产生微小差异。缓解措施：

1. 核心战斗计算使用定点数或整型模拟（如伤害 = 整数，不涉及小数）
2. 必须使用浮点的场景（如寻路成本），在回放中使用相同平台或容忍微小偏差
3. SyncCheckpoint 的哈希值对浮点使用 `to_bits()` 比较，不比较 `==`

---

## 3. 同步校验（Sync Checkpoint）

### 3.1 检查点插入

每 N 帧（默认 N=60，即 1 秒 @60fps）插入一个 SyncCheckpoint：

```rust
struct SyncCheckpoint {
    /// 帧序号
    frame_number: u64,

    /// 关键状态哈希
    state_hash: [u8; 32],

    /// 哈希包含的 Entity 列表（用于调试）
    hashed_entities: Vec<EntityId>,
}
```

### 3.2 哈希计算

```rust
/// 计算所有"标记为 replay_key 字段"的状态哈希
fn compute_sync_hash(world: &World) -> [u8; 32] {
    let mut hasher = Sha256::new();

    // 遍历所有标记了 #[replay_key] 的 Component
    for entity in world.iter_entities() {
        for component in entity.components() {
            if component.has_replay_key_attr() {
                hasher.update(component.serialize_replay_key());
            }
        }
    }

    hasher.finalize().into()
}
```

### 3.3 回放时校验

```
回放每一帧
    │
    ├── 执行 Commands
    ├── 推进确定性 RNG
    ├── 如果当前帧有 SyncCheckpoint:
    │      ├── 计算当前 state_hash
    │      ├── 与录制的 state_hash 比较
    │      ├── 匹配 → ✅ 继续
    │      └── 不匹配 → ❌ 报告回放断裂
    └── 下一帧
```

### 3.4 回放断裂处理

```
SyncCheckpoint 不匹配
    │
    ├── 1. 记录断裂帧序号 + 期望哈希 + 实际哈希
    ├── 2. 尝试从上一个检查点重新回放（最多 3 次）
    ├── 3. 如果仍失败：
    │      ├── 标记 Replay 为 "desynced"
    │      ├── 切换到 "尽力回放" 模式（跳过后续校验）
    │      └── 在回放结果中标记 desync 帧
    └── 4. 生成 desync 报告（含帧序列、哈希差异、RNG 状态）
```

---

## 4. 录制策略

### 4.1 录制动

```
战斗开始 ──→ 开始录制
                │
                ├── 创建 Replay 文件
                ├── 记录初始种子（initial_seed）
                ├── 记录初始状态（队伍/地图/先攻排序）
                │
                ▼
        每帧录制：
            ├── 收集所有玩家/AI 输入
            ├── 打包为 Command 列表
            ├── 每 N 帧创建 SyncCheckpoint
            └── 写入 Frames
                │
                ▼
战斗结束 ──→ 完成录制
                ├── 写入总帧数、结果元数据
                ├── 计算 body checksum
                └── 关闭文件
```

### 4.2 录制粒度

| 粒度 | 适用场景 | 文件大小（1 小时战斗） |
|------|---------|---------------------|
| **每 Tick** (60fps) | 精确回放、调试 | ~50-100 MB |
| **每 Command** | 普通回放 | ~5-10 MB |
| **每帧聚合** | 生产环境 | ~10-20 MB |

推荐生产环境使用 **每 Command 粒度**：只录制有输入/决策的帧，静默帧跳过。

### 4.3 AI 决策录制

AI 决策在录制时同时记录**决策结果**和**决策依据**：

```rust
struct AIDecision {
    entity_id: EntityId,
    decision_type: AIDecisionType,

    /// 最终选定的行动
    chosen_action: AIAction,

    /// 决策依据摘要（调试用途，回放时可选）
    reasoning: Option<AIDecisionReasoning>,
}

enum AIDecisionType {
    MoveAction,
    AbilitySelection,
    TargetSelection,
    ReactionChoice,
    FormationChoice,
}
```

回放时 AI 系统**不重新计算决策**，直接读取录制的 `AIDecision` Command。这保证了 AI 行为的完全确定性，同时也意味着修改 AI 逻辑后旧 Replay 仍然可回放。

---

## 5. 回放控制

### 5.1 回放模式

| 模式 | 速度 | 校验 | 用途 |
|------|------|------|------|
| 正常 | 1x | SyncCheckpoint | 观看回放 |
| 快速 | 4x / 8x | SyncCheckpoint | 快速浏览 |
| 极速 | MAX (逐帧无渲染) | 仅头尾 | 自动化测试 |
| 步进 | 逐帧 | 每帧 | 调试 |
| 验证 | MAX | 全部 SyncCheckpoint | CI/测试断言 |

### 5.2 回放场景

| 场景 | 录制源 | 回放方式 |
|------|--------|---------|
| **Bug 复现** | 玩家复现步骤 | 逐帧步进调试 |
| **自动化测试** | CI 生成 | 验证模式，断言最终结果 |
| **玩家分享** | 战斗录制 | 正常模式观看 |
| **反作弊验证** | 服务端录制 | 验证模式，检查关键同步点 |

---

## 6. Replay 与各领域 Schema 的关系

| 领域 | 参与回放？ | 录制内容 | replay-safe 标记 |
|------|-----------|---------|-----------------|
| Combat | ✅ 核心 | 回合/攻击/移动 Command | `true` |
| Tactical | ✅ 核心 | 移动路径/位置变化 | `true` |
| Spell | ✅ 核心 | 施法/专注 Command | `true` |
| Reaction | ✅ 核心 | 反应决策 Command | `true` |
| Effect | ✅ 核心 | Effect 生命周期由 Combat 驱动 | `true` |
| Summon | ✅ 是 | 召唤/消失 Command | `true` |
| Terrain | ✅ 是 | 表面变化（由 Effect 驱动） | `true` |
| Progression | ❌ 否 | — | `false` |
| Inventory | ❌ 否 | — | `false` |
| Quest | ❌ 否 | — | `false` |
| Narrative | ❌ 否 | — | `false` |
| Economy | ❌ 否 | — | `false` |
| Crafting | ❌ 否 | — | `false` |
| Party | ❌ 否 | — | `false` |
| CampRest | ❌ 否 | — | `false` |

只有标记为 `replay-safe: true` 的领域参与回放录制/校验。

---

## 7. 调试工具

### 7.1 Replay 校验工具

命令行工具用于验证 Replay 文件：

```bash
# 验证 replay 完整性
fre-replay verify path/to/replay.frreplay

# 步进回放到指定帧，打印状态
fre-replay step path/to/replay.frreplay --frame 142 --print-state

# 比较两个 replay 的同步点差异
fre-replay diff replay_a.frreplay replay_b.frreplay

# 从 replay 导出 metadata
fre-replay info path/to/replay.frreplay --json
```

### 7.2 回放断裂报告

当回放 desync 发生时，生成结构化报告：

```json
{
  "replay_file": "battle_001.frreplay",
  "desync_frame": 142,
  "expected_hash": "a1b2c3d4e5f6...",
  "actual_hash": "ffeeddccbbaa...",
  "rng_state_at_desync": { "seed": 12345, "consumed": 67 },
  "last_commands": [
    { "frame": 140, "type": "MoveUnit", "entity": "unit_003" },
    { "frame": 141, "type": "UseAbility", "entity": "unit_001" }
  ],
  "divergent_entities": ["unit_001", "npc_008"]
}
```

---

## 8. Future Extension

- **多段 Replay**: 将长时间回放分为多个段文件，支持随机跳转
- **回放编辑**: 支持在回放中插入/修改命令（用于"what-if"分析）
- **直播回放**: Replay 数据流通过 WebSocket 实时推送观看者
- **自动 Bug 报告**: 检测到 desync 时自动生成 Replay + 报告上传
- **压缩算法演进**: 帧间预测编码（参考视频编码的 I/P/B 帧思想）
