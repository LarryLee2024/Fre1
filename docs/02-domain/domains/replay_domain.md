---
id: 02-domain.replay
title: Replay（回放）领域规则 v1.0
status: draft
owner: domain-designer
created: 2026-06-21
updated: 2026-06-21
tags:
  - domain
  - replay
  - cross-cutting
---


## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| ReplayFrame | 单帧录制的增量数据，包含该帧内的所有命令和 RNG 种子快照 | 负责：单帧数据容器；不负责：帧的校验与验证 |
| ReplayLog | 完整的回放日志，包含元数据头（ReplayHeader）和帧序列（ReplayFrame[]） | 负责：回放数据的完整持久化表示；不负责：录制/回放的运行时状态 |
| ReplayHeader | 回放元数据头，记录版本号、游戏版本、会话标识、初始种子、参与者列表 | 负责：回放的身份与版本信息；不负责：帧级别的数据 |
| ReplayMode | 回放系统的三大运行状态：空闲（Idle）、录制中（Recording）、回放中（Playing） | 负责：状态标识与转换；不负责：状态转换的触发逻辑 |
| DeterministicRng | 确定性随机数生成器，4 个独立 RNG 流（Combat/Drop/AI/World），每流使用 ChaCha12 CSPRNG | 负责：所有业务随机操作的统一入口；不负责：随机数的业务含义 |
| RngSeeds | 4 个 RNG 流的种子集合，录制时保存、回放时恢复 | 负责：种子的分组管理；不负责：种子值的生成策略 |
| BattleUnitId | 战场单位稳定标识字符串，格式 "bu:{team_index}:{unit_index}"，用于录制/回放时跨实体生命周期标识单位 | 负责：单位在回放上下文中的稳定标识；不负责：实体的运行时身份 |
| BattleUnitRegistry | 战场单位注册表，维护 Entity 与 BattleUnitId 的双向映射 | 负责：实体标识的转换服务；不负责：映射的生命周期管理 |
| SyncCheckpoint | 同步检查点，记录指定帧号处的世界状态哈希值，用于回放完整性验证 | 负责：回放正确性的周期性验证；不负责：单帧级别的校验 |
| RecordingSession | 录制会话，管理一次录制从开始到结束的全过程状态 | 负责：录制中的帧缓冲和元数据管理；不负责：帧数据的持久化存储 |
| PlaybackSession | 回放会话，管理一次回放从加载到完成的全过程状态 | 负责：回放中的帧推进和命令分发；不负责：回放完成后的数据清理 |

### 已对齐项目术语

- **Combat**：Replay 录制/回放战斗全流程，Combat 领域通过桥接层与 Replay 协作
- **Event History**：Event History（ADR-059）是 Replay 的互补系统。Replay 录制输入命令，Event History 记录输出事件。二者帧号对齐
- **DeterministicRng**：定义在 `shared/random/`，被所有需要随机数的领域消费
- **Cue**：Cue（表现信号）不参与回放确定性，回放不录制 Cue 信号

---

## 2. 状态机

### 回放系统宏观状态机

```
           ┌──────────────────────────────────────────────────┐
           │                 ReplayMode                        │
           │                                                    │
           │  Idle（空闲）                                     │
           │    │                                                │
           │    ├── [战斗开始 | 手动开始录制]                    │
           │    │   └──→ Recording（录制中）                    │
           │    │          │  [逐帧录制命令和种子]               │
           │    │          └──→ [战斗结束 | 手动停止] → Idle    │
           │    │                                                │
           │    └── [加载回放文件]                               │
           │         └──→ Playing（回放中）                     │
           │                │  [逐帧回放命令]                     │
           │                ├──→ [回放完成] → Idle               │
           │                ├──→ [手动停止] → Idle               │
           │                └──→ [校验失败] → Idle（标记异常）   │
           └──────────────────────────────────────────────────┘
```

### 状态转换表

| 当前状态 | 目标状态 | 触发条件 | 触发动作 |
|----------|----------|----------|----------|
| Idle | Recording | 战斗开始事件 / 显式录制指令 | 创建 RecordingSession，初始化帧缓冲，记录初始种子 |
| Recording | Idle | 战斗结束事件 / 显式停止指令 | 完成当前帧，生成 ReplayLog，清理会话 |
| Idle | Playing | 回放文件加载完成 | 创建 PlaybackSession，设置 ReplayModeGuard，恢复种子 |
| Playing | Idle | 回放完成（所有帧执行完毕） | 触发 PlaybackCompleted 事件，清理会话，恢复 Idle 状态 |
| Playing | Idle | 手动停止回放 | 触发 PlaybackStopped 事件，清理会话 |
| Playing | Idle | SyncCheckpoint 校验失败 | 触发 PlaybackFailed（校验失败）事件，清理会话 |

### 禁止的状态转换

- 🟥 禁止：Recording → Playing — 理由：录制中不可同时回放，两模式互斥
- 🟥 禁止：Playing → Recording — 理由：回放中不可同时录制，两模式互斥
- 🟥 禁止：Recording → Idle 不完整 — 理由：录制结束必须生成完整的 ReplayLog（含最终帧），不可丢弃
- 🟥 禁止：Playing → Idle 不清理 — 理由：回放结束必须恢复 DeterministicRng 到安全状态，避免影响后续操作

---

## 3. 不变量（Invariants）

### 3.1 种子决定论

- **条件**：任何 RNG 使用场景
- **不变量**：相同的初始种子 + 相同的命令序列，必须产生完全相同的 RNG 输出序列
- **违反后果类型**：🔴 程序错误
- **违反后果**：回放结果与原始录制不一致，回放验证失败，属于系统 Bug

### 3.2 帧完整录制

- **条件**：Recording 模式下，每一帧结束时
- **不变量**：每帧必须完整记录该帧内发生的所有命令和帧末种子状态，不得遗漏
- **违反后果类型**：🔴 程序错误
- **违反后果**：遗漏命令导致回放时决策缺失，回放路径偏离，属于系统 Bug

### 3.3 无可信外部随机源

- **条件**：任何业务系统中的随机操作
- **不变量**：业务代码禁止使用 `rand::random()`、`thread_rng()` 或其他非确定性随机源。所有随机操作必须通过 `DeterministicRng` 的 4 个流之一进行
- **违反后果类型**：🔴 程序错误
- **违反后果**：非确定性随机数导致回放不可重现，属于系统 Bug

### 3.4 ReplayLog 不可变

- **条件**：ReplayLog 生成后
- **不变量**：录制完成后的 ReplayLog（A 录 制 的 数 据）不可被任何系统修改。回放模式下只能读取，不可写入
- **违反后果类型**：🔴 程序错误
- **违反后果**：修改后的回放数据无法代表真实游戏过程，丧失回放的可信度

### 3.5 版本兼容性

- **条件**：ReplayLog 加载时
- **不变量**：ReplayLog 的游戏版本必须与当前运行版本兼容（主版本号一致，次版本号 >= 录制版本）
- **违反后果类型**：🔴 规则失败
- **违反后果**：版本不匹配时拒绝加载并提示用户，属于预期业务分支

### 3.6 回放模式下禁止外部状态读取

- **条件**：Playing 模式下
- **不变量**：回放进行中禁止读取系统时间、文件系统、网络状态等非确定性外部状态
- **违反后果类型**：🔴 程序错误
- **违反后果**：外部状态变化导致回放结果偏离原始录制，属于系统 Bug

### 3.7 表现层不影响确定性

- **条件**：任何游戏运行模式
- **不变量**：表现层（Cue、VFX、SFX、UI 动画）产生的帧不能影响逻辑层的确定性。表现层是逻辑层结果的消费方，不可反向影响逻辑
- **违反后果类型**：🔴 程序错误
- **违反后果**：表现层影响逻辑确定性导致回放结果不一致，属于架构级 Bug

### 3.8 RNG 流隔离

- **条件**：多域同时使用 RNG 时
- **不变量**：4 个 RNG 流（Combat/Drop/AI/World）之间严格隔离。战斗随机数消耗不影响掉落随机数，反之亦然
- **违反后果类型**：🔴 程序错误
- **违反后果**：RNG 流互相污染导致一个域的随机行为影响另一个域，回放时跨域依赖无法复现

### 3.9 帧号单调递增

- **条件**：录制或回放全过程中
- **不变量**：ReplayFrame 的帧号必须严格单调递增，无跳跃、无回退
- **违反后果类型**：🔴 程序错误
- **违反后果**：帧号乱序导致 SyncCheckpoint 无法对齐，回放校验失败

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：业务代码直接调用 `rand::random()` 或 `thread_rng()` — 理由：破坏回放确定性，所有随机操作必须通过 DeterministicRng
- 🟥 禁止：录制后修改 ReplayFrame 或 ReplayLog 的数据 — 理由：破坏回放数据的完整性和可信度
- 🟥 禁止：回放模式下读取系统时间/文件系统/网络状态 — 理由：外部状态不可决定论复现
- 🟥 禁止：表现层（Cue/VFX/SFX）影响逻辑层执行路径 — 理由：表现层必须为逻辑层的纯消费方
- 🟥 禁止：Recording 模式下切换到 Playing 模式（或反向）— 理由：录制与回放互斥
- 🟥 禁止：版本不匹配时强行加载 ReplayLog — 理由：逻辑可能不一致，应拒绝加载并提示
- 🟥 禁止：ReplayLog 包含 Event History 数据 — 理由：Event History 是输出事件记录，属于互补系统而非回放输入数据
- 🟥 禁止：跨越不同 RNG 流调用随机数（如 AI 系统使用 Combat 流）— 理由：每个域必须使用自己分配的 RNG 流
- 🟥 禁止：回放文件中包含文件路径/用户标识等环境敏感信息 — 理由：ReplayLog 应可跨环境共享用于调试

---

## 5. 流程定义

### 5.1 录制开始

- **输入**：录制触发信号（战斗开始事件 / 手动开始指令）、当前 DeterministicRng 种子、参战单位列表
- **处理**：
  1. 创建 RecordingSession，记录当前帧号计数器起点
  2. 记录 ReplayHeader：版本号、游戏版本、会话标识、初始种子快照
  3. 注册所有参战单位的 BattleUnitId（Entity → String 映射）
  4. 在单位实体上挂载 BattleUnitId Component
  5. 切换 ReplayMode = Recording
  6. 发布 RecordingStarted 事件
- **输出**：RecordingSession（活跃录制状态）、RecordingStarted 事件
- **失败处理**：已在 Recording 模式下收到录制信号 → 忽略（幂等）→ 这是**规则失败**（预期业务分支，重复触发不终止已有录制）

### 5.2 单帧录制

- **输入**：帧结束信号，本帧内产生的所有命令，当前 DeterministicRng 各流种子
- **处理**：
  1. 收集本帧内所有 RecordedCommand（玩家/AI 输入、系统决策）
  2. 创建 ReplayFrame，包含：
     a. frame_number（单调递增）
     b. commands[ ]（本帧的命令列表）
     c. rng_seeds（帧末 4 流种子快照）
  3. 将 ReplayFrame 追加到 RecordingSession 的帧缓冲
  4. 如果达到检查点间隔（SyncCheckpoint），记录世界状态哈希
  5. 推进帧号计数器
- **输出**：ReplayFrame 追加到缓冲，可选 SyncCheckpoint
- **失败处理**：本帧无命令 → 录制空帧（仅含种子快照）→ 这是**规则失败**（预期业务分支，无操作的帧仍需记录种子状态以保证帧号对齐）

### 5.3 录制停止

- **输入**：录制停止信号（战斗结束事件 / 手动停止指令），活跃的 RecordingSession
- **处理**：
  1. 完成当前帧录制（如有未完成帧）
  2. 关闭 RecordingSession：
     a. 计算 ReplayLog 的统计元数据（总帧数、总命令数、持续时间）
     b. 生成完整的 ReplayLog（header + frames）
  3. 清理 BattleUnitRegistry（移除所有 BattleUnitId Component）
  4. 切换 ReplayMode = Idle
  5. 发布 RecordingStopped 事件
- **输出**：ReplayLog（完整回放数据）、RecordingStopped 事件
- **失败处理**：无活跃的 RecordingSession 时收到停止信号 → 忽略 → 这是**程序错误**（系统异常，不应在非录制模式下收到停止信号）

### 5.4 回放开始

- **输入**：ReplayLog 加载请求，ReplayLog 数据
- **处理**：
  1. 校验版本兼容性（不变量 3.5）
  2. 校验 ReplayLog 完整性（header → frames 结构校验）
  3. 创建 PlaybackSession，加载 ReplayLog
  4. 设置 ReplayModeGuard.is_replay = true（通知所有系统进入回放模式）
  5. 从 ReplayHeader 恢复 DeterministicRng 的初始种子（set_all_seeds）
  6. 重建 BattleUnitRegistry（从帧数据或 header 中的参与者列表）
  7. 切换到 Playing 模式
  8. 发布 PlaybackStarted 事件
- **输出**：PlaybackSession（活跃回放状态）、PlaybackStarted 事件
- **失败处理**：版本不兼容 → 拒绝加载，提示版本不匹配 → 这是**规则失败**（预期业务分支）；数据损坏 → 拒绝加载，标记为损坏数据 → 这是**规则失败**（预期业务分支）

### 5.5 单帧回放

- **输入**：当前帧号，PlaybackSession 中的下一帧 ReplayFrame
- **处理**：
  1. 读取 ReplayFrame 中的命令列表
  2. 将每条 RecordedCommand 分发到对应的目标系统（通过 BattleUnitId → Entity 映射转换）
  3. 目标系统执行命令（决定论路径——相同输入产生相同输出）
  4. 帧结束时，验证 DeterministicRng 的种子是否与 ReplayFrame 中记录的一致
  5. 如有 SyncCheckpoint，验证世界状态哈希
  6. 推进帧号计数器
  7. 发布 FramePlayed 事件（调试用）
- **输出**：命令执行完成，种子一致性确认，可选 SyncCheckpoint 验证结果
- **失败处理**：种子不匹配 → 暂停回放，触发 PlaybackFailed（确定性偏离）→ 这是**程序错误**（系统异常，回放结果偏离原始录制）；SyncCheckpoint 哈希不匹配 → 暂停回放，触发 PlaybackFailed（状态不一致）→ 这是**程序错误**（系统异常，世界状态偏离预期）

### 5.6 回放完成

- **输入**：所有 ReplayFrame 执行完毕 / 手动停止信号
- **处理**：
  1. 如果是正常完成，执行最终 SyncCheckpoint 验证
  2. 清理 PlaybackSession
  3. 清理 BattleUnitRegistry
  4. 设置 ReplayModeGuard.is_replay = false
  5. 切换 ReplayMode = Idle
  6. 重置 DeterministicRng 到安全初始状态（可选）
  7. 发布 PlaybackCompleted / PlaybackStopped 事件
- **输出**：回放结束确认，PlaybackCompleted/PlaybackStopped 事件
- **失败处理**：回放中途异常（如 SyncCheckpoint 校验失败）→ 强制停止，记录失败原因 → 这是**程序错误**（系统异常，回放数据与运行时代码不一致）

---

## 6. 领域事件

| 事件名 | 触发时机 | 携带数据 | 订阅者 |
|--------|----------|----------|--------|
| RecordingStarted | 录制开始时 | session_id, initial_seed, participant_count, timestamp | Combat（桥接层初始化录制状态）、UI（显示录制指示器）、日志（LogCode: REP001） |
| RecordingStopped | 录制停止时 | session_id, frame_count, command_count, duration | Combat（桥接层清理录制状态）、UI（隐藏录制指示器）、日志（LogCode: REP002） |
| PlaybackStarted | 回放开始时 | log_id, version, frame_count, initial_seed | Combat（桥接层初始化回放状态）、UI（切换到回放模式显示）、日志（LogCode: REP003） |
| PlaybackCompleted | 回放正常完成时 | log_id, frames_played, checks | UI（回放结束提示）、日志（LogCode: REP004） |
| PlaybackStopped | 回放被手动停止时 | log_id, frames_played, stopped_by | UI（回放被中断提示）、日志（LogCode: REP005） |
| PlaybackFailed | 回放校验失败时 | log_id, failed_frame, failure_reason（种子不匹配/哈希校验失败/数据损坏）, expected_value, actual_value | UI（显示错误信息）、日志（LogCode: REP006） |
| SyncCheckpointPassed | SyncCheckpoint 验证通过时 | frame_number, world_hash | 调试工具、日志（LogCode: REP007） |
| SyncCheckpointFailed | SyncCheckpoint 验证失败时 | frame_number, expected_hash, actual_hash, discrepancy_details | 调试工具、日志（LogCode: REP008） |

### 事件订阅关系图

```
RecordingStarted
    │
    ├──→ Combat（桥接层）：初始化 BattleUnitRegistry，创建 RecordingSession
    ├──→ UI：显示录制状态指示器
    └──→ Log：记录录制开始日志

RecordingStopped
    │
    ├──→ Combat（桥接层）：清理 BattleUnitRegistry，完成 ReplayLog
    ├──→ UI：隐藏录制状态指示器
    └──→ Log：记录录制元数据（帧数、命令数、时长）

PlaybackStarted
    │
    ├──→ Combat（桥接层）：初始化 PlaybackSession，恢复种子，重建 BattleUnitRegistry
    ├──→ UI：切换到回放模式，隐藏玩家输入
    └──→ Log：记录回放开始日志

PlaybackCompleted / PlaybackStopped
    │
    ├──→ Combat（桥接层）：清理回放状态
    ├──→ UI：退出回放模式，恢复正常输入
    └──→ Log：记录回放结束日志

PlaybackFailed / SyncCheckpointFailed
    │
    ├──→ Combat（桥接层）：暂停回放，记录失败现场
    ├──→ UI：显示错误面板（帧号、期望值 vs 实际值）
    └──→ Log：记录详细失败信息（用于 Bug 复现）
```

### Event History 互补关系

| 维度 | Replay | Event History（ADR-059） |
|------|--------|--------------------------|
| 录制数据 | 输入命令（RecordedCommand + RNG 种子） | 输出事件（StoredEvent 结构化快照） |
| 目的 | 确定性验证与 Bug 复现 | 事后分析与可观测性 |
| 完整性 | 必须完整，缺一不可 | 环形缓冲区，溢出丢弃 |
| 持久化 | .replay 文件序列化 | 运行时内存存储 |
| 因果关系 | 原因 | 结果 |

- Replay 不依赖 Event History：Replay 的确定性验证功能完整独立
- Event History 在 Replay 模式下正常工作：回放过程中产生的事件同样写入 EventStore
- EventStore 不写入 Replay 文件：事件历史是输出数据，不属于确定性输入的一部分
- 两者共享帧号：帧号是对齐 Replay 帧和 EventStore 事件的桥梁

---

## 7. 桥接契约：Replay ↔ 各域协作

### 7.1 桥接原则

Replay 域是横切关注点，通过桥接层与各业务域协作。桥接层属于各业务域的 `integration/` 模块，遵循以下原则：

- **桥接层属于下游域**：如 Combat 域创建 `combat/integration/replay/` 桥接模块，适配 Replay 接口
- **最小侵入**：桥接层通过 Hook 和 Observer 接入现有事件流，不修改核心类型
- **录制与回放分离**：录制侧在生命周期事件上挂载；回放侧在 ReplayModeGuard 下注入命令

### 7.2 Replay ↔ Combat 协作契约

| 协作点 | 机制 | 方向 |
|--------|------|------|
| 战斗开始 → 开始录制 | Observer: OnBattleStart | Combat → Replay |
| 单位动作 → 录制命令 | Observer: UnitActionComplete | Combat → Replay |
| 战斗结束 → 停止录制 | Observer: OnBattleEnd | Combat → Replay |
| 回放模式 → 注入命令 | ReplayModeGuard 查询，dispatch_replay_command | Replay → Combat |
| 回放模式 → 阻止输入 | ReplayModeGuard 查询，skip_input | Replay → Input |
| 单位标识映射 | BattleUnitRegistry（Resource）+ BattleUnitId（Component） | 共享数据 |
| RNG 种子管理 | DeterministicRng（Resource） | 共享数据 |

### 7.3 Replay ↔ 其他域协作契约

| 域 | 录制内容 | 回放方式 | 确定性要求 |
|----|----------|----------|------------|
| Combat | CombatIntent、回合结束、战斗开始/结束 | 同种子 + 同命令序列重放 | 确定性 RNG（Combat 流） |
| Tactical | MoveCommand、朝向变化 | 注入路径命令 | 寻路算法使用固定种子或确定性实现 |
| Ability | AbilityCastCommand | 注入技能释放命令 | Spec 快照包含种子 |
| Reaction | 反应触发条件 | 由 Combat 流程自动重放 | 无额外要求 |
| Modifier | 不录制（无随机逻辑） | 不重放（由 Combat 流程自动应用） | 无 |
| Cue | 不录制 | 不重放 | 无要求（纯表现，非确定性） |
| AI | AI 决策日志（调试用） | AI 系统使用 AI 流 RNG | AI 随机决策使用 AI 流 |

### 7.4 命令映射表

| 录制命令（ReplayCommand 变体） | 含义 | 消费域 | 回放侧处理 |
|-------------------------------|------|--------|------------|
| UseAbility { caster, ability_def_id, target } | 单位使用技能 | Combat | 触发 UnitActionComplete |
| SkipTurn { unit } | 单位跳过回合 | Combat | 触发 UnitActionComplete(skip) |
| UnitMove { unit, path } | 单位移动 | Tactical | 注入移动命令 |
| ConfirmTargets { caster, ability_def_id, selected_targets } | 确认目标 | Targeting | 注入目标选择 |

### 7.5 RNG 流分配表

| RNG 流 | 分配给 | 典型用途 |
|--------|--------|----------|
| Combat | 战斗系统 | 命中判定、暴击判定、伤害浮动 |
| Drop | 掉落/制造系统 | 战利品生成、制造随机产出 |
| AI | AI 决策系统 | AI 行动选择、行为随机性 |
| World | 世界事件系统 | 探索事件、遭遇生成 |

---

## 8. 与已有架构的对齐校验

- ✅ 架构边界：Replay 域位于横切关注点层，桥接层位于各业务域的 `integration/replay/` 目录
- ✅ 与 ADR-041 一致：4 流确定性 RNG、ReplayFrame 格式、SyncCheckpoint 验证机制
- ✅ 与 ADR-048 一致：桥接层属于下游域、BattleUnitRegistry 双向标识映射、录制/回放分离
- ✅ 与 Event History（ADR-059）互补：Replay 录制输入，Event History 记录输出，共享帧号对齐
- ✅ 最小侵入：桥接层通过 Observer/Hook 接入，不修改现有核心类型
- ✅ RNG 流隔离：4 个流为 Combat/Drop/AI/World，各域使用自己的 RNG 流
- ✅ 表现层隔离：Cue/VFX 不参与回放确定性，逻辑/表现层分离
- ✅ LocalizationKey：本领域涉及的用户可见文本使用 LocalizationKey 而非硬编码文本（宪法 §22）

---

## 9. 自检清单

- [x] 所有术语有唯一定义，与项目已有术语一致
- [x] 业务规则无"可能"、"也许"等模糊表述
- [x] 已检查 `docs/02-domain/` 下相关文档，无冲突
- [x] 未涉及代码实现细节（函数名、trait 名等）
- [x] 领域模型能完整覆盖录制开始、帧录制、录制停止、回放开始、帧回放、回放完成等全场景
- [x] 所有不变量和约束条件已识别（9 条不变量）
- [x] 禁止事项已明确列出（9 条禁止）
- [x] 回放系统状态机定义清晰（Idle → Recording → Idle，Idle → Playing → Idle）
- [x] 每个操作有完整的流程定义（录制开始、单帧录制、录制停止、回放开始、单帧回放、回放完成）
- [x] 桥接契约清晰（Replay ↔ Combat、Replay ↔ 其他域、命令映射、RNG 流分配）
