# Delayed Commands 延迟命令

## 1. 新特性概述

Bevy 0.19 新增 Delayed Commands，允许将任意命令延迟到未来某个时间点执行。这是 0.19 对 SRPG 项目最有价值的新功能之一。

在回合制战棋中，大量逻辑天然具有"延迟执行"的特征——死亡动画播放后移除实体、Buff 持续一段时间后过期、技能冷却后恢复、DOT 间隔触发伤害等。此前这些场景都需要 Timer + 状态机的样板代码，而 Delayed Commands 将其压缩为一行声明式调用，大幅降低代码复杂度。

## 2. API 详解

### 2.1 基本用法

```rust
fn delayed_spawn(mut commands: Commands) {
    commands.delayed().secs(1.0).spawn(DummyComponent);
}
```

`commands.delayed()` 返回一个延迟命令构建器，`.secs(f32)` 指定延迟秒数，后续链式调用与普通 `Commands` 一致（`spawn`、`entity(...).insert/remove/despawn`、`trigger` 等）。

### 2.2 延迟操作已存在实体

```rust
fn delayed_spawn_then_insert(mut commands: Commands) {
    let mut delayed = commands.delayed();
    let entity = delayed.secs(0.5).spawn_empty().id();
    delayed.secs(1.5).entity(entity).insert(DummyComponent);
}
```

同一个 `delayed` 构建器可以注册多个不同延迟时间的命令，且延迟命令返回的 `Entity` 可被后续延迟命令引用。上例中：0.5 秒后生成空实体，1.5 秒后向该实体插入组件。

### 2.3 延迟触发事件

```rust
commands.delayed().secs(3.0).trigger(BurnTick);
```

延迟触发 Observer 事件，与 `commands.trigger()` 语义一致，只是执行时间被推迟。同样支持 `trigger_targets`：

```rust
commands.delayed().secs(3.0).trigger_targets(BurnTick, target_entity);
```

## 3. 对 SRPG 项目的核心价值

### 3.1 替代 Timer 样板代码

此前 SRPG 项目中大量使用 Timer 来实现延迟效果，每种场景都需要：

1. 定义一个 Timer 组件
2. 在系统中 tick 该 Timer
3. 在 Timer 完成时执行逻辑
4. 清理 Timer 组件

常见的 Timer 类型包括：

| Timer 组件 | 用途 |
|---|---|
| `AnimationTimer` | 动画帧切换 / 动画结束回调 |
| `BuffTimer` | Buff 持续时间到期移除 |
| `EffectTimer` | 特效播放完毕后清理 |
| `DOTTimer` | DOT 伤害间隔触发 |
| `CooldownTimer` | 技能冷却恢复 |

Delayed Commands 可以将上述一次性延迟场景的代码量减少 60-80%，消除 Timer 组件定义、系统注册、状态清理等样板代码。

### 3.2 死亡动画

**以前：**

```rust
// 播放死亡动画
// 1.5秒后移除实体
// 需要 Timer + State 机器

#[derive(Component)]
struct DeathDespawnTimer(Timer);

fn tick_death_despawn(time: Res<Time>, mut query: Query<(Entity, &mut DeathDespawnTimer)>, mut commands: Commands) {
    for (entity, mut timer) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
```

**现在：**

```rust
fn on_death(trigger: Trigger<DeathTrigger>, mut commands: Commands) {
    let entity = trigger.target();
    // 播放死亡动画
    commands.entity(entity).insert(DeathAnimation);
    // 1.5秒后移除实体
    commands.delayed().secs(1.5).entity(entity).despawn();
}
```

从"定义组件 + 注册系统 + tick 逻辑 + 清理"缩减为一行声明式调用。

### 3.3 DOT 伤害

```rust
fn apply_burn(trigger: Trigger<BurnApplied>, mut commands: Commands) {
    let target = trigger.target();
    // 立即造成初始伤害
    commands.entity(target).insert(Burning);
    // 3秒后触发 DOT tick
    commands.delayed().secs(3.0).trigger_targets(BurnTick, target);
}
```

对于需要多次 tick 的 DOT，可以在 `BurnTick` 的 Observer 中重新注册下一次延迟：

```rust
fn on_burn_tick(trigger: Trigger<BurnTick>, mut commands: Commands) {
    let target = trigger.target();
    // 造成 DOT 伤害
    commands.trigger_targets(DamageEvent { amount: 5 }, target);
    // 如果仍在燃烧，注册下一次 tick
    // 注意：需要查询 Burning 组件确认仍在生效
    commands.delayed().secs(3.0).trigger_targets(BurnTick, target);
}
```

### 3.4 Buff 过期

```rust
fn apply_buff(mut commands: Commands, target: Entity, duration: f32) {
    commands.entity(target).insert(ShieldBuff);
    commands.delayed().secs(duration).entity(target).remove::<ShieldBuff>();
}
```

> **注意**：此模式适用于不可驱散的 Buff。对于可驱散的 Buff，参见 [4.3 推荐模式](#43-推荐模式delayedbattlecommand)。

### 3.5 技能冷却

```rust
fn cast_ability(mut commands: Commands, caster: Entity, cooldown: f32) {
    commands.entity(caster).insert(AbilityOnCooldown);
    commands.delayed().secs(cooldown).entity(caster).remove::<AbilityOnCooldown>();
}
```

技能冷却是典型的"一次性延迟 + 无需取消"场景，非常适合 Delayed Commands。

### 3.6 连击系统

```rust
fn execute_combo(mut commands: Commands, attacker: Entity) {
    // 第一击
    commands.trigger_targets(FirstHit, attacker);
    // 0.3秒后第二击
    commands.delayed().secs(0.3).trigger_targets(SecondHit, attacker);
    // 0.6秒后第三击
    commands.delayed().secs(0.6).trigger_targets(ThirdHit, attacker);
}
```

连击系统此前需要复杂的状态机（ComboState + ComboTimer + ComboStep），现在可以用多个延迟触发直接表达时序。

## 4. 重要限制：不支持取消

### 4.1 官方说明

> "No blessed cancellation mechanism yet"

Delayed Commands 一旦注册，无法通过官方 API 取消。这是当前版本最重要的限制。

### 4.2 风险场景

在 SRPG 项目中，以下场景可能导致已注册的延迟命令执行时目标已失效：

| 风险场景 | 具体表现 |
|---|---|
| 目标死亡 | 延迟 `.entity(target).remove::<ShieldBuff>()` 执行时 target 已被 despawn |
| 地图切换 | 场景销毁后延迟命令仍尝试操作已不存在的实体 |
| 战斗结束 | 战斗结束后延迟命令仍触发战斗逻辑 |
| Buff 被驱散 | ShieldBuff 已被驱散，延迟 remove 无意义但不会报错 |

### 4.3 推荐模式：DelayedBattleCommand

对于需要安全校验的延迟操作，建议封装业务层校验：

```rust
/// 延迟战斗命令，内嵌 BattleId 校验
struct DelayedBattleCommand {
    battle_id: BattleId,
    target: Entity,
    action: DelayedAction,
}

enum DelayedAction {
    RemoveBuff(TypeId),
    Despawn,
    TriggerEffect(EffectId),
}

fn execute_delayed_battle_command(
    mut commands: Commands,
    query: Query<&BattleId, With<BattleRoot>>,
    delayed: Res<DelayedBattleCommands>,
) {
    for cmd in &delayed.commands {
        // 校验战斗是否仍在进行
        if query.get(cmd.target).is_ok() {
            // 执行命令
        }
        // 否则静默忽略
    }
}
```

此模式的核心思路：**延迟命令执行时先校验上下文是否仍然有效，无效则静默跳过**。

### 4.4 替代方案

对于需要取消的场景，仍然使用 Timer + 手动取消：

| 场景 | 推荐方案 |
|---|---|
| 长时间 Buff（如持续到战斗结束） | Timer + 手动取消 |
| 可驱散的 Buff | Timer + 驱散时取消 |
| 条件触发的延迟效果 | Timer + 条件检查 |

## 5. 与现有 Timer 的对比

| 场景 | Timer | Delayed Commands |
|------|-------|------------------|
| 一次性延迟 | 需要状态机 | 一行代码 |
| 可取消 | 天然支持 | 不支持 |
| 循环触发 | 天然支持 | 需要重新注册 |
| 短生命周期效果 | 样板代码多 | 简洁 |
| 长生命周期效果 | 合适 | 需要额外校验 |
| 代码量 | 多（组件+系统+tick） | 少（一行声明） |
| 调试可见性 | 组件可查询 | 命令队列不可查询 |
| ECS 一致性 | 显式组件驱动 | 隐式命令队列 |

## 6. 迁移策略

### 6.1 立即迁移

以下场景属于"一次性延迟 + 无需取消"，可以安全地立即迁移到 Delayed Commands：

| 场景 | 迁移收益 |
|---|---|
| 死亡动画延迟 despawn | 消除 DeathDespawnTimer 组件及对应系统 |
| 技能冷却 | 消除 CooldownTimer 组件及对应系统 |
| 简单 DOT tick | 消除 DOTTimer 组件，用延迟 trigger 替代 |
| 连击系统 | 消除 ComboState/ComboTimer，用多行延迟 trigger 替代 |
| 特效延迟清理 | 消除 EffectTimer 组件 |

### 6.2 保持 Timer

以下场景因需要取消或循环，暂不迁移：

| 场景 | 保留原因 |
|---|---|
| 循环 Buff tick（每回合触发） | 需要循环触发，Delayed Commands 需重新注册 |
| 可驱散的 Buff | 驱散时需要取消延迟效果 |
| 需要取消的延迟效果 | Delayed Commands 不支持取消 |
| 需要查询剩余时间的 UI | Timer 组件可直接读取 remaining_secs |

### 6.3 混合模式

两种机制可以共存，按场景选择：

- **短生命周期效果** → Delayed Commands（简洁、声明式）
- **长生命周期 / 可取消效果** → Timer（可控、可查询）
- **同一系统中**可以同时使用两种机制，互不冲突

## 7. 注意事项

1. **World 销毁时自动失效**：Delayed Commands 在 World 销毁时自动失效，不会悬空执行。场景切换时无需手动清理。
2. **延迟时间基于真实时间**：延迟时间基于 `Time<Real>` 而非游戏帧，不受帧率影响。但也不受时间缩放（time scaling）影响，如需暂停时的延迟效果需要额外处理。
3. **暂无内置取消机制**：建议封装业务层校验（如 `DelayedBattleCommand` 模式），在执行前验证上下文有效性。
4. **不要在 Delayed Command 中持有 Entity 引用而不做有效性检查**：延迟期间 Entity 可能已被 despawn，操作无效 Entity 不会 panic 但逻辑上可能不正确。
5. **命令队列不可查询**：与 Timer 组件不同，已注册的延迟命令无法通过 ECS 查询检查，调试时需注意。
6. **Observer 中的延迟命令**：在 Observer 回调中使用 `commands.delayed()` 是安全的，且是最常见的使用模式。
