---
id: 01-architecture.ADR-021
title: ADR-021 — Turn State Machine
status: approved
owner: architect
created: 2026-06-16
updated: 2026-06-18
supersedes: none
partially-superseded-by: ADR-050 (BattlePhase 注册方式从独立 States 转为 SubState，绑定到 GameState::Combat；枚举值 Preparation/Battle/Victory/Defeat 保持不变)
---

# ADR-021: 回合状态机设计

## 状态

**Approved**（部分被 ADR-050 取代）— 依赖 ADR-000（Feature Module Map）和 `docs/04-data/domains/tactical_schema.md`，本架构决策正式生效。

> ⚠️ **ADR-050 变更说明**：BattlePhase 从独立 `States` 转为 `GameState::Combat` 的 `SubStates`。枚举值（Preparation/Battle/Victory/Defeat）保持不变，回合内流程仍由 `CombatPipelineDriver` 驱动。详见 ADR-050 §2。

## 背景

回合制战棋的核心是回合流程管理。根据 SRPG 专项规则（§四），回合必须划分为标准化阶段，每个阶段对应独立的 System 集与触发点。回合切换通过状态机驱动，禁止手动调用回合切换函数。

## 引用的领域规则与数据架构

- `docs/02-domain/domains/combat_domain.md` — Combat 领域规则（§2.2 回合行动规则, §5.2 回合流转）
- `docs/04-data/domains/combat_schema.md` — Combat Schema（CombatState, TurnOrder, ActionPoints）
- `.trae/rules/SRPG专项规则.md` §四 — 回合系统规范
- `.trae/rules/ECS规则.md` §四 — 状态管理使用 Bevy States

## 决策

### 1. 回合状态层次结构

使用 Bevy `SubStates` 实现两层次状态机：

```
外层: BattlePhase (宏观战斗阶段)

  Preparation (部署阶段)
       │
       ▼
  Battle (战斗进行中)
       │
       ├── TurnLoop (回合循环) ← 内层状态机
       │
       ▼
  Victory / Defeat (战斗结束)

内层: TurnLoop (回合循环 SubState)

  TurnStart (回合开始)
       │
       ▼
  PhaseCheck (阶段判定 — 移动/行动/技能)
       │
       ▼
  UnitAction (单位行动中)
       │
       ▼
  TurnSettlement (回合结算 — Buff Tick/Duration 减1)
       │
       ▼
  TurnEnd (回合结束 → 下一个单位/队伍)
```

### 2. 外层状态：BattlePhase

```rust
/// 宏观战斗阶段 — States
#[derive(States, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum BattlePhase {
    #[default]
    Preparation,    // 战前部署
    Battle,         // 战斗中
    Victory,        // 胜利
    Defeat,         // 失败
}
```

### 3. 内层状态：TurnSubState

```rust
/// 回合内子状态 — SubStates
/// 父状态: BattlePhase::Battle
#[derive(SubStates, Clone, Eq, PartialEq, Hash, Debug)]
#[source(BattlePhase = BattlePhase::Battle)]
pub enum TurnSubState {
    TurnStart,          // Phase 1: 回合开始
    PhaseCheck,         // Phase 2: 阶段判定
    UnitAction,         // Phase 3: 单位行动
    TurnSettlement,     // Phase 4: 回合结算
    TurnEnd,            // Phase 5: 回合结束
}
```

### 4. 每个阶段的 System 职责

```rust
// src/core/domains/combat/systems/turn_systems.rs

/// 进入 TurnStart
fn on_enter_turn_start(
    mut turn_queue: ResMut<TurnQueue>,
    mut next_state: ResMut<NextState<TurnSubState>>,
) {
    let current = turn_queue.current();
    // 1. 给当前单位发放 ActionPoint
    current.reset_action_points();
    // 2. OnTurnStart Trigger → 触发"回合开始"类 Effect
    commands.trigger(OnTurnStart { unit: current.entity });
    // 3. 进入下一个子阶段
    next_state.set(TurnSubState::PhaseCheck);
}

/// PhaseCheck: 判定单位可以执行什么行动
fn phase_check(
    turn_queue: Res<TurnQueue>,
    unit_query: Query<(&ActionPoints, &MovementLeft)>,
    mut next_state: ResMut<NextState<TurnSubState>>,
) {
    let unit = turn_queue.current();
    let (ap, mp) = unit_query.get(unit.entity).unwrap();
    if ap.current > 0 && mp.current > 0 {
        // 可以移动和行动 → 等待玩家/AI 输入
        next_state.set(TurnSubState::UnitAction);
    } else if ap.current > 0 {
        // 只能行动
        next_state.set(TurnSubState::UnitAction);
    } else {
        // 无事可做 → 跳过
        next_state.set(TurnSubState::TurnSettlement);
    }
}

/// UnitAction: 等待行动完成
fn unit_action(
    mut action_events: EventReader<UnitActionComplete>,
    mut next_state: ResMut<NextState<TurnSubState>>,
) {
    for _ in action_events.read() {
        next_state.set(TurnSubState::TurnSettlement);
    }
}

/// TurnSettlement: Buff Tick / Duration 结算
fn turn_settlement(
    mut next_state: ResMut<NextState<TurnSubState>>,
) {
    // 1. 触发 OnTurnEnd 领域事件
    commands.trigger(OnTurnEnd { unit: current.entity });
    // 2. Buff Duration -1
    // 3. 检查"回合结束"类 Effect
    // 4. 进入 TurnEnd
    next_state.set(TurnSubState::TurnEnd);
}

/// TurnEnd: 切换到下一个单位
fn turn_end(
    mut turn_queue: ResMut<TurnQueue>,
    mut battle_phase: ResMut<NextState<BattlePhase>>,
    mut turn_sub_state: ResMut<NextState<TurnSubState>>,
) {
    // 1. 切换队列中的下一个单位
    turn_queue.advance();

    // 2. 如果是新队伍的回合 → 触发 BetweenTurns 事件
    if turn_queue.just_changed_team() {
        commands.trigger(BetweenTurns {
            team: turn_queue.current_team(),
        });
    }

    // 3. 检查战斗结束条件
    if turn_queue.is_victory() {
        battle_phase.set(BattlePhase::Victory);
    } else if turn_queue.is_defeat() {
        battle_phase.set(BattlePhase::Defeat);
    } else {
        // 回到 TurnStart
        turn_sub_state.set(TurnSubState::TurnStart);
    }
}
```

### 5. 回合队列

```rust
/// TurnQueue — 管理行动顺序
#[derive(Resource)]
pub struct TurnQueue {
    entries: Vec<TurnEntry>,
    current_index: usize,
    round_number: u32,
}

pub struct TurnEntry {
    pub entity: Entity,
    pub team: TeamId,
    pub initiative: u32,    // 先攻值（用于排序）
}
```

### 6. State 驱动的事件链

```
BattlePhase::Battle (OnEnter)
       │
       ▼
TurnSubState::TurnStart (OnEnter)
       │
       ├── ActionPoint 重置
       ├── commands.trigger(OnTurnStart { unit })
       └── → TurnSubState::PhaseCheck
              │
              ▼
       TurnSubState::PhaseCheck (OnEnter)
              │
              ├── 检查 AP/MP → 进入 UnitAction
              └── 无行动 → TurnSettlement
                     │
                     ▼
       TurnSubState::UnitAction (等待)
              │
              └── UnitActionComplete Event → TurnSettlement
                     │
                     ▼
       TurnSubState::TurnSettlement (OnEnter)
              │
              ├── Buff Duration -1
              ├── commands.trigger(OnTurnEnd { unit })
              └── → TurnSubState::TurnEnd
                     │
                     ▼
       TurnSubState::TurnEnd (OnEnter)
              │
              ├── turn_queue.advance()
              ├── 战斗结束？ → Victory/Defeat
              └── 继续 → TurnStart
```

### 7. 与回合阶段相关的领域事件

| 事件 | 触发时机 | 监听者 |
|------|---------|--------|
| `OnTurnStart { unit }` | TurnStart Phase | Buff（"每回合开始时"）、被动 |
| `OnUnitAction { unit }` | UnitAction 开始时 | 技能预览、UI 高亮 |
| `OnTurnEnd { unit }` | TurnSettlement Phase | Buff（"每回合结束时"）、DOT |
| `BetweenTurns { team }` | 切换到新队伍时 | 领域效果、环境效果 |
| `OnRoundEnd { round }` | 所有队伍结束 | 全局结算、召唤物消失 |
| `OnBattleStart` | BattlePhase::Battle OnEnter | 初始化战斗相关系统 |
| `OnBattleEnd { result }` | Victory/Defeat | 经验结算、战利品、回放结束 |

## Module Design

> **Note**: Module 重新归属 `combat/`（v5.0 架构总纲明确"回合流程、先攻、伤害结算、胜负"归 Combat 域）。原 ADR 草案指向 `tactical/`，此处修正。

```
src/core/domains/combat/
  ├── plugin.rs              — CombatPlugin（含 TurnPhasePlugin 的子注册）
  ├── components.rs          — BattlePhase, TurnSubState, TurnQueue, TurnEntry, ActionPoints
  ├── systems/
  │   ├── mod.rs
  │   └── turn_systems.rs    — 各阶段 System + 状态转移
  ├── events.rs              — 回合相关领域事件
  └── integration/turn/      — get_current_turn(), get_turn_queue()（ADR-046）
```

## Communication Design

| 通信 | 机制 | 方向 |
|------|------|------|
| 阶段转移 | `NextState<TurnSubState>` | turn_phase 内部 |
| 回合领域事件 | Trigger (`OnTurnStart`, `OnTurnEnd`) | turn_phase → observer 链 |
| 行动完成通知 | Event (`UnitActionComplete`) | 外部 combat/ability → turn_phase |
| 战斗结束 | `NextState<BattlePhase>` | turn_phase → 全局 |

## 边界定义

### 允许
- 外部通过 `UnitActionComplete` Event 通知回合系统"我行动完了"
- 外部通过 `OnTurnStart/OnTurnEnd` Trigger 接入回合生命周期
- turn_phase 读取单位属性（Layer 2）以判定可否行动

### 🟥 禁止
- 外部直接 `turn_queue.advance()` 或 `next_state.set(TurnSubState::...)`
- 回合阶段中执行非法的业务逻辑（如 TurnStart 中伤害结算）
- 在 OnEnter/OnExit 中执行重型操作（必须惰性到 Update System）
- 外部跳过 UnitAction 直接进入 TurnSettlement

## Forbidden

| 禁止行为 | 理由 |
|---------|------|
| 手动调用回合切换函数 | 必须通过 State 驱动 |
| 在 OnEnter 中执行重型逻辑 | OnEnter 应轻量，重型逻辑放 Update |
| 外部直接修改 TurnQueue.index | TurnQueue 应为 turn_phase 私有 |
| 跳过 PhaseCheck 阶段 | 每回合必须进行能力判定 |
| 同一 System 跨多个阶段 | 每阶段独立 System 集 |

## Definition / Instance Design

- **Definition**: `TurnPhaseDef` (config: 每队伍最大行动点数、阶段时间等)
- **Instance**: `TurnQueue` (Resource), `BattlePhase` (State), `TurnSubState` (SubState)
- **Persistence**: `TurnQueue` 存档（用于读档后恢复当前回合）

## 后果

### 正面
- 两层次状态机清晰分离宏观流程和回合内流程
- Bevy SubStates 天然绑定 OnEnter/OnExit 生命周期
- 五个标准化阶段与 SRPG 专项规则完全对齐
- 领域事件驱动让外部系统可以监听回合生命周期

### 负面
- SubState 的 Source 绑定到 `BattlePhase::Battle`，不支持在 Preparation 阶段使用回合逻辑
- 初版只需要最基本的回合切换，五阶段可能显得"过度设计"

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 单层状态机 | Battle 和 Turn 两层次混在一起，状态爆炸 |
| Event 驱动的阶段转移 | 不如 State 声明式清晰，调试困难 |
| 简单 bool 标记 | 无法保证阶段顺序，不符合 SRPG 规范 |

## 评审要点

- [ ] 五个子阶段是否覆盖了所有回合内场景？是否需要 ExtraAction 阶段？
- [ ] BetweenTurns 事件的确切含义——队伍切换时还是回合切换时？
- [ ] Preparation 阶段是否足够灵活（布阵、对话、购买）？
- [ ] ActionPoint 和 MovementLeft 的恢复时机是否正确？
