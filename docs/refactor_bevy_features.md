# 基于 Bevy 0.18 官方特性的激进重构方案

> 依据：`docs/11.md` Bevy 0.18 Feature 价值评估
> 原则：用官方能力替换自造轮子，优先最佳实践，不考虑短期成本

---

## 现状诊断

| 自造轮子 | 代码位置 | Bevy 0.18 官方替代 |
|----------|----------|-------------------|
| 手写鼠标拾取（cursor_to_coord） | `input.rs:157-172` | `bevy_picking` + `sprite_picking` |
| 手写范围高亮（spawn Sprite） | `ui/highlight.rs` | `bevy_gizmos` |
| 手写相机控制器 | `ui/camera.rs` | `bevy_camera_controller`（研究参考） |
| 手写中文字体管理（CnFont） | `assets.rs` | `system_font_discovery` |
| 硬编码 UI 主题常量 | `ui/theme.rs` | `bevy_settings` |
| 无 Reflect 体系 | 全项目零 Reflect derive | `reflect_auto_register` |
| 无 World 序列化 | 无法存档/快照/回放 | `bevy_world_serialization` + `serialize` |
| 无输入焦点管理 | UI 输入与游戏输入冲突 | `bevy_input_focus` |
| 无系统级调试 | 只能靠日志 | `bevy_debug_stepping` + `track_location` |
| 无运行时控制台 | 无法运行时修改属性 | `bevy_remote` |

---

## 阶段一：Cargo.toml Feature 升级

### 目标
启用所有 11.md 推荐的 Feature，建立基础设施。

### 任务

#### 1.1 升级 Cargo.toml
```toml
[dependencies]
bevy = { version = "0.18.1", features = [
    "2d",
    # 必开
    "bevy_state",
    "bevy_ui",
    "bevy_text",
    "bevy_sprite",
    "bevy_gizmos",
    "bevy_log",
    "file_watcher",
    "multi_threaded",
    "png",
    # 强烈推荐
    "bevy_picking",
    "sprite_picking",
    "ui_picking",
    "bevy_dev_tools",
    "bevy_debug_stepping",
    "bevy_remote",
    "bevy_world_serialization",
    "serialize",
    "reflect_auto_register",
    "track_location",
    # 推荐
    "bevy_input_focus",
    "bevy_settings",
    "system_font_discovery",
    "bevy_ci_testing",
    "schedule_data",
] }

[features]
default = []
dev = ["bevy/file_watcher", "bevy/bevy_dev_tools", "bevy/bevy_debug_stepping"]
release = []  # 发布时关闭 dev_tools/debug_stepping/track_location/bevy_remote/file_watcher
```

#### 1.2 验证编译
- `cargo build` 通过
- `cargo test` 通过
- 确认无 feature 冲突

---

## 阶段二：bevy_picking 替换手写鼠标拾取

### 目标
用 `bevy_picking` + `sprite_picking` + `ui_picking` 替换 `cursor_to_coord` 手动射线检测。

### 当前问题
- `input.rs` 手动计算 `viewport_to_world_2d` → `world_to_coord`
- 每个点击都需要遍历所有单位查找匹配坐标
- UI 按钮和游戏场景的点击没有统一处理

### 任务

#### 2.1 为地图格子添加 Pickable
- 每个 Tile Sprite 添加 `Pickable` 组件
- Tile 携带 `GridPosition`，picking 回调直接获取坐标
- 无需手动 `cursor_to_coord`

#### 2.2 为单位 Sprite 添加 Pickable
- 每个 Unit Sprite 添加 `Pickable` 组件
- 点击单位直接获得 Entity，无需遍历

#### 2.3 用 Picking 事件重写 input.rs
- 监听 `Pointer<Click>` / `Pointer<Over>` / `Pointer<Out>` 事件
- 替换 `handle_click` 中的手动坐标计算
- `HoveredEntity` 通过 `Pointer<Over>` 自动更新

#### 2.4 删除 cursor_to_coord
- 删除 `input.rs` 中的 `cursor_to_coord` 函数
- 删除对 `Camera`/`GlobalTransform`/`Window` 的手动查询

#### 2.5 UI 按钮统一 Picking
- 行动菜单按钮通过 `ui_picking` 处理
- 删除 `action_menu.rs` 中的手动按钮交互检测

### 预期收益
- 删除 ~50 行手动射线检测代码
- 点击精度提升（像素级 vs 格子级）
- UI 和游戏场景统一交互模型

---

## 阶段三：bevy_gizmos 替换 Sprite 范围高亮

### 目标
用 `bevy_gizmos` 替换 spawn Sprite 方式的移动/攻击范围显示。

### 当前问题
- `show_move_range` 每次调用 spawn 数十个 Sprite Entity
- `show_attack_range` 同样 spawn Sprite
- `clear_markers` 需要手动 despawn 所有标记 Entity
- 大量 Entity 创建/销毁影响性能

### 任务

#### 3.1 移动范围用 Gizmos 绘制
```rust
fn draw_move_range(mut gizmos: Gizmos, /* query reachable tiles */) {
    for (coord, _) in reachable {
        let rect = Rect::from_center_size(world_pos, Vec2::splat(tile_size));
        gizmos.rect_2d(rect.center(), rect.size(), theme.movable_range);
    }
}
```

#### 3.2 攻击范围用 Gizmos 绘制
- 同理，用 `gizmos.rect_2d()` 绘制

#### 3.3 选中高亮用 Gizmos 绘制
- 用 `gizmos.rect_2d()` 替换 `SelectionHighlight` Sprite

#### 3.4 删除标记组件和清理系统
- 删除 `MovableRange`、`AttackRange`、`SelectionHighlight` 组件
- 删除 `clear_markers`、`clear_selection` 函数
- 删除 `show_move_range`、`show_attack_range`、`spawn_selection_highlight` 函数
- Gizmos 每帧自动清除，无需手动清理

#### 3.5 寻路调试用 Gizmos
- 在 Debug 模式下用 Gizmos 显示寻路路径
- 用不同颜色显示路径代价

### 预期收益
- 删除 ~120 行 Sprite 标记代码
- 消除 Entity 创建/销毁开销
- Gizmos 自动清除，无内存泄漏风险
- 调试时可视化更灵活

---

## 阶段四：Reflect 体系 + 序列化基础

### 目标
为所有 Component/Resource 添加 Reflect，建立 Inspector/编辑器/序列化基础。

### 当前问题
- 全项目零 Reflect derive
- 无法在 Inspector 中查看/修改组件
- 无法序列化 World（存档/快照/回放）
- BattleRecord 只能用 serde，无法利用 Bevy Reflect

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

需要添加 Reflect 的组件：
- `Unit`, `UnitName`, `Dead`, `Selected`
- `GridPosition`, `MovingUnit`
- `Attributes`, `GameplayTags`, `TraitGrantedTags`
- `SkillSlots`, `SkillCooldowns`
- `ActiveBuffs`
- `TraitCollection`
- `AiBehaviorId`
- `EquipmentSlots`, `CombatInventory`

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
- 各 Registry（SkillRegistry, BuffRegistry 等）

#### 4.3 核心 Enum/Struct 添加 Reflect
- `Faction`, `AttributeKind`, `ModifierOp`
- `GameplayTag`, `TagName`
- `SkillTargeting`, `SkillCondition`
- `BuffInstance`, `TraitEffect`
- `BattleEntry`

#### 4.4 注册 Reflect 类型
```rust
app.register_type::<Unit>()
    .register_type::<Attributes>()
    .register_type::<GridPosition>()
    // ...
```

#### 4.5 启用 bevy_world_serialization
- 验证 World 序列化/反序列化
- 为存档系统建立基础
- 为 Golden Test 提供快照能力

### 预期收益
- Inspector 自动显示所有组件字段
- 编辑器/工具链的基础设施
- 存档/快照/回放的技术基础
- `bevy_remote` 可运行时查看/修改属性

---

## 阶段五：system_font_discovery 替换手写字体管理

### 目标
用 `system_font_discovery` 替换手写 `CnFont` 资源管理。

### 当前问题
- `assets.rs` 手动加载 `fonts/Arial Unicode.ttf`
- `CnFont` Resource 全局持有字体 Handle
- 打包时需要携带字体文件（几 MB）
- 所有 UI 组件需要手动设置 `text_font.font = cn_font.handle.clone()`

### 任务

#### 5.1 启用 system_font_discovery
- Cargo.toml 已添加 feature
- Bevy 自动发现系统已安装的中文字体

#### 5.2 删除 CnFont Resource
- 删除 `src/assets.rs` 中的 `CnFont` struct 和 `init_cn_font` 系统
- 删除所有 `Res<CnFont>` 依赖

#### 5.3 删除字体文件
- 删除 `assets/fonts/Arial Unicode.ttf`

#### 5.4 UI 组件使用默认字体
- Bevy 发现系统字体后自动使用
- 删除 `setup_ui_font` 系统中的手动字体设置
- 如果需要指定字体，使用 `FontHandle` 通过 AssetServer 加载系统字体

### 预期收益
- 删除 ~30 行字体管理代码
- 减少打包体积（不携带字体文件）
- 跨平台自动适配系统字体

---

## 阶段六：bevy_settings 替换硬编码主题

### 目标
用 `bevy_settings` 替换 `UiTheme` 中的硬编码常量。

### 当前问题
- `ui/theme.rs` 中所有颜色/字号/间距都是硬编码常量
- 无法运行时调整
- 无法持久化用户偏好
- 无法支持无障碍（色盲模式、字号放大）

### 任务

#### 6.1 创建 GameSettings Resource
```rust
#[derive(Resource, Reflect, Serialize, Deserialize)]
#[reflect_resource]
pub struct GameSettings {
    pub audio: AudioSettings,
    pub video: VideoSettings,
    pub input: InputSettings,
    pub accessibility: AccessibilitySettings,
    pub ui: UiSettings,
}
```

#### 6.2 迁移 UiTheme 常量到 GameSettings
- 字号 → `ui.font_sizes`
- 颜色 → `ui.colors`
- 间距 → `ui.spacing`
- 支持 RON 配置文件覆盖

#### 6.3 添加无障碍设置
- 色盲模式（替换颜色方案）
- 字号缩放因子
- 自动战斗速度

#### 6.4 设置持久化
- 使用 `bevy_settings` 自动保存/加载
- 配置文件路径：`settings.ron`

### 预期收益
- 用户可自定义 UI 外观
- 支持无障碍功能
- 配置持久化
- 为未来设置菜单提供数据源

---

## 阶段七：bevy_input_focus 解决输入冲突

### 目标
用 `bevy_input_focus` 统一管理 UI 和游戏输入焦点。

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

#### 8.3 bevy_remote
- 运行时通过控制台查看/修改 Entity 属性
- 运行时查看 Resource 状态
- GM 工具基础

#### 8.4 schedule_data
- 分析 System 执行顺序和依赖关系
- 可视化 System 调度图

### 预期收益
- 从"只能靠日志"升级到"系统级调试"
- Buff 链/Observer 链问题可逐步追踪
- 运行时修改属性，无需重编译

---

## 阶段九：bevy_camera_controller 研究与适配

### 目标
研究官方相机控制器，替换或参考改进手写相机。

### 当前问题
- `ui/camera.rs` 手写 WASD 平移 + 滚轮缩放
- 缺少边缘滚屏、聚焦角色、平滑移动
- 缺少相机边界限制

### 任务

#### 9.1 研究官方 bevy_camera_controller
- 了解 PanCamera/FreeCamera API
- 评估是否可直接使用

#### 9.2 如果可用：替换手写相机
- 删除 `CameraController` 组件
- 删除 `camera_control` 系统
- 使用官方相机控制器

#### 9.3 如果不可用：参考改进
- 添加边缘滚屏
- 添加聚焦角色功能
- 添加平滑移动/缩放
- 添加相机边界限制

#### 9.4 战棋专用相机功能
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

### 当前问题
- 技能效果通过 `EffectHandler` trait 分发
- 新增效果类型需要修改代码（添加新的 Handler 实现）
- 无法通过配置定义新效果类型

### 任务

#### 10.1 研究 reflect_functions API
- 了解 FunctionRegistry 机制
- 评估与当前 EffectHandler 的兼容性

#### 10.2 设计 Reflect-based 效果系统
```rust
// 未来：效果函数注册到 FunctionRegistry
app.register_function("apply_damage", apply_damage_fn);
app.register_function("apply_heal", apply_heal_fn);
app.register_function("apply_buff", apply_buff_fn);

// 配置中引用函数名
effects: [
    { function: "apply_damage", params: { amount: 50 } },
    { function: "apply_buff", params: { buff_id: "burn", duration: 3 } },
]
```

#### 10.3 渐进迁移
- 保留当前 EffectHandler 作为 fallback
- 新效果类型优先使用 reflect_functions
- 逐步迁移旧效果到新系统

### 预期收益
- 新增效果类型无需修改代码
- 配置中可直接引用效果函数
- 为脚本系统/Mod 系统提供基础
- 技能编辑器可动态选择效果函数

---

## 执行优先级

| 优先级 | 阶段 | 核心价值 | 预估影响 |
|--------|------|---------|---------|
| **P0** | 阶段一：Feature 升级 | 所有后续阶段的基础 | 1 文件 |
| **P0** | 阶段二：bevy_picking | 删除手写拾取，统一交互 | ~5 文件 |
| **P0** | 阶段三：bevy_gizmos | 删除 Sprite 标记，性能提升 | ~4 文件 |
| **P1** | 阶段四：Reflect 体系 | Inspector/序列化/编辑器基础 | ~30 文件 |
| **P1** | 阶段五：system_font | 删除字体管理，减少包体积 | ~8 文件 |
| **P2** | 阶段六：bevy_settings | 用户偏好/无障碍 | ~3 文件 |
| **P2** | 阶段七：bevy_input_focus | 输入冲突解决 | ~5 文件 |
| **P2** | 阶段八：调试基础设施 | 系统级调试 | ~3 文件 |
| **P3** | 阶段九：相机控制器 | 相机体验提升 | ~2 文件 |
| **P3** | 阶段十：Reflect Functions | 数据驱动技能（前瞻） | 设计文档 |

---

## 风险与注意事项

1. **bevy_picking 与 Grid 系统兼容性**：picking 是像素级，需要确保与格子坐标的映射正确
2. **bevy_gizmos 性能**：大量格子时 Gizmos 绘制性能需验证
3. **Reflect derive 宏爆炸**：30+ 组件添加 Reflect 可能导致编译时间增加
4. **system_font_discovery 跨平台**：Linux/Windows/macOS 系统字体路径不同
5. **bevy_camera_controller 成熟度**：0.18 版本可能不够完善，需评估
6. **reflect_functions 稳定性**：目前可能还是实验性 API

---

## 预期总收益

| 指标 | 当前 | 重构后 |
|------|------|--------|
| 手写拾取代码 | ~50 行 | 0 行（官方 picking） |
| Sprite 标记代码 | ~120 行 | 0 行（Gizmos） |
| 字体管理代码 | ~30 行 | 0 行（系统字体） |
| 硬编码主题常量 | ~80 行 | 配置驱动 |
| Reflect 覆盖率 | 0% | 100%（核心类型） |
| 序列化能力 | 无 | World 级序列化 |
| 调试能力 | 日志 + egui | +系统级调试+远程控制 |
| 输入焦点管理 | 无 | 完整焦点系统 |
| 包体积 | 含字体文件 | 减少字体文件 |
