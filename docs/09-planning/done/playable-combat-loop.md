---
id: 09-planning.playable-combat-loop
title: Playable Combat Loop — 可玩的战斗循环
status: completed
completed: 2026-06-23
owner: architect
created: 2026-06-23
tags:
  - combat
  - game-flow
  - ai
  - screens
  - integration-test
---

# Playable Combat Loop

闭合完整的游戏循环：MainMenu → PartySetup → Combat → Result → MainMenu。

## 背景

当前状态：

```
MainMenu✅ → PartySetup✅ → Combat✅ → Result❌ 空
                                          → GameOver❌ 空
                                          → MainMenu❌ 未接线
战斗中：玩家行动 ✅ | 敌方行动 ❌ 死锁 | 胜负判定 ❌ 无状态转换 | 结算 ❌ 无屏幕
```

## 实施步骤

### Step 1: 简单敌方 AI（自动跳过敌方回合）

**目标**：敌方单位在轮到其行动时自动触发 `UnitActionComplete`，不阻塞管线。

**改动文件**：
- `src/core/domains/combat/pipeline/driver.rs`

**实现方案**：
在 `combat_pipeline_driver` 的 `"unit_action"` 分支中，检查当前单位是否为敌方（通过 `TurnQueue.current().team_id` 与 `"Player"` 对比）：
- 若为敌方 → `commands.trigger(UnitActionComplete { unit })` + 不暂停（`driver.paused = false`）
- 若为玩家 → 保持现有暂停行为

```rust
"unit_action" => {
    step_unit_action(&mut commands, &turn_queue);
    let should_pause = turn_queue.current()
        .map(|entry| entry.team_id.as_str() == "Player")
        .unwrap_or(false);
    if !should_pause {
        // 敌方单位：自动结束回合
        if let Some(current) = turn_queue.current() {
            commands.trigger(UnitActionComplete { unit: current.entity });
        }
    }
    driver.paused = should_pause;
}
```

### Step 2: 战斗结束 → Result/GameOver 状态转换

**目标**：`BattlePhase::Victory` → `GameState::Result`，`BattlePhase::Defeat` → `GameState::GameOver`。

**改动文件**：
- 新建 `src/app/scenes/battle_end.rs` — Observer 监听 `BattleEnded` 事件，通过 `StateTransitionQueue` 转换状态

**实现方案**：
```rust
pub fn on_battle_ended_transition(
    trigger: On<BattleEnded>,
    mut queue: ResMut<StateTransitionQueue>,
) {
    let state = if trigger.event().victory {
        GameState::Result
    } else {
        GameState::GameOver
    };
    queue.push(TransitionRequest::Change(state));
}
```
- 在 `scenes/plugin.rs` 注册：`app.add_observer(on_battle_ended_transition);`

### Step 3: ResultScreen（胜利/失败结果展示）

**目标**：战斗结束后显示结果屏幕，含"返回主菜单"按钮。

**改动文件**：
- 新建 `src/app/scenes/result/` — 含 `mod.rs`，遵循 PartySetup 模式

**实现方案**：
- `spawn_result_screen`：全屏面板 + "Victory!" / "Defeat!" 标题文本 + "Back to Main Menu" 按钮
- `despawn_result_screen`：OnExit 清理
- 按钮点击 → `TransitionRequest::Change(MainMenu)`
- 注册到 `scenes/plugin.rs`：`register_scene(GameState::Result, spawn_result_screen, despawn_result_screen)`

### Step 4: GameOverScreen（游戏结束展示）

**目标**：失败时显示游戏结束画面。

**改动文件**：
- 新建 `src/app/scenes/game_over/` — 与 Result 类似但更简约

**实现方案**：
- `spawn_game_over_screen`：全屏面板 + "Game Over" 标题 + "Main Menu" 按钮
- `despawn_game_over_screen`：OnExit 清理
- 注册到 `scenes/plugin.rs`：`register_scene(GameState::GameOver, spawn_game_over_screen, despawn_game_over_screen)`

### Step 5: 集成测试

**目标**：验证完整战斗循环的端到端流程。

**改动文件**：
- 新建 `tests/combat_flow.rs`

**测试场景**：
1. 玩家攻击敌方 → 造成伤害
2. 敌方自动结束回合 → 切回玩家
3. 玩家攻击同一敌方 → 击杀
4. 战斗结束（敌方全灭=Victory）→ 转换到 Result 状态
5. 验证 GameState 转换链

## 依赖关系

```
Step 1 (Enemy AI) ─┐
                    ├── 无依赖
Step 2 (State Transition) ─── 依赖 Step 1（需要战斗可结束）
Step 3 (ResultScreen) ─── 依赖 Step 2
Step 4 (GameOverScreen) ─── 依赖 Step 2
Step 5 (Integration Test) ─── 依赖 Steps 1-4
```

## 优先级

Step 1 + Step 2 是核心依赖，必须先做。
Step 3 + Step 4 可以并行。
Step 5 最后做。

## 影响范围

| 模块 | 改动方式 |
|------|----------|
| `combat/pipeline/driver.rs` | 修改 UnitAction 分支（~5行） |
| `app/scenes/plugin.rs` | 添加 observer + 2 个 scene 注册 |
| `app/scenes/battle_end.rs` | 新建（~20行） |
| `app/scenes/result/mod.rs` | 新建（~80行） |
| `app/scenes/game_over/mod.rs` | 新建（~60行） |
| `tests/combat_flow.rs` | 新建（~150行） |
