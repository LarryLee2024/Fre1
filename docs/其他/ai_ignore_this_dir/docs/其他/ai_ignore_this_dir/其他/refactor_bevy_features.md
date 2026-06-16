# 基于 Bevy 0.18.1 官方特性的激进重构方案

> 依据：`docs/11.md` Bevy 0.18 Feature 价值评估 + `docs/best_practices.md` 最佳实践
> 原则：用官方能力替换自造轮子，优先最佳实践，不考虑短期成本
> 版本约束：所有方案必须基于 **Bevy 0.18.1** 实际可用 API，0.19 特性标注为前瞻

---

## 现状诊断

| 自造轮子 | 代码位置 | Bevy 0.18.1 官方替代 | 可行性 |
|----------|----------|---------------------|--------|
| 手写鼠标拾取（cursor_to_coord） | `input.rs` | `bevy_picking` + `sprite_picking`（已含于 `2d`） | 部分可行（单位 Picking，格子保留逻辑计算） |
| 手写范围高亮（spawn Sprite） | `ui/highlight.rs` | `bevy_gizmos`（已含于 `2d`） | 仅调试用，正式高亮保留 Sprite |
| 手写相机控制器 | `ui/camera.rs` | `bevy_camera_controller` + `pan_camera` | 需研究评估 |
| 手写中文字体管理（CnFont） | `assets.rs` | `system_font_discovery`（**0.19 特性**） | 降级：优化 CnFont 管理 |
| 硬编码 UI 主题常量 | `ui/theme.rs` | `bevy_settings`（**0.19 特性**） | 降级：自定义 GameSettings + RON |
| 无 Reflect 体系 | 全项目零 Reflect derive | `reflect_auto_register`（已含于 `2d`） | 完全可行 |
| 无 World 序列化 | 无法存档/快照/回放 | `bevy_scene` + `serialize` | 完全可行 |
| 无输入焦点管理 | UI 输入与游戏输入冲突 | `bevy_input_focus`（已含于 `2d`） | 完全可行 |
| 无系统级调试 | 只能靠日志 | `bevy_debug_stepping` + `track_location` | 完全可行 |
| 无运行时控制台 | 无法运行时修改属性 | `bevy_remote` | 完全可行 |

### 已知代码问题（审查发现）

| 问题 | 位置 | 影响 |
|------|------|------|
| `UiTheme::default()` 与 `Res<UiTheme>` 混用 | `highlight.rs` 3 处用 `UiTheme::default()` | 范围高亮颜色不响应运行时修改 |
| `CnFont` 在 5 个文件中使用 | `unit_info.rs`/`inventory_panel.rs`/`spawn.rs`/`grid.rs`/`combat_vfx_handler.rs` | 删除影响超出 UI 层 |
| `clear_markers` 在 AI 管线中调用 | `command_handler.rs` 2 处 + `intent.rs` 2 处 | 删除需同步修改 AI |
| `MovableRange`/`AttackRange` 在 AI Query 中 | `intent.rs` 2 处 Query | 删除需同步修改 AI |
| `bevy-inspector-egui` 缺少 Reflect 支持 | 全项目零 Reflect | Inspector 功能受限 |

---

## 阶段一：Cargo.toml Feature 升级

### 目标
启用 Bevy 0.18.1 实际可用的 Feature，去除冗余声明。

### 关键认知：`"2d"` 元 Feature 已隐式包含

```
"2d" 依赖链：
├── default_app → bevy_input_focus, bevy_log, bevy_state, bevy_asset, reflect_auto_register
├── default_platform → multi_threaded, default_font
├── 2d_bevy_render → 2d_api → common_api → bevy_gizmos, bevy_text, bevy_image, png
├── ui → ui_api → bevy_ui, common_api → bevy_sprite, bevy_gizmos, bevy_text, png
└── picking → bevy_picking, sprite_picking, ui_picking, mesh_picking
```

**以下 Feature 已被 `"2d"` 隐式包含，无需显式声明**：
`bevy_state`, `bevy_ui`, `bevy_text`, `bevy_sprite`, `bevy_gizmos`, `bevy_log`, `png`, `multi_threaded`, `bevy_picking`, `sprite_picking`, `ui_picking`, `bevy_input_focus`, `reflect_auto_register`

### 任务

#### 1.1 升级 Cargo.toml
```toml
[dependencies]
bevy = { version = "0.18.1", features = [
    "2d",  # 元 Feature，已包含上述 13 个子 Feature

    # === 需要显式启用的 Feature ===
    # 序列化
    "serialize",                  # serde 支持
    "bevy_scene",                 # DynamicScene / World 快照（0.19 重命名为 bevy_world_serialization）

    # 开发工具
    "bevy_dev_tools",             # 官方开发工具集
    "bevy_debug_stepping",        # System 单步调试
    "bevy_remote",                # 运行时远程控制台
    "track_location",             # 错误追踪 System 来源文件/行号
    "file_watcher",               # 资产热重载

    # Reflect 增强
    "reflect_documentation",      # 反射文档注释（编辑器基础）

    # 相机
    "bevy_camera_controller",     # 官方相机控制器（需配合 pan_camera 子 feature）

    # 测试
    "bevy_ci_testing",            # CI 测试支持
] }

[features]
default = []
dev = ["bevy/file_watcher", "bevy/bevy_dev_tools", "bevy/bevy_debug_stepping"]
release = []  # 发布时关闭 dev_tools/debug_stepping/track_location/bevy_remote/file_watcher
```

#### 1.2 不可用 Feature 清单（属于 Bevy 0.19，降级处理）

| 0.19 Feature | 0.18.1 替代方案 | 对应阶段 |
|-------------|----------------|---------|
| `bevy_world_serialization` | `bevy_scene` + `DynamicSceneBuilder` | 阶段四 |
| `bevy_settings` | 自定义 `GameSettings` Resource + RON 持久化 | 阶段六 |
| `system_font_discovery` | 保留 `CnFont`，优化管理方式 | 阶段五 |
| `schedule_data` | 暂不启用 | — |

#### 1.3 验证编译
- `cargo build` 通过
- `cargo test` 通过
- 确认无 feature 冲突

---

## 阶段二：bevy_picking 部分替换手写鼠标拾取

### 目标
用 `bevy_picking` 替换单位点击的手动遍历，保留格子点击的逻辑计算。

### 设计决策：Picking 与 Grid 架构兼容

**问题**：为空格子创建 Sprite Entity 以支持 Picking，违反地图铁律（Grid 优先于 Entity）。

**方案**：
- **单位点击** → `bevy_picking`（Unit Sprite 天然存在，添加 `Pickable` 零成本）
- **空格子点击** → 保留 `cursor_to_coord` 逻辑计算（不创建 Tile Entity）
- **UI 按钮** → `ui_picking`（已含于 `2d`，自动工作）

### 任务

#### 2.1 为单位 Sprite 添加 Pickable
- 每个 Unit Sprite 添加 `Pickable` 组件
- 点击单位直接获得 Entity，无需遍历所有单位匹配坐标
- `HoveredEntity` 通过 `Pointer<Over>` / `Pointer<Out>` 自动更新

#### 2.2 用 Picking 事件重写单位点击逻辑
- 监听 `Pointer<Click>` 事件（仅 Unit Entity）
- 替换 `handle_click` 中的单位遍历匹配代码
- 保留 `cursor_to_coord` 用于空格子点击

#### 2.3 修复 HoveredEntity 更新
- 当前：每帧 Query 所有单位比较坐标
- 重构后：`Pointer<Over>` → 设置 `HoveredEntity`，`Pointer<Out>` → 清除

#### 2.4 UI 按钮自动 Picking
- `ui_picking` 已含于 `2d`，UI 按钮自动支持
- 行动菜单按钮交互无需手动处理

### 预期收益
- 删除 ~30 行单位遍历匹配代码
- 悬停检测从 O(n) 遍历 → O(1) 事件驱动
- UI 和游戏场景统一交互模型
- 保留 Grid 逻辑层独立性

### 保留代码
- `cursor_to_coord`：空格子点击仍需逻辑计算
- `Camera`/`GlobalTransform`/`Window` 查询：空格子点击依赖

---

## 阶段三：bevy_gizmos 调试可视化 + Sprite 高亮优化

### 目标
Gizmos 仅用于调试可视化，正式范围高亮保留 Sprite 方式但优化管理。

### 设计决策：Gizmos 只能画线框，不能替代半透明填充

**关键事实**：`bevy_gizmos 0.18.1` 的 `rect_2d()` 内部调用 `lineloop_2d()`，**只绘制线框轮廓**。

当前代码使用 `Sprite { color: Color::srgba(0.3, 0.6, 1.0, 0.4) }` 生成**半透明填充色块**，这是 SRPG 移动/攻击范围的标准视觉表现。Gizmos 无法替代。

正确 API：
```rust
// Gizmos rect_2d 签名（0.18.1）
gizmos.rect_2d(
    Isometry2d::from_translation(world_pos),  // 不是 Vec2
    Vec2::splat(tile_size),                    // 尺寸
    Color::srgba(0.3, 0.6, 1.0, 0.3),         // 颜色（但只画线框）
);
```

### 任务

#### 3.1 Gizmos 用于调试可视化（新增）
```rust
/// 寻路路径调试
fn debug_path(gizmos: Gizmos, path: &[(IVec2, f32)]) {
    for (coord, cost) in path {
        let pos = Isometry2d::from_translation(coord.as_vec2() * TILE_SIZE);
        gizmos.rect_2d(pos, Vec2::splat(TILE_SIZE * 0.9), Color::srgba(0.0, 1.0, 0.0, 0.5));
    }
}

/// AI 决策调试
fn debug_ai_intent(gizmos: Gizmos, intent: &CombatIntent) {
    // 移动目标
    let pos = Isometry2d::from_translation(intent.move_target.as_vec2() * TILE_SIZE);
    gizmos.rect_2d(pos, Vec2::splat(TILE_SIZE), Color::srgba(1.0, 1.0, 0.0, 0.5));
    // 攻击目标
    // ...
}

/// 占据网格调试
fn debug_occupancy(gizmos: Gizmos, grid: &OccupancyGrid) {
    for (coord, _) in grid.cells.iter() {
        let pos = Isometry2d::from_translation(coord.as_vec2() * TILE_SIZE);
        gizmos.rect_2d(pos, Vec2::splat(TILE_SIZE * 0.3), Color::srgba(1.0, 0.0, 0.0, 0.3));
    }
}
```

#### 3.2 修复 UiTheme 混用问题
- `highlight.rs` 3 处 `UiTheme::default()` → `Res<UiTheme>`
- 确保范围高亮颜色响应运行时修改

#### 3.3 优化 Sprite 高亮管理（可选）
- 考虑使用对象池复用 Sprite Entity，避免频繁 spawn/despawn
- 或使用单个 Entity + 多个 Sprite 的方式减少 Entity 数量

#### 3.4 删除标记组件（需同步修改 AI 管线）
- 删除 `MovableRange`、`AttackRange`、`SelectionHighlight` 组件
- 修改 `intent.rs` 中 2 处 Query（改用 `HoveredEntity` 或其他方式判断）
- 修改 `command_handler.rs` 中 2 处 `clear_markers` 调用
- Gizmos 每帧自动清除，无需手动清理

### 预期收益
- 新增寻路/AI/占据网格调试可视化
- 修复 UiTheme 运行时不生效的 Bug
- 减少标记 Entity 的创建/销毁（如果采用对象池）
- AI 管线不再依赖标记组件

### 保留代码
- Sprite 方式的正式范围高亮（半透明填充）
- `show_move_range`/`show_attack_range` 核心逻辑

---

## 阶段四：Reflect 体系 + bevy_scene 序列化

### 目标
为所有 Component/Resource 添加 Reflect，建立 Inspector/编辑器/序列化基础。

### 与 bevy-inspector-egui 的协同

项目已依赖 `bevy-inspector-egui = "0.36"`，该 crate 需要 Reflect 才能完整工作。当前全项目零 Reflect，导致 Inspector 只能显示有限信息。添加 Reflect 后 Inspector 自动获得完整组件查看/修改能力。

### 任务

#### 4.1 核心 Component 添加 Reflect
```rust
#[derive(Component, Reflect, Debug)]
#[reflect_component]
pub struct Unit {
    pub faction: Faction,
    pub acted: bool,
}
```

需要添加 Reflect 的组件（按优先级排序）：

**P0 — Inspector 最高价值**：
- `Unit`, `UnitName`, `Dead`, `Selected`
- `GridPosition`, `MovingUnit`
- `Attributes`, `GameplayTags`

**P1 — 调试常用**：
- `SkillSlots`, `SkillCooldowns`
- `ActiveBuffs`, `TraitCollection`
- `AiBehaviorId`

**P2 — 编辑器/序列化需要**：
- `EquipmentSlots`, `CombatInventory`
- `TraitGrantedTags`

#### 4.2 核心 Resource 添加 Reflect
```rust
#[derive(Resource, Reflect, Default, Debug)]
#[reflect_resource]
pub struct TurnOrder { ... }
```

需要添加 Reflect 的资源：
- `TurnOrder`, `TurnState`
- `GameMap`, `OccupancyGrid`, `TerrainGrid`
- `BattleRecord`, `CombatLog`
- `SelectedUnitView`, `HoveredEntity`

#### 4.3 核心 Enum/Struct 添加 Reflect
- `Faction`, `AttributeKind`, `ModifierOp`
- `GameplayTag`, `TagName`
- `SkillTargeting`, `SkillCondition`
- `BuffInstance`, `TraitEffect`
- `BattleEntry`

#### 4.4 注册 Reflect 类型
```rust
// 在各 Plugin::build 中注册
app.register_type::<Unit>()
    .register_type::<Attributes>()
    .register_type::<GridPosition>()
    // ...
```

#### 4.5 启用 bevy_scene 序列化
- 使用 `DynamicSceneBuilder` 创建 World 快照
- 为存档系统建立基础
- 为 Golden Test 提供快照能力
- 为 BattleRecord 回放提供数据源

```rust
// 0.18.1 正确 API（不是 world.serialize()）
use bevy::scene::DynamicSceneBuilder;

fn save_snapshot(world: &mut World) -> Vec<u8> {
    let scene = DynamicSceneBuilder::from_world(world)
        .extract_entities(entities.iter().copied())
        .build();
    scene.serialize_ron().unwrap().into_bytes()
}
```

### 预期收益
- Inspector 自动显示所有组件字段（配合已有的 bevy-inspector-egui）
- 编辑器/工具链的基础设施
- 存档/快照/回放的技术基础
- `bevy_remote` 可运行时查看/修改属性
- `reflect_documentation` 支持编辑器自动生成字段说明

---

## 阶段五：CnFont 管理优化（0.18.1 无 system_font_discovery）

### 目标
`system_font_discovery` 是 0.19 特性，0.18.1 不可用。优化现有 CnFont 管理方式。

### 当前问题
- `CnFont` 在 **5 个文件**中使用（非仅 UI 层）：
  - `ui/panels/unit_info.rs` — UI 字体
  - `ui/panels/inventory_panel.rs` — UI 字体
  - `character/spawn.rs` — 角色名字标签
  - `map/grid.rs` — 地图格子文字
  - `ui/combat_vfx_handler.rs` — 战斗飘字
- 打包时需携带 `fonts/Arial Unicode.ttf`（数 MB）

### 任务

#### 5.1 统一字体获取接口
- 创建 `FontProvider` trait 或统一函数
- 所有需要字体的地方通过统一接口获取
- 为未来迁移到 `system_font_discovery` 预留扩展点

#### 5.2 支持字体回退链
- 优先使用 AssetServer 加载的字体
- 回退到 Bevy 默认字体（`default_font` 已含于 `2d`）
- 中文字符缺失时自动回退

#### 5.3 字体文件优化
- 评估是否可用更小的中文字体子集
- 或使用系统字体路径（macOS: `/System/Library/Fonts/`，Windows: `C:\Windows\Fonts\`）

#### 5.4 未来迁移路径（0.19）
- 升级到 0.19 后启用 `system_font_discovery`
- 删除 `CnFont` Resource
- 删除字体文件
- 所有 `Res<CnFont>` 替换为默认字体

### 预期收益
- 统一字体管理，减少散落的字体获取代码
- 为 0.19 迁移预留扩展点
- 可能减少打包体积

---

## 阶段六：自定义 GameSettings + RON 持久化（0.18.1 无 bevy_settings）

### 目标
`bevy_settings` 是 0.19 特性，0.18.1 不可用。自定义 GameSettings Resource + RON 配置文件。

### 当前问题
- `ui/theme.rs` 所有颜色/字号/间距硬编码在 `Default` impl 中
- `highlight.rs` 3 处使用 `UiTheme::default()` 而非 `Res<UiTheme>`（不响应运行时修改）
- 无法持久化用户偏好
- 无法支持无障碍（色盲模式、字号放大）

### 任务

#### 6.1 创建 GameSettings Resource
```rust
#[derive(Resource, Reflect, Serialize, Deserialize, Default)]
#[reflect_resource]
pub struct GameSettings {
    pub ui: UiSettings,
    pub accessibility: AccessibilitySettings,
    pub gameplay: GameplaySettings,
}

#[derive(Reflect, Serialize, Deserialize)]
pub struct UiSettings {
    pub font_scale: f32,           // 字号缩放因子
    pub color_scheme: ColorScheme, // 颜色方案（正常/色盲友好）
}

#[derive(Reflect, Serialize, Deserialize)]
pub struct AccessibilitySettings {
    pub color_blind_mode: ColorBlindMode,
    pub auto_battle_speed: f32,
}

#[derive(Reflect, Serialize, Deserialize)]
pub struct GameplaySettings {
    pub animation_speed: f32,
    pub show_damage_numbers: bool,
}
```

#### 6.2 迁移 UiTheme 到 GameSettings
- 字号 → `ui.font_scale` 缩放
- 颜色 → `ui.color_scheme` 切换
- 修复 `highlight.rs` 的 `UiTheme::default()` → `Res<UiTheme>`

#### 6.3 RON 配置持久化
```rust
impl GameSettings {
    pub fn load() -> Self {
        std::fs::read_to_string("settings.ron")
            .ok()
            .and_then(|s| ron::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Ok(s) = ron::ser::to_string_pretty(self, Default::default()) {
            let _ = std::fs::write("settings.ron", s);
        }
    }
}
```

#### 6.4 未来迁移路径（0.19）
- 升级到 0.19 后替换为 `bevy_settings`
- `GameSettings` 字段映射到 `bevy_settings` API
- RON 格式兼容

### 预期收益
- 用户可自定义 UI 外观
- 支持无障碍功能
- 配置持久化
- 修复 UiTheme 运行时不生效的 Bug
- 为未来设置菜单提供数据源

---

## 阶段七：bevy_input_focus 解决输入冲突

### 目标
用 `bevy_input_focus`（已含于 `2d`）统一管理 UI 和游戏输入焦点。

### 当前问题
- 打开背包/技能面板时，WASD 仍然移动相机
- 输入框获得焦点时，按键仍然触发游戏操作
- 没有统一的焦点管理机制

### 任务

#### 7.1 集成 bevy_input_focus
- 当 UI 面板打开时，游戏输入自动禁用
- 当 UI 面板关闭时，游戏输入自动恢复

#### 7.2 修改输入系统
- `handle_click`、`camera_control`、`handle_end_turn` 添加焦点检查
- 只有游戏焦点时才处理输入

#### 7.3 UI 面板焦点管理
- 打开面板时声明焦点
- 关闭面板时释放焦点
- ESC 键优先关闭面板，其次取消操作

#### 7.4 拖拽支持（未来）
- `bevy_input_focus` + `ui_picking` 组合支持拖拽物品/技能/装备

### 预期收益
- 消除 UI/游戏输入冲突
- 为未来聊天框/搜索框/重命名提供基础
- 统一的焦点生命周期管理

---

## 阶段八：bevy_debug_stepping + track_location + bevy_remote

### 目标
建立系统级调试基础设施。

### 任务

#### 8.1 bevy_debug_stepping
- 启用后可逐步执行 System
- 用于调试 Buff 链/Observer 链/回合流程
- 在 Debug 面板中添加控制按钮

#### 8.2 track_location
- 错误信息自动标注 System 来源文件和行号
- Observer/Hook/Message 触发时可追踪来源
- 与结构化日志配合，快速定位问题

#### 8.3 bevy_remote
- 运行时通过控制台查看/修改 Entity 属性（需 Reflect 支持，阶段四前置）
- 运行时查看 Resource 状态
- GM 工具基础

### 预期收益
- 从"只能靠日志"升级到"系统级调试"
- Buff 链/Observer 链问题可逐步追踪
- 运行时修改属性，无需重编译

---

## 阶段九：bevy_camera_controller 研究与适配

### 目标
研究官方相机控制器（需 `pan_camera` 子 feature），替换或参考改进手写相机。

### 关键认知
`bevy_camera_controller` 默认 `default-features = false`，2D 项目需显式启用 `pan_camera` 子 feature。

### 任务

#### 9.1 启用 pan_camera 子 feature
```toml
# Cargo.toml 需要修改为：
bevy = { version = "0.18.1", features = [
    # ... 其他 feature ...
    "bevy_camera_controller",  # 启用相机控制器
] }
# 注意：pan_camera 子 feature 可能需要通过 bevy_internal 启用
# 需要验证是否可以直接在 bevy crate 中启用
```

#### 9.2 研究官方 PanCamera API
- 了解 PanCamera 组件 API
- 评估是否可直接用于 2D 战棋

#### 9.3 如果可用：替换手写相机
- 删除 `CameraController` 组件
- 删除 `camera_control` 系统
- 使用官方 PanCamera

#### 9.4 如果不可用：参考改进
- 添加边缘滚屏
- 添加聚焦角色功能
- 添加平滑移动/缩放
- 添加相机边界限制

#### 9.5 战棋专用相机功能
- 聚焦当前行动单位
- 聚焦战斗事件
- 战场全景/缩放切换

### 预期收益
- 更流畅的相机体验
- 减少手写代码
- 参考官方最佳实践

---

## 阶段十：Reflect Functions 数据驱动技能（前瞻）

### 目标
研究 `reflect_functions` 作为数据驱动技能系统的未来方向。

### 当前双 Handler 体系

项目存在两套并行的 Handler 体系：

| 体系 | 位置 | 处理类型 |
|------|------|---------|
| `EffectHandler` | `core/effect/handler.rs` | Damage/Heal/Buff/Cleanse |
| `TraitEffectHandler` | `character/traits/handlers.rs` | GrantTag/ModifyAttribute/ApplyBuff |

`reflect_functions` 需要同时覆盖两套体系。

### 任务

#### 10.1 研究 reflect_functions API
- 了解 FunctionRegistry 机制
- 评估与当前 EffectHandler + TraitEffectHandler 的兼容性

#### 10.2 设计 Reflect-based 效果系统
```rust
// 未来：效果函数注册到 FunctionRegistry
app.register_function("apply_damage", apply_damage_fn);
app.register_function("apply_heal", apply_heal_fn);
app.register_function("apply_buff", apply_buff_fn);
app.register_function("grant_tag", grant_tag_fn);
app.register_function("modify_attribute", modify_attribute_fn);

// 配置中引用函数名
effects: [
    { function: "apply_damage", params: { amount: 50 } },
    { function: "apply_buff", params: { buff_id: "burn", duration: 3 } },
    { function: "grant_tag", params: { tag: "BURN" } },
]
```

#### 10.3 渐进迁移
- 保留当前 EffectHandler + TraitEffectHandler 作为 fallback
- 新效果类型优先使用 reflect_functions
- 逐步迁移旧效果到新系统

### 预期收益
- 新增效果类型无需修改代码
- 配置中可直接引用效果函数
- 为脚本系统/Mod 系统提供基础
- 技能编辑器可动态选择效果函数

---

## 阶段十一：实验性 Feature 观察（0.18.1 已存在但标记 experimental）

### 目标
跟踪官方实验性 Feature，为未来采用做准备。

### 任务

#### 11.1 bevy_ui_widgets（`experimental_bevy_ui_widgets`）
- 官方 UI Widget 层拆分
- 未来可能提供 VBox/HBox/Panel/Tabs/ScrollView
- 当前手写 Widget 库（`ui/layout.rs`）需观察是否可替换

#### 11.2 bevy_feathers（`experimental_bevy_feathers`）
- 官方组件库（类似 Qt Widgets / Flutter Material）
- 未来可能直接搭建 CharacterPanel/InventoryPanel
- **不要急着造自己的 UI DSL，先观察官方**

#### 11.3 reflect_functions
- 函数注册表（阶段十的基础）
- 目前实验性 API，稳定性待观察

### 预期收益
- 避免重复造轮子
- 为 0.19/0.20 迁移做准备
- 减少 Widget 库维护成本

---

## 执行优先级

| 优先级 | 阶段 | 核心价值 | 前置依赖 | 预估影响 |
|--------|------|---------|---------|---------|
| **P0** | 一：Feature 升级 | 所有后续阶段的基础 | 无 | 1 文件 |
| **P0** | 二：bevy_picking | 单位点击优化 | 阶段一 | ~5 文件 |
| **P0** | 三：Gizmos 调试 + 高亮修复 | 调试可视化 + Bug 修复 | 阶段一 | ~6 文件 |
| **P1** | 四：Reflect + bevy_scene | Inspector/序列化基础 | 阶段一 | ~30 文件 |
| **P1** | 五：CnFont 优化 | 统一字体管理 | 无 | ~5 文件 |
| **P2** | 六：GameSettings | 用户偏好/无障碍 | 阶段四（Reflect） | ~5 文件 |
| **P2** | 七：bevy_input_focus | 输入冲突解决 | 无 | ~5 文件 |
| **P2** | 八：调试基础设施 | 系统级调试 | 阶段四（Reflect） | ~3 文件 |
| **P3** | 九：相机控制器 | 相机体验提升 | 阶段一 | ~2 文件 |
| **P3** | 十：Reflect Functions | 数据驱动技能（前瞻） | 阶段四 | 设计文档 |
| **P3** | 十一：实验性 Feature | 未来观察 | 无 | 观察文档 |

---

## 风险与注意事项

### 已验证风险

1. **Gizmos 只能画线框**：`rect_2d()` 调用 `lineloop_2d()`，无法替代半透明填充 Sprite。正式范围高亮必须保留 Sprite 方式
2. **Picking 与 Grid 架构冲突**：为空格子创建 Entity 违反地图铁律。Picking 仅用于 Unit Sprite，格子点击保留逻辑计算
3. **`bevy_camera_controller` 需子 feature**：默认无功能，2D 项目需 `pan_camera`
4. **`clear_markers` 跨层调用**：AI 管线（intent.rs）依赖标记组件，删除需同步修改
5. **`UiTheme::default()` 混用**：highlight.rs 不响应运行时修改，需统一为 `Res<UiTheme>`

### 潜在风险

6. **Reflect derive 编译时间**：30+ 组件添加 Reflect 可能增加编译时间
7. **bevy_scene 序列化兼容性**：`DynamicSceneBuilder` 对复杂组件的序列化可能有限制
8. **bevy_camera_controller 成熟度**：0.18 版本可能不够完善，需评估
9. **reflect_functions 稳定性**：目前是实验性 API
10. **experimental features 不稳定**：`bevy_ui_widgets`/`bevy_feathers` API 可能随时变更

### best_practices.md 同步修正

以下内容需同步修正 `docs/best_practices.md`：

| 位置 | 问题 | 修正 |
|------|------|------|
| 12.2 必开 Feature | `bevy_world_serialization` 在 0.18.1 不存在 | 改为 `bevy_scene` |
| 12.5 bevy_world_serialization | `world.serialize()` / `World::deserialize()` 不是 0.18.1 API | 改为 `DynamicSceneBuilder` |
| 10.2 Gizmos 调试 | `Color::srgba()` 传入 `rect_2d` 只画线框 | 标注"仅线框，不填充" |
| 12.1 必开 Feature | 列出 8 个已被 `2d` 隐式包含的 Feature | 标注"已含于 2d" |

---

## 预期总收益

| 指标 | 当前 | 重构后 |
|------|------|--------|
| 手写单位遍历代码 | ~30 行 | 0 行（Picking 事件） |
| 调试可视化 | 无 | Gizmos 寻路/AI/占据网格 |
| UiTheme Bug | highlight.rs 不响应运行时修改 | 修复 |
| Reflect 覆盖率 | 0% | 100%（核心类型） |
| Inspector 能力 | 有限（无 Reflect） | 完整组件查看/修改 |
| 序列化能力 | 无 | DynamicScene 快照 |
| 调试能力 | 日志 + egui | +系统级调试+远程控制 |
| 输入焦点管理 | 无 | 完整焦点系统 |
| 字体管理 | 散落 5 处 | 统一接口 |
| 设置持久化 | 无 | GameSettings + RON |
