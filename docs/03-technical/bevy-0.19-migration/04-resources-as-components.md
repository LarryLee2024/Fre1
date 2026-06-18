# Resources as Components — Resource 统一为 Component

> Bevy 0.19 迁移系列文档 | 适用于基于 Bevy 0.18.1 的 SRPG 项目

## 1. 新特性概述

Bevy 0.19 将 Resource 底层统一为 Component，存储在 Singleton Entity 上。这是 0.19 最具战略价值的底层变化。

在 Bevy 的 ECS 中，Resource 和 Component 本质上都是"存储在世界中的数据"，唯一的区别是基数（cardinality）：Resource 是"最多存在一个"的 Component。然而，这种人为的分离长期导致两套内部机制并存，且 Resource 无法享受 Component 的许多高级特性（Hook、Observer、Relationship 等）。

0.19 的统一意味着：

- **引擎内部**：消除了 Resource / Component 的重复机制，降低维护成本
- **开发者层面**：Resource 获得了与 Component 对等的能力，不再有"二等公民"
- **架构层面**：Entity + Component 成为唯一的数据模型，DDD 聚合根的映射更加自然

## 2. 核心变化

### 2.1 以前（Bevy ≤ 0.18）

- Resource 和 Component 是两套独立系统
- Resource 没有 Hook、Observer、Relationship
- 引擎内部维护两套存储和访问机制
- Resource 只能通过 `Res<T>` / `ResMut<T>` 访问
- 无法在 Query 中同时查询 Resource 和 Component
- Resource 无法添加额外的 Component 作为元数据

### 2.2 现在（Bevy 0.19）

- Resource 存储为 Singleton Entity 上的 Component
- 底层统一，减少内部重复机制
- `Res<T>` / `ResMut<T>` 语法仍然有效（语法糖不变）
- Resource Entity 可以拥有 Component 的所有高级特性

### 2.3 新增能力

Resource 现在可以：

| 能力 | 说明 | 对 SRPG 的价值 |
|------|------|----------------|
| 添加生命周期 Observer | `on_insert`、`on_remove` 等 | 监听 BattleState 插入/移除，自动触发初始化/清理 |
| 添加自定义 Hook | 组件级生命周期回调 | TurnState 初始化时自动重置回合计数器 |
| 添加 Relationship | 与其他 Entity 建立关联 | BattleState → TurnState 的聚合关系 |
| 添加额外 Component | 为 Resource Entity 附加元数据 | 为 BattleState 添加 `ActiveBattle` 标记 |
| 标记为 Immutable | 防止运行时修改 | 保护 GameConfig 等配置型 Resource |
| Query 混合查询 | 同时查询 Resource 和 Component | 统一的数据访问模式 |

### 2.4 不支持

- **不支持改变 Resource 的存储类型**：Resource 有一致的插入和访问模式，暴露存储类型选择不是有用的性能杠杆
- **`Res<T>` / `ResMut<T>` 语法不变**：现有代码无需修改访问方式

## 3. 对 DDD 架构的影响

### 3.1 思维模型转变

```
以前：Resource = 全局变量（特殊的、受限的）
现在：Resource = 世界中的特殊实体（Singleton Entity）
```

这个转变对 DDD 架构有深远影响：

- **BattleState** 不再是"特殊存在"，而是战斗聚合根的 Singleton Entity
- **TurnState** 可以拥有 Observer，自动响应回合状态变化
- **InputState** 可以有 Relationship，关联当前输入目标 Entity

### 3.2 Resource Hook 时代

以前，Resource 的生命周期管理只能通过 System 手动处理。现在，Resource 可以像 Component 一样拥有 Hook 和 Observer：

```rust
#[derive(Resource)]
struct BattleState {
    phase: BattlePhase,
}

// 未来可以为 Resource 添加 Observer
// BattleState 插入时触发初始化逻辑
// BattleState 移除时触发清理逻辑
```

这意味着：
- **初始化逻辑**可以绑定到 Resource 的 `on_insert`，而非散落在各个 Startup System 中
- **清理逻辑**可以绑定到 Resource 的 `on_remove`，而非依赖 `OnExit` 状态
- **状态变化**可以通过 Observer 自动传播，减少手动 `is_changed()` 检查

### 3.3 统一数据模型

```
Entity + Component + Resource 本质都是数据：
- Entity  = 身份（Identity）
- Component = 实体数据（Entity Data）
- Resource  = 单例实体数据（Singleton Entity Data）
```

这让 DDD 的聚合根概念更统一：

| DDD 概念 | ECS 映射（以前） | ECS 映射（0.19） |
|----------|------------------|-------------------|
| 战斗聚合 | BattleRoot Entity + BattleState Resource（割裂） | BattleRoot Entity + BattleState Resource（统一在 Singleton Entity 上） |
| 回合聚合 | TurnRoot Entity + TurnState Resource（割裂） | TurnRoot Entity + TurnState Resource（可通过 Relationship 连接） |
| 配置聚合 | GameConfig Resource（无实体身份） | GameConfig Resource Entity（有实体身份，可附加元数据） |

### 3.4 聚合边界更清晰

Resource Entity 化后，可以用 Relationship 连接 Resource Entity，让聚合边界更清晰：

```
BattleRoot Entity
  ├── ChildOf → BattleState Resource Entity
  ├── ChildOf → TurnState Resource Entity
  └── ChildOf → MapState Resource Entity
```

这种结构让战斗聚合的内部关系一目了然，也便于 Save/Replay 时序列化整个聚合。

## 4. 项目中的 Resource 清单与影响评估

### 4.1 核心 Resource

| Resource | 当前用途 | 0.19 影响 | 优先级 |
|----------|----------|-----------|--------|
| `BattleState` | 战斗阶段管理 | 可添加 Observer 监听状态变化，自动触发阶段转换逻辑 | 高 |
| `TurnState` | 回合管理 | 可添加 Hook 做回合初始化/清理，替代手动 System | 高 |
| `InputState` | 输入管理 | 可添加 Relationship 关联输入目标 Entity | 中 |
| `MapState` | 地图状态 | 可添加 Observer 监听地图切换，自动触发地形更新 | 中 |

### 4.2 配置 Resource

| Resource | 当前用途 | 0.19 影响 | 优先级 |
|----------|----------|-----------|--------|
| `GameConfig` | 游戏配置 | 可迁移到 `SettingsGroup`，利用 Bevy 0.19 新增的 `bevy_settings` 持久化 | 中 |
| `AudioConfig` | 音频配置 | 可迁移到 `SettingsGroup`，配合 `PreferencesPlugin` 自动持久化 | 低 |

### 4.3 系统 Resource

| Resource | 当前用途 | 0.19 影响 | 优先级 |
|----------|----------|-----------|--------|
| `Time` | 时间管理 | 引擎内部已统一，无需关注 | 无 |
| `Assets<T>` | 资产管理 | 引擎内部已统一，无需关注 | 无 |

### 4.4 影响评估总结

- **零破坏性**：所有现有 `Res<T>` / `ResMut<T>` 代码无需修改
- **增量收益**：逐步为关键 Resource 添加 Observer/Hook/Relationship
- **架构利好**：Resource Entity 化让 DDD 聚合根映射更自然

## 5. 迁移策略

### 5.1 立即可做（0.19 升级时）

- **理解新模型**：Resource = Singleton Entity，不再是"特殊的全局变量"
- **代码不需要改动**：`Res<T>` / `ResMut<T>` 语法仍然有效
- **验证现有行为**：确保所有 Resource 的读写行为与 0.18 一致

### 5.2 逐步采用（0.19 稳定后）

按优先级逐步为 Resource 添加新能力：

**第一阶段：Observer（高优先级）**

为关键 Resource 添加 Observer，替代手动 `is_changed()` 检查：

```rust
// 以前：在 System 中手动检查 BattleState 变化
fn check_battle_state_change(state: Res<BattleState>) {
    if state.is_changed() {
        // 处理变化
    }
}

// 未来：直接为 Resource 添加 Observer
app.add_observer(on_battle_state_changed);
```

**第二阶段：Hook（中优先级）**

为 Resource 添加生命周期 Hook，替代散落在各处的初始化/清理逻辑：

```rust
// BattleState 插入时自动初始化
// BattleState 移除时自动清理
```

**第三阶段：Relationship（低优先级）**

利用 Resource + Component 混合查询和 Relationship，建立聚合间的关联：

```rust
// BattleRoot Entity → BattleState Resource Entity
// TurnRoot Entity → TurnState Resource Entity
```

### 5.3 未来规划（0.20+）

- Resource Entity 化后，考虑用 Relationship 连接 Resource Entity
- 例如：`BattleRoot Entity → BattleState Resource Entity`
- 这会让战斗聚合的边界更清晰，也便于 Save/Replay 时序列化
- 关注 Bevy 后续版本对 Resource Hook/Observer 的正式 API 支持

## 6. 代码示例

### 6.1 为 Resource 添加 Observer

```rust
// 以前：需要在 System 中手动检查 BattleState 变化
fn check_battle_state_change(state: Res<BattleState>) {
    if state.is_changed() {
        // 处理变化
    }
}

// 未来：直接为 Resource 添加 Observer
app.add_observer(on_battle_state_changed);

fn on_battle_state_changed(trigger: Trigger<OnInsert, BattleState>) {
    // BattleState 被插入时自动触发
    info!("BattleState initialized!");
}
```

### 6.2 Resource + Component 混合查询

```rust
// 查询所有拥有特定 Component 的 Resource Entity
fn query_resources(query: Query<&BattleState, With<ActiveBattle>>) {
    for state in &query {
        // ...
    }
}

// 这意味着可以为 Resource Entity 附加标记 Component
// 例如：为当前活跃的战斗 Resource 添加 ActiveBattle 标记
```

### 6.3 Resource 生命周期管理

```rust
// 以前：手动管理 Resource 的插入和移除
commands.insert_resource(BattleState::new());
commands.remove_resource::<BattleState>();

// 未来：可以添加 Hook 和 Observer
// BattleState 插入时自动初始化
// BattleState 移除时自动清理
// 无需在 OnEnter/OnExit 状态中手动处理
```

### 6.4 Immutable Resource

```rust
// 配置型 Resource 可以标记为 Immutable，防止运行时意外修改
// 这对 GameConfig 等定义态数据特别有用
// 符合项目红线：严禁修改定义态配置数据
```

### 6.5 Resource 间的 Relationship

```rust
// 未来：通过 Relationship 建立聚合间的关联
// BattleState Resource Entity → TurnState Resource Entity
// 让战斗聚合的内部关系显式化
```

## 7. 注意事项

1. **`Res<T>` / `ResMut<T>` 语法不变**：现有代码无需修改，这是零破坏性升级
2. **不要急于将所有 Resource 改为 Component**：Resource 的"单例"语义是架构约束，不是技术限制，保持 `#[derive(Resource)]` 语义清晰
3. **Resource 的"单例"语义**：一个 World 中最多只有一个 `T` 类型的 Resource，这是架构约束而非技术限制
4. **Observer/Hook API 可能演进**：0.19 是 Resource 统一的第一步，后续版本可能继续增强 API
5. **关注 0.20+ 的 Resource Entity 化进展**：Bevy 可能提供更直接的 Resource Entity 操作 API
6. **不要绕过 Effect/Modifier 管线**：即使 Resource 可以添加 Observer，也不应在 Observer 中直接修改战斗数值，必须通过管线
7. **Save/Replay 兼容性**：Resource Entity 化可能影响序列化格式，需与 @data-architect 确认

## 8. 架构信号

0.19 的 Resource 统一透露了 Bevy 的长期方向：

```
一切数据都是 Entity + Component
Observer / Hook / Relationship 是通用机制
ECS 正在从"组件存储系统"升级为"关系 + 生命周期 + 观察器驱动的数据模型"
```

这对 DDD 项目是利好：

| 维度 | 以前 | 0.19+ |
|------|------|-------|
| 数据模型 | Entity + Component + Resource（三套） | Entity + Component（统一） |
| 生命周期 | 只有 Component 有 Hook | Resource 也有 Hook |
| 观察机制 | 只有 Component 有 Observer | Resource 也有 Observer |
| 关系建模 | 只有 Entity 间有 Relationship | Resource Entity 也可以有 Relationship |
| 聚合根映射 | 聚合根 = Entity，聚合状态 = Resource（割裂） | 聚合根 = Entity，聚合状态 = Resource Entity（统一） |

**核心结论**：领域模型可以更统一地使用 ECS 机制，Resource 不再是"二等公民"，DDD 的聚合根概念与 ECS 的映射更加自然。

---

> 参考：[Bevy 0.19 Release Notes — Resources as Components](https://bevy.org/news/bevy-0-19/#resources-as-components)
