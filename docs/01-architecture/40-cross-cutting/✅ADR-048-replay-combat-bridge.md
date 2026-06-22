# ADR-048: Replay→Combat 桥接层设计

## 状态
Accepted

## 背景

Replay 系统（`infra::replay` + `core::capabilities::runtime::replay`）已实现完整的录制/回放引擎（90 个测试覆盖），Combat 域（`core::domains::combat`）也已实现完整的回合制战斗管线（80+ 个测试覆盖），但两者之间存在**零桥接代码**：

1. **CombatPipelineDriver** 设计为在 `unit_action` 阶段暂停等待外部输入，通过 `on_unit_action_complete` observer 恢复，但该事件从未被触发
2. **ReplayCommand** 使用 String ID 标识单位，而 Combat 使用 `Entity` 句柄，无转换层
3. **Combat 无 RNG 使用**，虽然 `DeterministicRng` 和 `GameRng` 基础设施已可用
4. 两个系统的 Plugin 都已注册于 App 中，但互不知晓对方

本 ADR 设计桥接层，使 Combat 域可与 Replay 系统协作，实现战斗的录制与确定性回放。

## 引用的领域规则与架构决策

- `docs/01-architecture/README.md` §6.2 — integration/ 模式（Facade + SystemParam）
- `docs/01-architecture/40-cross-cutting/ADR-047-content-loading-pipeline.md` — Content 管线
- `docs/01-architecture/20-tactical-combat/ADR-021-turn-state-machine.md` — 战斗域架构
- `docs/02-domain/domains/combat_domain.md` — 战斗领域规则
- `docs/02-domain/domains/replay_domain.md` — 回放领域规则（注：当前无此文档，本 ADR 涵盖 replay 域与 combat 域的接口契约）

## 决策

### 架构原则

1. **桥接层属于 Combat 域**：在 `core::domains::combat::integration::replay/` 下创建桥接模块。Combat 适配 Replay 接口，而非相反。这保持了 Infra(L2)→Core(L1) 的正确依赖方向。
2. **最小侵入**：不修改 CombatPipelineDriver、TurnQueue、CombatParticipant 等现有类型的内部逻辑。桥接层通过 Hook 和 Observer 接入现有事件流。
3. **录制与回放分离**：录制侧（`recording.rs`）在战斗生命周期事件上挂载；回放侧（`playback.rs`）在 `ReplayModeGuard::is_replay` 为 true 时注入命令。

### Entity↔String 映射策略

采用 **BattleUnitRegistry** 资源，在战斗开始时建立所有参战实体的稳定标识映射：

| 概念 | 类型 | 说明 |
|------|------|------|
| 战场内唯一标识 | `BattleUnitId(String)` | 格式 `"bu:{team_index}:{unit_index}"`，如 `"bu:0:0"` |
| 注册表（Resource） | `BattleUnitRegistry` | `HashMap<Entity, BattleUnitId>` + `HashMap<BattleUnitId, Entity>` |
| 生命周期 | 战斗开始时创建 → 战斗结束时清除 | 不跨会话持久化——存档/读档用 `PersistentEntityId` |

**热路径优化**：为提高 `Entity→BattleUnitId` 查询性能，新增 `BattleUnitId` Component 直接挂在实体上（`Component` 查询优于 `HashMap` 查询）。

### ReplayCommand→Combat 动作映射

Combat 域当前动作类型与 `ReplayCommand` 的映射：

| Combat 动作 | ReplayCommand 变体 | 触发时机 |
|-------------|-------------------|----------|
| 单位使用技能 | `UseAbility { caster, ability_def_id, target }` | `unit_action` pause 恢复前 |
| 单位跳过回合 | `SkipTurn { unit }` | `unit_action` pause 恢复前 |
| 单位移动 | `UnitMove { unit, path }` | 移动系统执行后 |
| 确认目标 | `ConfirmTargets { caster, ability_def_id, selected_targets }` | 目标选择完成后 |

### 确定性与 RNG

- 录制时：将当前 `DeterministicRng` 种子记录到 `ReplayHeader.initial_seed`
- 回放时：ReplayPlugin 的 `rng_sync_system` 自动从 `PlaybackSession` 同步种子到 `DeterministicRng`
- Combat 系统如需随机数，应通过 `ResMut<DeterministicRng>` 读取（使用 `Combat` 流），而非 `GameRng` 或 `thread_rng()`

## Module Design

在 `src/core/domains/combat/integration/replay/` 下创建：

```
combat/integration/replay/
├── mod.rs              # Module root + Plugin registration
├── registry.rs         # BattleUnitRegistry Resource + BattleUnitId Component
├── recording.rs        # Recording lifecycle systems
│   ├── start_recording_on_battle_begin()    # OnBattleStart → init RecordingSession
│   ├── record_unit_action()                 # Intercept UnitActionComplete → record ReplayCommand
│   └── stop_recording_on_battle_end()       # OnBattleEnd → stop RecordingSession
├── playback.rs         # Playback command dispatch
│   ├── dispatch_replay_command()            # Update system: read PlaybackSession → fire UnitActionComplete
│   └── skip_player_input_during_replay()    # PreUpdate: prevent real input during playback
└── tests/
    ├── mod.rs
    ├── registry_test.rs
    ├── recording_test.rs
    └── playback_test.rs
```

各文件职责：

### `mod.rs`
- 声明子模块
- 定义 `CombatReplayBridgePlugin`，注册所有系统和 Resource

### `registry.rs`
```rust
/// 战场单位标识 Component（挂在每个参战实体上）
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BattleUnitId(pub String);

/// 战场单位注册表 Resource（双向查询）
#[derive(Resource, Debug, Default)]
pub struct BattleUnitRegistry {
    entity_to_id: HashMap<Entity, BattleUnitId>,
    id_to_entity: HashMap<BattleUnitId, Entity>,
}

impl BattleUnitRegistry {
    pub fn register(&mut self, entity: Entity, id: BattleUnitId);
    pub fn get_id(&self, entity: &Entity) -> Option<&BattleUnitId>;
    pub fn get_entity(&self, id: &BattleUnitId) -> Option<&Entity>;
    pub fn is_empty(&self) -> bool;
    pub fn clear(&mut self);
}
```

### `recording.rs`
```rust
/// OnBattleStart → 初始化 RecordingSession，注册所有参战单位
pub(crate) fn start_recording_on_battle_begin(
    trigger: On<'_, '_, OnBattleStart>,
    participants: Query<(Entity, &CombatParticipant)>,
    mut registry: ResMut<BattleUnitRegistry>,
    mut recording: ResMut<RecordingSession>,
    mut game_time: ResMut<GameTime>,
    rng: Res<DeterministicRng>,
);

/// 拦截管道中的 UnitActionComplete → 记录为 ReplayCommand
pub(crate) fn record_unit_action(
    trigger: On<'_, '_, UnitActionComplete>,
    registry: Res<BattleUnitRegistry>,
    mut recording: ResMut<RecordingSession>,
);

/// OnBattleEnd → 停止录制，清理注册表
pub(crate) fn stop_recording_on_battle_end(
    trigger: On<'_, '_, OnBattleEnd>,
    mut recording: ResMut<RecordingSession>,
    mut registry: ResMut<BattleUnitRegistry>,
);
```

### `playback.rs`
```rust
/// Update 系统：回放模式下，当管线暂停时读取回放命令并触发 UnitActionComplete
pub(crate) fn dispatch_replay_command(
    mode: Res<ReplayModeGuard>,
    playback: Res<PlaybackSession>,
    pipeline: Res<CombatPipelineDriver>,
    registry: Res<BattleUnitRegistry>,
    mut commands: Commands,
);

/// PreUpdate：回放模式下阻止真实玩家输入
pub(crate) fn skip_player_input_during_replay(
    mode: Res<ReplayModeGuard>,
    mut input_state: ResMut<InputState>,
);
```

## Communication Design

| 通信机制 | 使用场景 |
|----------|----------|
| **Trigger**（`commands.trigger()`） | `UnitActionComplete` 事件（CombatPipelineDriver 已有 observer） |
| **Observer**（`app.add_observer()`） | `OnBattleStart`、`OnBattleEnd`、`UnitActionComplete`（已有事件） |
| **Observer**（本 ADR 新增） | `OnBattleStart` → 启动录制；`OnBattleEnd` → 停止录制 |
| **Query API** | `is_replay()` 辅助函数：`mode.0.is_replay` |
| **Resource** | `RecordingSession`、`PlaybackSession`、`ReplayModeGuard`、`BattleUnitRegistry`、`DeterministicRng`、`BattleUnitRegistry` |

### 录制数据流

```
OnBattleStart (Trigger)
  │
  ↓
start_recording_on_battle_begin (Observer)
  ├── 枚举所有 CombatParticipant 实体
  ├── 为每个实体分配 BattleUnitId（Component + Registry 双向注册）
  ├── 创建 RecordingSession（from ReplayHeader）
  │     ├── version: 1
  │     ├── game_version: env!("CARGO_PKG_VERSION")
  │     ├── session_id: "battle_{timestamp}"
  │     ├── initial_seed: current DeterministicRng seed
  │     └── participants: [BattleUnitId's]
  └── 存入 ResMut<RecordingSession>
  
...战斗进行...

unit_action → 管线暂停
  │
  ▼
外部系统（AI/玩家）执行动作 → 触发 UnitActionComplete
  │
  ├── record_unit_action (Observer)
  │     └── 注册监听 UnitActionComplete → 转换为 ReplayCommand → 记录到 session
  │
  ├── on_unit_action_complete (原有 observer)
  │     └── pipeline.resume()
  │
  └── 继续管线

OnBattleEnd (Trigger)
  │
  ↓
stop_recording_on_battle_end (Observer)
  ├── session.stop() → ReplayLog
  ├── 可选择将 ReplayLog 保存到文件或测试上下文
  └── 清理 BattleUnitRegistry
```

### 回放数据流

```
加载 ReplayLog → 创建 PlaybackSession
  │
  ▼
BattlePhase::Preparation
  │
  ▼
OnEnter::Battle
  ├── initialize_turn_order（正常执行—参战实体相同则 TurnQueue 一致）
  ├── 回放模式下额外：重建 BattleUnitRegistry（从 replay frame 的 entities 重建）
  │
  ▼
管线运行到 unit_action → pause
  │
  ├── dispatch_replay_command (Update system, !is_driving + is_replay)
  │     ├── 读取 PlaybackSession.current_commands()
  │     ├── 找到匹配的 ReplayCommand（当前回合单位）
  │     ├── BattleUnitId → Entity 转换
  │     └── commands.trigger(UnitActionComplete { unit })
  │
  ├── on_unit_action_complete (原有 observer)
  │     └── pipeline.resume()
  │
  └── 继续管线到下一单位
  
回放完成 → ReplayCompleted event → 自动清理
```

## 边界定义

| 方向 | 允许 | 禁止 |
|------|------|------|
| 桥接层→Combat 域 | 读取 CombatPipelineDriver、TurnQueue、BattlePhase State；写入 CombatParticipant 上的 Component | 🟥 修改 pipeline 内部逻辑（step 函数 body） |
| 桥接层→Replay 系统 | 读取/写入 RecordingSession、PlaybackSession、ReplayModeGuard、DeterministicRng | 🟥 修改 replay 引擎的录制/回放核心逻辑 |
| 桥接层→其他域 | 读取 InputState（回放时阻止输入） | 🟥 依赖任何业务域的内部类型 |
| Combat 域→桥接层 | 无需知晓桥接层存在 | 🟥 Combat 域代码中引用 bridge module |

**核心约束**：桥接层是一个纯挂载层——它 hook 现有事件流，不修改核心逻辑。移除桥接层后，Combat 域应完全独立可运行（仅失去录制/回放能力）。

## Forbidden（禁止事项）

- 🟥 禁止修改 `CombatPipelineDriver`、`CombatParticipant`、`TurnQueue` 等现有 combat 核心类型
- 🟥 禁止在 `infra::replay` 中引用任何 combat 域类型（依赖方向：桥接层吸收 combat→replay 引用）
- 🟥 禁止录制侧出现回放逻辑，或回放侧出现录制逻辑（明确分离）
- 🟥 禁止添加第五个通信机制——仅使用已有的 Trigger/Observer/Resource/Query API
- 🟥 禁止桥接层依赖任何业务域语义——`ReplayCommand::Custom` 是跨域扩展点
- 🟥 禁止在录制/回放路径中使用非确定性操作（时间戳、thread_rng、随机 UUID）
- 🟥 禁止桥接层代码直接引用 `ReplayCommand` 枚举变体之外的结构——对 combat 内部类型的引用必须走 `CombatParticipant` Query 或 `BattleUnitRegistry`

## Definition / Instance Design

### Definition（不可变配置）
| 类型 | 说明 | 存放位置 |
|------|------|----------|
| `ReplayHeader` | 回放头部元数据（version, game_version, session_id, initial_seed, participants） | `core::runtime::replay::foundation::types` |
| `ReplayCommand` 枚举 | 8 个变体定义可录制的动作类型 | `core::runtime::replay::foundation::types` |
| `ReplayLog` | 完整回放日志（header + frames） | `core::runtime::replay::foundation::values` |

### Instance（运行时状态）
| 类型 | 角色 | 生命周期 |
|------|------|----------|
| `RecordingSession` (Resource) | 录制会话状态 | 战斗开始时创建 → 战斗结束时销毁 |
| `PlaybackSession` (Resource) | 回放会话状态 | 回放加载时创建 → 回放完成时销毁 |
| `BattleUnitRegistry` (Resource) | Entity↔String 双向映射表 | 战斗开始时创建 → 战斗结束时清理 |
| `BattleUnitId` (Component) | 实体上的稳定标识 | 战斗开始时插入 → 战斗结束时移除 |
| `ReplayModeGuard` (Resource) | 当前是否为回放模式 | 全局单例，由 PlaybackSession 控制 |

## 后果

### 正面
- Combat 域获得完整的战斗录制/确定性回放能力
- 基础设施使用率从 19% → ~45%（新增 1 个系统接入 replay 基础设施）
- 每个战斗 Bug 可录制成回放测试，实现 TestGuardian 的 "Bug → 回放测试" 流程
- 回放能力为后续 save/load 系统提供确定性验证基础
- 桥接层可独立测试，不修改现有 combat 核心逻辑

### 负面
- 桥接层增加了~400 行新代码
- 录制/回放增加了战斗启动的固定开销（Entity↔String 注册）
- 所有使用 RNG 的 combat 系统必须改用 `DeterministicRng`（当前 combat 无 RNG，负面影响为零）
- 后续新增 combat 动作类型时需要同步更新桥接层

## 替代方案

| 方案 | 说明 | 放弃原因 |
|------|------|----------|
| 在 `infra::replay` 中创建 `combat_bridge` | infra 层引用 combat 类型 | 🟥 违反 Core←Infra 依赖方向（ADR-002） |
| 修改 `ReplayCommand` 使其支持 `Entity` 而非 String | 侵蚀 replay 系统的跨域通用性 | Replay 设计为 String-based 以保持域无关性 |
| 在 `CombatPipelineDriver` 中内联录制逻辑 | 侵入核心类型，违反单一职责 | 桥接层作为独立模块更易测试和维护 |
| 用事件总线（EventBus）替代直接 `commands.trigger(UnitActionComplete)` | 增加不必要的间接层 | 现有 observer 模式可直接使用 |

---

*遵循 ADR-006（Capabilities/Domains 双轴）和 ADR-021（Combat）的架构边界。*
