---
id: 01-architecture.battle-fsm-design
title: Battle FSM Design
status: draft
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - architecture
  - design
---

# 战斗状态机设计

> Version: 1.1
> Status: Proposed
> 来源：`docs/其他/31遗漏.md` Section 二（第232-239行）、`docs/其他/76.md` §八（调度与确定性）

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

> **优化来源**：`docs/其他/39.md` — FSM 必须作为 Component 而非 Resource 挂载，支持多战场并行

### 2.1.1 FSM 存储模式：Component vs Resource

> ⚠️ **§1.1.7 过度设计警告**：以下 Component-based FSM 方案为"多战场并行"需求设计。当前项目为单战场 SRPG，**应优先使用 Bevy 原生 `States/SubStates` 机制（§2.3.6）**，仅在明确需要多战场支持时才迁移为 Component-based FSM。

🟥 **FSM 状态必须是 Component，不是 Resource**（仅在多战场场景下）。

将 BattleFsm 作为全局 Resource 会导致无法支持"多战场并存"（如主线关卡 + 支线副本同时运行）。正确做法是将 FSM 挂载到每个战场实体上：

```rust
#[derive(Component, Reflect)]
pub struct BattleFsm {
    pub current_phase: BattlePhase,
    pub current_sub_state: SubState,
    pub transition_history: VecDeque<TransitionRecord>, // 用于 Replay/Audit
}

// 每个战场实体（BattleArena）持有自己的 FSM
// 系统通过 Query<&mut BattleFsm> 并行处理多个战场
```

优势：
- 多战场并行：为未来联机/后台模拟预留架构空间
- 天然并行：`Query<&mut BattleFsm>` 可自动并行化
- 隔离性：一个战场的 FSM 转换不会影响另一个

**Bevy 0.18+ 实现要点**：
- 使用 `#[derive(States)]` 的 SubState 机制时，每个战场实体需要独立的 FSM Component
- 避免使用全局 `NextState<BattlePhase>`，改用 `Commands` 或直接修改 Component 字段

**当前推荐方案（§2.3.6 合规）**：
```rust
// ✅ 当前推荐：使用 Bevy 原生 States 机制
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum BattlePhase {
    #[default]
    PreBattle,
    RoundStart,
    PlayerPhase,
    EnemyPhase,
    TurnEnd,
    VictoryCheck,
    RoundEnd,
    PostBattle,
}

// 通过 NextState<BattlePhase> 驱动状态转换
// OnEnter/OnExit 自动触发钩子
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

### 2.6 Guard / Action / Effect 三段式

> **优化来源**：`docs/其他/39.md` — 物理分离 Guard（纯函数只读）、Action（同步变更）、Effect（事件发射），防止在 Guard 中执行变更操作

状态转换的执行逻辑必须物理分离为三个阶段：

```
Guard（守卫）→ Action（动作）→ Effect（效果）
  纯函数        同步变更        事件发射
  只读查询      修改 Component    发送 Domain Event
```

| 阶段 | 职责 | 可执行操作 | 禁止操作 |
|------|------|-----------|----------|
| Guard | 判断"能不能转" | 读取 Context、计算布尔值 | 修改任何 Component/Resource |
| Action | 处理"状态切换的即时逻辑" | 播放音效、修改标记 Component | 发送 Domain Event |
| Effect | 触发"后续领域事件" | 发送 TurnStarted、UnitDamaged | 修改 FSM 状态 |

**单元测试友好**：
- Guard 可纯函数测试：给定 Context → 断言 true/false
- Effect 可通过事件监听器验证：断言事件被正确发送
- Action 可通过状态断言验证

**反模式警示**：
- ❌ 在 Guard 中调用 `asset_server.load()`（IO 操作破坏确定性）
- ❌ 在 Guard 中修改 Component（破坏纯函数约束）
- ❌ 在 Effect 中直接修改 FSM 状态（必须通过下一帧 Guard 重新评估）

### 2.7 GuardContext 预计算

> **优化来源**：`docs/其他/39.md` — 批量收集所有查询数据到 GuardContext，避免每帧 ECS 随机访问

Guard 需要访问 ECS 世界数据，但频繁调用 `query.get()` 会产生严重的随机访问开销。解决方案是在 Phase 进入时一次性预计算所有 Guard 所需数据：

```rust
#[derive(Resource)]
pub struct GuardContext {
    pub active_unit_stats: HashMap<Entity, UnitStatsSnapshot>,
    pub map_terrain_cache: TerrainGrid,
    pub buff_registry_snapshot: Vec<BuffId>,
    pub range_cache: HashMap<Entity, RangeInfo>,  // SRPG 射程/视线预计算
}

// Guard 只读 Context，不直接查 ECS
fn can_transition_to_attack(ctx: &GuardContext, unit: Entity) -> bool {
    ctx.active_unit_stats.get(&unit).map_or(false, |s| s.mp >= 10)
}
```

**性能收益**：将 N 次随机 ECS 查询压缩为 1 次批量收集 + N 次 HashMap 查找，性能提升 10-100 倍。

**实现时机**：
- `OnEnter(BattlePhase::RoundStart)` 时重建 GuardContext
- `OnEnter(BattlePhase::PlayerPhase)` / `OnEnter(BattlePhase::EnemyPhase)` 时刷新

### 2.8 转换优先级协议

> **优化来源**：`docs/其他/39.md` — 显式 priority: u32 字段，高优先级短路评估，避免多规则冲突时的不可预测行为

当多条 TransitionRule 同时满足时，需要明确的裁决机制：

```
Transition 优先级协议：
1. 每条规则必须有显式 priority: u32 字段
2. 同优先级时，按声明顺序（RON 文件中的先后）裁决
3. 高优先级规则触发后，立即短路，不再评估后续规则
4. 优先级数值规范：
   - 0-99   = 常规行动（移动、攻击、技能）
   - 100-199 = 打断/反击
   - 200-299 = 死亡/退场
   - 300+   = 系统级强制转换
```

```rust
#[derive(Deserialize)]
pub struct TransitionRule {
    pub from: BattlePhase,
    pub to: BattlePhase,
    pub priority: u32,           // 必须显式声明
    pub guard: String,           // Guard 函数名
    pub action: String,          // Action 函数名
    pub effect: String,          // Effect 函数名
}
```

### 2.9 一帧延迟反模式（One-Frame Lag）

> **优化来源**：`docs/其他/39.md` — Guard 在第 N 帧评估，Transition 在第 N+1 帧通过 Commands 应用

FSM 转换存在一帧延迟，这是 ECS 架构的固有特性，必须显式记录：

```
帧 N：  Guard 评估通过 → Action 执行 → Effect 发送事件
帧 N+1：Commands 应用状态转换 → OnEnter/OnExit 钩子触发
```

**影响**：
- 状态转换不是即时的，Event → FSM 的反馈必须通过下一帧 Guard 重新评估
- 不要在同一帧内假设状态已经改变
- Replay 系统需要记录 `(tick, from, to, trigger_rule_id, seed)` 以精准复现

**确定性保证**：
- FSM → Event：单向输出，FSM 只负责发出事实事件
- Event → FSM：绝对禁止，事件监听器不能直接修改 BattleFsm Component
- 反馈路径：通过下一帧 Guard 重新评估实现

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

## 4.5 回合内调度时序（SRPG Lite-GAS 对齐）

> **来源**：`docs/其他/76.md` §八（调度与确定性）— 所有战斗逻辑挂载到回合固定阶段
> **ADR-026 扩展**：Effect Pipeline 内部扩展为 Effect → Stacking → Execution → Modifier → Attribute → Tag → Cue 全链路

回合内所有战斗逻辑必须挂载到回合状态机的固定阶段，完全不用帧更新驱动。以下是精确的调度时序：

```
┌─────────────────────────────────────────────────────────┐
│  TurnStarted（Phase 入口，OnEnter(RoundStart) 触发）     │
├─────────────────────────────────────────────────────────┤
│  1. Buff Tick & 持续效果结算                              │
│     ├── 递减所有 DurationPolicy::Turns 的 remaining_turns│
│     ├── DoT 结算（毒/燃烧等持续伤害）                     │
│     ├── HoT 结算（再生等持续治疗）                        │
│     ├── Stun 结算（晕眩单位标记 acted=true）              │
│     └── 到期 Buff 标记为待移除                            │
│                                                         │
│  2. ⭐ 回合开始 Trigger 触发                              │
│     ├── TriggerRegistry 分发 OnTurnStart                  │
│     ├── 匹配的 Trigger Handler 返回 EffectDef[]           │
│     └── EffectDef[] 压入 ExecutionStack（LIFO）           │
│                                                         │
│  3. ⭐ ExecutionStack 解析（递归）                         │
│     ├── 弹出栈顶 Entry                                   │
│     ├── Condition 检查                                    │
│     │   ├── 通过 → 进入 Effect Pipeline                   │
│     │   │              ├── Generate（纯函数）              │
│     │   │              ├── Modify（ModifierRule 匹配）     │
│     │   │              └── Execute（副作用 + Message）     │
│     │   │              ↓                                  │
│     │   │         Effect 完成 → 新事件触发 → 可能再压栈    │
│     │   └── 失败 → 跳过，弹出下一个                        │
│     ├── 栈深度 ≥ MAX_STACK_DEPTH(32) → 强制弹出 + WARN   │
│     └── 栈空 → 本轮触发链结束                              │
│                                                         │
│  4. 过期 Buff 清理                                        │
│     ├── 移除待移除 Buff                                   │
│     ├── 清理 Modifier                                     │
│     └── 触发 BuffRemoved 事件                             │
│                                                         │
│  5. 冷却扣减                                              │
│     └── SkillCooldowns.tick()                             │
│                                                         │
├─────────────────────────────────────────────────────────┤
│  PlayerPhase / EnemyPhase（单位行动阶段）                  │
│                                                         │
│  每个单位行动时的子时序：                                  │
│                                                         │
│  SelectUnit → MoveUnit → ActionMenu → SelectTarget       │
│      ↓                                                   │
│  ExecuteAction:                                           │
│    1. SkillCast 请求进入                                  │
│    2. Requirement 检查（can_use 验证）                    │
│    3. Cost 扣除（MP/HP/弹药）                              │
│    4. Targeting 目标解析                                  │
│    5. ⭐ Effect Pipeline（Generate → Modify → Execute）
       ├── Generate（生成 Effect 意图）
       ├── Stacking（堆叠策略匹配：Replace/RefreshDuration/StackAdd/StackMax）
       ├── Execution（公式执行：Damage/Heal/Shield Execution trait 分发）
       ├── Modifier（ModifierRule 修饰匹配）
       ├── Attribute（基础/派生属性刷新）
       ├── Tag（标签变更）
       └── Cue（表现事件下发 → UI/特效/音效订阅）   │
│       ↓                                                  │
│    6. ⭐ Trigger 触发 → ExecutionStack 解析               │
│       ├── OnAttack / AfterAttack / OnDamage 等            │
│       ├── 匹配 → 返回 EffectDef[] → 压入 Stack            │
│       └── Stack 解析（深度 ≤ 32，防止无限递归）             │
│    7. Settlement（冷却设置、状态标记）                      │
│                                                         │
├─────────────────────────────────────────────────────────┤
│  TurnEnded（Phase 出口，OnEnter(TurnEnd) 触发）           │
├─────────────────────────────────────────────────────────┤
│  6. 回合结束 Trigger 触发                                 │
│     ├── TriggerRegistry 分发 OnTurnEnd                    │
│     └── → ExecutionStack 解析                              │
│                                                         │
│  7. Buff 到期最终清理                                     │
│  8. 冷却扣减（再次确保）                                   │
│  9. acted 重置 + 队列重建                                  │
│                                                         │
├─────────────────────────────────────────────────────────┤
│  VictoryCheck → RoundEnd / PostBattle                     │
└─────────────────────────────────────────────────────────┘
```

### 关键约束

| 约束 | 说明 | 违反后果 |
|------|------|---------|
| 🟥 Buff Tick 只发生在 RoundStart | 不在 ExecuteAction 阶段 tick | 持续效果在单位行动后提前结算 |
| 🟥 Trigger 分发走 Registry | 不在 System 中硬编码匹配 | 新增 Trigger 需改核心代码 |
| 🟥 ExecutionStack 深度上限 32 | 防止无限递归 | 栈溢出、游戏崩溃 |
| 🟥 Effect Pipeline 是唯一执行通道 | Trigger 产生的 EffectDef 也走 Pipeline | 修饰规则不生效 |
| 🟥 冷却扣减只在 TurnEnd | 不在其他阶段 tick | 冷却期缩短、不同步 |
| 🟩 Replay 确定性 | 同种子 + 同指令序列 = 同结果 | Replay 不可复现 |

### 与领域文档的对应

| 时序步骤 | 负责领域 | 参考文档 |
|---------|---------|---------|
| Buff Tick | Buff 领域 | `docs/02-domain/buff/buff-rules.md` §5.3 |
| Trigger 分发 | Trigger 领域 | `docs/02-domain/trigger/trigger-rules.md` |
| ExecutionStack 解析 | Trigger 领域 | `docs/02-domain/trigger/trigger-rules.md` |
| Effect Pipeline | Effect 领域（一级） | `docs/02-domain/effect/effect-rules.md` |
| Execution 算式执行 | Execution 领域（新增） | `docs/02-domain/execution/execution-rules.md` |
| Cue 表现事件 | Cue 领域（新增） | `docs/02-domain/cue/cue-rules.md` |
| Stacking 堆叠策略 | Stacking 领域 | `docs/02-domain/stack-policy/stack-policy-rules.md` |
| Skill 执行管线 | Skill 领域 | `docs/02-domain/skill/skill-rules.md` |
| Cooldown tick | Skill 领域 | `docs/02-domain/skill/skill-rules.md` |
| Victory Check | Battle 领域 | `docs/02-domain/battle/battle-rules.md` |

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
| 🟥 在 Guard 中直接查询 ECS 世界 | 随机访问开销 | 性能下降 10-100 倍 |
| 🟥 在 Guard 中执行 IO 操作（如 asset_server.load） | 破坏确定性 | Replay 不可复现 |
| 🟥 事件监听器直接修改 BattleFsm Component | 破坏单向数据流 | FSM 状态不可预测 |
| 🟥 使用 Timer 控制状态持续时间 | 依赖系统时钟 | 确定性破坏，应用 Tick 计数 |

---

## 6. SRPG 专项补充

> **优化来源**：`docs/其他/39.md` — SubState 预留、Transition History、Effect 执行分层

| FSM 设计点 | SRPG 专项建议 | 理由 |
|-----------|--------------|------|
| SubState 枚举 | 预留 `WaitingForPlayerInput` 和 `WaitingForAiDecision` 两个独立子状态 | 玩家操作和 AI 思考的时间尺度完全不同，分离后可独立做超时/加速逻辑 |
| Transition History | 记录 `(tick, from, to, trigger_rule_id, seed)` | Replay 和 Bug 复现的核心数据源，比字符串日志有价值 100 倍 |
| Guard Context | 包含"视线/射程预计算结果" | SRPG 中射程/视线计算是最重的操作，必须在 Phase 入口缓存 |
| Effect 执行 | 区分 `ImmediateEffect` 和 `QueuedEffect` | 伤害结算立即生效，但动画/音效入队延迟执行，避免阻塞 FSM 推进 |

---

## 7. 附录

### 7.1 新增 Transition Rule Checklist

> **优化来源**：`docs/其他/39.md` — 新增转换规则时的标准检查流程

新增任何 Transition Rule 时，必须完成以下检查：

- [ ] 是否定义了明确的 `priority: u32`？
- [ ] Guard 是否纯函数且只读 GuardContext？
- [ ] Effect 是否只发事件、不改 FSM 状态？
- [ ] 是否有对应的单元测试覆盖 Guard 真/假两种情况？
- [ ] 是否在 Replay 测试中验证了确定性？
- [ ] 是否避免了在同一帧内假设状态已改变（One-Frame Lag）？

### 7.2 常见反模式

> **优化来源**：`docs/其他/39.md` — FSM 实践中的典型错误

- ❌ 在 Guard 中调用 `asset_server.load()`（IO 操作破坏确定性）
- ❌ 用 `Timer` 控制状态持续时间（应使用 Tick 计数）
- ❌ 在 Action 中修改其他实体的 Component（应通过 Effect 发事件）
- ❌ 事件监听器直接修改 BattleFsm（应通过下一帧 Guard 重新评估）
- ❌ 将 FSM 作为全局 Resource（应作为 Component 挂载到战场实体）

---

## 8. 交叉引用

| 文档 | 关系 |
|------|------|
| `docs/02-domain/turn/turn-rules.md` | TurnPhase SubState 在 PlayerPhase/EnemyPhase 内激活 |
| `docs/02-domain/battle/battle-rules.md` | 胜负条件判定、Effect Pipeline 执行 |
| `docs/02-domain/effect/effect-rules.md` | Effect 作为一级领域（v2.0 新增） |
| `docs/02-domain/trigger/trigger-rules.md` | TriggerRegistry + ExecutionStack 调度 |
| `docs/02-domain/buff/buff-rules.md` | Buff Tick 生命周期、三层标签重建 |
| `docs/01-architecture/00-overview/app-bootstrap.md` | AppState 层级定义、Schedule/SystemSet 组织 |
| `docs/01-architecture/02-ecs-patterns/determinism-rules.md` | FSM 状态转换必须确定性 |
| `docs/01-architecture/schedules_design.md` | 系统在正确 Schedule 中注册 |
| `docs/01-architecture/01-battle-gas/skill-buff-abstraction.md` | Effect/Trigger/ExecutionStack 完整抽象模型 |
| `docs/02-domain/replay-rules.md` | 回放系统记录 FSM 状态转换序列 |
| `docs/00-governance/ai-constitution-complete.md` | §2.3.6 状态管理、§16.0.1-16.0.5 生命周期、§1.1.7 避免过度设计 |

---

## 宪法合规说明

| 条款 | 合规状态 | 说明 |
|------|---------|------|
| 🟩 §2.3.6 状态管理 | ✅ 合规 | 使用 `#[derive(States)]` + SubState 机制 |
| 🟩 §11.1.1 回合阶段标准化 | ✅ 合规 | RoundStart → PlayerPhase/EnemyPhase → TurnEnd → VictoryCheck → RoundEnd |
| 🟩 §16.0.1 OnEnter/OnExit 轻量 | ✅ 合规 | 钩子只做轻量操作（发送消息、清理标记） |
| 🟩 §16.0.4 FSM 职责 | ✅ 合规 | FSM 只负责流转，不包含业务逻辑 |
| 🟥 §1.1.7 避免过度设计 | ⚠️ 需关注 | Component-based FSM 方案为未来多战场需求设计，当前应使用 Bevy 原生 States |
| 🟥 §20.6.1 禁止 todo!() | ⚠️ 需关注 | 代码示例中的 todo!() 仅为占位，实现时必须替换 |
