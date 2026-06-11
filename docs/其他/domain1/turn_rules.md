# 回合领域规则 (Turn Rules)

## 1. 领域概述

回合系统管理 SRPG 的游戏流程状态机、行动队列和回合切换。采用敏捷驱动行动队列，所有单位按 Initiative 降序行动，队列耗尽时回合结束。遵循 **状态机负责流程，不负责业务细节**原则。

### 核心原则

- **状态机负责流程，不负责业务细节**
- **OnEnter / OnExit 保持轻量**
- **Message 负责跨 Feature 广播**：TurnStarted / TurnEnded
- **敏捷驱动行动队列**：Initiative 决定行动顺序

---

## 2. AppState — 游戏主状态

```rust
pub enum AppState {
    MainMenu,   // 主菜单
    InGame,     // 游戏中
    GameOver,   // 游戏结束
}
```

---

## 3. TurnPhase — 回合阶段

```rust
#[derive(SubStates)]
#[source(AppState = AppState::InGame)]
pub enum TurnPhase {
    SelectUnit,     // 选择单位
    MoveUnit,       // 移动阶段
    ActionMenu,     // 行动菜单（右键弹出）
    SelectTarget,   // 选择攻击目标
    ExecuteAction,  // 执行攻击
    WaitAction,     // 待机
    TurnEnd,        // 回合结束
}
```

### 3.1 阶段流转

```
SelectUnit → MoveUnit → ActionMenu → SelectTarget → ExecuteAction → SelectUnit
                     ↘ WaitAction → SelectUnit
                     ↘ ActionMenu → WaitAction
SelectUnit → TurnEnd → SelectUnit（新回合）
```

### 3.2 GameSet — 系统集合

```rust
pub enum GameSet {
    Camera,  // 相机初始化
    Map,     // 地图生成
    Unit,    // 单位生成
    Ui,      // UI 生成
}
```

**执行顺序**：Camera → Map → Unit → Ui（OnEnter(InGame) 时链式执行）

---

## 4. TurnOrder — 行动队列

```rust
#[derive(Resource)]
pub struct TurnOrder {
    pub queue: Vec<Entity>,       // 按 Initiative 降序
    pub current_index: usize,     // 当前行动索引
    pub turn_number: u32,         // 当前回合号
}
```

### 4.1 行动队列构建

```rust
TurnOrder::build(units: &[(Entity, f32)]) -> Vec<Entity>
```

- 按 Initiative 降序排列
- 相同 Initiative 保持原始顺序（稳定排序）
- 空队列返回空 Vec

### 4.2 核心操作

| 方法 | 说明 |
|------|------|
| `current_unit()` | 获取当前行动单位 |
| `advance()` | 前进到下一个单位，None 表示队列耗尽 |
| `current_faction(units)` | 当前行动单位的阵营 |

### 4.3 行动流程

```
1. current_unit() → 当前行动单位
2. 执行行动（移动/攻击/待机）
3. advance() → 下一个单位
4. advance() 返回 None → 队列耗尽 → TurnEnd
```

---

## 5. TurnState — 回合状态

```rust
#[derive(Resource)]
pub struct TurnState {
    pub current_faction: Faction,
    pub turn_number: u32,
}
```

默认值：`Faction::Player`, `turn_number = 1`

---

## 6. 辅助资源

### 6.1 AiTimer

```rust
#[derive(Resource)]
pub struct AiTimer {
    pub timer: Timer,  // 0.8秒，Once 模式
}
```

AI 行动延迟计时器，防止 AI 瞬间完成所有行动。

### 6.2 NeedsResolve

```rust
#[derive(Resource)]
pub struct NeedsResolve(pub bool);
```

标记是否需要结算持续效果（Buff tick 等），防止 SelectUnit 多次进入时重复结算。

---

## 7. 消息

### 7.1 TurnStarted

```rust
#[derive(Message)]
pub struct TurnStarted {
    pub turn: u32,
}
```

新回合开始时发送，回合结束重建队列后也发送。

### 7.2 TurnEnded

```rust
#[derive(Message)]
pub struct TurnEnded {
    pub turn: u32,
}
```

当前回合结束时发送。

### 7.3 ForceEndTurn

```rust
#[derive(Message)]
pub struct ForceEndTurn;
```

玩家按 E 强制结束回合时发送，队列自然耗尽后进入 TurnEnd。

---

## 8. 回合生命周期

### 8.1 游戏开始（OnEnter(InGame)）

```
1. init_turn_order（在 GameSet::Unit 之后）
2. 发送 TurnStarted { turn: 1 }
3. 重建行动队列（所有存活单位按 Initiative 降序）
4. 更新 current_faction
5. 重置 AI 计时器
```

### 8.2 回合结束（OnEnter(TurnEnd)）

```
1. 发送 TurnEnded { turn: old_turn }
2. 消费 ForceEndTurn 消息
3. turn_number += 1
4. 重置所有单位 acted = false
5. 设置 needs_resolve = true
6. 重建行动队列（所有存活单位按 Initiative 降序）
7. 更新 current_faction
8. 重置 AI 计时器
9. 发送 TurnStarted { turn: new_turn }
10. 切换到 TurnPhase::SelectUnit
```

### 8.3 队列耗尽

```
advance() 返回 None → 队列耗尽 → 切换到 TurnPhase::TurnEnd
```

---

## 9. 关键约束

1. **行动顺序由 Initiative 决定**：降序排列，不区分阵营
2. **队列耗尽自动进入 TurnEnd**：advance() 返回 None 时触发
3. **回合结束重置所有单位 acted**：确保新回合所有单位可行动
4. **NeedsResolve 防止重复结算**：只在回合结束时设置一次
5. **ForceEndTurn 消费即丢弃**：不需要额外操作，队列自然耗尽
6. **GameSet 链式执行**：Camera → Map → Unit → Ui，保证初始化顺序
7. **init_turn_order 在 Unit 之后**：确保单位已生成
8. **AI 计时器 0.8 秒**：Once 模式，每回合重置
9. **TurnPhase 是 SubState**：仅在 InGame 时激活
10. **回合结束总是回到 SelectUnit**：不跳过任何阶段
