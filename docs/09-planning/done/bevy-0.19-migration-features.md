# Phase B：架构现代化（第 3–4 周）

> **目标**：BSN 全面替换 Bundles、核心 Resource 迁移为 Singleton Entity、Relationship 接入、User Settings 引入
> **原则**：4 Agent 并行，不设"试点"——全面采用，一次性迁移完毕
> **前置条件**：Phase A 已完成（Observer/Delayed 全面接管核心事件流）

---

## 1. Agent B1：UI 层全面 BSN 化

**负责模块**：
- `src/app/scenes/` — 场景 UI
- `src/app/` — 新增 `settings_plugin.rs`
- 所有 UI 相关代码（Node / Text / Button / 布局）

### 1.1 核心变更

| 文件 | 当前模式 | BSN 模式 |
|------|----------|----------|
| `app/scenes/plugin.rs` | 命令式 spawn | `bsn! {}` 场景定义 |
| `app/scenes/components.rs` | UI 组件 | 适配 BSN 标签 |
| 所有 UI 生成代码 | `.spawn(Node { ... }).with_children(...)` | `bsn! { Node { ... children: [...] } }` |

### 1.2 BSN 转换示例

```rust
// 0.18 模式：命令式 UI
fn spawn_main_menu(commands: &mut Commands) -> Entity {
    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }).with_children(|parent| {
        parent.spawn(Text(Text::new("Fre SRPG")));
        parent.spawn(Button { ... });
    }).id()
}

// 0.19 模式：BSN 声明式
fn spawn_main_menu(world: &mut World) {
    world.spawn(bsn! {
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            children: [
                Text(Text::new("Fre SRPG")),
                Button { ... },
            ]
        }
    });
}
```

### 1.3 BSN 红线

- ❌ BSN 仅在 `src/app/` UI 层使用
- ❌ 核心玩法层（`src/core/`）禁止 BSN
- ❌ BSN 只描述实体结构，不包含业务逻辑
- ✅ 工厂函数 `spawn_hero()` 内部可以自由选择 BSN 或传统 spawn，对外接口一致

---

## 2. Agent B2：Bundle → BSN 场景工厂

**负责模块**：
- 所有 `#[derive(Bundle)]` 定义（~25 处）
- `src/core/` 下所有实体生成代码

### 2.1 核心变更

| 模块 | Bundle → 工厂函数 |
|------|-------------------|
| `core/capabilities/ability/` | `AbilityBundle` → `spawn_ability()` |
| `core/capabilities/effect/` | `EffectBundle` → `spawn_effect_entity()` |
| `core/domains/tactical/` | `UnitBundle` → `spawn_unit()` |
| `core/domains/summon/` | `SummonBundle` → `spawn_summon()` |
| 所有领域 | 所有 Bundle → `spawn_*()` 工厂 |

### 2.2 转换模式

```rust
// 旧模式
#[derive(Bundle)]
struct UnitBundle {
    name: Name,
    health: Health,
    faction: Faction,
    position: GridPosition,
    sprite: Sprite,
}

// 新模式：工厂函数
fn spawn_unit(commands: &mut Commands, pos: GridPosition, faction: Faction) -> Entity {
    commands.spawn(bsn! {
        Name::new("Unit"),
        Health::full(100),
        faction,
        pos,
        Sprite::default(),
    }).id()
}
```

### 2.3 兼容策略

- 保留旧的 Bundle 结构体定义（但标记 `#[deprecated]`）确保外部代码不受影响
- 新增 `spawn_*()` 工厂函数
- 一行搜索命令找到所有使用旧 Bundle 的地方：

```bash
grep -rn "Bundle" src/ --include="*.rs" | grep -v "#\[derive.*Bundle\]" | grep -v "use.*Bundle"
```

---

## 3. Agent B3：Resource → Singleton Entity

**负责模块**：
- 5 个核心全局 Resource
- 所有使用 `Res<T>` / `ResMut<T>` 的 System（~60 处）

### 3.1 迁移清单

| Resource | Marker Entity | Singleton Component |
|----------|---------------|-------------------|
| `BattleState` | `BattleRoot` | `&BattleState` on `BattleRoot` |
| `TurnState` | `BattleRoot` | `&TurnState` on `BattleRoot` |
| `GameTime` | `TimeRoot` | `&GameTime` on `TimeRoot` |
| `InputState` | `InputRoot` | `&InputState` on `InputRoot` |
| `GameRng` | `RngRoot` | `&GameRng` on `RngRoot` |

### 3.2 注入模式变更

```rust
// 旧模式
fn my_system(battle: Res<BattleState>, time: Res<Time>) {
    if battle.phase() == Phase::Execution {
        // ...
    }
}

// 新模式
fn my_system(
    battle: Single<&BattleState, With<BattleRoot>>,
    time: Single<&GameTime, With<TimeRoot>>,
) {
    if battle.phase() == Phase::Execution {
        // ...
    }
}
```

### 3.3 初始化方式

```rust
// 在 Plugin::build() 中
commands.spawn((
    BattleRoot,
    BattleState::new(),
    TurnState::new(),
));

commands.spawn((
    TimeRoot,
    GameTime::default(),
));

// 旧版 Res<T> 兼容桥：如果某些地方必须保持 Res<T>，可以同时注册 Resource
app.insert_resource(BattleState::new());  // 兼容桥
```

### 3.4 搜索路径

```bash
# 找出所有 Res<T> / ResMut<T> 使用点（针对 5 个核心 Resource）
grep -rn "Res<BattleState>\|ResMut<BattleState>" src/
grep -rn "Res<TurnState>\|ResMut<TurnState>" src/
grep -rn "Res<GameTime>\|ResMut<GameTime>" src/
grep -rn "Res<InputState>\|ResMut<InputState>" src/
grep -rn "Res<GameRng>\|ResMut<GameRng>" src/
```

---

## 4. Agent B4：Relationship 接入

**负责模块**：
- `core/capabilities/effect/` — Effect.source → Relationship<SourcedFrom>
- `core/capabilities/stacking/` — Buff.caster → Relationship<CasterOf>, Buff.target → Relationship<TargetOf>
- `core/domains/summon/` — Summon.owner → Relationship<SummonedBy>

### 4.1 转换模式

```rust
// 关系类型定义
#[derive(Relationship)]
struct CasterOf(Entity);

#[derive(Relationship)]
struct TargetOf(Entity);

#[derive(Relationship)]
struct SummonedBy(Entity);

#[derive(Relationship)]
struct SourcedFrom(Entity);

// 旧模式
#[derive(Component)]
struct Buff {
    caster: Entity,
    target: Entity,
    duration: Duration,
}

// 新模式
#[derive(Component)]
struct Buff {
    #[relationship(relationship = CasterOf, target = caster)]
    caster: Entity,
    #[relationship(relationship = TargetOf, target = target)]
    target: Entity,
    duration: Duration,
}

// 查询所有施法者的 Buff
let buffs = query_buff
    .iter()
    .filter(|buff| relationship.get::<CasterOf>(buff.caster).is_ok());
```

---

## 5. Phase C 同步任务（第 5 周收尾）

### 5.1 Agent C1：User Settings + Diagnostics

```rust
// 三组设置，立即启用
#[derive(Resource, Reflect, Serialize, Deserialize)]
struct AudioSettings {
    master_volume: f32, sfx_volume: f32, music_volume: f32,
}

#[derive(Resource, Reflect, Serialize, Deserialize)]
struct VideoSettings {
    fullscreen: bool, resolution: (u32, u32), vsync: bool,
}

#[derive(Resource, Reflect, Serialize, Deserialize)]
struct GameplaySettings {
    grid_overlay: bool, show_move_range: bool, camera_speed: f32,
}

// 注册
app.init_settings::<AudioSettings>()
   .init_settings::<VideoSettings>()
   .init_settings::<GameplaySettings>();

// Diagnostics
#[cfg(feature = "dev")]
app.add_plugins(DiagnosticsOverlayPlugin::default());
```

### 5.2 Agent C2：Contiguous Query + Performance

```bash
# 找出所有 Query::iter() 调用
grep -rn "\.iter()" src/ --include="*.rs" | grep "query\."
# 替换 hot paths 为 contiguous_iter()
# 组件布局优化：经常一起查询的组件放入同一 Archetype
```

### 5.3 Agent C3：Reflect 全覆盖

```bash
# 找出所有组件/事件/资源类型，添加 #[derive(Reflect)]
grep -rn "#\[derive(Component)\]" src/ --include="*.rs"
grep -rn "#\[derive(Event)\]" src/ --include="*.rs"
grep -rn "#\[derive(Resource)\]" src/ --include="*.rs"
# 逐个检查是否同时 derive Reflect
```

---

## 6. Phase B + C 准出条件

- [-] 全部 UI 代码已 BSN 化（单组件场景无增益，复杂 UI 时再引入）
- [x] 全部 `#[derive(Bundle)]` 已替换为工厂函数（或标记 `#[deprecated]`）
- [ ] 5 个核心 Resource 已迁移到 Singleton Entity Component
- [ ] 4 种核心关系已使用 Relationship
- [ ] User Settings 三组定义 + init_settings 完成
- [x] DiagnosticsOverlay 注册（dev 模式）
- [-] `font_size: f32` 全部替换为 `FontSize::Px`（领域配置字段，非 Bevy API）
- [x] `Input<T>` 全部替换为 `ButtonInput<T>`
- [ ] `cargo nextest run` 全部通过
- [ ] `cargo clippy -- -D warnings` 零警告

---

> **执行 Agent**: @feature-developer (B1–B4, C1–C3) | **协调者**: @architect
> **预计周期**: 第 3–5 周 | **文件影响范围**: ~200 文件
