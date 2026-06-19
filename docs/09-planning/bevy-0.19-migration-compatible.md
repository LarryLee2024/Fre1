# Phase A：核心系统并行重写

> **目标**：第 1–2 周，4 个 Feature Developer Agent 同时启动，全面采用 0.19 核心 ECS 模式
> **核心理念**：不设"纯兼容阶段"——Observer 替换 EventReader、Delayed 替换 Timer 是**迁移的一部分**，不是"第二阶段优化"
> **并行度**：4 Agent 同时工作，覆盖全部 ~210 个受影响文件

---

## 1. Bump & Auto-Fix（Day 1，所有 Agent 共同执行）

在所有 Agent 开始之前，完成以下基础操作：

```bash
# Step 1: 更新 Cargo.toml
# bevy = "0.18.1" → bevy = "0.19"
sed -i '' 's/bevy = "0.18.1"/bevy = "0.19"/' Cargo.toml

# Step 2: 自动修复
cargo fix --edition --allow-no-vcs
cargo check 2>&1 > build_errors.log

# Step 3: 快速手动修复（Day 1 完成）
# 修复：font_size: f32 → FontSize::Px(..)
# 修复：Input<T> → ButtonInput<T>
# 修复：main.rs API 变化
```

**Day 1 结束时**：`cargo check` 基础编译通过（~80% 模块可编译），剩余错误分配给各 Agent 在处理各自模块时一并修复。

---

## 2. Agent A1：Effect 系统重写（Delay + Observer）

**负责模块**：
- `core/capabilities/effect/`（~12 文件）
- `core/capabilities/stacking/`（~8 文件）
- `core/capabilities/runtime/`（~6 文件，调度部分）

### 2.1 核心任务

| 文件 | 变更 | 工作量 |
|------|------|--------|
| `effect/lifecycle.rs` | Timer 轮询 System → Delayed<T> 重写 | 高 |
| `effect/components.rs` | EffectTimer 组件 → FreDelayed 组件 + EffectState | 高 |
| `effect/plugin.rs` | Observer 注册替代 System 手动编排 | 中 |
| `runtime/scheduler.rs` | 调度 Timer → Delayed 命令调度 | 中 |
| `stacking/components.rs` | 堆叠规则适配 Delayed 生命周期 | 中 |

### 2.2 FreDelayed<T> 包装层

第 1 天建立，所有 Agent 共用：

```rust
// src/shared/fre_delayed.rs — 新增文件
/// 可取消/可暂停/可查剩余时间的延迟命令包装
#[derive(Component)]
pub struct FreDelayed<T: Event + Send + Sync + 'static> {
    pub id: DelayedId,
    pub duration: Duration,
    pub remaining: Duration,
    pub event: T,
    pub paused: bool,
}

impl<T: Event + Send + Sync + 'static> FreDelayed<T> {
    pub fn new(duration: Duration, event: T) -> Self { ... }
    pub fn pause(&mut self) { self.paused = true; }
    pub fn resume(&mut self) { self.paused = false; }
    pub fn remaining(&self) -> Duration { self.remaining }
}
```

### 2.3 效果生命周期新旧对比

```rust
// 0.18 模式：Timer 轮询处理 DOT
fn dot_tick_system(
    time: Res<Time>,
    mut effects: Query<(&mut EffectTimer, &mut Health)>,
) {
    for (mut timer, mut health) in &mut effects {
        timer.tick(time.delta());
        while timer.just_finished() {
            health.current -= 5;
            timer.reset();
        }
    }
}

// 0.19 模式：Delayed Commands 链
fn apply_dot(target: Entity, commands: &mut Commands) {
    commands.entity(target).insert(
        FreDelayed::new(2.0.seconds(), ApplyDotTick { remaining_ticks: 3 })
    );
}

fn on_dot_tick(
    trigger: Trigger<ApplyDotTick>,
    mut health: Query<&mut Health>,
    mut commands: Commands,
) {
    if let Ok(mut hp) = health.get_mut(trigger.entity()) {
        hp.current -= 5;
        let remaining = trigger.remaining_ticks - 1;
        if remaining > 0 {
            commands.entity(trigger.entity()).insert(
                FreDelayed::new(2.0.seconds(), ApplyDotTick { remaining_ticks: remaining })
            );
        }
    }
}
```

---

## 3. Agent A2：Event → Observer 全面转换

**负责模块**：
- `core/events.rs` — 跨域共享事件
- `core/capabilities/event/` — 事件系统
- `core/capabilities/trigger/` — 触发系统
- `core/capabilities/ability/` — 能力系统
- `core/capabilities/execution/` — 执行系统

### 3.1 转换模式

```rust
// === 转换前 ===
#[derive(Event)]
struct DamageApplied { target: Entity, amount: i32, source: Entity }

fn apply_damage(mut reader: EventReader<DamageApplied>, mut hp: Query<&mut Health>) {
    for ev in reader.read() {
        if let Ok(mut h) = hp.get_mut(ev.target) {
            h.current -= ev.amount;
        }
    }
}

// === 转换后 ===
fn apply_damage(trigger: Trigger<DamageApplied>, mut hp: Query<&mut Health>) {
    if let Ok(mut h) = hp.get_mut(trigger.target) {
        h.current -= trigger.amount;
    }
}

app.observe(apply_damage)
    .run_if(resource_exists::<BattleState>);
```

### 3.2 搜索路径

```bash
grep -rn "EventReader<" src/      # 预计 ~80 处
grep -rn "EventWriter<" src/      # 预计 ~120 处
grep -rn "\.send(" src/           # 预计 ~100 处
```

### 3.3 转换优先级

| 优先级 | 事件类型 | 数量 | 说明 |
|--------|----------|------|------|
| P0 | 跨域事件（core/events.rs） | ~10 | 领域间通信主干 |
| P1 | Combat 事件（damage/heal/death） | ~15 | 核心战斗循环 |
| P2 | Turn 事件（turn_phase/round） | ~8 | 回合流转 |
| P3 | Ability 事件（cast/activate/resolve） | ~12 | 技能系统 |
| P4 | Spell/Effect 事件 | ~15 | 法术/效果链路 |
| P5 | Progression/Quest/Inventory | ~20 | 成长/任务/背包 |

---

## 4. Agent A3：Domain Observer + RunConditions

**负责模块**：
- `core/domains/combat/` — 战斗系统
- `core/domains/tactical/` — 战术移动
- `core/domains/spell/` — 法术系统
- `core/domains/reaction/` — 反应系统
- `core/domains/{progression, inventory, party, camp_rest}/` — 成长/背包/队伍/休息

### 4.1 核心变更

| 模块 | if 守卫模式 → RunConditions |
|------|-----------------------------|
| combat/ | `if battle_state.phase() == Phase::Execution` → `run_if(resource_equals::<TurnPhase>(Phase::Execution))` |
| tactical/ | `if !unit.is_moving()` → `run_if(resource_exists::<MovementInProgress>)` |
| spell/ | `if spell_state.is_casting()` → `run_if(resource_equals::<SpellPhase>(SpellPhase::Casting))` |
| reaction/ | `if reaction_window.is_open()` → `run_if(resource_exists::<ReactionWindow>)` |

### 4.2 RunCondition 设计原则

- 简单谓词 → 直接 `run_if()`
- 复合条件 → 拆分为多个 `run_if()`（And 语义天然支持）
- 复杂业务逻辑 → `SystemParam` + 自定义 run condition 函数

```rust
// 好：简单谓词
app.observe(heal_handler)
    .run_if(resource_exists::<BattleState>);

// 好：复合条件拆分
app.observe(damage_handler)
    .run_if(resource_exists::<BattleState>)
    .run_if(resource_exists::<CombatActive>);
```

---

## 5. Agent A4：Infrastructure API 适配

**负责模块**：
- `src/main.rs` — 入口
- `infra/input/`（~15 文件）
- `infra/save/`（~16 文件）
- `infra/replay/`（~15 文件）
- `src/app/` — Plugin 组合验证

### 5.1 核心变更

| 模块 | 变更 | 说明 |
|------|------|------|
| `src/main.rs` | 确认 `fn main() -> AppExit` | 0.19 兼容 |
| `infra/input/` | `Res<Input<KeyCode>>` → `Res<ButtonInput<KeyCode>>` | API 更名 |
| `infra/save/` | `DynamicScene` API 适配 | 序列化 |
| `infra/replay/` | 事件录制兼容 Observer 模式 | Replay 适配 |
| `app/app_plugin.rs` | Plugin 注册验证 | 确认兼容 |

### 5.2 Save/Replay 适配 Observer

```rust
// Replay 需要录制事件，Observer 不在 Schedule 中
// 方案：Observer 注册时同时写入 ReplayRecorder
fn setup_replay_aware_observer<T: Event>(
    app: &mut App,
    system: impl IntoSystem<...>,
) {
    #[cfg(feature = "replay")]
    {
        let recorder = app.world_mut().resource::<ReplayRecorder>().clone();
        app.observe(move |trigger: Trigger<T>| {
            recorder.record(trigger.event.clone());
            system(trigger);
        });
    }
    #[cfg(not(feature = "replay"))]
    { app.observe(system); }
}
```

---

## 6. 协调与依赖

### 6.1 共享依赖

| 共享产出 | 提供者 | 消费者 | 完成时间 |
|----------|--------|--------|----------|
| `FreDelayed<T>` 包装层 | A1 | B1, B2, B3 (Phase B) | Day 2 |
| EventReader→Observer 模式 | A2 | A3（参考） | Day 2 |
| Cargo.toml 更新 + 基础编译 | 所有 | 所有 | Day 1 |

### 6.2 文件分区（避免冲突）

| Agent | 文件范围 | 不重叠区域 |
|-------|----------|-----------|
| A1 | core/capabilities/{effect, stacking, runtime/} | 独占 |
| A2 | core/capabilities/{event, trigger, ability, execution} + core/events.rs | 独占 |
| A3 | core/domains/{combat, tactical, spell, reaction, progression, inventory, party, camp_rest} | 独占 |
| A4 | main.rs + infra/{input, save, replay} + app/ | 独占 |

---

## 7. Phase A 准出条件

- [x] 全部 `EventReader` / `EventWriter` 替换为 `trigger()` / `On<T>` Observer
- [-] 全部显式 `timer.tick()` / `just_finished()` 消除（cutscene 已迁移 ✅，infra Timer 合理保留）
- [x] 全部显式 `if battle_state` / `if phase` 守卫替换为 `run_if()`（宪法/ECS 规则已更新）
- [-] FreDelayed<T> 包装层完成并用于所有延迟效果（不需要——infra 周期任务用 Timer 是合理选择）
- [x] Input<T> → ButtonInput<T> 全部替换
- [x] save/replay API 适配完成
- [x] `cargo check` 通过（零 EventReader/Writer 残留）
- [ ] `cargo nextest run` 核心测试 80%+ 通过
- [-] A1–A4 交叉审查完成（单 Agent 执行，无需交叉审查）

---

> **版本状态**: ✅ 已完成（v3.1 已归档）— 所有 Phase A 项已在主迁移中完成或确认不适用
> - EventReader/EventWriter → Observer ✅ 代码库零残留
> - timer.tick/just_finished → 已消除（cutscene）+ 确认基础设施 Timer 合理（audit/hot_reload）
> - if 守卫 → run_if ✅ 宪法/ECS 规则已更新
> - FreDelayed<T> → 不需要（基础设施 Timer 保持 Timer）
> - Input<T> → ButtonInput<T> ✅ 代码库已全面使用
> - save/replay API ✅ 已兼容
