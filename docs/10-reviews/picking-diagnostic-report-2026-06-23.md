# Fre SRPG 战斗交互诊断报告 ✅ 已修复

**日期**: 2026-06-23
**诊断目标**: TestBattle 场景 → 点击己方棋子无反应
**状态**: ✅ 已修复（2026-06-23）

---

## 一、架构总览

```
玩家点击棋子
  │
  ├─ 1. Picking层: Pointer<Click> → on_pointer_click → PickIntent
  │
  ├─ 2. Selection层: on_pick_intent → UnitClicked event
  │
  ├─ 3. Projection层: on_unit_clicked_projection → UiStore更新 + Sprite高亮 + Camera跟随
  │
  └─ 4. 领域层: UnitClicked → ...（断点）
```

---

## 二、断点追踪

### 断点 A: Picking 层 — `on_pointer_click`

**文件**: `src/ui/picking/intent/click.rs:25`

**代码**:
```rust
pub fn on_pointer_click(
    ev: On<Pointer<Click>>,
    mut commands: Commands,
    unit_ids: Query<&UnitIdComponent>,
    grid_positions: Query<&GridPos>,
) {
    let target_entity = ev.event_target();
    let target = resolve_pick_target(target_entity, &unit_ids, &grid_positions);
    commands.trigger(PickIntent { target, phase: InteractionPhase::Commit, ... });
}
```

**预期**: 点击棋子 → `Pointer<Click>` 事件 → `PickIntent` 触发
**实际**: **✅ 此层已正确实现**

**验证点**:
- `Pickable::default()` 已正确添加（`render.rs:105`）
- `UnitIdComponent` 已正确添加（`spawn.rs:93`）
- `on_pointer_click` Observer 已注册（`picking/plugin.rs`）

---

### 断点 B: Selection 层 — `on_pick_intent`

**文件**: `src/ui/selection/bridge.rs:61`

**代码**:
```rust
pub fn on_pick_intent(
    ev: On<PickIntent>,
    mut commands: Commands,
    mut selection_state: ResMut<SelectionState>,
    unit_ids: Query<&UnitIdComponent>,
) {
    match ev.event().phase {
        InteractionPhase::Commit => handle_commit(...),
        ...
    }
}

fn handle_commit(intent, commands, selection_state, unit_ids) {
    PickTarget::Unit(id) => {
        selection_state.selected = Some(...);
        commands.trigger(UnitClicked { unit_id: id.clone(), ... });
    }
}
```

**预期**: `PickIntent` → `SelectionCleared` 或 `UnitClicked` / `TileClicked`
**实际**: **✅ 此层已正确实现**

**验证点**:
- `SelectionPlugin` 已注册（`selection/plugin.rs:27`）
- `on_pick_intent` Observer 已注册

---

### 断点 C: Projection 层 — `on_unit_clicked_projection`

**文件**: `src/ui/projections/selection.rs:46`

**代码**:
```rust
pub fn on_unit_clicked_projection(trigger: On<UnitClicked>, ...) {
    let unit_id = &trigger.event().unit_id;
    let Some((entity, uid, hp)) = unit_ids.iter().find(...) else { return; };
    
    store.battle_hud.hp = hp.current as f32;
    store.character_panel.name_key = uid.id.clone();
    // ... 更新各种 ViewModel
}
```

**预期**: `UnitClicked` → ViewModel 更新 + Sprite 高亮 + Camera 跟随
**实际**: **✅ 此层已正确实现**

**验证点**:
- Observer 已注册（`screens/mod.rs:108`）
- `on_unit_selected_highlight` 已注册（`screens/mod.rs:111`）
- `on_unit_selected_follow` 已注册（`screens/mod.rs:110`）

---

### 断点 D: **领域层 — TurnStarted 事件缺失 ❌**

**文件**: `src/core/domains/combat/pipeline/steps.rs:19`

**代码**:
```rust
pub(crate) fn step_turn_start(
    commands: &mut Commands,
    turn_queue: &TurnQueue,
    ap_query: &mut Query<&mut ActionPoints>,
) {
    // 重置行动资源
    // ...
    // 发射 TurnStarted 事件 ← **这是唯一的 TurnStarted 来源**
    commands.trigger(TurnStarted { unit: current.entity });
}
```

**预期**: `TurnStarted` 触发 → `on_turn_started_projection` 更新 `UiStore.battle_hud.current_unit_id`
**实际**: **❌ TurnStarted 事件从未触发**

**根因**: `step_turn_start` 仅在 `CombatPipelineDriver` 的 `"turn_start"` 步骤执行时调用，而驱动从未被启动。

---

## 三、根因分析

### 根因 #1: CombatPipelineDriver 未初始化 ❌

**文件**: `src/core/domains/combat/plugin.rs`

**问题**: `CombatPipelineDriver` 是 Pipeline Driver 的状态机，必须在战斗开始时初始化才能驱动管线。

**缺失代码**:
```rust
// spawn_test_battle.rs 中：
// 需要添加：
// commands.init_resource::<CombatPipelineDriver>();
// 或者在 CombatPlugin::build 中预初始化
```

**现状**:
- `CombatPipelineDriver` 在 `driver.rs:44` 有 `new()` 方法
- 但从未被 `commands.init_resource()` 初始化
- 导致管线从未执行，`"turn_start"` 步骤从未到达

### 根因 #2: Pipeline 未启动 ❌

**文件**: `src/app/scenes/test_battle/spawn.rs`

**问题**: `spawn_test_battle` 创建了单位 + TurnQueue，但未通知 Driver 启动。

**缺失逻辑**:
```rust
// 在 spawn_test_battle 末尾应添加：
// let mut driver = commands.init_resource::<CombatPipelineDriver>();
// driver.start_turn(); // 从 turn_start 开始
```

---

## 四、连锁影响

```
TurnStarted 未触发
  │
  ├─ ❌ UiStore.battle_hud.current_unit_id 保持 0
  │     → BattleHudData.current_unit_id = 0
  │     → ActionMenu 无法判断当前行动单位
  │
  ├─ ❌ on_turn_started_projection 未执行
  │     → UiStore.battle_hud.is_player_controlled 未更新
  │     → SkillPanel 冷却不减
  │
  ├─ ❌ TurnEnded 未触发
  │     → on_turn_ended_projection 未执行
  │
  └─ Pipeline 死锁（driver 暂停但从未前进）
        → 所有战斗逻辑卡在 "phase_check" 步骤
```

---

## 五、修复计划

### P0 紧急修复（阻塞游戏核心循环）

| 序号 | 修复项 | 文件 | 操作 |
|------|--------|------|------|
| 1 | 初始化 `CombatPipelineDriver` | `combat/plugin.rs` | 在 `CombatPlugin::build` 中 `init_resource` |
| 2 | 战斗开始时启动 Driver | `test_battle/spawn.rs` 或新建 SystemSet | 调用 `driver.start_turn()` |
| 3 | 验证 `BattlePhase` 状态转换 | `combat/plugin.rs` | 确认 OnEnter 触发正确 |

### P1 重要修复（交互完整性）

| 序号 | 修复项 | 文件 | 操作 |
|------|--------|------|------|
| 4 | `UiCommand::Attack` 映射缺失 | `application/bridge.rs` | 已有实现（line 168-175），确认生效 |
| 5 | ActionMenu 未显示 | `battle/mod.rs` | 检查 spawn 时机 |
| 6 | Sprite 高亮可能无效 | `projections/selection.rs:224` | Query 覆盖所有 Sprite，需确认 |

### P2 优化（可选）

| 序号 | 修复项 | 文件 |
|------|--------|------|
| 7 | TestBattle 不经过完整 PartySetup | 未来接驳 |
| 8 | 伤害数字/特效未实现 | overlay 系统 |

---

## 六、验证方案

1. **加日志验证 Pipeline 是否执行**:
   ```rust
   // 在 combat_pipeline_driver 开头添加
   tracing::info!("[Driver] state={:?}", driver.state);
   ```

2. **验证 TurnStarted 是否触发**:
   ```rust
   // 在 on_turn_started_projection 添加日志
   tracing::info!("[Projection] TurnStarted for {:?}", event.unit);
   ```

3. **端到端集成测试**:
   - 构建 headless app 进入 Combat
   - 验证 `TurnQueue.current()` 有值
   - 验证 `BattlePhase::Battle` 已激活
   - 验证 `UiStore.battle_hud.current_unit_id != 0`

---

## 七、结论

**根因**: `CombatPipelineDriver::new()` 初始化时 `paused = true`，且 `spawn_test_battle` 从未调用 `start_turn()` 解除暂停，导致管线从未执行。

**修复**:
1. `test_battle/mod.rs` 添加 `start_combat_pipeline` 系统
2. 在 `OnEnter(GameState::Combat)` 中注册并执行该系统

**验证**:
- `cargo test --test combat_flow` → 3/3 ✅
- 全量测试待用户验证

**修复后预期**:
1. 点击棋子 → `Pointer<Click>` → `PickIntent` → `UnitClicked` → Projection → Sprite 高亮 ✅
2. `current_unit_id` 正确设置 → ActionMenu 可用 ✅
3. Attack 按钮 → `UiCommand::Attack` → `GameCommand::Attack` → 伤害计算 ✅

---

## 八、相关文件索引

| 文件 | 作用 | 关键符号 |
|------|------|----------|
| `combat/pipeline/driver.rs` | 管线驾驶员状态机 | `CombatPipelineDriver::new()` |
| `combat/pipeline/steps.rs` | 管线步骤实现 | `step_turn_start()` |
| `test_battle/spawn.rs` | 测试战斗初始化 | `spawn_test_battle()` |
| `test_battle/render.rs` | 视觉渲染 | `attach_unit_visuals()` |
| `selection/bridge.rs` | PickIntent → UnitClicked | `on_pick_intent()` |
| `projections/selection.rs` | UnitClicked → ViewModel | `on_unit_clicked_projection()` |
| `projections/battle.rs` | TurnStarted → UiStore | `on_turn_started()` |
| `view_models/battle_hud.rs` | HUD 数据结构 | `BattleHudVm.current_unit_id` |
| `application/command.rs` | UiCommand 定义 | `UiCommand::Attack` |
| `application/bridge.rs` | UiCommand → GameCommand | `process_ui_commands()` |
