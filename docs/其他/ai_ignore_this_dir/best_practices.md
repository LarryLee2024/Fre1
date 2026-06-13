# Bevy 0.18+ 大型 SRPG 项目最佳实践

> 基于项目铁律、Bevy 0.18 特性分析、实际项目经验总结

---

## 目录

1. [架构设计原则](#一架构设计原则)
2. [ECS 数据流设计](#二ecs-数据流设计)
3. [模块化与插件系统](#三模块化与插件系统)
4. [数据驱动架构](#四数据驱动架构)
5. [属性与 Modifier 系统](#五属性与-modifier-系统)
6. [战斗系统设计](#六战斗系统设计)
7. [UI 系统设计](#七ui-系统设计)
8. [地图与寻路](#八地图与寻路)
9. [测试策略](#九测试策略)
10. [调试与可观测性](#十调试与可观测性)
11. [性能优化](#十一性能优化)
12. [Bevy 0.18 特性利用](#十二bevy-018-特性利用)
13. [代码组织规范](#十三代码组织规范)
14. [长期维护](#十四长期维护)

---

## 一、架构设计原则

### 1.1 Feature First（最高优先级）

按业务拆模块，不按技术拆模块。

```
✅ 推荐
src/
├── character/    # 角色系统
├── battle/       # 战斗系统
├── skill/        # 技能系统
├── buff/         # 状态效果
├── map/          # 地图系统
├── ui/           # 界面系统
├── ai/           # AI 系统
├── turn/         # 回合系统
└── core/         # 共享基础

❌ 避免
src/
├── components/   # 所有组件
├── systems/      # 所有系统
├── events/       # 所有事件
└── resources/    # 所有资源
```

**理由**：Feature 模块边界清晰，技术模块容易产生循环依赖。

### 1.2 Definition / Instance 分离

配置（Definition）永远独立于运行时状态（Instance）。

```rust
// Definition：配置层，不可变，可序列化
#[derive(Deserialize, Clone)]
pub struct BuffDef {
    pub id: String,
    pub name: String,
    pub modifiers: Vec<AttributeModifierDef>,
    pub duration: u32,
}

// Instance：运行时，可变，带状态
pub struct ActiveBuff {
    pub def_id: String,
    pub remaining_turns: u32,
    pub source: Entity,
}
```

**铁律**：配置文件不包含运行时状态，运行时状态不修改配置。

### 1.3 Rule / Content 分离

代码负责规则，配置负责内容。

```rust
// 代码：伤害计算规则（稳定）
pub fn calculate_damage(atk: f32, def: f32, multiplier: f32) -> i32 {
    ((atk - def) * multiplier).max(1.0) as i32
}

// 配置：具体伤害数值（频繁调整）
// assets/skills/fireball.ron
(
    id: "fireball",
    damage_multiplier: 1.5,
    effects: ["burn"],
)
```

**新增内容优先修改配置，不修改逻辑代码。**

### 1.4 Logic / Presentation 分离

逻辑产生结果，表现负责展示。

```rust
// 逻辑层：计算伤害值
pub struct DamageApplied {
    pub target: Entity,
    pub amount: i32,
}

// 表现层：播放受击动画（监听事件）
fn play_hit_animation(query: Query<&DamageApplied>) { ... }
```

**铁律**：伤害计算不能依赖 UI 和特效。

---

## 二、ECS 数据流设计

### 2.1 Entity 只是 ID

```rust
// ❌ 错误：把 Entity 当对象
player_entity.hp -= damage;

// ✅ 正确：通过 Component 访问
fn apply_damage(
    mut query: Query<&mut Attributes>,
    damage: Res<DamageEvent>,
) {
    if let Ok(mut attrs) = query.get_mut(damage.target) {
        attrs.hp -= damage.amount;
    }
}
```

### 2.2 Component 存状态，System 存行为

```rust
// Component：纯数据
#[derive(Component)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

// System：纯逻辑
pub fn regen_health(mut query: Query<&mut Health>) {
    for mut health in &mut query {
        health.current = (health.current + 1).min(health.max);
    }
}
```

### 2.3 通信机制分层

| 机制 | 用途 | 示例 |
|------|------|------|
| **Hook** | 组件固有行为 | `Dead` 添加时自动移除移动组件 |
| **Observer** | 局部响应 | 角色死亡后播放动画 |
| **Message** | 跨 Feature 广播 | 回合结束、战斗结束 |

```rust
// Hook：组件级别的自动行为
impl Hook for Dead {
    fn on_add(entity: Entity, world: &mut World) {
        // 自动移除移动相关组件
        world.entity_mut(entity).remove::<Movable>();
    }
}

// Observer：事件响应
app.add_observer(|trigger: Trigger<DeathEvent>, mut commands: Commands| {
    commands.entity(trigger.target()).insert(Dead);
});

// Message：跨模块广播
#[derive(Message)]
pub struct TurnEnded {
    pub faction: Faction,
}
```

### 2.4 Tag Component 优于 bool

```rust
// ❌ 避免
#[derive(Component)]
pub struct Unit {
    pub is_dead: bool,
    pub is_stunned: bool,
    pub is_invulnerable: bool,
}

// ✅ 推荐
#[derive(Component)]
pub struct Dead;

#[derive(Component)]
pub struct Stunned;

#[derive(Component)]
pub struct Invulnerable;
```

### 2.5 Required Components 表达依赖

```rust
#[derive(Component)]
#[require(Health, Attributes, Faction)]
pub struct Unit;
```

### 2.6 Resource 不是全局变量仓库

```rust
// ❌ 错误：所有状态都塞进 Resource
#[derive(Resource)]
pub struct GameState {
    pub player_hp: i32,
    pub enemy_count: u32,
    pub current_turn: u32,
    pub score: i32,
}

// ✅ 正确：只保存真正的全局状态
#[derive(Resource)]
pub struct TurnState {
    pub current_faction: Faction,
    pub turn_number: u32,
}
```

---

## 三、模块化与插件系统

### 3.1 Plugin 职责单一

```rust
// 每个 Plugin 只负责一个业务领域
pub struct BuffPlugin;
impl Plugin for BuffPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BuffRegistry::load_from_dir("assets/buffs"))
           .add_systems(OnEnter(TurnPhase::Buff), tick_buffs);
    }
}
```

### 3.2 模块只暴露公共接口

```rust
// lib.rs
pub mod buff {
    pub use domain::BuffRegistry;
    pub use instance::{ActiveBuffs, apply_buff};
    // 内部实现不暴露
}

// domain.rs
pub struct BuffRegistry { ... }  // pub
struct InternalCache { ... }     // 私有
```

### 3.3 跨模块通信通过 Message/Observer

```rust
// ❌ 错误：直接访问其他模块内部
use crate::character::internal::heal_entity;

// ✅ 正确：通过事件通信
events.send(HealEvent { target, amount });
```

---

## 四、数据驱动架构

### 4.1 统一配置格式

推荐使用 RON（Rusty Object Notation）：

```rust
// assets/buffs/burn.ron
(
    id: "burn",
    name: "灼烧",
    duration: 3,
    modifiers: [
        (kind: Defense, op: Add, value: -2.0),
    ],
    dot_damage: 2,
    tags: [DEBUFF, BURN, FIRE],
)
```

### 4.2 Registry 模式

```rust
#[derive(Resource, Default)]
pub struct BuffRegistry {
    pub buffs: HashMap<String, BuffData>,
}

impl RegistryLoader for BuffRegistry {
    type Item = BuffDef;

    fn register_item(&mut self, item: BuffDef) {
        self.buffs.insert(item.id.clone(), item.into());
    }

    fn register_defaults(&mut self) {
        // 内置默认数据
    }
}
```

### 4.3 配置兼容性优先

```rust
#[derive(Deserialize)]
pub struct BuffDef {
    #[serde(default)]
    pub version: u32,  // 版本字段用于兼容
    pub id: String,
    // ...
}
```

### 4.4 热重载支持

```toml
# Cargo.toml
[features]
dev = ["bevy/file_watcher"]
```

```rust
// 开发期启用热重载
#[cfg(feature = "dev")]
app.add_plugins(bevy::asset::AssetPlugin {
    watch_for_changes: true,
    ..default()
});
```

---

## 五、属性与 Modifier 系统

### 5.1 Primary / Derived 分离

```rust
// Primary：基础属性，可直接修改
#[derive(Component)]
pub struct PrimaryStats {
    pub strength: f32,
    pub vitality: f32,
    pub agility: f32,
}

// Derived：派生属性，实时计算
pub struct DerivedStats {
    pub attack: f32,   // = strength * 2
    pub defense: f32,  // = vitality * 1.5
    pub max_hp: f32,   // = vitality * 10
}
```

### 5.2 统一 Modifier 管线

```rust
#[derive(Clone)]
pub struct Modifier {
    pub source: ModifierSource,
    pub stat: StatKind,
    pub op: ModifierOp,  // Add, Multiply, Override
    pub value: f32,
}

pub fn calculate_final(stat: f32, modifiers: &[Modifier]) -> f32 {
    let mut add_sum = 0.0;
    let mut mul_product = 1.0;

    for m in modifiers {
        match m.op {
            ModifierOp::Add => add_sum += m.value,
            ModifierOp::Multiply => mul_product *= (1.0 + m.value),
            ModifierOp::Override => return m.value,
        }
    }

    (stat + add_sum) * mul_product
}
```

### 5.3 不要到处直接修改最终属性

```rust
// ❌ 错误：直接修改
attrs.attack += 5.0;

// ✅ 正确：通过 Modifier 管线
modifier_queue.push(Modifier {
    source: ModifierSource::Buff("attack_up".into()),
    stat: StatKind::Attack,
    op: ModifierOp::Add,
    value: 5.0,
});
```

---

## 六、战斗系统设计

### 6.1 Pipeline 模式

```rust
pub struct BattlePipeline;

impl BattlePipeline {
    pub fn execute_turn(world: &mut World) {
        // 1. 检查胜利条件
        Self::check_victory(world);

        // 2. 处理当前单位行动
        Self::process_action(world);

        // 3. 结算持续效果（DoT/HoT）
        Self::tick_status_effects(world);

        // 4. 检查死亡
        Self::check_deaths(world);

        // 5. 推进回合
        Self::advance_turn(world);
    }
}
```

### 6.2 Effect Queue 模式

```rust
#[derive(Resource, Default)]
pub struct EffectQueue {
    pub pending: Vec<PendingEffect>,
}

pub struct PendingEffect {
    pub source: Entity,
    pub target: Entity,
    pub data: EffectData,
}

// 系统只消费 EffectQueue，不直接修改状态
pub fn execute_effects(
    mut queue: ResMut<EffectQueue>,
    mut query: Query<&mut Attributes>,
) {
    for effect in queue.pending.drain(..) {
        if let Ok(mut attrs) = query.get_mut(effect.target) {
            apply_effect(&mut attrs, &effect.data);
        }
    }
}
```

### 6.3 Buff / Debuff 统一机制

```rust
// Buff 本质是临时 Modifier
pub struct ActiveBuff {
    pub def_id: String,
    pub modifiers: Vec<Modifier>,
    pub remaining_turns: u32,
    pub source: Entity,
}

// 净化 = 移除所有带 DEBUFF 标签的 Modifier
pub fn cleanse(target: Entity, world: &mut World) {
    // 移除所有 debuff modifiers
}
```

---

## 七、UI 系统设计

### 7.1 UI 只展示状态

```rust
// UI 系统只读取数据，不修改业务状态
pub fn update_health_bar(
    query: Query<(&Health, &mut HealthBar)>,
) {
    for (health, mut bar) in &query {
        bar.0 = health.current as f32 / health.max as f32;
    }
}
```

### 7.2 业务逻辑不要直接操作 UI

```rust
// ❌ 错误：战斗系统直接操作 UI
fn apply_damage(
    mut health_query: Query<&mut Health>,
    mut ui_query: Query<&mut HealthBar>,
) {
    // 业务逻辑和 UI 耦合
}

// ✅ 正确：通过事件解耦
fn apply_damage(
    mut health_query: Query<&mut Health>,
    mut events: EventWriter<DamageApplied>,
) {
    // 只处理业务逻辑
    events.send(DamageApplied { target, amount });
}

// UI 系统单独监听事件
fn on_damage_applied(query: Query<&DamageApplied, Changed<DamageApplied>>) {
    // 更新 UI
}
```

### 7.3 优先使用官方 Widget

```rust
// Bevy 0.18+ 官方 UI Widget 体系
use bevy::ui::widgets::{Button, Text, Node};

// 关注 bevy_feathers 组件库发展
// 关注 bevy_ui_widgets 未来方向
```

### 7.4 Input Focus 统一管理

```rust
// 使用 bevy_input_focus 管理焦点
app.add_plugins(bevy::input_focus::InputFocusPlugin);

// 防止输入冲突：对话框打开时禁用游戏输入
fn handle_focus(focus: Res<InputFocus>) {
    if focus.is_ui_focused() {
        // 禁用游戏输入
    }
}
```

---

## 八、地图与寻路

### 8.1 地图优先看成 Grid 数据

```rust
#[derive(Resource)]
pub struct TerrainGrid {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<TerrainId>,
}

impl TerrainGrid {
    pub fn get(&self, x: i32, y: i32) -> Option<&TerrainId> {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            Some(&self.tiles[(y * self.width as i32 + x) as usize])
        } else {
            None
        }
    }
}
```

### 8.2 OccupancyGrid 独立存在

```rust
#[derive(Resource, Default)]
pub struct OccupancyGrid {
    pub cells: HashMap<IVec2, Entity>,
}

impl OccupancyGrid {
    pub fn is_occupied(&self, coord: IVec2) -> bool {
        self.cells.contains_key(&coord)
    }

    pub fn get_entity(&self, coord: IVec2) -> Option<Entity> {
        self.cells.get(&coord).copied()
    }
}
```

### 8.3 寻路数据运行时生成

```rust
// A* 寻路
pub fn find_path(
    start: IVec2,
    end: IVec2,
    grid: &TerrainGrid,
    occupancy: &OccupancyGrid,
) -> Option<Vec<IVec2>> {
    // 实现 A* 算法
}
```

### 8.4 地图数据与渲染分离

```rust
// 数据层：TerrainGrid（逻辑）
// 渲染层：TileSprite（表现）
// 两者通过 Entity 关联
```

---

## 九、测试策略

### 9.1 三级测试体系

| 级别 | 目标 | 示例 |
|------|------|------|
| **单元测试** | 验证规则 | 伤害计算、Buff 效果、属性修饰 |
| **集成测试** | 验证 Feature | 装备系统、背包系统、战斗流程 |
| **场景测试** | 验证玩家流程 | 完整战斗回合、技能释放链路 |

### 9.2 单元测试：纯函数优先

```rust
#[test]
fn 伤害计算_基础攻击() {
    let dmg = calculate_damage(10.0, 3.0, 1.0);
    assert_eq!(dmg, 7);
}

#[test]
fn buff_攻击力增加() {
    let mut attrs = Attributes::default();
    let buff = BuffData {
        modifiers: vec![Modifier {
            stat: StatKind::Attack,
            op: ModifierOp::Add,
            value: 5.0,
        }],
    };
    apply_buff(&mut attrs, &buff);
    assert_eq!(attrs.attack, 15.0);
}
```

### 9.3 集成测试：使用 Mock

```rust
#[test]
fn 装备系统_装备武器增加攻击力() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // 使用 Mock 数据，不依赖文件系统
    let weapon = WeaponDef {
        id: "sword",
        modifiers: vec![Modifier {
            stat: StatKind::Attack,
            op: ModifierOp::Add,
            value: 10.0,
        }],
    };

    // 测试逻辑
    let mut equipment = Equipment::default();
    equip(&mut equipment, &weapon);
    assert_eq!(equipment.total_attack_bonus(), 10.0);
}
```

### 9.4 避免测试依赖文件系统

```rust
// ❌ 错误：测试依赖外部文件
#[test]
fn 测试buff加载() {
    let registry = BuffRegistry::load_from_dir("assets/buffs");
    assert!(registry.get("burn").is_some());
}

// ✅ 正确：使用默认数据
#[test]
fn 测试buff加载() {
    let mut registry = BuffRegistry::default();
    registry.register_defaults();
    assert!(registry.get("burn").is_some());
}
```

### 9.5 Snapshot 测试

```rust
use insta::assert_snapshot;

#[test]
fn 伤害计算_snapshot() {
    let result = calculate_damage(10.0, 3.0, 1.5);
    assert_snapshot!(result);
}
```

---

## 十、调试与可观测性

### 10.1 统一使用 tracing

```rust
use tracing::{info, warn, error, debug};

info!(
    target: "battle",
    damage = %amount,
    target = ?target_entity,
    "伤害已应用"
);
```

### 10.2 Gizmos 调试可视化

> **注意**：`rect_2d()` 只绘制线框轮廓，**不会**绘制填充矩形。如需半透明填充叠加层，请使用 Sprite 实体。

```rust
// 寻路调试（仅线框轮廓）
fn draw_path(gizmos: &mut Gizmos, path: &[IVec2]) {
    for coord in path {
        gizmos.rect_2d(
            Isometry2d::from_translation(coord.as_vec2() * TILE_SIZE),
            Vec2::splat(TILE_SIZE * 0.9),
            Color::srgba(0.0, 1.0, 0.0, 0.3),
        );
    }
}

// 攻击范围调试（仅线框轮廓）
fn draw_attack_range(gizmos: &mut Gizmos, range: &AttackRange) {
    for coord in range.coords() {
        gizmos.rect_2d(
            Isometry2d::from_translation(coord.as_vec2() * TILE_SIZE),
            Vec2::splat(TILE_SIZE * 0.9),
            Color::srgba(1.0, 0.0, 0.0, 0.3),
        );
    }
}

// 如需半透明填充叠加层，使用 Sprite 实体
fn spawn_filled_overlay(
    commands: &mut Commands,
    coord: IVec2,
    color: Color,
) {
    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::splat(TILE_SIZE * 0.9)),
            ..default()
        },
        Transform::from_translation(coord.as_vec2().extend(1.0)),
    ));
}
```

### 10.3 bevy_inspector_egui 集成

```rust
app.add_plugins(WorldInspectorPlugin::new());
```

### 10.4 关键系统支持单步执行

```rust
// 使用 bevy_debug_stepping
app.add_plugins(bevy::debug_stepping::DebugSteppingPlugin);
```

---

## 十一、性能优化

### 11.1 先正确，再优化

```rust
// 1. 先写出正确代码
// 2. 用 profiler 找热点
// 3. 只优化热点
```

### 11.2 Changed 过滤优于全量扫描

```rust
// ❌ 每帧遍历所有实体
fn update_all_health_bars(query: Query<(&Health, &mut HealthBar)>) { ... }

// ✅ 只处理变化的实体
fn update_changed_health_bars(query: Query<(&Health, &mut HealthBar), Changed<Health>>) { ... }
```

### 11.3 Reflect 不要参与高频计算

```rust
// ❌ 运行时用反射计算属性
let value = attrs.get_reflected("attack");

// ✅ 编译时确定
let value = attrs.attack;
```

### 11.4 缓存必须定义失效条件

```rust
#[derive(Resource)]
pub struct PathCache {
    pub paths: HashMap<(IVec2, IVec2), Vec<IVec2>>,
    pub dirty: bool,  // 缓存失效条件
}

// 当地图变化时设置 dirty = true
```

### 11.5 Feature 裁剪

```toml
# 开发期
[features]
dev = ["bevy/file_watcher", "bevy-inspector-egui"]

# 发布期：关闭调试功能
default = []
```

---

## 十二、Bevy 0.18 特性利用

### 12.1 必开 Feature

> **注意**：`2d` 元 Feature 已包含以下子 Feature，无需重复列出：
> `bevy_state`、`bevy_ui`、`bevy_text`、`bevy_sprite`、`bevy_gizmos`、`bevy_log`、`bevy_picking`、`sprite_picking`、`ui_picking`、`bevy_input_focus`、`reflect_auto_register`、`png`、`multi_threaded`。

```toml
bevy = { version = "0.18", features = [
    "2d",                # 2D 元 Feature（已包含上述子 Feature）
    "file_watcher",      # 热重载
    "serialize",         # 序列化
] }
```

### 12.2 强烈推荐 Feature

```toml
"bevy_scene",                 # 存档/Replay（0.19 中更名为 bevy_world_serialization）
"reflect_auto_register",     # Inspector 支持
"bevy_debug_stepping",       # 单步调试
"bevy_dev_tools",            # 开发工具
```

### 12.3 bevy_picking 用于点击检测

```rust
// 替代手动射线检测
app.add_plugins(bevy::picking::PluginGroup);

// 选格子、选角色、选敌人统一处理
fn handle_pick(
    query: Query<&PickSelection>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        // 处理点击
    }
}
```

### 12.4 bevy_state 管理游戏状态

```rust
#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    Loading,
    MainMenu,
    InGame,
    Battle,
    Dialogue,
}

// 状态切换自动触发 OnEnter/OnExit
app.add_systems(OnEnter(AppState::Battle), setup_battle);
app.add_systems(OnExit(AppState::Battle), cleanup_battle);
```

### 12.5 bevy_scene 用于存档

```rust
use bevy::scene::DynamicSceneBuilder;

// 存档：使用 DynamicSceneBuilder 提取实体并序列化为 RON
fn save_game(world: &mut World) -> Option<String> {
    let scene = DynamicSceneBuilder::from_world(world)
        .extract_entities(world.iter_entities().map(|e| e.id()))
        .build();
    scene.serialize_ron().ok()
}
```

---

## 十三、代码组织规范

### 13.1 文件大小警戒

- 单文件超过 **500 行**开始警觉
- 单文件超过 **1000 行**必须拆分
- 单函数超过 **100 行**开始警觉

### 13.2 避免巨型文件

```
❌ 避免
src/battle/systems.rs      # 所有战斗系统
src/character/components.rs # 所有角色组件
src/utils.rs               # 工具函数垃圾桶

✅ 推荐
src/battle/damage.rs       # 伤害计算
src/battle/effects.rs      # 效果结算
src/buff/domain.rs         # Buff 定义
src/buff/instance.rs       # Buff 实例
```

### 13.3 函数设计

```rust
// ✅ 函数像目录，不像小说
fn validate_action(action: &Action) -> Result<(), ActionError> {
    let target = acquire_target(action)?;
    let cost = calculate_cost(action)?;
    validate_resources(cost)?;
    Ok(())
}

// 一个函数一个主要职责
// 函数名描述意图，不描述过程
// 优先 Early Return
```

### 13.4 内部辅助函数

```rust
// 使用 _ 前缀表示模块内部使用
fn _internal_helper() { ... }

// 或通过私有可见性
fn private_helper() { ... }  // 不导出即可
```

---

## 十四、长期维护

### 14.1 代码首先写给人看

```rust
// ❌ 聪明但难懂
let r = a.iter().filter(|x| x.0 > 0).map(|x| x.1).sum::<f32>();

// ✅ 明确且易读
let total_hp: f32 = units
    .iter()
    .filter(|unit| unit.is_alive())
    .map(|unit| unit.health.current)
    .sum();
```

### 14.2 简单优于优雅

```rust
// ❌ 过度抽象
trait DamageCalculator {
    fn calculate(&self, ctx: &DamageContext) -> DamageResult;
}

// ✅ 足够简单
fn calculate_damage(atk: f32, def: f32) -> i32 {
    (atk - def).max(1.0) as i32
}
```

### 14.3 删代码通常比写代码更有价值

```rust
// 定期审查：哪些代码没用到？
// 删除死代码 > 注释掉死代码
```

### 14.4 架构定期复盘

```markdown
# 每季度架构复盘清单
- [ ] 模块边界是否清晰？
- [ ] 是否有循环依赖？
- [ ] 配置格式是否稳定？
- [ ] 测试覆盖率是否足够？
- [ ] 工具链是否需要升级？
```

---

## 附录：项目检查清单

### 启动新功能前

- [ ] 是否符合 Feature First 原则？
- [ ] Definition / Instance 是否分离？
- [ ] 是否可以通过配置实现，而非硬编码？
- [ ] 是否有现成的官方能力可以复用？

### 代码审查时

- [ ] 单文件是否超过 500 行？
- [ ] 函数是否超过 100 行？
- [ ] 是否有循环依赖？
- [ ] 测试是否覆盖核心规则？
- [ ] 是否使用了 tracing 记录关键状态变化？

### 发布前

- [ ] 关闭调试 Feature
- [ ] 移除 println! 日志
- [ ] 检查序列化兼容性
- [ ] 运行完整测试套件
- [ ] 性能 Profile 通过

---

> **核心原则**：大多数独立游戏死于复杂度，而不是性能。优先保证代码可维护，再考虑优化。
