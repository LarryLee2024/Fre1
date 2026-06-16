---
id: 01-architecture.component-design-rules
title: Component Design Rules
status: draft
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - architecture
  - design
  - rules
---

# Component 设计规范

Version: 2.0
Status: Proposed
Source: `docs/其他/31遗漏.md` 第一节、`docs/其他/43.md` — 深度技术审查
Related: `docs/01-architecture/README.md` ECS 章节、`docs/01-architecture/00-overview/layer-contracts.md`

> **优化来源**: `docs/其他/43.md` — Definition Component 第四类、Hook 跨组件安全、Change Detection 防御性编程、数据视图方法

---

## 概述

本文档定义 Bevy ECS 项目中 Component 的分类体系、设计约束、命名规范和禁止事项。

核心问题：Bevy 项目后期 90% 的混乱，都始于组件越写越大、职责越混，最终变成不可维护的大泥球。

本规范建立"四位一体"组件分类系统，将所有 Component 划分为四个明确类别，每类有严格的职责边界。

---

## 组件分类体系（四位一体）

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

// 🟩 推荐：使用 Is 前缀的 Marker（阵营/属性相关）
#[derive(Component)]
pub struct IsAlly;

#[derive(Component)]
pub struct IsEnemy;

#[derive(Component)]
pub struct IsFlying;

#[derive(Component)]
pub struct IsDead;

// 🟩 允许：临时状态 Marker（不加 Is 前缀）
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
- 🟥 **绝对禁止**包含任何**修改自身状态**的业务逻辑方法（如 `take_damage`、`apply_buff`）
- 🟩 **允许**纯读取的数据视图方法（如 `is_alive()`、`ratio()`），只要方法不修改 `&mut self`
- 🟩 字段应为 `pub`，通过 System 直接读写
- 🟩 不包含跨模块协调逻辑
- 🟩 每个 Data Component 只负责一个数据关注点

> **优化来源**: `docs/其他/43.md` — Data Component 的"纯数据"定义过严，需区分"业务逻辑"与"数据视图"

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

**允许的纯读取数据视图方法**：

```rust
#[derive(Component)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    #[inline(always)]
    pub fn is_alive(&self) -> bool { self.current > 0 }  // ✅ 纯读取视图

    #[inline(always)]
    pub fn ratio(&self) -> f32 { self.current as f32 / self.max as f32 }  // ✅ 纯计算

    pub const fn max_limit() -> i32 { 9999 }  // ✅ 常量

    // 🟥 禁止：修改自身状态（必须放在 System 中）
    // pub fn take_damage(&mut self, amount: i32) {
    //     self.current = (self.current - amount).max(0);
    // }
}
```

**原则**：只要方法不修改 `&mut self`，且不访问外部 World/Assets，纯读取的辅助方法是鼓励的。

**禁止的 Data Component**：
```rust
// 🟥 禁止：数据组件包含修改自身状态的业务逻辑方法
#[derive(Component)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    pub fn take_damage(&mut self, amount: i32) {  // 🟥 业务逻辑应放在 System 中
        self.current = (self.current - amount).max(0);
    }
}

// 🟥 禁止：数据组件修改其他 Component
#[derive(Component)]
pub struct Attributes { ... }

impl Attributes {
    pub fn apply_buff(&self, buff: &BuffData) {  // 🟥 跨模块协调
        // ...
    }
}
```

**判断标准**：
> 如果这个组件需要存储"某个东西的值"，但不需要自己做任何修改处理 → Data Component。

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

### 4. Definition Component（定义组件）

> **优化来源**: `docs/其他/43.md` — SRPG 大量静态配置（UnitClassDef、SkillDef）需要第四类组件

**定义**：从 AssetServer 加载的静态配置数据，运行时绝对不可变。通常作为 Resource 或挂载在预制体 Entity 上。

**特征**：
- 🟥 **绝对禁止**在 System 中获取 Definition Component 的 `&mut` 引用
- 🟥 **绝对禁止**运行时修改 Definition Component 的任何字段
- 🟩 命名必须以 `Def` 或 `Config` 结尾（如 `UnitClassDef`、`SkillConfig`），以区分运行时的 Instance 数据
- 🟩 从 AssetServer 加载，通过 `Res<T>` 或只读 `&T` 访问

**允许的 Definition Component**：
```rust
#[derive(Component, Reflect)]
pub struct UnitClassDef {
    pub class_id: String,
    pub base_stats: Stats,
    pub skill_slots: Vec<SkillId>,
}

#[derive(Component, Reflect)]
pub struct SkillDef {
    pub skill_id: String,
    pub name_key: String,       // 本地化 Key，非硬编码文本
    pub desc_key: String,
    pub damage: i32,
    pub cooldown: u32,
    pub element: Element,
}

#[derive(Resource, Reflect)]
pub struct BattleRulesDef {
    pub turn_order: TurnOrderRule,
    pub damage_formula: DamageFormula,
    pub max_team_size: usize,
}
```

**禁止的 Definition Component**：
```rust
// 🟥 禁止：运行时修改 Definition
fn bad_system(mut skill_def: ResMut<SkillDef>) {
    skill_def.damage += 10; // 🟥 绝对禁止！Definition 不可变
}

// 🟥 禁止：Definition 命名不规范
#[derive(Component)]
pub struct UnitClassData { }  // 应使用 UnitClassDef
```

**判断标准**：
> 如果这个组件是从文件加载的静态配置，运行时不需要修改 → Definition Component。

---

## 组件命名规范

### 命名模式：`[Entity][Concept]`

| 类别 | 模式 | 示例 |
|------|------|------|
| Marker | `IsXxx` / `Xxx` | `IsAlly`, `IsEnemy`, `IsFlying`, `IsDead`, `Dead`, `Selected` |
| Data | `EntityConcept` | `GridPosition`, `UnitName`, `UnitId`, `PersistentTags` |
| Status | `EntityAction` | `MovingUnit`, `CastingSkill`, `PendingEffect` |
| Definition | `EntityDef` / `EntityConfig` | `UnitClassDef`, `SkillConfig`, `BattleRulesDef` |
| UI Marker | `XxxBg` / `XxxFg` | `HpBarBg`, `HpBarFg` |

> **优化来源**: `docs/其他/43.md` — Marker Component 命名建议统一使用 `Is` 前缀（IsAlly, IsDead, IsFlying），提升语义清晰度

**Marker 命名规则**：
- 🟩 表示阵营/属性的 Marker 统一使用 `Is` 前缀：`IsAlly`、`IsEnemy`、`IsFlying`、`IsDead`
- 🟩 表示临时状态的 Marker 可不加 `Is` 前缀：`Dead`、`Selected`、`Acted`、`Frozen`、`Stunned`
- 🟥 **禁止**使用 `XxxManager`、`XxxData`、`XxxState` 等 OOP 风格命名

---

## 单一职责原则

### 规则

🟥 **一个 Component = 一个数据关注点。**

### 8 字段拆分阈值（参考阈值）

> **优化来源**: `docs/其他/43.md` — 8 字段是参考阈值，核心是关注点分离，非硬性规则

**8 字段是"参考阈值"，不是硬性规则**。核心判断标准是"关注点分离"：

```
Component 是否需要拆分？
├─ 超过 8 个字段？→ 通常必须拆分（除非所有字段属于同一关注点）
├─ 字段分为明显不同的关注点？→ 即使不足 8 字段也必须拆分
├─ 部分字段的生命周期不同？→ 拆分（如逻辑坐标 vs 表现坐标）
├─ 字段的修改频率差异大？→ 拆分（高频变更 vs 低频变更）
├─ 极端性能场景且经团队评审？→ 可允许少量字段合并（需记录理由）
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

### 版本号升级规则

> **优化来源**: `docs/其他/43.md` — 版本号升级规则与数据迁移示例

| 变更类型 | 版本号变化 | 示例 |
|---------|-----------|------|
| 新增字段（有默认值） | 小版本 +1（如 1.0 → 1.1） | `v2 新增 critical_rate: f32 = 0.0` |
| 删除字段 | 大版本 +1（如 1.x → 2.0） | 移除已废弃字段 |
| 字段类型变更 | 大版本 +1 | `cooldown: u32 → cooldown: f32` |

**数据迁移示例**：
```rust
// v1 → v2 迁移逻辑
fn migrate_skill_cooldowns(v1: SkillCooldownsV1) -> SkillCooldownsV2 {
    SkillCooldownsV2 {
        version: 2,
        cooldowns: v1.cooldowns,
        new_field: Default::default(), // 兼容旧数据，使用默认值填充
    }
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
| 🟥 Data Component 包含修改自身状态的方法 | 违反 ECS 数据/逻辑分离原则 | 业务逻辑放在 System 中 |
| 🟥 Status Component 在状态退出后残留 | 导致系统误判，状态不一致 | 在 OnExit 系统中自动清理 |
| 🟥 跨领域的"上帝组件" | 破坏模块边界，修改一个字段影响所有依赖方 | 按关注点拆分为多个组件 |
| 🟥 组件超过 8 个字段不拆分 | 职责混杂，修改频率不同导致维护困难 | 按生命周期和关注点拆分 |
| 🟥 可序列化组件无 version 字段 | 无法处理数据迁移 | 添加 version 字段 |
| 🟥 手写 `is_xxx: bool` 字段 | Bevy 位掩码优化失效，查询性能下降 | 使用 Marker Component |
| 🟥 组件命名使用 `XxxManager` | OOP 风格封装 Entity，违反 ECS 原则 | 使用纯数据 Component + System |
| 🟥 在 Hook 中修改其他 Component | 形成隐式数据依赖，可能导致死锁或 Panic | 通过 Observer/Event 协调跨组件联动 |
| 🟥 System 中无脑获取 `&mut T` | 即使值没变也会触发 Changed，导致下游 System 每帧执行 | 只需读取时用 `&T`，修改前先判断值是否改变 |
| 🟥 Definition Component 运行时修改 | 静态配置不可变，修改会导致数据不一致 | 使用 `Res<T>` 只读访问 |
| 🟥 使用 `#[require]` 跨子系统耦合 | 破坏模块边界，导致组件隐式依赖 | 通过 Plugin 批量注册或显式插入 |

---

## 允许的模式

### 模式1：Marker + Hook 组合（安全用法）

> **优化来源**: `docs/其他/43.md` — Hook 中禁止跨组件修改，必须通过 Observer/Event 协调

🟥 **绝对禁止**在 Component Hook（OnAdd/OnInsert/OnRemove）中通过 `get_mut` 修改同一 Entity 上的其他 Component。Hook 执行顺序在复杂场景下难以预测，跨组件修改会形成隐式数据依赖，可能导致死锁或 Panic。

```rust
// ✅ 安全：Hook 只触发事件，跨组件协调交给 Observer/System
#[component(on_add = Dead::on_add_dead)]
pub struct Dead;

impl Dead {
    fn on_add_dead(mut world: DeferredWorld, context: HookContext) {
        // ✅ 安全：只触发事件，让 Observer 处理跨组件逻辑
        world.commands().trigger_targets(UnitDiedEvent(context.entity), context.entity);
        
        // 🟥 禁止：在 Hook 中直接修改其他 Component
        // if let Some(mut unit) = world.get_mut::<Unit>(context.entity) {
        //     unit.acted = true;
        // }
        // world.commands().entity(context.entity).remove::<Selected>();
    }
}

// ✅ 正确：通过 Observer 监听事件，处理跨组件联动
fn on_unit_died(
    trigger: Trigger<UnitDiedEvent>,
    mut commands: Commands,
    mut query: Query<&mut Unit>,
) {
    let entity = trigger.target();
    if let Ok(mut unit) = query.get_mut(entity) {
        unit.acted = true;
    }
    commands.entity(entity).remove::<Selected>();
}
```

**规则**：
- 🟥 **禁止**在 Component Hook 中通过 `get_mut` 修改同一 Entity 上的其他 Component
- 🟥 **禁止**在 Hook 中执行阻塞操作（如资源卸载、网络请求）
- 🟩 Hook 应只做"自身清理"或"触发事件"
- 🟩 跨组件联动必须通过 Observer 监听事件，或在专属 cleanup_system 中处理

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

### 变更检测防御性编程

> **优化来源**: `docs/其他/43.md` — Changed<T> 性能陷阱，无脑 &mut 导致每帧误触发

🟥 **禁止**在 System 中无脑获取 `&mut T`。如果只需要读取，必须用 `&T`。

**规则 1**：🟥 禁止在 System 中无脑获取 `&mut T`。如果只需要读取，必须用 `&T`。

**规则 2**：修改 Data Component 前，必须先判断值是否真的改变：

```rust
// ✅ 正确：防御性修改，避免无效触发 Changed
fn update_health(
    mut query: Query<&mut Health>,
    new_values: Res<HealthUpdates>,
) {
    for mut health in &mut query {
        if health.current != new_values.hp {
            health.current = new_values.hp; // 仅在值真正改变时才赋值
        }
    }
}

// 🟥 错误：直接赋值，即使 new_hp 和 current 相同，也会触发 Changed
fn bad_update_health(
    mut query: Query<&mut Health>,
    new_hp: Res<i32>,
) {
    for mut health in &mut query {
        health.current = *new_hp; // 每帧都触发 Changed，下游 System 每帧都执行
    }
}
```

**规则 3**：使用 `item.is_changed()` 在 System 内部做二次判断：

```rust
fn process_changed_items(query: Query<(&Item, Entity), Changed<Item>>) {
    for (item, entity) in &query {
        // 即使 Changed<T> 触发了，也要检查是否真正需要处理
        if item.is_changed() {
            // 只有值真正改变时才执行昂贵操作
            expensive_calculation(entity);
        }
    }
}
```

---

## SRPG 专项组件设计建议

> **优化来源**: `docs/其他/43.md` — SRPG 回合制特性的组件专项建议

| 组件类型 | SRPG 专项建议 | 理由 |
|---------|-------------|------|
| Status Component | 必须包含 `source: Entity`（施加者）和 `duration: u32`（剩余回合） | SRPG 的 Buff/Debuff 和临时状态极多，记录来源用于计算仇恨/反击，记录持续时间用于自动清理 |
| Data Component | 坐标类组件必须区分**逻辑坐标**（`IVec2`）和**表现坐标**（`Vec3`） | 逻辑坐标用于寻路/计算（低频变更），表现坐标用于动画插值（每帧变更）。分离后可大幅减少 `Changed<GridPosition>` 的误触发 |
| Marker Component | 引入 `NeedsPathfinding`、`NeedsVisionUpdate` 等"脏标记" | 寻路和视线计算极重。通过 Marker 标记"需要重算"的单位，System 只处理带有该 Marker 的实体，计算完后移除 Marker |
| Definition Component | 技能/职业/特质的静态配置统一使用 `XxxDef` 命名 | SRPG 有大量从 RON 文件加载的配置（`SkillDef`、`UnitClassDef`、`TraitDef`），必须与运行时 Instance 数据物理隔离 |

---

## 组件设计决策树

> **优化来源**: `docs/其他/43.md` — Component 设计决策树，帮助开发者在写代码前快速判断

在添加新组件前，按以下决策树判断：

```text
我要加一个新组件：
├─ 它只是为了在 Query 中过滤实体吗？ (是/否)
│  └─ 是 → Marker Component (零字段，如 IsAlly, Dead, Selected)
├─ 它的生命周期是临时的吗？ (如：正在移动、正在施法)
│  └─ 是 → Status Component (必须有明确的 OnExit 清理逻辑)
├─ 它是从文件加载的静态配置吗？ (如：技能基础属性、职业定义)
│  └─ 是 → Definition Component (运行时只读，命名为 XxxDef)
└─ 它是实体运行时的持久状态吗？ (如：当前血量、坐标)
   └─ 是 → Data Component
       ├─ 字段 > 8 个且分属不同关注点？→ 必须拆分！
       ├─ 包含修改自身状态的方法？→ 移到 System 中！
       └─ 只有纯读取的辅助方法？→ 允许保留！
```

---

## 宪法合规说明

| 条款 | 合规状态 | 说明 |
|------|---------|------|
| 🟩 §2.1.2 数据与行为强制分离 | ✅ 合规 | Component 只存数据，业务逻辑在 System 中 |
| 🟩 §2.1.3 Tag Component 优先原则 | ✅ 合规 | Marker Component 零字段，禁止 `is_xxx: bool` |
| 🟩 §2.2.1 Hook 生命周期 | ✅ 合规 | Hook 只做自身清理或触发事件 |
| 🟩 §2.3.2 依赖声明 | ✅ 合规 | 使用 `#[require(Component)]` |
| 🟩 §2.3.3 变更检测 | ✅ 合规 | 使用 Bevy 原生 `Added/Changed/Removed` |
| 🟥 §1.1.2 定义与实例分离 | ✅ 合规 | Definition Component 运行时只读 |
| 🟥 §1.2.1 强类型 ID | ✅ 合规 | 使用 `UnitId`、`SkillId` 等强类型 |

⚠️ **§2.1.2 合规备注**：本文档允许 Data Component 包含纯读取的数据视图方法（如 `is_alive()`、`ratio()`）。宪法原文 "Component 只能存储纯数据状态，绝对禁止包含任何逻辑" 的严格解释禁止所有方法。本文档的宽松解释基于以下理由：纯 `&self` 只读方法不修改状态、不访问外部 World/Assets，本质上是数据的便捷访问器而非业务逻辑。此设计决策需在架构复盘时重新评估。

## 交叉引用

| 文档 | 关系 |
|------|------|
| `docs/01-architecture/README.md` | ECS 章节（Component、Tag Component、Hook、Observer） |
| `docs/01-architecture/00-overview/layer-contracts.md` | 组件应归属正确的业务模块 |
| `docs/02-domain/ecs_communication_rules.md` | 组件通信方式（Hook、Observer、Message） |
| `docs/01-architecture/00-overview/plugin-design.md` | 组件通过 Plugin 注册 |
| `docs/其他/31遗漏.md` | 本文档的原始需求来源（第 184-191 行） |
| `docs/AI开发宪法完整版.md` | §2.1.2 数据与行为分离、§2.1.3 Tag Component、§2.2.1 Hook、§2.3.3 变更检测 |
