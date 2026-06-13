对于你的项目，我的答案是：

> **不要直接裸用 `info!()`、`warn!()`；但也不要学 UE 那样搞一个巨大的日志系统。**
>
> 最佳实践是：
>
> **做一个极薄的 Domain Log Facade（领域日志薄封装）。**

---

# 为什么不推荐直接用 tracing

假设你项目做了两年。

你代码里到处都是：

```rust
info!(
    attacker=?attacker,
    target=?target,
    damage=damage,
    "UnitAttacked"
);
```

然后有一天你决定：

```text
增加 BattleId
增加 ReplayId
增加 SessionId
增加 TraceId
增加 Log Version
```

你会发现：

```text
全项目几百处日志
全部要改
```

---

再例如：

未来你想：

```text
控制台日志

+
战斗回放日志

+
埋点统计

+
Bug复现日志
```

如果直接用 tracing：

```rust
info!("UnitAttacked");
```

那就没法扩展。

---

# 为什么也不推荐 UE 那种宏地狱

UE经常这样：

```cpp
UE_LOG(LogBattle, Warning, TEXT("xxx"));
```

甚至：

```cpp
ABILITY_LOG(...)
COMBAT_LOG(...)
NETWORK_LOG(...)
```

一大堆宏。

---

Rust 本身已经有：

```rust
tracing
```

没必要再发明一套。

---

# 对你项目最合适的结构

建立：

```text
src/
└── infrastructure/
    └── logging/
        ├── mod.rs
        ├── battle.rs
        ├── skill.rs
        ├── buff.rs
        └── quest.rs
```

---

例如：

```rust
// battle.rs

pub fn unit_attacked(
    attacker: Entity,
    target: Entity,
    damage: i32,
) {
    tracing::info!(
        attacker=?attacker,
        target=?target,
        damage=damage,
        "UnitAttacked"
    );
}
```

---

业务代码：

```rust
battle_log::unit_attacked(
    attacker,
    target,
    damage,
);
```

而不是：

```rust
info!(...)
```

---

# 更进一步（我推荐）

其实你的项目已经有：

```text
Observer
Domain Event
```

所以日志甚至可以不手写。

---

例如：

```rust
pub struct UnitAttacked {
    pub attacker: Entity,
    pub target: Entity,
    pub damage: i32,
}
```

---

然后：

```rust
world.trigger(UnitAttacked {
    ...
});
```

---

统一 Observer：

```rust
fn log_unit_attacked(
    trigger: On<UnitAttacked>,
) {
    info!(
        attacker=?trigger.attacker,
        target=?trigger.target,
        damage=trigger.damage,
        "UnitAttacked"
    );
}
```

---

这样：

```text
领域事件
↓
日志
```

自动绑定。

---

以后新增：

```text
BattleReplay

Analytics

Achievement

Quest
```

全部监听同一个事件。

---

# 针对你的项目，我会定一个规则

## 禁止

```rust
info!("...");
warn!("...");
debug!("...");
```

直接出现在业务代码。

---

## 允许

```rust
battle_log::unit_attacked(...)
```

或者：

```rust
world.trigger(UnitAttacked)
```

---

## 最终推荐

如果项目规模：

```text
< 3万行
```

直接：

```rust
tracing::info!
```

也没问题。

---

如果项目规模：

```text
3万~10万行
```

（你未来大概率会到）

推荐：

```text
Domain Event
    ↓
Log Observer
    ↓
tracing
```

架构。

---

对于你这种 DDD + SRPG + AI协作项目，我甚至会把日志当成：

```text
Domain Event 的一个消费者
```

而不是一个独立系统。

也就是说：

```text
UnitAttacked Event
├── BattleLog UI
├── Replay Recorder
├── Achievement System
├── Quest System
└── Logger
```

Logger 只是最后那个 Observer。

这是我认为最符合你项目长期演化的方案。
