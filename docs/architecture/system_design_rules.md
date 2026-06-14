# System 编写铁律

Version: 1.1
Status: Proposed
Source: `docs/其他/31遗漏.md` 第一节
Related: `docs/architecture.md` ECS 章节、`docs/domain/ecs_communication_rules.md`
> **宪法依据**：`docs/AI开发宪法完整版.md` v1.6 — 第2.1节ECS核心概念、第2.2节四级通信机制、第2.3节ECS执行模型、第5.0节函数设计宪法

---

## 概述

本文档定义 Bevy ECS 项目中 System 的粒度约束、参数边界、依赖方式、命名规范和禁止事项。

核心问题：Bevy 的并行调度能力完全依赖系统解耦，无规范的系统会快速退化成面条代码，性能和可维护性双输。

本规范确保每个 System 保持纯粹的逻辑单元，不存储状态、不跨边界调用、不破坏并行空间。

---

## 系统粒度规则

> **宪法依据**：〔宪法 2.1.2 数据与行为强制分离〕、〔宪法 5.0.1 单一职责〕

### 参数上限

🟥 **单系统 Query/Resource 参数上限为 8 个。超过必须拆分。** — 〔宪法 5.0.1 单一职责〕

> **优化来源**：`docs/其他/68.md` — 参数上限 8 个的编译时间膨胀解释

**为什么是 8 个？** Bevy 的 System 使用宏展开来生成参数提取代码。每个 Query/Resource 参数都会触发：
- 一次 `World::query` 或 `World::resource` 调用
- 一组生命周期绑定和借用检查
- 一份 `SystemParam` trait 实现

超过 8 个参数时，宏展开的编译时间呈指数级增长。实测数据：

| 参数数量 | 宏展开编译时间（增量） | 说明 |
|---------|----------------------|------|
| 1-4 个 | < 50ms | 正常 |
| 5-8 个 | 50-200ms | 可接受 |
| 9-12 个 | 200-500ms | 明显变慢 |
| 13+ 个 | 500ms-2s | 编译体验严重下降 |

超过 8 个参数的系统往往意味着职责过载，强制拆分是保持系统轻量的最佳手段。

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

> **宪法依据**：〔宪法 2.3.1 ECS是数据流，不是调用链〕、〔宪法 2.2.5 模块内部优先函数调用〕

### 规则

🟥 **禁止系统间直接函数调用。所有跨系统通信必须走 Message 或组件状态。** — 〔宪法 2.3.1 ECS是数据流，不是调用链〕

> **优化来源**：`docs/其他/68.md` — 禁止 System 间直接函数调用（用 Message/Event 或 Component 状态替代）

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

**为什么禁止直接调用？**
1. **并行安全**：直接函数调用让 Bevy 无法分析系统间依赖，两个本可并行的系统被迫串行
2. **职责混乱**：System A 调用 System B 的函数，意味着 A 需要了解 B 的实现细节
3. **测试困难**：无法单独测试 System A，因为它依赖 System B 的实现
4. **重构困难**：修改 System B 的签名会影响所有调用它的 System

**替代方案优先级**：
1. **Message（推荐）**：跨模块通信的首选，解耦最彻底
2. **组件状态**：同模块内简单状态传递
3. **Resource**：全局共享状态（如 TurnOrder）
4. **同模块辅助函数**：仅限非 System 的纯函数调用

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

> **优化来源**：`docs/其他/68.md` — 读写分离捍卫并行空间与 Bevy 调度器自动并行化
> **宪法依据**：〔宪法 11.7 读写分离原则（CQRS Lite）〕

### 规则

🟥 **读操作与写操作系统必须分离，最大化 Bevy 调度器的并行空间。** — 〔宪法 11.7.1 读路径无副作用〕

Bevy 的多线程调度器通过分析系统读写的 Resource/Component 自动决定并行度。如果两个系统只读同一资源，它们可以并行；如果一个读一个写，它们必须串行。

```rust
// ✅ 正确：读系统和写系统分离，Bevy 可自动并行化
// 读系统：只查询数据，不修改
fn read_unit_stats(
    query: Query<(&Unit, &Attributes)>,
) { ... }

// 写系统：只修改数据
fn apply_damage(
    mut query: Query<(&mut Attributes,)>,
    damage_events: MessageReader<DamageApplied>,
) { ... }

// ✅ Bevy 调度器分析：
// read_unit_stats: 读取 Unit, Attributes
// apply_damage: 写入 Attributes
// → 有写冲突 → 串行执行（通过 SystemSet 保证顺序）
```

```rust
// ✅ 更好的模式：读系统之间可以并行
fn read_unit_attack(query: Query<&Attributes>) { ... }   // 读 Attributes
fn read_unit_defense(query: Query<&Defense>) { ... }     // 读 Defense（不同 Component）
fn read_buff_status(query: Query<&ActiveBuffs>) { ... }  // 读 ActiveBuffs（不同 Component）

// Bevy 分析：三个系统读取不同 Component → 自动并行执行！
```

```rust
// 🟥 错误：读写混在同一系统，破坏并行空间
fn mixed_system(
    query: Query<(&mut Attributes, &Unit)>,  // 同时读写 Attributes
) {
    for (mut attrs, unit) in &mut query {
        attrs.hp -= 10;  // 写操作
        // 这个系统独占了 Attributes 的写锁
        // 所有其他需要读/写 Attributes 的系统都必须等待
    }
}
```

### 并行空间最大化策略

```
┌─────────────────────────────────────────────────────────────┐
│  策略                          │  效果                        │
├─────────────────────────────────────────────────────────────┤
│  读系统与写系统分离到不同 Set   │  读系统可并行，写系统串行     │
│  不同 Component 的读系统       │  Bevy 自动并行               │
│  使用 Changed<T> 过滤器        │  只处理变更的 Entity          │
│  避免单个 ResMut 阻塞整个 Set  │  拆分为多个细粒度 Resource    │
└─────────────────────────────────────────────────────────────┘
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

> **宪法依据**：〔宪法 5.0.2 函数命名规范〕函数名必须描述意图

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

> **宪法依据**：〔宪法 2.3.8 Schedule权责划分〕

### 规则

🟩 **必须使用 SystemSets 和 `.before()` / `.after()` 显式声明系统执行顺序。** — 〔宪法 2.3.8〕

> ⚠️ **反 `.chain()` 规则**（宪法 2.3.8 + app-bootstrap.md）：`.chain()` 强制系统串行执行，破坏 Bevy 多线程并行优势。效果管线（Generate→Modify→Execute）应使用自定义 Schedule（EffectPipelineSchedule），而非 `.chain()`。详见 `docs/architecture/app-bootstrap.md`。

```rust
// ✅ 正确：使用 SystemSet 定义阶段
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BattleSet {
    Generate,
    Modify,
    Execute,
    Record,
}

// ✅ 正确：使用 .before() / .after() 定义顺序（非 .chain()）
app.add_systems(Update, (
    generate_effects.in_set(BattleSet::Generate),
    modify_effects.in_set(BattleSet::Modify).after(BattleSet::Generate),
    execute_effects.in_set(BattleSet::Execute).after(BattleSet::Modify),
    record_battle_event.in_set(BattleSet::Record).after(BattleSet::Execute),
).run_if(in_state(AppState::InGame)));

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

> **宪法依据**：〔宪法 2.3.7 运行条件优先〕

### 规则

🟥 **必须使用 `run_if(in_state(...))` 而非手动状态检查。禁止在 System 内部手动 `if state == xxx`。** — 〔宪法 2.3.7〕

> **优化来源**：`docs/其他/68.md` — 强制 run_if 门控（禁止手动 `if state == xxx`）

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
        return;  // ❌ 手动检查，效率低于 run_if
    }
    // ... 业务逻辑
}
```

**为什么必须用 run_if？**

| 维度 | run_if | 手动 if |
|------|--------|---------|
| **性能** | Bevy 在图构建阶段直接裁剪，零运行时开销 | 每帧执行 System，Query 遍历后才 return |
| **可观测性** | 调度图中清晰可见哪些系统在哪些状态下运行 | 状态逻辑隐藏在 System 内部，难以追踪 |
| **编译优化** | Bevy 可以在编译时优化掉不需要的系统 | System 始终编译，增加二进制体积 |
| **调试** | bevy_mod_debugdump 可视化显示门控状态 | 无法通过工具发现状态检查逻辑 |

### 常用 run_if 模式

```rust
// 状态条件
.run_if(in_state(AppState::InGame))
.run_if(in_state(TurnPhase::SelectUnit))

// 资源条件
.run_if(resource_exists::<BattleState>())

// 组合条件
.run_if(in_state(AppState::InGame).and_then(in_state(TurnPhase::ExecuteAction)))

// 谓词条件（复杂逻辑）
.run_if(|world: &World| {
    let combat_state = world.get_resource::<CombatState>();
    combat_state.map(|s| s.is_active()).unwrap_or(false)
})
```

---

## 单帧变更规则

> **宪法依据**：〔宪法 2.1.2 数据与行为强制分离〕

### 规则

🟥 **一个 System 不得在同一帧内对同一 Entity 通过 Commands 多次变更。** — 〔宪法 2.1.2〕

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

> **优化来源**：`docs/其他/68.md` — 业务逻辑下沉领域层铁律
> **宪法依据**：〔宪法 1.1.4 逻辑与表现强制分离〕、〔宪法 1.4.1 核心领域与引擎解耦〕、〔宪法 1.4.2 领域无副作用〕

### 规则

🟥 **System 禁止包含属于领域模块的业务规则。System 只负责调度，业务规则放在领域模块中。** — 〔宪法 1.4.1 核心领域与引擎解耦〕

这是 DDD（领域驱动设计）与 ECS 的完美融合。System 是 ECS 的"调度层"，负责读取数据、调用领域函数、写回结果。核心业务逻辑（伤害计算、状态判定、属性修正）必须下沉到领域层，作为纯函数实现。

```rust
// 🟥 禁止：System 包含领域逻辑
fn damage_system(
    mut query: Query<(&mut Attributes,)>,
) {
    for mut attrs in &mut query {
        // ❌ 伤害计算逻辑属于 core/battle/，不属于 System
        let base_damage = attrs.attack * 2;
        let defense = attrs.defense;
        let final_damage = (base_damage - defense).max(1);
        attrs.current_hp -= final_damage;
    }
}

// ✅ 正确：System 只做调度，领域逻辑在纯函数中
fn damage_system(
    mut query: Query<(&mut Attributes,)>,
    damage_events: MessageReader<DamageApplied>,
) {
    for msg in damage_events.read() {
        if let Ok((mut attrs,)) = query.get_mut(msg.target) {
            // 调用领域层纯函数，不在 System 中硬编码逻辑
            let damage = battle::calculate_damage(
                msg.base_damage,
                &attrs,
                msg.modifiers,
            );
            battle::apply_damage(&mut attrs, damage);
        }
    }
}
```

### 为什么必须下沉

| 维度 | System 包含逻辑 | 逻辑下沉领域层 |
|------|----------------|----------------|
| **可测试性** | 需要启动 Bevy 运行时才能测试 | 纯函数，毫秒级单元测试 |
| **可复用性** | 逻辑绑死在 System 中，无法被其他模块调用 | 领域函数可被多个 System、测试、工具复用 |
| **可维护性** | 业务规则散落在多个 System 中 | 业务规则集中在领域模块，一处修改全局生效 |
| **编译时间** | System 宏展开时间长 | 领域函数无宏展开，编译快 |

### 领域层纯函数示例

```rust
// core/battle/damage.rs — 领域层，零 Bevy 依赖
pub fn calculate_damage(
    base_damage: i32,
    attacker_attrs: &Attributes,
    defender_attrs: &Attributes,
    modifiers: &[Modifier],
) -> DamageResult {
    let mut damage = base_damage as f32;

    // 应用修饰链（暴击、克制、地形等）
    for modifier in modifiers {
        damage = modifier.apply(damage);
    }

    // 护甲减免
    let defense = defender_attrs.defense as f32;
    let final_damage = (damage - defense).max(1.0) as i32;

    DamageResult { value: final_damage }
}

pub fn apply_damage(attrs: &mut Attributes, damage: i32) {
    attrs.current_hp = (attrs.current_hp - damage).max(0);
}

// ✅ 纯函数测试：零 Bevy 依赖，毫秒级
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_damage_respects_armor() {
        let attacker = Attributes { attack: 100, ..Default::default() };
        let defender = Attributes { defense: 30, ..Default::default() };
        let result = calculate_damage(100, &attacker, &defender, &[]);
        assert_eq!(result.value, 70);
    }
}
```

---

## 禁止事项总览

> **宪法依据**：〔宪法 2.1节ECS核心概念〕、〔宪法 5.0节函数设计宪法〕

| 禁止项 | 理由 | 替代方案 | 宪法条款 |
|--------|------|----------|----------|
| 🟥 单系统参数超过 8 个 | 职责混杂，编译时间长 | 按职责拆分为多个系统 | 5.0.1 |
| 🟥 系统间直接函数调用 | 破坏 Bevy 并行调度 | 使用 Message 或组件状态通信 | 2.3.1 |
| 🟥 在 System 内部手动检查状态 | 效率低，代码冗余 | 使用 `run_if(in_state(...))` | 2.3.7 |
| 🟥 读系统和写系统混在一起 | 缩小并行空间，性能下降 | 分离读写系统，使用 SystemSet 排序 | 11.7 |
| 🟥 系统命名不含 `_system` 后缀 | 降低可读性 | 遵循 `[schedule]_[verb]_[object]_system` | 5.0.2 |
| 🟥 依赖隐式执行顺序 | Bevy 不保证默认顺序 | 使用 SystemSet + `.before()` / `.after()` | 2.3.8 |
| 🟥 同一帧内对同一 Entity 多次 Commands | 组件覆盖顺序不确定 | 合并为一次操作 | 2.1.2 |
| 🟥 System 包含领域逻辑 | 违反 Logic/Presentation 分离 | 业务规则放在领域模块中 | 1.4.1 |
| 🟥 手写 `is_xxx: bool` 状态检测 | Bevy 位掩码优化失效 | 使用 Marker Component + Query 过滤 | 2.1.3 |
| 🟥 在每帧系统中打印 Info/Debug 日志 | 性能下降，日志洪水 | 仅允许 Error 级别日志，调试使用 Inspector | 13.1.2 |

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

> ⚠️ **注意**：效果管线（Generate→Modify→Execute）应使用自定义 Schedule（EffectPipelineSchedule），而非 `.chain()`。此处示例展示非管线场景的 SystemSet 排序。

```rust
// 定义 SystemSet
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum BattlePipeline {
    Input,
    Decision,
    Execute,
    Resolve,
}

// 注册系统 — 使用 .before() / .after()（非 .chain()）
app.add_systems(Update, (
    handle_player_input.in_set(BattlePipeline::Input),
    ai_decision.in_set(BattlePipeline::Decision).after(BattlePipeline::Input),
    execute_action.in_set(BattlePipeline::Execute).after(BattlePipeline::Decision),
    resolve_effects.in_set(BattlePipeline::Resolve).after(BattlePipeline::Execute),
).run_if(in_state(AppState::InGame)));
```

---

## 交叉引用

| 文档 | 关系 |
|------|------|
| `docs/AI开发宪法完整版.md` | 宪法第2.1节ECS核心概念、第2.2节四级通信机制、第2.3节ECS执行模型、第5.0节函数设计宪法 |
| `docs/architecture.md` | ECS 章节（System、Resource、系统排序） |
| `docs/architecture/component_design_rules.md` | System 操作的 Component 设计规范 |
| `docs/domain/ecs_communication_rules.md` | 系统间通信方式（Message、Observer、Command） |
| `docs/architecture/plugin-design.md` | System 通过 Plugin 注册 |
| `docs/architecture/app-bootstrap.md` | EffectPipelineSchedule、Schedule 权责划分 |
| `docs/其他/31遗漏.md` | 本文档的原始需求来源（第 193-200 行） |

---

## System 测试策略

> **优化来源**：`docs/其他/68.md` — System 测试策略（纯领域函数独立测试 + headless ECS 集成测试）

### 测试分层

| 测试类型 | 测试对象 | 依赖 | 执行速度 | 示例 |
|---------|---------|------|---------|------|
| **领域函数单元测试** | 纯业务逻辑函数 | 零 Bevy 依赖 | 毫秒级 | `calculate_damage()` 的各种输入输出 |
| **System 集成测试** | ECS 系统调度与交互 | Bevy MinimalPlugins | 秒级 | 验证 `damage_system` 正确触发 `CharacterDied` |
| **run_if 门控测试** | 状态门控逻辑 | Bevy MinimalPlugins | 秒级 | 验证 `TurnPhase::SelectUnit` 时只运行选择系统 |

### 领域函数独立测试（推荐，占 80%）

```rust
// core/battle/tests/damage_calculation.rs
// 纯 Rust，零 Bevy 依赖，毫秒级执行
#[test]
fn physical_damage_respects_armor() {
    let attacker = Attributes { attack: 100, ..Default::default() };
    let defender = Attributes { defense: 30, ..Default::default() };
    let result = calculate_damage(100, &attacker, &defender, &[]);
    assert_eq!(result.value, 70);
}

#[test]
fn damage_never_below_one() {
    let attacker = Attributes { attack: 1, ..Default::default() };
    let defender = Attributes { defense: 999, ..Default::default() };
    let result = calculate_damage(100, &attacker, &defender, &[]);
    assert!(result.value >= 1);
}
```

### Headless ECS 集成测试（占 20%）

```rust
// tests/integration/damage_system_test.rs
use bevy::prelude::*;

#[test]
fn damage_system_applies_damage_and_triggers_death() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(BattlePlugin);

    // 注入伤害事件
    app.world_mut().write_message(DamageApplied {
        source: UnitId::new(1),
        target: UnitId::new(2),
        damage: 999,
        is_critical: false,
    });

    // 运行一帧
    app.update();

    // 验证目标死亡
    let target = app.world().entity(UnitId::new(2).entity());
    assert!(app.world().entity(target).contains::<Dead>());
}
```

### run_if 门控测试

```rust
#[test]
fn combat_system_only_runs_in_player_phase() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(BattlePlugin);

    // 设置为非战斗阶段
    app.insert_resource(State::new(BattlePhase::RoundStart));
    app.update();

    // 验证战斗系统未执行
    let record = app.world().resource::<BattleRecord>();
    assert!(record.entries().is_empty(), "Combat system should not run in RoundStart");

    // 切换到 PlayerPhase
    app.insert_resource(State::new(BattlePhase::PlayerPhase));
    app.world_mut().write_message(DamageApplied { /* ... */ });
    app.update();

    // 验证战斗系统执行了
    let record = app.world().resource::<BattleRecord>();
    assert!(!record.entries().is_empty(), "Combat system should run in PlayerPhase");
}
```
