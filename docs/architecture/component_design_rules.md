# Component 设计规范

Version: 1.0
Status: Proposed
Source: `docs/其他/31遗漏.md` 第一节
Related: `docs/architecture.md` ECS 章节、`docs/architecture/layer-contracts.md`

---

## 概述

本文档定义 Bevy ECS 项目中 Component 的分类体系、设计约束、命名规范和禁止事项。

核心问题：Bevy 项目后期 90% 的混乱，都始于组件越写越大、职责越混，最终变成不可维护的大泥球。

本规范建立"三位一体"组件分类系统，将所有 Component 划分为三个明确类别，每类有严格的职责边界。

---

## 组件分类体系（三位一体）

### 1. Marker Component（标记组件）

**定义**：零数据的纯布尔标记，仅用于 Query 过滤和状态判断。

**特征**：
- 🟥 **绝对禁止**携带任何字段
- 仅用于 `With<T>` / `Without<T>` 查询过滤
- 可被 Bevy 优化为位掩码，查询性能远超布尔字段
- 通过 `commands.entity(e).insert(Dead)` 添加，`commands.entity(e).remove::<Dead>()` 移除

**允许的 Marker Component**：
```rust
#[derive(Component)]
pub struct Dead;

#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct Acted;

#[derive(Component)]
pub struct IsAlly;

#[derive(Component)]
pub struct IsEnemy;

#[derive(Component)]
pub struct Frozen;

#[derive(Component)]
pub struct Stunned;
```

**禁止的 Marker Component**：
```rust
// 🟥 禁止：Marker 携带字段
#[derive(Component)]
pub struct Dead {
    pub cause: DeathCause,  // 这变成了数据组件
}

// 🟥 禁止：布尔字段替代 Marker
pub struct Unit {
    pub is_dead: bool,     // 应使用 Dead Marker Component
    pub is_frozen: bool,   // 应使用 Frozen Marker Component
}
```

**判断标准**：
> 如果这个组件只需要回答"是/否"，不需要携带任何数据 → Marker Component。

---

### 2. Data Component（数据组件）

**定义**：纯数据容器，存储不可变或低频变更的状态数据。

**特征**：
- 🟥 **绝对禁止**包含任何业务逻辑方法（仅允许 `impl Default` 和纯数据辅助方法）
- 字段应为 `pub`，通过 System 直接读写
- 不包含跨模块协调逻辑
- 每个 Data Component 只负责一个数据关注点

**允许的 Data Component**：
```rust
#[derive(Component, Reflect)]
pub struct GridPosition {
    pub coord: IVec2,
}

#[derive(Component, Reflect)]
pub struct UnitName(pub String);

#[derive(Component, Reflect)]
pub struct UnitId(pub String);

#[derive(Component, Reflect)]
pub struct UnitRace(pub String);

#[derive(Component, Reflect)]
pub struct UnitClass(pub String);

#[derive(Component, Reflect)]
pub struct AiBehaviorId(pub String);
```

**禁止的 Data Component**：
```rust
// 🟥 禁止：数据组件包含业务逻辑方法
#[derive(Component)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    pub fn take_damage(&mut self, amount: i32) {  // 业务逻辑应放在 System 中
        self.current = (self.current - amount).max(0);
    }

    pub fn is_alive(&self) -> bool {  // 这是查询逻辑，不是数据
        self.current > 0
    }
}

// 🟥 禁止：数据组件修改其他 Component
#[derive(Component)]
pub struct Attributes { ... }

impl Attributes {
    pub fn apply_buff(&self, buff: &BuffData) {  // 跨模块协调
        // ...
    }
}
```

**判断标准**：
> 如果这个组件需要存储"某个东西的值"，但不需要自己做任何处理 → Data Component。

---

### 3. Status Component（状态组件）

**定义**：生命周期有限的临时状态，绑定到特定游戏阶段（如回合阶段、行动状态）。

**特征**：
- 必须在状态结束时自动清理
- 🟥 **绝对禁止**在状态退出后残留
- 通常携带与临时状态相关的运行时数据
- 通过 `OnExit(State)` 系统或 Hook 自动清理

**允许的 Status Component**：
```rust
#[derive(Component, Reflect)]
pub struct MovingUnit {
    pub path: Vec<IVec2>,
    pub current_index: usize,
    pub speed: f32,
    pub elapsed: f32,
    pub next_phase: TurnPhase,
}

#[derive(Component, Reflect)]
pub struct CastingSkill {
    pub skill_id: SkillId,
    pub target: Entity,
    pub cast_time: f32,
}

#[derive(Component, Reflect)]
pub struct PendingEffect {
    pub effects: Vec<EffectDef>,
}
```

**禁止的 Status Component**：
```rust
// 🟥 禁止：Status Component 在状态退出后残留
// 如果 MovingUnit 在移动完成后未被移除，会导致：
// 1. 移动动画重复播放
// 2. 后续系统误判单位仍在移动中

// 🟥 禁止：Status Component 被多个不相关的状态共享
#[derive(Component)]
pub struct InProgress;  // 太模糊，无法判断清理时机
```

**清理规则**：
```rust
// ✅ 正确：在 OnExit 中清理 Status Component
fn cleanup_moving_units(
    mut commands: Commands,
    query: Query<Entity, With<MovingUnit>>,
) {
    for entity in &query {
        commands.entity(entity).remove::<MovingUnit>();
    }
}

// ✅ 正确：通过 Hook 自动清理（Dead 移除 Selected）
#[component(on_add = Dead::on_add_dead)]
pub struct Dead;
```

**判断标准**：
> 如果这个组件有明确的"开始"和"结束"时机，且结束后必须清理 → Status Component。

---

## 组件命名规范

### 命名模式：`[Entity][Concept]`

| 类别 | 模式 | 示例 |
|------|------|------|
| Marker | `IsXxx` / `Xxx` | `IsAlly`, `Dead`, `Selected` |
| Data | `EntityConcept` | `GridPosition`, `UnitName`, `UnitId`, `PersistentTags` |
| Status | `EntityAction` | `MovingUnit`, `CastingSkill`, `PendingEffect` |
| UI Marker | `XxxBg` / `XxxFg` | `HpBarBg`, `HpBarFg` |

### 禁止的命名模式

- 🟥 `components.rs`（违反 Feature First 原则）
- 🟥 `XxxManager`（OOP 风格，不是 ECS）
- 🟥 `XxxData`（与 Definition 的 XxxData 混淆）
- 🟥 `XxxState`（与 Bevy State 混淆）

---

## 单一职责原则

### 规则

🟥 **一个 Component = 一个数据关注点。**

当 Component 超过 8 个字段时，必须拆分。

### 拆分判断

```
Component 是否需要拆分？
├─ 超过 8 个字段？→ 必须拆分
├─ 字段分为明显不同的关注点？→ 拆分
├─ 部分字段的生命周期不同？→ 拆分
└─ 否则 → 保持单一 Component
```

### 拆分示例

```rust
// 🟥 拆分前：God Component（超过 8 个字段）
#[derive(Component)]
pub struct UnitState {
    pub hp: i32,
    pub max_hp: i32,
    pub mp: i32,
    pub max_mp: i32,
    pub attack: i32,
    pub defense: i32,
    pub speed: i32,
    pub position: IVec2,
    pub faction: Faction,
    pub acted: bool,
}

// ✅ 拆分后：各司其职
#[derive(Component)]
pub struct Health { pub current: i32, pub max: i32 }

#[derive(Component)]
pub struct Mana { pub current: i32, pub max: i32 }

#[derive(Component)]
pub struct GridPosition { pub coord: IVec2 }

#[derive(Component)]
pub struct Unit { pub faction: Faction, pub acted: bool }
```

---

## 序列化规范

### 版本字段

🟥 **所有可序列化组件必须携带 `version` 字段。**

```rust
#[derive(Component, Reflect, serde::Serialize, serde::Deserialize)]
pub struct SkillCooldowns {
    pub version: u32,  // 必须
    pub cooldowns: HashMap<SkillId, u32>,
}
```

### 序列化约束

- 🟥 Definition（配置）组件禁止在运行时修改
- 🟥 Instance（运行时）组件必须引用 Definition ID，不保存 Definition 本身
- 🟩 序列化使用 `#[reflect(Serialize, Deserialize)]` 属性

---

## 禁止事项总览

| 禁止项 | 理由 | 替代方案 |
|--------|------|----------|
| 🟥 Marker Component 携带字段 | Marker 应零数据，否则查询性能下降 | 改用 Data Component |
| 🟥 Data Component 包含业务方法 | 违反 ECS 数据/逻辑分离原则 | 业务逻辑放在 System 中 |
| 🟥 Status Component 在状态退出后残留 | 导致系统误判，状态不一致 | 在 OnExit 系统中自动清理 |
| 🟥 跨领域的"上帝组件" | 破坏模块边界，修改一个字段影响所有依赖方 | 按关注点拆分为多个组件 |
| 🟥 组件超过 8 个字段不拆分 | 职责混杂，修改频率不同导致维护困难 | 按生命周期和关注点拆分 |
| 🟥 可序列化组件无 version 字段 | 无法处理数据迁移 | 添加 version 字段 |
| 🟥 手写 `is_xxx: bool` 字段 | Bevy 位掩码优化失效，查询性能下降 | 使用 Marker Component |
| 🟥 组件命名使用 `XxxManager` | OOP 风格封装 Entity，违反 ECS 原则 | 使用纯数据 Component + System |

---

## 允许的模式

### 模式1：Marker + Hook 组合

```rust
// Marker 触发固有行为
#[component(on_add = Dead::on_add_dead)]
pub struct Dead;

impl Dead {
    fn on_add_dead(mut world: DeferredWorld, context: HookContext) {
        // 固有行为：标记已行动、移除选中
        if let Some(mut unit) = world.get_mut::<Unit>(context.entity) {
            unit.acted = true;
        }
        world.commands().entity(context.entity).remove::<Selected>();
    }
}
```

### 模式2：Data Component + Required Components

```rust
// Unit 生成时自动插入所需组件
#[derive(Component)]
#[require(Attributes, SkillSlots, ActiveBuffs, GridPosition)]
pub struct Unit {
    pub faction: Faction,
    pub acted: bool,
}
```

### 模式3：Status Component + 自动清理

```rust
// 移动状态组件，移动完成后自动移除
#[derive(Component)]
pub struct MovingUnit {
    pub path: Vec<IVec2>,
    pub current_index: usize,
    pub speed: f32,
    pub elapsed: f32,
    pub next_phase: TurnPhase,
}

// 系统：移动完成后移除 Status Component
fn finish_movement(
    mut commands: Commands,
    query: Query<(Entity, &MovingUnit), With<MovingUnit>>,
) {
    for (entity, moving) in &query {
        if moving.is_finished() {
            commands.entity(entity).remove::<MovingUnit>();
        }
    }
}
```

---

## 变更检测规范

🟥 **必须优先使用 Bevy 原生的 `Added<T>`、`Changed<T>`、`Removed<T>` 过滤器。**

```rust
// ✅ 正确：使用 Bevy 变更检测
fn on_health_changed(
    query: Query<(&Health, Entity), Changed<Health>>,
) { ... }

// 🟥 禁止：手写状态标记字段检测变更
#[derive(Component)]
pub struct Health {
    pub current: i32,
    pub changed: bool,  // 手动标记，效率低于 Bevy 原生检测
}
```

---

## 交叉引用

| 文档 | 关系 |
|------|------|
| `docs/architecture.md` | ECS 章节（Component、Tag Component、Hook、Observer） |
| `docs/architecture/layer-contracts.md` | 组件应归属正确的业务模块 |
| `docs/architecture/ecs_communication_rules.md` | 组件通信方式（Hook、Observer、Message） |
| `docs/architecture/plugin-design.md` | 组件通过 Plugin 注册 |
| `docs/其他/31遗漏.md` | 本文档的原始需求来源（第 184-191 行） |
