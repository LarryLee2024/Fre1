# System 编写铁律

Version: 1.0
Status: Proposed
Source: `docs/其他/31遗漏.md` 第一节
Related: `docs/architecture.md` ECS 章节、`docs/domain/ecs_communication_rules.md`

---

## 概述

本文档定义 Bevy ECS 项目中 System 的粒度约束、参数边界、依赖方式、命名规范和禁止事项。

核心问题：Bevy 的并行调度能力完全依赖系统解耦，无规范的系统会快速退化成面条代码，性能和可维护性双输。

本规范确保每个 System 保持纯粹的逻辑单元，不存储状态、不跨边界调用、不破坏并行空间。

---

## 系统粒度规则

### 参数上限

🟥 **单系统 Query/Resource 参数上限为 8 个。超过必须拆分。**

```rust
// 🟥 禁止：参数超过 8 个
fn bad_system(
    units: Query<(&Unit, &Attributes, &ActiveBuffs, &SkillSlots, &GridPosition, &EquipmentSlots)>,
    buffs: Res<BuffRegistry>,
    skills: Res<SkillRegistry>,
    map: Res<GameMap>,
    turn: Res<TurnOrder>,
    phase: Res<State<TurnPhase>>,
) { ... }

// ✅ 正确：拆分为多个职责清晰的系统
fn check_unit_buffs(
    units: Query<(&Unit, &ActiveBuffs)>,
    buffs: Res<BuffRegistry>,
) { ... }

fn check_unit_skills(
    units: Query<(&Unit, &SkillSlots)>,
    skills: Res<SkillRegistry>,
) { ... }
```

### 拆分判断

```
System 是否需要拆分？
├─ 超过 8 个 Query/Resource 参数？→ 必须拆分
├─ 函数体超过 100 行？→ 考虑拆分
├─ 包含多个不相关的职责？→ 拆分
├─ 需要不同的 run_if 条件？→ 拆分
└─ 否则 → 保持单一 System
```

---

## 禁止跨系统函数调用

### 规则

🟥 **禁止系统间直接函数调用。所有跨系统通信必须走 Message 或组件状态。**

```rust
// 🟥 禁止：System A 直接调用 System B 的函数
fn system_a(/* ... */) {
    system_b_function(data);  // 直接调用，破坏并行调度
}

// ✅ 正确：通过 Message 通信
fn system_a(/* ... */) {
    // 处理完后发送消息
    writer.write(SomeMessage { data });
}

fn system_b(/* ... */) {
    // 消费消息
    for msg in reader.read() { ... }
}

// ✅ 正确：通过组件状态通信
fn system_a(/* ... */) {
    // 修改组件状态
    component.state = new_value;
}

fn system_b(/* ... */) {
    // 读取组件状态
    if component.state == expected_value { ... }
}
```

### 例外：同模块内直接调用

🟦 **同模块内的 System 可以直接调用内部辅助函数（非 System 函数）。**

```rust
// ✅ 允许：System 调用同模块的辅助函数
fn damage_system(/* ... */) {
    let result = calculate_damage(base, modifier);  // 辅助函数，不是 System
    apply_damage(entity, result);
}

// 🟥 禁止：System 调用另一个 System
fn damage_system(/* ... */) {
    apply_damage_system(/* ... */);  // 直接调用另一个 System
}
```

---

## 读写分离原则

### 规则

🟨 **读操作与写操作系统应分离，最大化 Bevy 调度器的并行空间。**

```rust
// ✅ 正确：读系统和写系统分离
// 读系统：只查询数据，不修改
fn read_unit_stats(
    query: Query<(&Unit, &Attributes)>,
) { ... }

// 写系统：只修改数据
fn apply_damage(
    mut query: Query<(&mut Attributes,)>,
    damage_events: MessageReader<DamageApplied>,
) { ... }

// ✅ 正确：通过 SystemSet 确保执行顺序
app.add_systems(Update, (
    read_unit_stats,         // 读系统
    apply_damage,            // 写系统
).chain())
```

### 常见模式

```
┌─────────────────────────────────────────────┐
│ 读系统（只查询）                              │
│ - 查询 Unit/Attributes/Buffs               │
│ - 计算派生值（不修改组件）                     │
│ - 更新 Resource（只读查询源）                 │
└─────────────────┬───────────────────────────┘
                  │ 通过组件状态或 Message 传递
                  ▼
┌─────────────────────────────────────────────┐
│ 写系统（只修改）                              │
│ - 修改 Attributes                           │
│ - 添加/移除 Component                        │
│ - 发送 Message                              │
└─────────────────────────────────────────────┘
```

---

## 系统命名规范

### 命名模式：`[schedule]_[verb]_[object]_system`

| 组成部分 | 说明 | 示例 |
|----------|------|------|
| schedule | 执行阶段（可选） | `update_`, `startup_` |
| verb | 动作（动词） | `apply_`, `check_`, `calculate_`, `spawn_`, `cleanup_` |
| object | 操作对象 | `buff_damage`, `unit_health`, `skill_cooldown` |
| suffix | 固定后缀 | `_system` |

### 命名示例

```rust
// ✅ 正确命名
fn update_apply_buff_damage_system(/* ... */) { ... }
fn check_unit_death_system(/* ... */) { ... }
fn calculate_movement_range_system(/* ... */) { ... }
fn spawn_unit_from_template_system(/* ... */) { ... }
fn cleanup_moving_units_system(/* ... */) { ... }
fn on_enter_select_unit_system(/* ... */) { ... }

// 🟥 禁止的命名
fn handle_stuff(/* ... */) { ... }     // 含糊不清
fn process(/* ... */) { ... }          // 没有对象
fn damage(/* ... */) { ... }           // 缺少后缀
fn do_damage_calculation(/* ... */) { ... }  // 过于冗长
```

---

## 系统排序

### 规则

🟩 **必须使用 SystemSets 和 `.chain()` / `.before()` / `.after()` 显式声明系统执行顺序。**

```rust
// ✅ 正确：使用 SystemSet 定义阶段
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BattleSet {
    Generate,
    Modify,
    Execute,
    Record,
}

// ✅ 正确：使用 .chain() 定义顺序
app.add_systems(Update, (
    generate_effects.in_set(BattleSet::Generate),
    modify_effects.in_set(BattleSet::Modify),
    execute_effects.in_set(BattleSet::Execute),
    record_battle_event.in_set(BattleSet::Record),
).chain().run_if(in_state(AppState::InGame)));

// ✅ 正确：使用 .before() / .after()
app.add_systems(Update, (
    update_ui.after(BattleSet::Execute),
    play_vfx.after(BattleSet::Execute),
).run_if(in_state(AppState::InGame)));
```

### 禁止的排序方式

```rust
// 🟥 禁止：依赖隐式顺序（Bevy 不保证默认顺序）
app.add_systems(Update, (system_a, system_b, system_c));

// 🟥 禁止：在 System 内部手动等待其他 System
fn system_a(/* ... */) {
    // 等待 system_b 完成（不可能，System 是并行的）
}
```

---

## Run Conditions

### 规则

🟨 **优先使用 `run_if(in_state(...))` 而非手动状态检查。**

```rust
// ✅ 正确：使用 run_if 条件
app.add_systems(Update, (
    select_unit_system.run_if(in_state(TurnPhase::SelectUnit)),
    move_unit_system.run_if(in_state(TurnPhase::MoveUnit)),
    execute_action_system.run_if(in_state(TurnPhase::ExecuteAction)),
));

// 🟥 禁止：在 System 内部手动检查状态
fn select_unit_system(
    phase: Res<State<TurnPhase>>,
) {
    if *phase.get() != TurnPhase::SelectUnit {
        return;  // 手动检查，效率低于 run_if
    }
    // ... 业务逻辑
}
```

### 常用 run_if 模式

```rust
// 状态条件
.run_if(in_state(AppState::InGame))
.run_if(in_state(TurnPhase::SelectUnit))

// 资源条件
.run_if(resource_exists::<BattleState>())

// 组合条件
.run_if(in_state(AppState::InGame).and_then(in_state(TurnPhase::ExecuteAction)))
```

---

## 单帧变更规则

### 规则

🟥 **一个 System 不得在同一帧内对同一 Entity 通过 Commands 多次变更。**

```rust
// 🟥 禁止：同一帧内对同一 Entity 多次 Commands 操作
fn bad_system(
    mut commands: Commands,
    query: Query<Entity, With<Unit>>,
) {
    for entity in &query {
        commands.entity(entity).insert(ComponentA);  // 第一次
        commands.entity(entity).insert(ComponentB);  // 第二次，可能冲突
    }
}

// ✅ 正确：合并为一次操作
fn good_system(
    mut commands: Commands,
    query: Query<Entity, With<Unit>>,
) {
    for entity in &query {
        commands.entity(entity().insert((ComponentA, ComponentB));  // 合并
    }
}
```

### 原因

Commands 是延迟执行的，多次写入同一 Entity 可能导致：
1. 组件覆盖顺序不确定
2. 性能下降（多次 World 变更）
3. Hook 触发顺序不可预测

---

## 系统中的业务逻辑边界

### 规则

🟥 **System 禁止包含属于领域模块的业务规则。System 只负责调度，业务规则放在领域模块中。**

```rust
// 🟥 禁止：System 包含领域逻辑
fn damage_system(
    mut query: Query<(&mut Attributes,)>,
) {
    for mut attrs in &mut query {
        // 伤害计算逻辑属于 core/battle/，不属于 System
        let base_damage = attrs.attack * 2;
        let defense = attrs.defense;
        let final_damage = (base_damage - defense).max(1);
        attrs.current_hp -= final_damage;
    }
}

// ✅ 正确：System 调用领域模块的函数
fn damage_system(
    mut query: Query<(&mut Attributes,)>,
    battle: Res<BattleService>,  // 领域服务
) {
    for mut attrs in &mut query {
        let damage = battle.calculate_damage(&attrs);
        battle.apply_damage(&mut attrs, damage);
    }
}
```

---

## 禁止事项总览

| 禁止项 | 理由 | 替代方案 |
|--------|------|----------|
| 🟥 单系统参数超过 8 个 | 职责混杂，编译时间长 | 按职责拆分为多个系统 |
| 🟥 系统间直接函数调用 | 破坏 Bevy 并行调度 | 使用 Message 或组件状态通信 |
| 🟥 在 System 内部手动检查状态 | 效率低，代码冗余 | 使用 `run_if(in_state(...))` |
| 🟥 读系统和写系统混在一起 | 缩小并行空间，性能下降 | 分离读写系统，使用 SystemSet 排序 |
| 🟥 系统命名不含 `_system` 后缀 | 降低可读性 | 遵循 `[schedule]_[verb]_[object]_system` |
| 🟥 依赖隐式执行顺序 | Bevy 不保证默认顺序 | 使用 SystemSet + `.chain()` / `.before()` / `.after()` |
| 🟥 同一帧内对同一 Entity 多次 Commands | 组件覆盖顺序不确定 | 合并为一次操作 |
| 🟥 System 包含领域逻辑 | 违反 Logic/Presentation 分离 | 业务规则放在领域模块中 |
| 🟥 手写 `is_xxx: bool` 状态检测 | Bevy 位掩码优化失效 | 使用 Marker Component + Query 过滤 |
| 🟥 在每帧系统中打印 Info/Debug 日志 | 性能下降，日志洪水 | 仅允许 Error 级别日志，调试使用 Inspector |

---

## 允许的模式

### 模式1：System + Message 通信

```rust
// 发送系统
fn check_death_system(
    query: Query<(Entity, &Health), (Changed<Health>, With<Unit>)>,
    mut writer: MessageWriter<CharacterDied>,
) {
    for (entity, health) in &query {
        if health.current <= 0 {
            writer.write(CharacterDied { entity });
        }
    }
}

// 消费系统
fn handle_death_system(
    mut reader: MessageReader<CharacterDied>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        commands.entity(msg.entity).insert(Dead);
    }
}
```

### 模式2：System + Resource 共享

```rust
// 写入 Resource
fn update_turn_order_system(
    mut turn_order: ResMut<TurnOrder>,
    query: Query<(&Unit, &Attributes), With<Unit>>,
) {
    // 计算并更新行动顺序
    turn_order.update(&query);
}

// 读取 Resource
fn select_next_unit_system(
    turn_order: Res<TurnOrder>,
) {
    let next = turn_order.current();
    // ...
}
```

### 模式3：SystemSet + 链式执行

```rust
// 定义 SystemSet
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum BattlePipeline {
    Input,
    Decision,
    Execute,
    Resolve,
}

// 注册系统
app.add_systems(Update, (
    handle_player_input.in_set(BattlePipeline::Input),
    ai_decision.in_set(BattlePipeline::Decision),
    execute_action.in_set(BattlePipeline::Execute),
    resolve_effects.in_set(BattlePipeline::Resolve),
).chain().run_if(in_state(AppState::InGame)));
```

---

## 交叉引用

| 文档 | 关系 |
|------|------|
| `docs/architecture.md` | ECS 章节（System、Resource、系统排序） |
| `docs/architecture/component_design_rules.md` | System 操作的 Component 设计规范 |
| `docs/domain/ecs_communication_rules.md` | 系统间通信方式（Message、Observer、Command） |
| `docs/architecture/plugin-design.md` | System 通过 Plugin 注册 |
| `docs/其他/31遗漏.md` | 本文档的原始需求来源（第 193-200 行） |
