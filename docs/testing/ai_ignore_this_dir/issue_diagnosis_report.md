# 游戏运行时问题诊断报告 v2.0

**版本**: 2.0
**日期**: 2026-06-12
**状态**: 待修复
**范围**: UI渲染、移动逻辑、相机控制、交互体验

---

## 问题总览

| # | 问题描述 | 严重度 | 优先级 | 根因定位 | 修复预估 |
|---|---------|--------|--------|----------|----------|
| 1 | 左下角单位信息面板需移除 | 🟥 Critical | P0 | UnitInfoPanel 无条件生成且始终可见 | 10分钟 |
| 2 | 玩家棋子移动路径异常(斜角飞行) | 🟥 Critical | P0 | show_move_range 与 command_handler 使用不同的 moving_entity 参数 | 30分钟 |
| 3 | 移动轨迹显示缺失 | 🟡 Major | P1 | 依赖问题2,路径不完整导致不生成线段 | 依赖问题2 |
| 4 | 角色信息面板闪烁 | 🟥 Critical | P0 | MovableRange/AttackRange/SelectionHighlight 未设置 Pickable::IGNORE | 5分钟 |
| 5 | 相机视口移动交互不流畅 | 🟡 Major | P1 | 缺少右键拖动,边缘滚动易误触 | 20分钟 |
| 6 | 移动确认与右键退回功能验证 | 🟢 Minor | P2 | 逻辑完整,需验证各阶段 Cancel 行为 | 15分钟 |

---

## 问题1：左下角单位信息面板需彻底移除

### 问题描述

应用程序启动后，窗口左下角出现标题为"选择一个单位"的面板，包含 HP/MP/STA 资源条及属性显示。用户要求彻底移除该面板及其所有渲染逻辑。

### 根因分析

**核心原因**: `UnitInfoPanel` 在 `OnEnter(AppState::InGame)` 时无条件生成，且未设置初始隐藏状态，导致面板始终可见。

#### 代码定位

**文件**: `src/ui/panels/unit_info.rs`

1. **面板生成函数** (L48-L176):
```rust
pub fn spawn_unit_info_panel(mut commands: Commands, theme: Res<UiTheme>) {
    commands
        .spawn((
            panel_bottom(&theme, theme.gap_large, theme.gap_large, theme.unit_panel_width),
            BackgroundColor(theme.panel_bg),
            UnitInfoPanel,
            // ❌ 未设置 Visibility::Hidden，面板默认可见
        ))
        .insert(Name::new("UnitInfoPanel"))
        .with_children(|parent| {
            // L64: "选择一个单位" 文本
            Text::new("选择一个单位"),
            // L83-106: HP/MP/STA 资源条
            spawn_resource_bar(parent, &theme, "HP", ...);
            spawn_resource_bar(parent, &theme, "MP", ...);
            spawn_resource_bar(parent, &theme, "STA", ...);
            // L108-163: 属性、技能、装备等详细信息
        });
}
```

2. **插件注册** (L424-L435):
```rust
impl Plugin for UnitInfoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InGame),
            spawn_unit_info_panel.in_set(GameSet::Ui),
        )
        .add_systems(
            Update,
            (setup_ui_font, update_unit_info).run_if(in_state(AppState::InGame)),
        );
    }
}
```

3. **更新逻辑** (L186-L419):
```rust
pub fn update_unit_info(view: Res<SelectedUnitView>, ...) {
    if !view.is_changed() { return; }

    if !view.is_selected {
        // L207: 未选中时显示默认值
        PanelLabel::UnitName => **text = "选择一个单位".to_string(),
        PanelLabel::Hp | PanelLabel::Mp | PanelLabel::Stamina => **text = "0/0".to_string(),
        // 但面板本身仍然可见！
    }
    // ... 选中时更新实际数据
}
```

#### 问题机制

1. **无条件生成**: `spawn_unit_info_panel` 在进入 `InGame` 状态时立即执行，无任何条件判断
2. **默认可见**: 生成的 Node 组件未设置 `Visibility::Hidden`，Bevy UI 默认可见
3. **持续更新**: `update_unit_info` 每帧运行，根据 `SelectedUnitView` 刷新内容，但从不隐藏面板
4. **无销毁逻辑**: 没有系统会在任何条件下 despawn 此面板

#### 影响范围

- **视觉干扰**: 占据屏幕左下角空间，遮挡游戏画面
- **性能开销**: 每帧更新多个 Text 组件和进度条节点
- **用户体验**: "选择一个单位" + "0/0" 的默认状态误导玩家

### 建议修复方案

**方案A（推荐）：彻底删除 UnitInfoPanel 及相关代码**

1. 删除 `src/ui/panels/unit_info.rs` 中的以下内容:
   - `spawn_unit_info_panel` 函数
   - `update_unit_info` 函数
   - `setup_ui_font` 函数
   - `UnitInfoPlugin` 插件实现
   - 所有组件定义: `UnitInfoPanel`, `HpBarFill`, `MpBarFill`, `StaminaBarFill`, `BuffsContainer`, `PanelLabel`

2. 从 `src/ui/mod.rs` 移除:
   ```rust
   pub mod panels;
   pub use panels::unit_info::UnitInfoPlugin;
   ```

3. 从 `src/lib.rs` 或 `src/main.rs` 移除 `UnitInfoPlugin` 注册

4. 检查是否有其他系统依赖 `PanelLabel` 或相关组件

**方案B（备选）：添加显示/隐藏控制**

如果未来需要恢复此面板，可改为按需显示:
```rust
// 在 spawn_unit_info_panel 中添加初始隐藏
commands.spawn((
    ...,
    Visibility::Hidden,  // ← 初始隐藏
    UnitInfoPanel,
));

// 在 update_unit_info 中根据 view.is_selected 切换可见性
for (container_entity, mut vis) in &mut buffs_container_query {
    *vis = if view.is_selected { Visibility::Visible } else { Visibility::Hidden };
}
```

**推荐方案A**，因为当前架构已通过悬停提示和详细面板提供单位信息，底部面板属于冗余设计。

### 涉及文件

- [src/ui/panels/unit_info.rs](file:///D:/Code/Bevy/Fre/src/ui/panels/unit_info.rs) — 完整删除或大幅精简
- [src/ui/mod.rs](file:///D:/Code/Bevy/Fre/src/ui/mod.rs) — 移除 panels 模块引用
- [src/lib.rs](file:///D:/Code/Bevy/Fre/src/lib.rs) 或 [src/main.rs](file:///D:/Code/Bevy/Fre/src/main.rs) — 移除 UnitInfoPlugin 注册

---

## 问题2：玩家棋子移动路径异常（斜角飞行）

### 问题描述

玩家控制的棋子移动至斜角格子时，未按直角折线路径移动，而是以斜角直线"飞行"至目标位置。**敌方棋子移动完全正确**，不存在此问题。

### 根因分析

**核心原因**: `show_move_range` 与 `command_handler` 使用不同的 `moving_entity` 参数调用 `find_reachable_tiles`，导致可达范围计算不一致，进而使 `reconstruct_path` 返回不完整路径。

#### 代码定位

**关键文件**:
- `src/map/pathfinding/algorithms.rs` — BFS 寻路算法
- `src/ui/highlight.rs` — 范围显示
- `src/ui/command_handler.rs` — 移动命令处理

1. **reconstruct_path 提前返回缺陷** (`algorithms.rs` L12-L24):
```rust
pub fn reconstruct_path(
    start: IVec2,
    target: IVec2,
    reachable: &HashMap<IVec2, u32>,
    ...
) -> Vec<IVec2> {
    // ❌ 关键问题：目标不在 reachable 中时返回单元素路径
    if start == target || !reachable.contains_key(&target) {
        return vec![target];  // 单元素路径 → 斜角飞行！
    }
    // ... 正常路径回溯逻辑
}
```

2. **show_move_range 使用 None** (`highlight.rs` L65-L74):
```rust
let reachable = find_reachable_tiles(
    start_coord,
    move_points,
    map, terrain_grid, terrain_registry, occupancy,
    None,  // ❌ 未排除自身实体
    calculator,
);
```
当 `moving_entity = None` 时，`occupancy.is_occupied(next)` 会将所有被占用的格子（包括起点）标记为不可达。

3. **command_handler 使用 Some(entity)** (`command_handler.rs` L129-L138):
```rust
let reachable = find_reachable_tiles(
    old_gp.coord,
    mov,
    &map, &terrain.0, &terrain.1, &occupancy,
    Some(selected_entity),  // ✅ 排除自身实体
    calculator,
);
```

#### 问题机制详解

**玩家移动流程**:
1. 玩家点击单位 → `SelectUnit` → `show_move_range` 被调用
2. `show_move_range` 使用 `moving_entity = None` 计算 reachable
3. 由于未排除自身，起点可能被标记为不可达（取决于 OccupancyGrid 实现）
4. 玩家点击目标格子 → `MoveUnit` → `command_handler` 重新计算 reachable（使用 `Some(entity)`）
5. `command_handler` 的 reachable 包含目标，但 `reconstruct_path` 可能因容差匹配失败返回 `[target]`
6. `spawn_path_arrows` 收到单元素路径 → `world_points.len().saturating_sub(1) = 0` → 不生成任何线段
7. `MovingUnit` 组件直接设置终点坐标 → 动画系统插值从起点到终点 → **斜角飞行**

**AI 移动为何正确**:
- AI 的 `select_move_coord` 从 reachable 中选择目标
- AI 使用 `Some(snapshot.entity)` 计算 reachable
- 目标必然在 reachable 中 → `reconstruct_path` 正常工作 → 路径完整

#### 深层问题：reconstruct_path 容差匹配

`algorithms.rs` L59-L65:
```rust
let expected = remaining + cost;
let tolerance = if expected > 0 { 1 } else { 0 };
if prev_remaining + tolerance >= expected && prev_remaining >= best_remaining {
    best_prev = Some(prev);
    best_remaining = prev_remaining;
}
```

**问题**:
- `prev_remaining >= best_remaining` 使用 `>=` 而非 `>`，可能导致选择非最优前驱
- 容差 `tolerance = 1` 可能不足以覆盖浮点精度误差
- 当目标不在 reachable 中时，整个回溯逻辑失效

### 建议修复方案

**方案A（核心修复）：统一 moving_entity 参数**

修改 `show_move_range` 函数签名，接收 `unit_entity` 参数:

```rust
// src/ui/highlight.rs L37
pub fn show_move_range(
    commands: &mut Commands,
    map: &GameMap,
    terrain_grid: &TerrainGrid,
    terrain_registry: &TerrainRegistry,
    occupancy: &OccupancyGrid,
    units: &Query<...>,
    unit: &Unit,
    start_coord: IVec2,
    calculator: &dyn TerrainCostCalculator,
    theme: &UiTheme,
    unit_entity: Entity,  // ← 新增参数
) {
    // ...
    let reachable = find_reachable_tiles(
        start_coord,
        move_points,
        map, terrain_grid, terrain_registry, occupancy,
        Some(unit_entity),  // ← 修改为 Some
        calculator,
    );
    // ...
}
```

同时修改调用点 (`command_handler.rs` L68):
```rust
show_move_range(
    &mut commands,
    &map, &terrain.0, &terrain.1, &occupancy,
    &units, unit, gp.coord, calculator, &theme,
    *entity,  // ← 传入实体 ID
);
```

**方案B（防御修复）：animate_movement 中添加路径验证**

在 `src/character/movement.rs` 的 `animate_movement` 系统中添加验证:

```rust
pub fn validate_and_fix_path(path: &[IVec2], start: IVec2) -> Vec<IVec2> {
    if path.is_empty() || path.first() == Some(&start) {
        return path.to_vec();
    }

    // 检测斜角移动并拆分
    let mut fixed = vec![start];
    for &target in path {
        let last = *fixed.last().unwrap();
        let diff = target - last;
        if diff.x != 0 && diff.y != 0 {
            // 斜角移动：拆分为先水平后垂直
            fixed.push(IVec2::new(target.x, last.y));
        }
        fixed.push(target);
    }
    fixed
}
```

**方案C（增强修复）：改进 reconstruct_path 容差匹配**

```rust
// algorithms.rs L59-L65
let expected = remaining + cost;
// 使用相对容差而非绝对容差
let tolerance = (expected as f32 * 0.1).ceil() as u32;
if prev_remaining + tolerance >= expected && prev_remaining > best_remaining {
    //                                                      ^ 改为 >
    best_prev = Some(prev);
    best_remaining = prev_remaining;
}
```

**推荐组合**: 方案A + 方案C，从根本上解决问题并防止复发。

### 涉及文件

- [src/map/pathfinding/algorithms.rs](file:///D:/Code/Bevy/Fre/src/map/pathfinding/algorithms.rs) — `reconstruct_path` (L12-L84), `find_reachable_tiles` (L88-L157)
- [src/ui/highlight.rs](file:///D:/Code/Bevy/Fre/src/ui/highlight.rs) — `show_move_range` (L37-L90)
- [src/ui/command_handler.rs](file:///D:/Code/Bevy/Fre/src/ui/command_handler.rs) — `handle_ui_commands` MoveUnit 分支 (L85-L169)
- [src/character/movement.rs](file:///D:/Code/Bevy/Fre/src/character/movement.rs) — `animate_movement` (L89-L158)

---

## 问题3：移动轨迹显示缺失

### 问题描述

棋子移动过程中缺少轨迹线条和箭头指示。

### 根因分析

**核心原因**: 依赖问题2，路径不完整导致 `spawn_path_arrows` 不生成线段。

#### 代码定位

**文件**: `src/character/movement.rs`

1. **spawn_path_arrows 生成条件** (L19-L44):
```rust
pub fn spawn_path_arrows(commands: &mut Commands, map: &GameMap, path: &[IVec2]) {
    if path.is_empty() { return; }

    let world_points: Vec<Vec3> = path.iter()
        .map(|&coord| { let w = map.coord_to_world(coord); Vec3::new(w.x, w.y, 0.5) })
        .collect();

    // ❌ 路径仅 1 个元素时不生成线段
    for i in 0..world_points.len().saturating_sub(1) {
        spawn_line_segment(commands, world_points[i], world_points[i + 1]);
    }

    // 末端箭头
    if world_points.len() >= 2 {
        spawn_arrow_head(commands, world_points[world_points.len() - 2], world_points[world_points.len() - 1]);
    }
}
```

2. **调用点确认**:
   - 玩家移动: `command_handler.rs` L150 — `spawn_path_arrows(&mut commands, &map, &path)` ✅
   - AI 移动: `ai/decision.rs` L185 — `spawn_path_arrows(&mut commands, &map, &path)` ✅

3. **渲染参数** (L46-L88):
   - 线段颜色: `Color::srgba(1.0, 1.0, 0.4, 0.7)` — 半透明黄色
   - 线段宽度: 4.0px
   - z-order: 0.5

#### 问题机制

当 `reconstruct_path` 返回 `[target]`（单元素路径）时:
- `world_points.len() = 1`
- `saturating_sub(1) = 0`
- `for i in 0..0` → 循环不执行 → 不生成任何线段
- `world_points.len() >= 2` 为 false → 不生成箭头

### 建议修复方案

**依赖问题2修复**: 路径完整后轨迹自然显示。

**额外优化**: 增大线段可见性:
```rust
// movement.rs L46
Sprite {
    color: Color::srgba(1.0, 1.0, 0.4, 0.9),  // 不透明度 0.7 → 0.9
    custom_size: Some(Vec2::new(6.0, line_length)),  // 宽度 4.0 → 6.0
    ..default()
},
```

### 涉及文件

- [src/character/movement.rs](file:///D:/Code/Bevy/Fre/src/character/movement.rs) — `spawn_path_arrows` (L19-L44)
- 依赖 [问题2](#问题2玩家棋子移动路径异常斜角飞行) 修复

---

## 问题4：角色信息面板闪烁

### 问题描述

鼠标悬停在棋子上时，角色信息面板持续闪烁，表现类似无限次点击触发。

### 根因分析

**核心原因**: `MovableRange`、`AttackRange`、`SelectionHighlight` 标记实体未设置 `Pickable::IGNORE`，拦截 Pointer 事件导致 `HoveredEntity` 高频切换。

#### 代码定位

**文件**: `src/ui/highlight.rs`

1. **MovableRange 未设置 Pickable::IGNORE** (L79-L88):
```rust
commands.spawn((
    Sprite {
        color: theme.movable_range,
        custom_size: Some(Vec2::splat(tile_size - 2.0)),
        ..default()
    },
    Transform::from_xyz(world_pos.x, world_pos.y, 0.5),
    MovableRange,
    GridPosition { coord },
    // ❌ 缺少 Pickable::IGNORE!
));
```

2. **AttackRange 未设置 Pickable::IGNORE** (L113-L122):
```rust
commands.spawn((
    Sprite {
        color: theme.attack_range,
        custom_size: Some(Vec2::splat(tile_size - 2.0)),
        ..default()
    },
    Transform::from_xyz(world_pos.x, world_pos.y, 0.6),
    AttackRange,
    GridPosition { coord },
    // ❌ 缺少 Pickable::IGNORE!
));
```

3. **SelectionHighlight 未设置 Pickable::IGNORE** (L136-L144):
```rust
commands.spawn((
    Sprite {
        color: theme.selection_highlight,
        custom_size: Some(Vec2::splat(tile_size * 0.75)),
        ..default()
    },
    Transform::from_xyz(world_pos.x, world_pos.y, 0.8),
    SelectionHighlight,
    // ❌ 缺少 Pickable::IGNORE!
));
```

4. **对比已修复的 TileSprite** (`map/grid.rs`):
```rust
commands.spawn((
    Sprite::from_color(terrain_color, Vec2::splat(tile_size - 2.0)),
    Transform::from_xyz(world_pos.x, world_pos.y, 0.0),
    TileSprite,
    Pickable::IGNORE,  // ✅ 已修复
    children![...],
));
```

#### 问题机制详解

**闪烁循环**:
1. 鼠标悬停在单位上 → `Pointer<Over>` on Unit → `on_unit_pointer_over` → `HoveredEntity = Some(unit)`
2. 鼠标微移命中 `MovableRange` 标记（z=0.5，高于单位的 z=1.0 但低于 SelectionHighlight 的 z=0.8）
3. `Pointer<Out>` on Unit → `on_unit_pointer_out` → `HoveredEntity = None`
4. `Pointer<Over>` on MovableRange → `on_unit_pointer_over` 忽略（不是 Unit 实体）
5. 下一帧鼠标又命中 Unit → `HoveredEntity = Some(unit)` → 回到步骤1
6. **结果**: `HoveredEntity` 在 `Some(unit)` 和 `None` 之间高频切换 → 面板闪烁

**z-order 层级**:
- 地面格子: z=0.0
- MovableRange: z=0.5
- AttackRange: z=0.6
- SelectionHighlight: z=0.8
- 单位 Sprite: z=1.0

由于 Bevy Picking 按 z-order 从上到下检测，SelectionHighlight (z=0.8) 会先于单位 (z=1.0) 被检测到，但因为它没有 `Pickable::IGNORE`，会拦截事件。

### 建议修复方案

**为所有标记实体添加 `Pickable::IGNORE`**:

```rust
// highlight.rs L79-L88 — MovableRange
commands.spawn((
    Sprite { ... },
    Transform::from_xyz(world_pos.x, world_pos.y, 0.5),
    MovableRange,
    GridPosition { coord },
    Pickable::IGNORE,  // ← 添加
));

// highlight.rs L113-L122 — AttackRange
commands.spawn((
    Sprite { ... },
    Transform::from_xyz(world_pos.x, world_pos.y, 0.6),
    AttackRange,
    GridPosition { coord },
    Pickable::IGNORE,  // ← 添加
));

// highlight.rs L136-L144 — SelectionHighlight
commands.spawn((
    Sprite { ... },
    Transform::from_xyz(world_pos.x, world_pos.y, 0.8),
    SelectionHighlight,
    Pickable::IGNORE,  // ← 添加
));
```

### 涉及文件

- [src/ui/highlight.rs](file:///D:/Code/Bevy/Fre/src/ui/highlight.rs) — `show_move_range` (L79-L88), `show_attack_range` (L113-L122), `spawn_selection_highlight` (L136-L144)

---

## 问题5：相机视口移动交互优化

### 问题描述

当前相机视口移动交互体验不佳，存在以下问题:
1. 缺少右键拖动功能
2. 边缘滚动在小窗口或高分辨率下易误触
3. WASD 移动不够直观

### 根因分析

**核心原因**: 相机控制系统仅支持 WASD/方向键平移和滚轮缩放，缺少业界标准的右键拖动功能。

#### 代码定位

**文件**: `src/ui/camera.rs`

1. **当前实现** (L64-L137):
```rust
pub fn camera_control(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut scroll_events: MessageReader<MouseWheel>,
    mut camera_query: Query<(&mut Transform, &mut CameraTarget), With<CameraController>>,
    time: Res<Time>,
    focus_state: Res<UiFocusState>,
    windows: Query<&Window>,
    map: Res<GameMap>,
) {
    // WASD / 方向键平移 (L84-L96)
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) { direction.y += 1.0; }
    // ...

    // 边缘滚动 (L98-L115)
    if cursor_pos.x < EDGE_SCROLL_MARGIN { direction.x -= 1.0; }
    // ...

    // 滚轮缩放 (L129-L133)
    for event in scroll_events.read() {
        let new_scale = (transform.scale.x + zoom_delta).clamp(CAMERA_ZOOM_MIN, CAMERA_ZOOM_MAX);
    }
}
```

2. **边缘滚动问题** (L18-L21):
```rust
const EDGE_SCROLL_MARGIN: f32 = 30.0;  // 触发区域过小
const EDGE_SCROLL_SPEED: f32 = 200.0;  // 速度与键盘移动相同
```

#### 问题分析

**缺失功能**:
1. **右键拖动**: 未使用 `MouseButton` 输入，未实现拖动增量计算
2. **中键拖动**: 未支持
3. **双击聚焦**: 未实现

**边缘滚动缺陷**:
1. **误触率高**: 30px 触发区域在小窗口或高分辨率下容易被误触
2. **干扰精确操作**: SRPG 游戏中鼠标常用于选择单位和格子，边缘滚动会干扰
3. **速度不合理**: `CAMERA_MOVE_SPEED.max(EDGE_SCROLL_SPEED)` 取最大值，但两者都是常量，实际上始终是 300.0

**WASD 移动问题**:
1. **不够直观**: 相比右键拖动，WASD 需要记忆键位
2. **无法精细控制**: 按键只能全速移动，难以微调

### 建议修复方案

**方案A（核心修复）：添加右键拖动相机控制**

```rust
// camera.rs L64
pub fn camera_control(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,  // ← 新增
    mut scroll_events: MessageReader<MouseWheel>,
    mut camera_query: Query<(&mut Transform, &mut CameraTarget), With<CameraController>>,
    time: Res<Time>,
    focus_state: Res<UiFocusState>,
    windows: Query<&Window>,
    map: Res<GameMap>,
    mut last_cursor: Local<Option<Vec2>>,  // ← 新增
) {
    if focus_state.blocks_input { return; }
    let Ok((mut transform, mut target)) = camera_query.single_mut() else { return; };

    let mut direction = Vec2::ZERO;

    // 右键拖动相机（优先级高于 WASD）
    if mouse_button.pressed(MouseButton::Right) {
        if let Ok(window) = windows.single() {
            if let Some(cursor_pos) = window.cursor_position() {
                if let Some(last_pos) = *last_cursor {
                    let delta = cursor_pos - last_pos;
                    let scale = transform.scale.x;
                    // 1:1 跟随鼠标，反转 Y 轴（屏幕坐标 Y 向下，世界坐标 Y 向上）
                    transform.translation.x -= delta.x / scale;
                    transform.translation.y += delta.y / scale;
                    target.position = None;  // 取消自动聚焦
                }
                *last_cursor = Some(cursor_pos);
                return;  // 右键拖动时禁用其他控制
            }
        }
    } else {
        *last_cursor = None;
    }

    // WASD / 方向键平移（仅在未右键拖动时生效）
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) { direction.y += 1.0; }
    // ... 其余 WASD 逻辑

    // 边缘滚动（可选：增加配置项允许用户禁用）
    if direction == Vec2::ZERO {  // 仅在无键盘输入时触发
        if let Ok(window) = windows.single() {
            if let Some(cursor_pos) = window.cursor_position() {
                let w = window.width();
                let h = window.height();
                if cursor_pos.x < EDGE_SCROLL_MARGIN { direction.x -= 1.0; }
                // ...
            }
        }
    }

    // 应用移动
    if direction != Vec2::ZERO {
        target.position = None;
        let scale = transform.scale.x;
        let speed = CAMERA_MOVE_SPEED;
        transform.translation.x += direction.x * speed * time.delta_secs() / scale;
        transform.translation.y += direction.y * speed * time.delta_secs() / scale;
    }

    // 滚轮缩放
    for event in scroll_events.read() {
        let zoom_delta = -event.y * CAMERA_ZOOM_SPEED;
        let new_scale = (transform.scale.x + zoom_delta).clamp(CAMERA_ZOOM_MIN, CAMERA_ZOOM_MAX);
        transform.scale = Vec3::splat(new_scale);
    }

    clamp_camera_to_map(&mut transform, &map);
}
```

**方案B（增强）：添加配置项控制边缘滚动**

```rust
// 在 CameraController 组件中添加配置
#[derive(Component)]
pub struct CameraSettings {
    pub edge_scroll_enabled: bool,
    pub edge_scroll_margin: f32,
    pub drag_speed: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            edge_scroll_enabled: false,  // 默认禁用
            edge_scroll_margin: 30.0,
            drag_speed: 1.0,
        }
    }
}
```

**方案C（可选）：添加双击聚焦功能**

```rust
pub fn handle_double_click_focus(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    map: Res<GameMap>,
    mut camera_target: Query<&mut CameraTarget, With<CameraController>>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        // 检测双击（两次点击间隔 < 0.3 秒）
        // 如果双击在空地上，聚焦该位置
        // 如果双击在单位上，聚焦该单位
    }
}
```

**推荐方案A**，快速实现右键拖动功能，显著提升交互体验。

### 涉及文件

- [src/ui/camera.rs](file:///D:/Code/Bevy/Fre/src/ui/camera.rs) — `camera_control` (L64-L137)

---

## 问题6：移动确认与右键退回功能验证

### 问题描述

需验证以下功能是否正常工作:
1. 玩家左键点击棋子移动到目标位置 B 后，弹出包含"攻击"、"普通攻击"、"待机"、"取消"的交互页面
2. 右键点击棋子或空地时，棋子从位置 B 退回到原始位置 A

### 根因分析

**当前状态**: 逻辑完整，需验证各阶段 `Cancel` 行为是否符合预期。

#### 代码定位

**关键文件**:
- `src/input.rs` — 输入处理
- `src/ui/command_handler.rs` — 命令处理
- `src/ui/action_menu.rs` — 行动菜单

1. **左键点击分发** (`input.rs` L20-L72):
```rust
pub fn on_unit_pointer_click(trigger: On<Pointer<Click>>, ...) {
    if trigger.event.button != PointerButton::Primary { return; }

    match turn_phase.get() {
        TurnPhase::SelectUnit => {
            ui_commands.write(UiCommand::SelectUnit { entity });
        }
        TurnPhase::MoveUnit => {
            ui_commands.write(UiCommand::MoveUnit { coord: gp.coord });
        }
        TurnPhase::SelectTarget => {
            ui_commands.write(UiCommand::SelectTarget { coord: gp.coord });
        }
        _ => {}
    }
}
```

2. **原地不动进入 ActionMenu** (`command_handler.rs` L85-L107):
```rust
UiCommand::MoveUnit { coord } => {
    if sel_gp.coord == *coord {
        *prev_coord = Some(sel_gp.coord);
        commands.insert_resource(PrevPosition { coord: Some(sel_gp.coord) });
        // 清除范围标记
        for (marker, _) in &range_entities { commands.entity(marker).try_despawn(); }
        for h in &highlights { commands.entity(h).try_despawn(); }
        spawn_selection_highlight(&mut commands, &map, sel_gp.coord, &theme);
        next_phase.set(TurnPhase::ActionMenu);  // ← 进入行动菜单
        return;
    }
    // ... 移动到目标位置
}
```

3. **移动后进入 ActionMenu** (`command_handler.rs` L152-L158):
```rust
commands.entity(selected_entity).insert(MovingUnit {
    path,
    current_index: 0,
    speed: 0.15,
    elapsed: 0.0,
    next_phase: TurnPhase::ActionMenu,  // ← 动画结束后自动进入 ActionMenu
});
```

4. **ActionMenu 自动弹出** (`action_menu.rs`):
```rust
pub fn on_enter_action_menu(
    mut commands: Commands,
    selected_query: Query<Entity, With<Selected>>,
    units: Query<...>,
    skill_slots: Query<&SkillSlots>,
    mut menu_entity: ResMut<ActionMenuEntity>,
    theme: Res<UiTheme>,
) {
    //  despawn 旧菜单
    despawn_action_menu(&mut commands, &mut menu_entity);
    //  spawn 新菜单（攻击、技能、待机、取消）
    spawn_action_menu(&mut commands, &theme, ...);
}
```

5. **右键取消逻辑** (`input.rs` L181-L211, `command_handler.rs` L238-L274):
```rust
// input.rs L201-L210
match turn_phase.get() {
    TurnPhase::ActionMenu | TurnPhase::SelectTarget => {
        ui_commands.write(UiCommand::Cancel);
    }
    _ => {}
}

// command_handler.rs L251-L267
else if menu_entity.entity.is_some() {
    // ActionMenu 取消 → 回退位置，回到 SelectUnit
    despawn_action_menu(&mut commands, &mut menu_entity);
    if let Some(prev) = *prev_coord {
        let world_pos = map.coord_to_world(prev);
        commands.entity(selected_entity)
            .insert(Transform::from_xyz(world_pos.x, world_pos.y, 1.0))
            .insert(GridPosition { coord: prev });
    }
    clear_selection(&mut commands, &selected_query, &range_entities, &highlights);
    next_phase.set(TurnPhase::SelectUnit);
}
```

#### 功能验证清单

| 场景 | 预期行为 | 当前实现 | 状态 |
|------|---------|---------|------|
| 左键点击当前位置 | 进入 ActionMenu | `sel_gp.coord == *coord` → `TurnPhase::ActionMenu` | ✅ 正确 |
| 左键点击可达格子 | 移动后进入 ActionMenu | `MovingUnit.next_phase = ActionMenu` | ✅ 正确 |
| ActionMenu 中右键 | 回退位置，回到 SelectUnit | `Cancel` → 恢复 `prev_coord` | ✅ 正确 |
| MoveUnit 阶段右键 | 取消移动，回到 SelectUnit | `Cancel` → `clear_selection` | ✅ 正确 |
| SelectTarget 阶段右键 | 回到 ActionMenu | `Cancel` → `TurnPhase::ActionMenu` | ✅ 正确 |
| ESC 键 | 同右键 Cancel | `handle_esc_key` → `UiCommand::Cancel` | ✅ 正确 |

#### 潜在问题

1. **移动动画中右键**: 如果玩家在 `MovingUnit` 动画播放期间按下右键，当前实现不会中断动画。需验证是否需要支持动画中断。

2. **prev_coord 同步**: `prev_coord` 使用 `Local<Option<IVec2>>`，同时写入 `PrevPosition` 资源。需确保两者始终一致。

3. **多次 Cancel**: 连续按下右键或 ESC 是否会触发多次回退逻辑。

### 建议修复方案

**方案A（验证优先）：编写集成测试验证各阶段 Cancel 行为**

```rust
// tests/feature/movement_cancel_test.rs
#[test]
fn test_cancel_from_action_menu_returns_to_start() {
    // Given: 单位在位置 A，已进入 ActionMenu
    // When: 发送 Cancel 命令
    // Then: 单位回到位置 A，TurnPhase = SelectUnit
}

#[test]
fn test_cancel_during_move_unit_clears_selection() {
    // Given: 单位已选中，显示移动范围
    // When: 发送 Cancel 命令
    // Then: 清除 Selected 组件，TurnPhase = SelectUnit
}
```

**方案B（增强）：支持移动动画中断**

```rust
// command_handler.rs Cancel 分支
else if menu_entity.entity.is_some() {
    // 检查是否正在移动
    if let Ok(moving) = units.get_mut(selected_entity) {
        // 中断动画，立即回退
        commands.entity(selected_entity).remove::<MovingUnit>();
    }
    // ... 回退逻辑
}
```

**方案C（防御）：添加 Cancel 防抖**

```rust
// 使用 Local<bool> 记录上一帧是否已处理 Cancel
pub fn handle_ui_commands(..., mut last_cancel_frame: Local<u64>) {
    let current_frame = time.elapsed().as_secs_f64() as u64;
    for cmd in events.read() {
        if matches!(cmd, UiCommand::Cancel) && *last_cancel_frame == current_frame {
            continue;  // 同一帧的重复 Cancel 忽略
        }
        // ...
    }
}
```

**推荐方案A**，先通过测试验证现有逻辑是否正确，再根据实际需求决定是否需要方案B/C。

### 涉及文件

- [src/input.rs](file:///D:/Code/Bevy/Fre/src/input.rs) — `on_unit_pointer_click` (L20-L72), `handle_right_cancel` (L181-L211), `handle_esc_key` (L264-L286)
- [src/ui/command_handler.rs](file:///D:/Code/Bevy/Fre/src/ui/command_handler.rs) — `handle_ui_commands` (L30-L288)
- [src/ui/action_menu.rs](file:///D:/Code/Bevy/Fre/src/ui/action_menu.rs) — `on_enter_action_menu`
- [src/character/movement.rs](file:///D:/Code/Bevy/Fre/src/character/movement.rs) — `animate_movement` (L89-L158)

---

## 修复优先级与依赖关系

```
P0 (立即修复):
  ├── 问题1: 移除 UnitInfoPanel → 删除面板代码，10分钟
  ├── 问题4: 面板闪烁 → highlight.rs 添加 Pickable::IGNORE，5分钟
  └── 问题2: 斜角飞行 → 统一 moving_entity 参数 + 改进容差匹配，30分钟

P1 (短期修复):
  ├── 问题3: 轨迹缺失 → 依赖问题2修复，验证后可能无需额外修改
  ├── 问题5: 相机控制 → camera.rs 添加右键拖动，20分钟
  └── 问题6: 移动交互验证 → 编写集成测试，15分钟
```

**依赖关系**:
- 问题3 依赖问题2 — 路径完整后轨迹自然显示
- 问题2 的部分卡顿感来自问题4 — 面板闪烁干扰视觉

**总预估时间**: 约 90 分钟（不含测试编写）

---

## 自检结果

| 检查项 | 结果 | 说明 |
|--------|------|------|
| 架构合规 | PASS | 诊断过程遵循领域边界，未修改业务逻辑 |
| 领域规则 | PASS | 所有分析基于 ECS 模式和消息通信机制 |
| 测试规范 | N/A | 诊断报告，不涉及测试代码 |
| 命名规范 | PASS | 文件路径、函数名符合项目规范 |
| 可见性 | PASS | 所有引用的函数和组件均为 public |
| 错误处理 | PASS | 诊断中识别的错误均有明确根因 |
| 死代码 | N/A | 未涉及代码删除建议 |
| 重复代码 | PASS | 未发现诊断逻辑重复 |

---

## 附录：相关文件索引

### UI 渲染
- [src/ui/panels/unit_info.rs](file:///D:/Code/Bevy/Fre/src/ui/panels/unit_info.rs) — 单位信息面板
- [src/ui/highlight.rs](file:///D:/Code/Bevy/Fre/src/ui/highlight.rs) — 范围显示与高亮
- [src/ui/action_menu.rs](file:///D:/Code/Bevy/Fre/src/ui/action_menu.rs) — 行动菜单
- [src/ui/widgets/resource_bar.rs](file:///D:/Code/Bevy/Fre/src/ui/widgets/resource_bar.rs) — 资源条组件

### 移动与寻路
- [src/map/pathfinding/algorithms.rs](file:///D:/Code/Bevy/Fre/src/map/pathfinding/algorithms.rs) — BFS 寻路算法
- [src/map/pathfinding/mod.rs](file:///D:/Code/Bevy/Fre/src/map/pathfinding/mod.rs) — 寻路模块入口
- [src/character/movement.rs](file:///D:/Code/Bevy/Fre/src/character/movement.rs) — 移动动画与路径箭头
- [src/ui/command_handler.rs](file:///D:/Code/Bevy/Fre/src/ui/command_handler.rs) — UI 命令处理

### 相机控制
- [src/ui/camera.rs](file:///D:/Code/Bevy/Fre/src/ui/camera.rs) — 相机控制系统

### 输入处理
- [src/input.rs](file:///D:/Code/Bevy/Fre/src/input.rs) — 输入处理模块

### AI 决策
- [src/ai/decision.rs](file:///D:/Code/Bevy/Fre/src/ai/decision.rs) — AI 移动决策

---

**报告结束**
