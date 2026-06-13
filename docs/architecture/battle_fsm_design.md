# 战斗状态机设计

> Version: 1.0
> Status: Proposed
> 来源：`docs/其他/31遗漏.md` Section 二（第232-239行）

---

## 1. 概述

战斗是 SRPG 最核心的生命周期，必须有**完整的有限状态机**管理从战斗开始到结束的全流程。本设计定义：

- 战斗阶段枚举及其与 Bevy State 的映射
- 每个状态的进入/退出钩子与扩展点
- 合法的状态转换路径（转换图）
- 状态边界必须满足的不变量

与 `turn_rules.md` 的关系：**Battle FSM 管理宏观战斗流程，TurnPhase 管理回合内微观阶段**。Battle FSM 是顶层状态机，TurnPhase 是 InGame 状态下的 SubState。

---

## 2. 设计

### 2.1 战斗阶段枚举

```rust
/// 战斗生命周期状态（映射到 AppState + GameOverState）
#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BattlePhase {
    /// 战斗初始化：加载关卡、生成地图、部署单位
    PreBattle,
    /// 回合开始：重建行动队列、发送 TurnStarted
    RoundStart,
    /// 玩家/敌方行动阶段（TurnPhase SubState 在此激活）
    PlayerPhase,
    /// 敌方行动阶段（AI 执行）
    EnemyPhase,
    /// 回合结算：胜负检查、回合号递增、acted 重置
    TurnEnd,
    /// 胜负判定：检查所有胜利/失败条件
    VictoryCheck,
    /// 回合结束收尾：发送 TurnEnded、准备下一回合
    RoundEnd,
    /// 战斗结束：发送 LevelCompleted、切换到 GameOver
    PostBattle,
}
```

### 2.2 与 Bevy State 的映射

Battle FSM 通过**两层 Bevy State** 实现：

```
AppState::InGame
  └── BattlePhase (SubState)
        ├── PreBattle
        ├── RoundStart
        ├── PlayerPhase     ← TurnPhase SubState 在此激活
        ├── EnemyPhase      ← TurnPhase SubState 在此激活
        ├── TurnEnd
        ├── VictoryCheck
        ├── RoundEnd
        └── PostBattle
```

#### 映射规则

| BattlePhase | AppState | GameOverState | TurnPhase |
|-------------|----------|---------------|-----------|
| PreBattle | InGame | Playing | 不激活 |
| RoundStart | InGame | Playing | 不激活 |
| PlayerPhase | InGame | Playing | 激活（SelectUnit → ... → TurnEnd） |
| EnemyPhase | InGame | Playing | 激活（SelectUnit → ... → TurnEnd） |
| TurnEnd | InGame | Playing | 不激活 |
| VictoryCheck | InGame | Playing/Transition | 不激活 |
| RoundEnd | InGame | Playing | 不激活 |
| PostBattle | GameOver | Victory/Defeat | 不激活 |

#### AppState 与 BattlePhase 的职责边界

```
AppState 负责：游戏宏观流程（MainMenu → InGame → GameOver）
BattlePhase 负责：战斗内流程（PreBattle → RoundStart → ... → PostBattle）

AppState::InGame 时，BattlePhase 控制战斗循环
AppState::GameOver 时，BattlePhase 不活跃
```

---

### 2.3 状态转换图

```
PreBattle → RoundStart → PlayerPhase → EnemyPhase → TurnEnd → VictoryCheck → RoundEnd
                ↑                                                              │
                └──────────────────── RoundStart ←─────────────────────────────┘
                                                                              
TurnEnd → VictoryCheck → PostBattle (终态达成时)
VictoryCheck → RoundEnd (未终态时)

PlayerPhase → EnemyPhase (玩家行动完毕)
EnemyPhase → TurnEnd (敌方行动完毕)
```

#### 合法转换表

| 从 | 到 | 条件 | 触发系统 |
|----|-----|------|---------|
| PreBattle | RoundStart | 关卡初始化完成 | `init_battle_system` |
| RoundStart | PlayerPhase | 回合开始完成 | `round_start_system` |
| PlayerPhase | EnemyPhase | 玩家方所有单位行动完毕或 ForceEndTurn | `phase_transition_system` |
| EnemyPhase | TurnEnd | 敌方所有单位行动完毕 | `phase_transition_system` |
| TurnEnd | VictoryCheck | 回合结算完成 | `turn_end_system` |
| VictoryCheck | RoundEnd | 未达到终态 | `victory_check_system` |
| VictoryCheck | PostBattle | 终态达成（Victory/Defeat） | `victory_check_system` |
| RoundEnd | RoundStart | 新回合开始 | `round_end_system` |
| PostGame | MainMenu | 玩家选择返回 | `post_battle_system` |

#### 非法转换（禁止）

| 从 | 到 | 禁止原因 |
|----|-----|---------|
| PreBattle | PlayerPhase | 跳过 RoundStart，行动队列未初始化 |
| PlayerPhase | TurnEnd | 跳过 EnemyPhase，敌方未行动 |
| PlayerPhase | VictoryCheck | 跳过回合结算 |
| PlayerPhase | PostBattle | 跳过所有后续阶段 |
| RoundEnd | PlayerPhase | 跳过 RoundStart，队列未重建 |
| TurnEnd | RoundStart | 跳过 VictoryCheck，胜负未判定 |
| PostBattle | 任意 | 终态不可逆 |

---

### 2.4 状态进入/退出钩子

每个 BattlePhase 状态定义 `OnEnter` 和 `OnExit` 系统。

#### OnEnter 钩子

| 状态 | OnEnter 系统 | 职责 | 禁止 |
|------|-------------|------|------|
| PreBattle | `init_turn_order` | 初始化行动队列 | 检查胜负条件 |
| RoundStart | `send_turn_started` | 发送 TurnStarted 消息 | 修改单位状态 |
| PlayerPhase | `highlight_player_units` | 高亮可行动单位 | 执行攻击逻辑 |
| EnemyPhase | `start_ai_decision` | 启动 AI 决策 | 直接执行攻击 |
| TurnEnd | `turn_end_cleanup` | 回合结算（详见 turn_rules.md） | 执行攻击逻辑 |
| VictoryCheck | `check_victory_conditions` | 五步胜负检查管线 | 修改战斗状态 |
| RoundEnd | `send_turn_ended` | 发送 TurnEnded 消息 | 执行业务逻辑 |
| PostBattle | `send_level_completed` | 发送 LevelCompleted 消息 | 继续战斗逻辑 |

#### OnExit 钩子

| 状态 | OnExit 系统 | 职责 | 禁止 |
|------|-----------|------|------|
| PreBattle | — | 无清理需求 | — |
| RoundStart | — | 无清理需求 | — |
| PlayerPhase | `cleanup_player_highlights` | 清理高亮标记 | 修改单位属性 |
| EnemyPhase | `cleanup_ai_state` | 清理 AI 临时状态 | 修改单位属性 |
| TurnEnd | — | 无清理需求 | — |
| VictoryCheck | — | 无清理需求 | — |
| RoundEnd | — | 无清理需求 | — |
| PostBattle | `cleanup_battle_resources` | 清理战斗资源 | 修改业务状态 |

---

### 2.5 扩展点

插件可以在以下位置注入行为：

| 扩展点 | 位置 | 允许行为 | 禁止行为 |
|--------|------|---------|---------|
| OnEnter(RoundStart) | 回合开始 | 发送消息、记录日志、触发 Trait 效果 | 修改单位状态 |
| OnExit(TurnEnd) | 回合结束 | Buff 过期检查、持续效果结算 | 执行攻击逻辑 |
| OnEnter(VictoryCheck) | 胜负判定前 | 准备判定数据 | 修改判定结果 |
| OnEnter(PostBattle) | 战斗结束 | 播放结算动画、保存结果 | 继续战斗逻辑 |

#### 扩展点注册规范

```rust
// ✅ 正确：通过 Plugin 在扩展点注入
impl Plugin for MyTraitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(BattlePhase::RoundStart), 
            trigger_trait_effects);
    }
}

// 🟥 错误：在 FSM 内部硬编码扩展逻辑
fn on_enter_round_start() {
    // 直接调用其他模块的函数
    buff_system::resolve_all_buffs();  // 违反模块边界
}
```

---

## 3. 不变量

### 不变量1：PreBattle 必须先于所有战斗阶段

```
任何战斗逻辑执行前，BattlePhase 必须已经过 PreBattle 阶段。
禁止：直接从 MainMenu 跳到 PlayerPhase
```

### 不变量2：RoundStart 必须在每个新回合执行

```
每个新回合开始前，BattlePhase 必须经过 RoundStart 阶段。
禁止：从 RoundEnd 直接跳到 PlayerPhase
```

### 不变量3：TurnEnd 必须在 VictoryCheck 之前

```
胜负检查必须在回合结算完成后执行。
禁止：从 TurnEnd 直接到 RoundEnd 跳过 VictoryCheck
```

### 不变量4：终态后不可逆

```
GameOverState 变为 Victory 或 Defeat 后：
- BattlePhase 必须进入 PostBattle
- 禁止从 PostBattle 回到任何战斗阶段
- PostBattle 后切换到 AppState::GameOver
```

### 不变量5：PlayerPhase 和 EnemyPhase 互斥

```
同一时刻只能处于 PlayerPhase 或 EnemyPhase 之一。
禁止：两者同时激活
```

### 不变量6：每个回合必须完整走完流程

```
RoundStart → PlayerPhase → EnemyPhase → TurnEnd → VictoryCheck → RoundEnd
禁止：跳过任何阶段（除终态时跳过 RoundEnd 直接到 PostBattle）
```

---

## 4. 规则总结

### FSM 职责边界

```
BattleFSM 负责：
  ✅ 战斗阶段的流转控制
  ✅ 状态进入/退出钩子
  ✅ 合法转换路径管理
  ✅ 扩展点暴露

BattleFSM 不负责：
  ❌ 具体伤害计算（由 Effect Pipeline 负责）
  ❌ 行动队列管理（由 TurnOrder 负责）
  ❌ 胜负条件的具体判定（由 VictoryCheck Pipeline 负责）
  ❌ Buff/效果的具体结算（由 Buff 领域负责）
```

### 与现有文档的关系

```
battle_fsm_design.md（本文档）    → 宏观战斗流程
turn_rules.md                     → 回合内微观阶段（TurnPhase SubState）
battle_rules.md                   → 胜负条件、Effect Pipeline
app-bootstrap.md                  → AppState 层级与启动
```

---

## 5. 禁止事项

| 禁止项 | 理由 | 违反后果 |
|--------|------|---------|
| 🟥 手动设置 BattlePhase 而不经过 NextState | 跳过 OnEnter/OnExit 生命周期 | 钩子未触发、状态不一致 |
| 🟥 在 OnEnter 中执行跨阶段跳转 | 破坏状态机的确定性 | 循环跳转、栈溢出 |
| 🟥 从 ActionPhase 直接跳到 PostBattle | 跳过所有后续阶段 | 回合结算缺失 |
| 🟥 跳过 TurnEnd 直接开始新回合 | acted 未重置、队列未重建 | 单位无法行动 |
| 🟥 在 FSM 内部包含业务逻辑 | 违反状态机职责单一 | 逻辑耦合、难以维护 |
| 🟥 状态机处理业务细节 | FSM 只负责流转 | 业务逻辑分散 |
| 🟥 循环状态转换（A→B→A→B 无终止） | 无限循环 | 游戏卡死 |
| 🟥 在 PlayerPhase 内执行敌方逻辑 | 职责混淆 | AI 和玩家逻辑耦合 |

---

## 6. 交叉引用

| 文档 | 关系 |
|------|------|
| `docs/domain/turn_rules.md` | TurnPhase SubState 在 PlayerPhase/EnemyPhase 内激活 |
| `docs/domain/battle_rules.md` | 胜负条件判定、Effect Pipeline 执行 |
| `docs/architecture/app-bootstrap.md` | AppState 层级定义、Schedule/SystemSet 组织 |
| `docs/architecture/determinism_rules.md` | FSM 状态转换必须确定性 |
| `docs/architecture/schedules_design.md` | 系统在正确 Schedule 中注册 |
| `docs/domain/replay_rules.md` | 回放系统记录 FSM 状态转换序列 |
