---
id: infrastructure.replay.schema.v1
title: Replay Schema — 回放系统数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: persistence
replay-safe: true
---

# Replay Schema — 回放系统数据架构

> **领域归属**: Infrastructure — C3 Runtime | **依赖 Schema**: 全部 Schema | **定义依据**: `docs/00-governance/Fre项目架构设计.md`, Data Law 010

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `ReplayLog` | Persistence | 完整的回放日志（命令序列） |
| `ReplayFrame` | Persistence | 单帧的命令集合 + 种子 |
| `ReplayCommand` | Persistence | 原子命令（玩家输入 / AI 决策） |
| `ReplayHeader` | Persistence | 回放日志元数据 |

---

## 2. Problem

Replay 是项目宪法 P0 要求——核心战斗逻辑必须可确定性重放。Schema 必须解决：
- 命令的原子记录格式（哪些操作需要录制）
- 确定性 RNG 的种子管理
- 关键状态校验点的 checksum 验证
- 版本兼容（旧版本回放在新版本中可用）

---

## 3. Schema Design

### 3.1 ReplayLog（Persistence 层）

```rust
/// 完整的回放日志。
struct ReplayLog {
    /// 头部元数据
    header: ReplayHeader,

    /// 命令帧序列
    frames: Vec<ReplayFrame>,

    /// 最终状态校验（可选，用于验证回放完整性）
    final_checksum: Option<u64>,
}
```

### 3.2 ReplayHeader（Persistence 层）

```rust
struct ReplayHeader {
    /// Schema 版本
    schema_version: u32,

    /// 录制的游戏版本
    game_version: String,

    /// 战斗/场景标识
    scene_id: String,

    /// 参与实体列表（初始状态）
    participants: Vec<EntityId>,

    /// 初始 RNG 种子
    initial_seed: u64,

    /// 总帧数
    total_frames: u64,

    /// 录制日期（仅用于显示，不用于回放逻辑）
    recorded_at: String,

    /// 录制耗时（毫秒，仅用于参考）
    duration_ms: Option<u64>,

    /// 扩展元数据
    metadata: HashMap<String, String>,
}
```

### 3.3 ReplayFrame（Persistence 层）

```rust
/// 单帧记录。
struct ReplayFrame {
    /// 帧序号（从 0 开始）
    frame_number: u64,

    /// 本帧的游戏内时间
    game_time: GameTime,

    /// 本帧的所有命令
    commands: Vec<ReplayCommand>,

    /// 本帧的 RNG 种子偏移
    /// 全局 RNG = initial_seed + frame_number
    rng_seed_offset: u64,

    /// 校验和（可选，关键状态哈希）
    checksum: Option<u64>,

    /// 校验范围（哪些 Entity 包含在 checksum 中）
    checksum_scope: Option<Vec<EntityId>>,
}
```

### 3.4 ReplayCommand（Persistence 层）

```rust
/// 原子命令——回放的最小可录制单元。
enum ReplayCommand {
    /// 单位移动
    UnitMove {
        unit: EntityId,
        path: Vec<GridPosition>,
    },

    /// 技能使用
    UseAbility {
        caster: EntityId,
        ability_spec_id: SpecId,
        target: AbilityTarget,
    },

    /// 物品使用
    UseItem {
        user: EntityId,
        item_instance_id: ItemInstanceId,
        target: Option<EntityId>,
    },

    /// 等待/跳过回合
    SkipTurn {
        unit: EntityId,
    },

    /// 对话选择
    DialogueChoice {
        speaker: EntityId,
        choice_id: String,
    },

    /// 反应触发确认（玩家确认是否触发反应）
    ReactionConfirm {
        reactor: EntityId,
        trigger_def_id: TriggerDefId,
        accepted: bool,
    },

    /// 目标选择确认（手动选择目标时）
    ConfirmTargets {
        caster: EntityId,
        ability_spec_id: SpecId,
        selected_targets: Vec<EntityId>,
    },

    /// 自定义命令（由 Domain 扩展）
    Custom {
        domain: String,
        command_type: String,
        params: HashMap<String, String>,
    },
}

enum AbilityTarget {
    /// 单体目标
    Single(EntityId),
    /// 区域目标（位置）
    Area(GridPosition),
    /// 无目标（如自我施法）
    None,
}
```

### 3.5 GameTime（Persistence 层）

```rust
/// 游戏内时间（确定性的时间表示，不依赖 wall-clock）。
struct GameTime {
    /// 已进行的回合数
    turn: u32,
    /// 本回合内的阶段
    phase: TurnPhase,
    /// 自阶段开始以来的帧数
    frame_in_phase: u64,
}

enum TurnPhase {
    Initiation,
    PlayerAction,
    EnemyAction,
    Resolution,
}
```

### 3.6 ReplayValidator（Runtime 层）

```rust
/// 回放验证器——用于录制时计算校验和，回放时比对。
struct ReplayValidator {
    /// 录制模式
    recording: bool,
    /// 当前帧号
    current_frame: u64,

    /// 累计校验和（XOR 所有帧的 checksum）
    accumulated_checksum: u64,

    /// 不一致记录（回放时发现偏差）
    mismatches: Vec<ReplayMismatch>,
}

struct ReplayMismatch {
    frame: u64,
    expected_checksum: u64,
    actual_checksum: u64,
    mismatched_entities: Vec<EntityId>,
}
```

### 3.7 ReplayLog Binary Format

```rust
/// 回放日志的二进制存储格式。
///
/// [Header (256 bytes)] [Frame 0] [Frame 1] ... [Frame N] [Footer (32 bytes)]
///
/// Header 固定大小，包含版本、种子、总帧数。
/// Frame 大小可变，以 frame_number 起始，checksum 结尾。
/// Footer 包含最終累計 checksum。
struct ReplayBinaryLayout {
    /// 魔数 "FREP" (Fre Replay)
    magic: [u8; 4],

    /// Header（序列化 ReplayHeader）
    header: Vec<u8>,

    /// Frame 数据（序列化 ReplayFrame 列表）
    frame_data: Vec<u8>,

    /// 压缩方式（0 = 无压缩，1 = zstd）
    compression: u8,

    /// 最終累計 checksum (SHA-256 truncated to 8 bytes)
    final_checksum: [u8; 8],
}
```

---

## 4. Layer Analysis

| 数据结构 | Layer | 持久化 | 说明 |
|----------|-------|--------|------|
| `ReplayHeader` | Persistence | 是（文件） | 回放文件元数据 |
| `ReplayFrame` | Persistence | 是（文件） | 命令帧 |
| `ReplayCommand` | Persistence | 是（文件） | 原子命令 |
| `ReplayValidator` | Runtime | 否 | 回放时的验证器 |

---

## 5. Dependency Analysis

| 依赖方向 | 依赖 Schema | 说明 |
|----------|------------|------|
| 依赖 | → CombatSchema | 录制战斗命令 |
| 依赖 | → AbilitySchema | 录制技能使用 |
| 依赖 | → MovementSchema | 录制移动路径 |
| 依赖 | → DialogueSchema | 录制对话选择 |

---

## 6. Validation Rules

| # | 规则 | 触发时机 | 校验逻辑 |
|---|------|----------|----------|
| V1 | 帧号连续 | 回放加载 | 帧号从 0 开始连续递增 |
| V2 | RNG 种子确定 | 回放执行 | RNG = initial_seed + frame_number |
| V3 | Command 实体存在 | 回放执行 | command 中的 EntityId 在场景中存在 |
| V4 | 版本兼容 | 回放加载 | schema_version ≤ 当前版本 |
| V5 | Checksum 一致 | 回放验证 | 每帧 checksum 与录制时一致 |

---

## 7. Replay Compatibility

Replay 系统的核心就是保证自身兼容性。关键原则：
1. **命令即事实**：ReplayLog 是唯一的 Truth Source
2. **确定性 RNG**：所有随机性通过 RNG 种子管理
3. **无 wall-clock 依赖**：所有时间基于 GameTime（回合 + 帧）
4. **版本迁移**：旧版本 ReplayLog 通过 Migration 升级到当前版本

---

## 8. Save Compatibility

ReplayLog 是独立的文件格式（.freplay），与游戏存档分离。

---

## 9. Migration Strategy

| 版本 | 变更 | 迁移策略 |
|------|------|----------|
| v1 | 初始版本 | — |
| v2（未来） | 新增命令类型 | 旧版本命令通过 Registry 映射，不支持的命令标记为 UNKNOWN 并跳过 |
| v3（未来） | 压缩格式变化 | 旧格式先解压再按新格式处理 |

---

## 10. Future Extension

- **增量回放**: 支持从指定帧号开始增量回放（用于调试）
- **回放加速/减速**: 回放速率控制
- **回放分支**: 在关键决策点创建回放分支（探索"如果那时做了不同选择"）
- **回放分析**: 自动分析回放数据提取统计信息（技能使用率、伤害统计）

---

## 11. Risks

| 风险 | 影响 | 缓解 |
|------|------|------|
| 二进制格式不可读 | 调试困难 | 提供 ReplayLog → JSON 转换工具 |
| 版本兼容断裂 | 旧回放在新版本无法使用 | 严格的 Migration 链和版本兼容测试 |
| 命令录制遗漏 | 某些操作未被录制导致回放偏差 | 所有 Command 必须通过 CommandBus 录制，遗漏检测 |
| AI 决策确定性 | AI 依赖 RNG 种子但实现有 bug | AI 模块测试中强制验证同一种子产生同一决策 |

---

## 12. Constitution Check

| 宪法条款 | 合规 | 说明 |
|----------|------|------|
| Replay First (P0) | ✅ | ReplayLog 是系统核心设计 |
| 确定性 RNG | ✅ | 种子 = initial_seed + frame_number |
| 无 wall-clock 依赖 | ✅ | 所有时间使用 GameTime |
| 版本迁移 | ✅ | schema_version + Migration 策略 |
