---
id: 09-planning.test-battle-scene
title: TestBattle 场景实施计划 — 零美术资产可运行战斗
status: draft
owner: feature-developer
created: 2026-06-21
updated: 2026-06-21
tags:
  - scene
  - combat
  - config
  - visualization
  - placeholder
---

# TestBattle 场景实施计划

## 目标

在零美术资产的前提下，让 `cargo run` 启动后直接看到一个可操作的战斗场景：
2v2 单位在网格上，可以点 UI 操作，伤害计算真实可用。

## 核心原则：视觉与逻辑严格分离

**每个实体同时挂载逻辑组件和视觉组件，两者通过不同 system 管理，互不感知。**

```rust
// spawn_unit 产生的实体结构（括号表示组件组）：
Entity {
    // ─── [逻辑组] 战斗系统依赖，永远不变 ───
    UnitIdComponent,      // String ID，桥接层使用
    HitPoints,            // 血量
    CombatParticipant,    // 阵营
    ActionPoints,         // 行动资源
    GridPosition,         // 格子坐标 (IVec2)

    // ─── [视觉组] 渲染层，随时可整体替换 ───
    Sprite,               // 着色矩形 → 将来换 Sprite::from_image()
    Transform,            // 位置 → 将来换 GridPosition→Transform 映射
}
```

**视觉替换只需改一个 system**（`apply_grid_position_to_transform` + `load_unit_sprite`），逻辑零改动。

## 架构图

```
content/configs/scenarios/test_battle.ron
  │ 定义：单位列表(位置/阵营/HP/技能)
  ▼
app/scenes/test_battle/
  ├── def.rs              ← 反序列化结构体
  ├── spawn.rs            ← 读取 config → spawn 实体
  └── render.rs           ← 占位视觉（着色矩形 + 文字标签）
```

## 实施步骤

### Step 1: 配置定义 (def.rs + RON)

```rust
// app/scenes/test_battle/def.rs
#[derive(Debug, Clone, Deserialize)]
pub struct TestBattleDef {
    pub units: Vec<UnitEntry>,
    pub grid: GridConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnitEntry {
    pub id: String,           // "unit_hero"
    pub name_key: String,     // "unit.hero.name"
    pub team: TeamId,         // Player | Enemy
    pub coord: (i32, i32),    // 网格坐标
    pub hp: u32,              // 初始血量
    pub max_hp: u32,          // 最大血量
    pub ap: u32,              // 可用行动点
}

#[derive(Debug, Clone, Deserialize)]
pub struct GridConfig {
    pub width: u32,
    pub height: u32,
    pub cell_size: f32,
}
```

RON 配置（`assets/configs/scenarios/test_battle.ron`）：

```ron
(
    units: [
        (id: "unit_hero",   name_key: "unit.hero.name",   team: Player, coord: (1, 1), hp: 100, max_hp: 100, ap: 2),
        (id: "unit_ally",   name_key: "unit.ally.name",   team: Player, coord: (1, 2), hp: 80,  max_hp: 80,  ap: 2),
        (id: "unit_goblin", name_key: "unit.goblin.name", team: Enemy,  coord: (4, 4), hp: 40,  max_hp: 40,  ap: 1),
        (id: "unit_orc",    name_key: "unit.orc.name",    team: Enemy,  coord: (4, 5), hp: 60,  max_hp: 60,  ap: 1),
    ],
    grid: (width: 6, height: 6, cell_size: 100.0),
)
```

### Step 2: Spawn 系统 (spawn.rs)

纯函数式读取配置 → 生成实体：

```rust
pub fn spawn_test_battle(
    mut commands: Commands,
    scenario: Res<TestBattleScenario>,  // 从 RON 加载
    asset_server: Res<AssetServer>,     // 用于创建占位白图
    theme: Res<Theme>,                  // 颜色来自主题
) {
    // 1. 初始化回合队列（TurnQueue + BattleStarted event）
    // 2. 遍历 units，每个生成：
    //    - 逻辑组件：UnitIdComponent, HitPoints, CombatParticipant, ActionPoints, GridPosition
    //    - 视觉组件：Sprite(着色), Transform(按坐标定位)
    // 3. 生成网格背景（Node 格子）
}
```

**关键设计：视觉组件通过独立 system 添加，而非直接在 spawn 中耦合。**

```rust
// 方案 A（推荐）— spawn 只产生逻辑实体，视觉由另一个 system 观察新实体并添加
// 优点：替换视觉只需改一个 system，spawn 零改动
pub fn attach_unit_visuals(
    mut commands: Commands,
    new_units: Query<Entity, (With<UnitIdComponent>, Without<Sprite>)>,
    asset_server: Res<AssetServer>,
    theme: Res<Theme>,
) {
    let white_texture = create_placeholder_texture(&asset_server); // 4x4 白图
    for entity in &new_units {
        commands.entity(entity).insert((
            Sprite::from_image(white_texture.clone()).with_color(theme.colors.unit_player),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
    }
}
```

### Step 3: 占位视觉 (render.rs)

| 元素 | 实现方式 | 替换方式 |
|------|---------|---------|
| 单位 | `Sprite` + 白图 tint 着色 | 纹理句柄 → AssetServer 加载真实贴图 |
| 网格 | `Gizmos::grid` 或 `Sprite` 格子背景 | 换 Tilemap |
| 单位名 | `Text2D`（永远保留，debug 用）| 样式/字体可调 |
| HP 条 | `Sprite` 比例缩放 | 去掉/换为 HUD 面板 |

颜色全部取自 `Theme` 资源（`theme.colors.unit_player` / `theme.colors.unit_enemy`），零硬编码。

### Step 4: 接入现有场景系统

通过 `ScenePlugin` 的状态机触发：

```rust
// app/scenes/ 增加 TestBattle 变体
pub enum SceneType {
    TestBattle,   // ← 新增
    MainMenu,
    InGame,
    // ...
}

// OnEnter(SceneType::TestBattle) → spawn_test_battle
// OnExit(SceneType::TestBattle) → despawn 所有战斗实体
```

### Step 5: 验证闭环

```
cargo run
  → App 启动
  → 场景系统自动进入 TestBattle
  → 网格 + 4 个单位可见（着色矩形 + 名字标签）
  → 可以操作：选单位 → EndTurn → 下一个单位高亮
  → 攻击敌方 → 伤害计算 → HP 变化 → 死亡 → Dead 标签
```

## 替换用例

| 需求变化 | 需要改什么 |
|---------|-----------|
| 换单位贴图为真实美术 | 改 `attach_unit_visuals` 中的 `Sprite` 创建方式 |
| 换网格为 Tilemap | 改网格生成 system |
| 换 2D 为 3D 模型 | 加 `SceneBundle`，换 `Transform` + `Sprite` → `Scene` |
| 加动画 | 加 `AnimationPlayer` 组件（视觉 system 加，逻辑 system 不知） |
| 换另一个战斗配置 | 换 RON 文件或改 `TestBattleScenario` 资源路径 |
| 正式场景对接关卡文件 | 换 `def.rs` 的数据源指正 (RON → LevelDef)，spawn 逻辑不动 |

## 不变量

1. **零硬编码颜色/数值** — 全部来自配置或 Theme
2. **视觉可整体切除** — 删除 `render.rs` 和相关 system，游戏逻辑仍完整运行
3. **不修改现有域代码** — 所有改动限 `app/scenes/test_battle/` 和 `assets/configs/`
4. **所有 1895+ 测试通过** — 不对现有测试造成影响
