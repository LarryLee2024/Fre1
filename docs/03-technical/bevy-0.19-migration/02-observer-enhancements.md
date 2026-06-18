# Observer 增强与关系系统

> 本文档分析 Bevy 0.19 中 Observer Run Conditions 与 Self-Referential Relationships 两个新特性，
> 评估其对 SRPG 项目（Ability/Effect/Buff/Turn/Character 等领域插件）的影响，
> 并给出迁移策略与架构建议。

---

## 1. Observer Run Conditions

### 1.1 新特性概述

Bevy 0.19 新增 **Observer Run Conditions**，允许 Observer 像 System 一样使用 `run_if` 条件。

此前，Observer 无法使用运行条件，只能在回调函数内部手动编写守卫逻辑。现在，Observer 与 System 在条件执行方面拥有了统一的 API 体验。

### 1.2 语法

```rust
#[derive(Resource)]
struct GamePaused(bool);

// 单条件：仅在游戏未暂停时触发
app.add_observer(
    on_damage.run_if(|paused: Res<GamePaused>| !paused.0)
);

// 多条件链式调用（AND 语义）：未暂停 且 Player 资源存在
app.add_observer(
    on_damage
        .run_if(|paused: Res<GamePaused>| !paused.0)
        .run_if(resource_exists::<Player>)
);
```

支持三种注册方式：

| 注册方式 | 示例 |
|---------|------|
| `app.add_observer()` | `app.add_observer(on_damage.run_if(\|p: Res<Pause>\| !p.0))` |
| `entity.observe()` | `entity.observe(on_click.run_if(\|p: Res<Pause>\| !p.0))` |
| `Observer` builder | `Observer::new(on_damage).run_if(\|p: Res<Pause>\| !p.0)` |

### 1.3 对 SRPG 项目的价值

#### 现状：大量 Observer 内部守卫代码

当前项目中，许多 Observer 在回调开头进行状态检查，不符合条件则提前返回：

```rust
fn on_damage_triggered(trigger: Trigger<DamageApplied>, battle_state: Res<BattleState>) {
    if battle_state.phase != BattlePhase::Running { return; }
    // 实际逻辑
}
```

这种模式的问题：
- **职责混淆**：Observer 既负责"是否应该响应"，又负责"如何响应"
- **不可复用**：守卫逻辑内嵌在函数体中，无法跨 Observer 复用
- **测试困难**：需要构造完整场景才能验证守卫行为
- **性能浪费**：Observer 仍然被调度执行，只是提前返回

#### 迁移后：声明式条件

```rust
app.add_observer(
    on_damage_triggered.run_if(|state: Res<BattleState>| state.phase == BattlePhase::Running)
);
```

优势：
- **职责分离**：条件判断与业务逻辑解耦
- **可复用**：条件闭包可提取为命名函数，跨 Observer 共享
- **声明式**：Observer 注册时即可直观看到触发前提
- **性能优化**：条件不满足时 Observer 完全不执行，跳过调度开销

#### 具体场景映射

| Observer | 原守卫代码 | 迁移后 run_if |
|----------|-----------|--------------|
| `DamageApplied` | `if battle_state.phase != Running { return; }` | `.run_if(\|s: Res<BattleState>\| s.phase == BattlePhase::Running)` |
| `TurnStarted` | `if battle_state.is_none() { return; }` | `.run_if(resource_exists::<BattleState>)` |
| `BuffTriggered` | `if !character.is_alive() { return; }` | `.run_if(character_alive)` |
| `AbilityCast` | `if character.has_status(Silenced) { return; }` | `.run_if(\|q: Query<&StatusSet>\| !q.get(entity).unwrap().has(Silenced))` |

其中 `character_alive` 可提取为可复用的条件函数：

```rust
fn character_alive(trigger: Trigger<impl Event>, query: Query<&Health>) -> bool {
    query.get(trigger.target()).map_or(false, |h| h.is_alive())
}
```

### 1.4 迁移建议

1. **渐进式迁移**：不要一次性重构所有 Observer，按领域模块逐步推进
2. **新增优先**：新增 Observer 必须使用 `run_if`，不再允许内部守卫
3. **提取共享条件**：将高频出现的条件提取为命名函数，放入各领域模块的 `conditions.rs`
4. **保留复杂守卫**：涉及多实体查询或复杂逻辑的守卫，暂时保留在函数体内，待模式成熟后再迁移
5. **测试覆盖**：迁移每个 Observer 时，确保其条件行为有对应测试

迁移优先级建议：

| 优先级 | 领域 | 原因 |
|-------|------|------|
| P0 | Combat | 战斗阶段守卫最多，收益最大 |
| P1 | Ability | 施法条件检查频繁 |
| P1 | Buff | 触发条件与存活状态强关联 |
| P2 | Turn | 回合状态守卫较简单 |
| P3 | Character | 守卫较少，可延后 |

---

## 2. Self-Referential Relationships

### 2.1 新特性概述

Bevy 0.19 允许自定义关系组件自引用，通过 `allow_self_referential` 属性声明。

默认情况下，Bevy 拒绝关系组件指向自身实体——插入时会发出警告并移除。这是因为结构化关系（如 `ChildOf`）会形成层级遍历，自引用会导致无限循环。但许多关系是纯语义的，自引用完全合法。

### 2.2 语法

```rust
#[derive(Component)]
#[relationship(relationship_target = PeopleILike, allow_self_referential)]
pub struct LikedBy(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = LikedBy)]
pub struct PeopleILike(Vec<Entity>);
```

关键点：
- `allow_self_referential` 是 opt-in 属性，必须显式声明
- 仅对自定义关系生效，`ChildOf` 仍然不允许自引用
- 插入自引用关系时不再产生警告

### 2.3 对 SRPG 项目的价值

#### 项目中的关系场景分析

| 关系 | 方向 | 自引用场景 | 是否需要 allow_self_referential |
|------|------|-----------|-------------------------------|
| `CasterOf(Target)` | 施放者 → 目标 | 自我施法（治疗自己） | **是** |
| `OwnerOf` | 主人 → 召唤物 | 不适用（主人≠召唤物） | 否 |
| `SummonedBy` | 召唤物 → 主人 | 不适用 | 否 |
| `AuraSource` | 光环实体 → 受影响者 | 自身也在光环范围内 | **是** |
| `Healing` | 治疗者 → 被治疗者 | 自我治疗 | **是** |
| `TargetOf` | 目标 ← 施放者 | 自我施法 | **是** |

#### 典型用例：自我治疗

```rust
#[derive(Component)]
#[relationship(relationship_target = HealingTargets, allow_self_referential)]
pub struct HealingTarget(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = HealingTarget)]
pub struct HealingTargets(Vec<Entity>);

// 治疗者可以治疗自己
fn cast_heal(mut commands: Commands, caster: Entity) {
    commands.entity(caster).insert(HealingTarget(caster));
}
```

#### 典型用例：光环自身生效

```rust
#[derive(Component)]
#[relationship(relationship_target = AuraAffectees, allow_self_referential)]
pub struct AuraAffectee(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = AuraAffectee)]
pub struct AuraAffectees(Vec<Entity>);

// 光环源自身也在影响范围内
fn apply_aura(mut commands: Commands, source: Entity) {
    commands.entity(source).insert(AuraAffectee(source));
}
```

### 2.4 注意事项

1. **`ChildOf` 仍然禁止自引用**：层级关系涉及递归遍历，自引用会无限循环，这是硬性约束
2. **纯语义关系才适合自引用**：`Likes`/`EmployedBy`/`Healing` 等不涉及遍历的关系可以自引用
3. **不影响现有 Parent/Child 层级**：自引用仅限自定义关系，不改变内置层级行为
4. **序列化兼容**：自引用关系在 Save/Replay 中需要正确处理，确保 Entity 映射时不会断裂
5. **查询注意**：自引用关系可能导致查询结果中出现重复实体，需要去重或特殊处理

---

## 3. Observer 与项目架构的深度结合

### 3.1 Observer 正在取代部分 EventReader 模式

#### 传统模式

```
Event → EventReader(System) → 处理逻辑 → 发送新 Event → EventReader(System) → ...
```

例如当前 Ability 系统：
```
CastAbilityEvent → ability_system → ApplyDamageEvent → damage_system → ...
```

#### Observer 模式

```
Event → Observer → 处理逻辑 → 触发新 Event → Observer → ...
```

迁移后：
```
CastAbility → Observer → DamageApplied → Observer → DeathTriggered → Observer
```

#### 对比

| 维度 | EventReader 模式 | Observer 模式 |
|------|-----------------|--------------|
| 注册方式 | 系统签名中声明 `EventReader<E>` | `app.add_observer(\|trigger: Trigger<E>\| ...)` |
| 作用域 | 全局，所有同类型事件 | 可限定到特定实体 |
| 条件执行 | 系统级 run_if | Observer 级 run_if（0.19 新增） |
| 链式响应 | 需要中间系统转发 | Observer 可直接触发新事件 |
| 与 GAS 对应 | 较远 | 更接近 UE GAS 设计思路 |

#### 对 Ability 系统的影响

Observer 模式更接近 GAS (Gameplay Ability System) 的设计思路：

```
传统：CastAbilityEvent → ability_system → ApplyDamageEvent → damage_system
GAS：CastAbility → Observer → DamageApplied → Observer → DeathTriggered
```

这意味着：
- **Ability 插件**：`CastAbility` 事件由 Observer 捕获，执行施法逻辑
- **Effect 插件**：`DamageApplied` 事件由 Observer 捕获，执行效果应用
- **Buff 插件**：`BuffTriggered` 事件由 Observer 捕获，执行 Buff 逻辑
- **Character 插件**：`DeathTriggered` 事件由 Observer 捕获，执行死亡处理

### 3.2 Observer 地狱防范

#### 风险：链式触发导致无限循环

```
DamageApplied → BuffTrigger → DamageApplied → BuffTrigger → ...
```

这是 Observer 模式最危险的问题。在 SRPG 项目中，以下场景尤其需要注意：

| 循环路径 | 触发条件 |
|---------|---------|
| Damage → Buff(反伤) → Damage → Buff(反伤) → ... | 反伤 Buff 对伤害来源造成伤害 |
| Heal → Buff(治疗加成) → Heal → ... | 治疗加成 Buff 触发额外治疗 |
| Death → Buff(复活) → Heal → ... | 复活 Buff 在死亡时触发治疗 |

#### 项目规范建议

1. **Observer 只能跨领域通信**：
   - Ability → Effect：允许
   - Effect → Buff：允许
   - Buff → Character：允许
   - **Buff → Buff：禁止**（领域内部走 System）

2. **领域内部仍然走 System**：
   - Buff 内部逻辑（叠加、刷新、移除）使用 System
   - Effect 内部逻辑（计算、应用、回退）使用 System

3. **不要在 Buff 内部用 Observer 调用另一个 Buff Observer**：
   - 如果需要 Buff 间交互，通过 Effect 插件中转
   - 例：`BurningBuff` 触发 `DamageApplied`，由 Effect 插件的 Observer 处理，而不是直接调用 `PoisonBuff` 的 Observer

4. **添加循环检测机制**：
   - 在 `BattleState` 中记录当前帧的 Observer 调用深度
   - 超过阈值（如 10 层）时记录警告并中断
   - 可通过 `run_if` 条件实现：

```rust
fn observer_depth_ok(state: Res<BattleState>) -> bool {
    state.observer_depth < MAX_OBSERVER_DEPTH
}

app.add_observer(
    on_buff_trigger
        .run_if(observer_depth_ok)
);
```

### 3.3 测试适配

#### 问题：Observer + Deferred Command + Delayed Command 混合使用

Bevy 0.19 中，Observer、Deferred Command 和 Delayed Command 的执行时机不同：

| 机制 | 执行时机 |
|------|---------|
| Observer 回调 | 事件触发时立即执行 |
| Deferred Command | 当前 stage 结束时批量执行 |
| Delayed Command | 指定延迟时间后执行 |

单次 `app.update()` 可能不足以让所有 Observer 和延迟命令执行完毕。

#### 建议：增加测试工具

```rust
/// 运行多帧，确保所有 Observer 和延迟命令执行完毕
pub fn run_frames(app: &mut App, frames: u32) {
    for _ in 0..frames {
        app.update();
    }
}
```

使用示例：

```rust
#[test]
fn damage_triggers_buff_then_death() {
    let mut app = App::new();
    // ... 设置

    // 发送伤害事件
    app.world_mut().send_event(DamageApplied { target: entity, amount: 100 });

    // 单次 update 不够：DamageApplied → BuffTrigger → DeathTriggered 需要多帧
    test_utils::run_frames(&mut app, 3);

    // 验证最终状态
    assert!(app.world().get::<Dead>(entity).is_some());
}
```

#### 测试注意事项

1. **Observer 链式触发**：每个 Observer 可能在不同帧执行，需要 `run_frames` 确保完整执行
2. **Delayed Command**：如果 Observer 触发了延迟命令，需要模拟时间推进
3. **确定性验证**：Observer 执行顺序可能与注册顺序相关，测试需注意顺序依赖

---

## 4. Relationship + Observer 组合：未来 GAS-Lite 的核心

### 4.1 组合模式

Relationship 与 Observer 的组合，构成了项目 GAS-Lite 架构的核心驱动机制：

```
CasterOf(Target)  →  DamageApplied  →  Observer  →  Has(BurningBuff)  →  Observer  →  Death
```

整个链路几乎没有传统 OOP 调度器，而是纯 ECS 关系驱动：

```
[Ability Entity]                [Effect Entity]               [Buff Entity]           [Character Entity]
    │                               │                              │                        │
    ├─ CasterOf(Target)             ├─ SourceOf(Ability)           ├─ AppliedTo(Target)     ├─ Has(BurningBuff)
    │  (关系: 指向目标)              │  (关系: 指向来源技能)         │  (关系: 指向目标)       │  (关系: Buff 标记)
    │                               │                              │                        │
    ▼                               ▼                              ▼                        ▼
 CastAbility Event          DamageApplied Event           BuffTriggered Event       DeathTriggered Event
    │                               │                              │                        │
    ▼                               ▼                              ▼                        ▼
 [Observer]                  [Observer]                    [Observer]                [Observer]
 (Effect 插件)               (Buff 插件)                   (Character 插件)          (UI/动画 插件)
```

### 4.2 与 UE GAS 的对应

| UE GAS 概念 | SRPG 项目对应 | 实现方式 |
|------------|-------------|---------|
| GameplayEffect | Effect Component + Observer | Effect 实体携带 EffectData 组件，Observer 响应 DamageApplied 等事件 |
| GameplayAbility | Ability Component + Observer | Ability 实体携带 AbilityData 组件，Observer 响应 CastAbility 等事件 |
| GameplayTag | Component Marker + Relationship | 标签作为 Marker Component，通过 Relationship 关联到实体 |
| AbilitySystemComponent | Entity + Relationships | 角色实体通过 CasterOf/Has 等关系连接所有 Ability/Effect/Buff |
| GameplayCue | Cue Component + Observer | Cue 实体通过 Observer 响应触发事件，播放表现 |

### 4.3 项目建议

#### 短期（0.19 迁移阶段）

1. **现有 Entity 字段逐步迁移为 Relationship**：
   - `EffectData.caster: Entity` → `CasterOf(Entity)` 关系组件
   - `BuffData.target: Entity` → `AppliedTo(Entity)` 关系组件
   - `SummonData.owner: Entity` → `SummonedBy(Entity)` 关系组件
   - 迁移时保持向后兼容，先并行存在，再逐步移除 Entity 字段

2. **新增领域行为优先用 Observer 注册**：
   - 新增的跨领域通信使用 Observer
   - 领域内部逻辑继续使用 System

3. **保持领域边界**：
   - Observer 跨领域通信
   - System 领域内逻辑
   - 严禁 Observer 在同一领域内形成链式调用

#### 中期（GAS-Lite 成型阶段）

4. **建立 Relationship 驱动的查询模式**：
   ```rust
   // 查询某角色所有 Buff
   fn get_all_buffs(character: Entity, query: Query<&HasBuff>) -> Vec<Entity> {
       query.get(character).map(|b| b.0.clone()).unwrap_or_default()
   }

   // 查询某技能的所有目标
   fn get_all_targets(ability: Entity, query: Query<&CasterOf>) -> Vec<Entity> {
       query.get(ability).map(|t| t.0.clone()).unwrap_or_default()
   }
   ```

5. **Observer 注册规范化**：
   - 每个领域插件在 `build()` 中集中注册所有 Observer
   - Observer 使用 `run_if` 声明触发条件
   - 跨领域 Observer 必须有明确的领域边界注释

6. **Save/Replay 适配**：
   - Relationship 的序列化/反序列化需要 Entity 映射
   - 自引用关系在 Entity 重映射时需要特殊处理
   - Observer 触发序列需要与 Replay 帧对齐

#### 长期（完整 GAS-Lite 阶段）

7. **完全 Relationship 化**：
   - 所有 Entity 引用字段迁移为 Relationship
   - 利用 BSN 的 Relationship 语法进行场景定义
   - 通过 Relationship 查询替代手动 Entity 查找

8. **Observer 管线可视化**：
   - 开发调试工具，可视化 Observer 触发链路
   - 检测循环依赖和深度过大的链路
   - 记录 Observer 执行时间，定位性能瓶颈

---

## 附录 A：迁移检查清单

### Observer Run Conditions 迁移

- [ ] 审计所有 Observer 回调中的 `if ... { return; }` 守卫
- [ ] 将简单守卫迁移为 `run_if` 条件
- [ ] 提取共享条件函数到 `conditions.rs`
- [ ] 更新 Observer 注册代码
- [ ] 补充条件行为的测试覆盖
- [ ] 更新编码规范：新增 Observer 必须使用 `run_if`

### Self-Referential Relationships 迁移

- [ ] 审计所有自定义 Relationship，识别自引用需求
- [ ] 为需要自引用的关系添加 `allow_self_referential`
- [ ] 验证 Save/Replay 中自引用关系的 Entity 映射
- [ ] 检查查询逻辑是否受自引用影响（去重）
- [ ] 更新数据架构文档

### 测试适配

- [ ] 实现 `test_utils::run_frames` 工具函数
- [ ] 审计现有测试中的 `app.update()` 调用，替换为 `run_frames`
- [ ] 添加 Observer 链式触发的集成测试
- [ ] 添加循环检测的边界测试

---

## 附录 B：相关参考

- Bevy 0.19 Release Notes：Observer Run Conditions / Self-Referential Relationships 章节
- 项目架构文档：`docs/01-architecture/README.md`
- 通信架构：`docs/01-architecture/00-foundation/ADR-002-ecs-communication.md`
- Ability 管线：`docs/01-architecture/10-capability-system/ADR-010-ability-pipeline.md`
- Effect 管线：`docs/01-architecture/10-capability-system/ADR-011-modifier-pipeline.md`
- 测试宪法：`docs/05-testing/test-spec.md`
