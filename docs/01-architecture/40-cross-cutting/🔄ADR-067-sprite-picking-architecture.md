---
id: 01-architecture.ADR-067
title: "ADR-067: Bevy Sprite Picking 架构"
status: Superseded
superseded_by: ADR-PICK-000
owner: architect
created: 2026-06-23
tags:
  - architecture
  - picking
  - input
  - sprite
  - ui
  - camera
  - cross-cutting
  - infra
  - presentation
---

# ADR-067: Bevy Sprite Picking 架构

## 状态

**Proposed**

## 背景

Fre SRPG 需要玩家点击 Sprite 单位实现单位选择。当前没有 picking 系统，Sprite 单位（纯色方块，通过 `Sprite { color, image: white_texture, custom_size }` 渲染）散落在世界空间（Camera2d），不可点击。

需求路径：点击蓝色方块（己方单位）→ 选中高亮 → ActionMenu 显示行动 → SkillPanel 显示技能。

当前相关架构：
- **Camera**: Camera2d + 自定义 Pose 系统（TargetPose/CurrentPose 分离），`write_to_transform` 在 PostUpdate 写入 Transform.translation
- **单位渲染**: Sprite 组件（含 custom_size），通过 SpriteBundle 在网格位置生成
- **UI**: 9-zone 全屏 BattleScreen 覆盖，Node 布局（无 BackgroundColor 根节点）
- **输入**: `infra/input/` 管理 InputAction，Phase 8 注册

## 引用的架构规则

- `docs/01-architecture/README.md` §4.2 — 四级通信机制（使用 trigger + Observer，禁止 EventWriter/EventReader）
- `docs/01-architecture/README.md` §2.1 — DDD 三层 + L2 Infra 层（Camera 在 infra/camera/）
- `docs/01-architecture/README.md` §6.1 — Plugin 注册顺序（Phase 0 DefaultPlugins → Phase 8 CameraPlugin）
- `ADR-064` — Camera 系统架构（Camera2d + Pose 分离 + write_to_transform in PostUpdate）
- `ADR-055` — UI 表现层架构（ViewModel + Projection + Screen 分离）
- `docs/06-ui/README.md` — UI/Presentation 架构总纲（逻辑与表现分离）
- `docs/02-domain/capabilities/` — Domain 设计不涉及 picking（picking 是输入/表现层问题）

### 引用说明

**领域规则**：Picking 是跨层基础设施问题，不独立对应某个 domain 规则文档。单位选择的选择域规则应在 tactical_domain.md 中定义，但 picking 机制本身（how to detect clicks）是纯基础设施问题。

**数据 Schema**：无需独立 picking Schema。Selection 资源由 @data-architect 定义（如已有 `docs/04-data/domains/tactical_schema.md`）。

**内容架构/Def**：Picking 不需要 Def 定义（无可加载配置）。

**UI 架构**：ViewModel（SkillPanelVm/BattleHudData 等）已存在，picking 结果将写入新的 Selection Resource 驱动这些 ViewModel。

## 评估：方案 A vs 方案 B

### 方案 A — 直接引入 bevy_sprite 的 SpritePickingPlugin

直接使用 `bevy_sprite::picking_backend::SpritePickingPlugin`，这是 Bevy 0.19 内置的 Sprite picking 后端。

#### Camera 兼容性分析

`sprite_picking` 系统（PreUpdate, PickingSystems::Backend）执行时：

1. 查询 cameras 并解构 `Projection::Orthographic(cam_ortho)` — Camera2d 默认使用 OrthographicProjection，兼容
2. 读取 `cam_transform`（GlobalTransform）用于光线计算 — GlobalTransform 在 PostUpdate 由 Bevy 的 TransformPropagate 更新
3. 使用 `RayMap.repopulate`（PreUpdate 处理）计算射线 — RayMap 也读取 GlobalTransform

**帧时序问题**：
- Fre 的 `write_to_transform` 在 PostUpdate 写入 Transform.translation
- Bevy 的 `TransformPropagate` 也在 PostUpdate 运行
- `sprite_picking` 在下一帧的 PreUpdate 运行
- 由于 `write_to_transform` 在默认 PostUpdate 顺序中运行于 `TransformPropagate` 之后，GlobalTransform 不反映最新 `write_to_transform` 的结果

**影响评估**：
- Camera2d + OrthographicProjection 技术上完全兼容
- 相机 GlobalTransform 滞后一帧（约 16ms @ 60fps）
- 在 SRPG 中，单位点击是**离散帧操作**，玩家在相机静止时点击，滞后不影响功能正确性
- Camera 和 Sprite 的 GlobalTransform 在同一时间坐标系，相对关系正确
- 滞后方向：相机移动中点击时，命中检测使用上一帧的相机位置，轻微偏移但不可感知
- 结论：**兼容，无需修改 Camera 系统**

**与 UI 的共存分析**：
- SpritePickingPlugin 和 Bevy UI 的 picking backend（Window backend）是独立后端
- 结果在 PickingPlugin 的 Hover 阶段合并排序
- UI Node 默认 `should_block_lower: true`（无 Pickable 组件时）
- 全屏 UI 会拦截 sprite 的 picking 事件

**解决方案**：在 UI 根节点设置 `Pickable::IGNORE`（`should_block_lower: false, is_hoverable: false`），使 UI 不阻挡 sprite 事件。具体可交互的 UI 元素（按钮等）单独设置 Pickable。

**对纯色 Sprite 的兼容性**：
- `SpritePickingMode::AlphaThreshold` 模式下，纯色 sprite（来自 `Sprite::from_color`）因为没有真实 Image asset，会默认视为"有效像素"（`break 'valid_pixel true`）
- 但更优方案：使用 `SpritePickingMode::BoundingBox`，因为 Fre 单位是方块 sprite，不需要 Alpha 检测
- `BoundingBox` 模式性能更好（无纹理采样），且对纯色 sprite 完全正确

**实现成本**：极低。仅需添加 plugin + observer，约 30 行代码。

### 方案 B — 自定义 Grid Backend

在 `bevy_picking::backend` 接口上实现自定义 Backend：读取 PointerLocation → 计算屏幕坐标 → Camera 转换到世界坐标 → 转换到网格坐标 → 查 GridMap 找单位。

#### 架构分析

**技术可行性**：
- `bevy_picking::backend` 提供了干净的接口：`PointerHits` 消息 + `RayMap` 资源
- 自定义 backend 只需在 `PickingSystems::Backend` set 中写一个 system，产生 `PointerHits`
- 50-100 行代码可实现基本功能（Bevy 官方文档称 custom backend 约 50 行）

**依赖耦合问题（关键问题）**：
- 需要导入 `core/domains/tactical/` 的 GridMap 类型来查单位位置
- 或者需要在 tactical domain 的 `integration/` 层暴露只读查询 API
- Backend 本身（输入检测）不应依赖业务领域类型
- 方案：Backend 只输出屏幕位置 → 网格位置的转换结果，网格→单位查找由 tactical domain 的 observer 完成

**与 SpritePickingPlugin 的冲突**：
- 两个 backend 可以共存
- SpritePickingPlugin 报告 Entity 级别的命中
- Grid Backend 报告网格级命中
- 两者会被 PickingPlugin 合并排序，可能产生重复事件

**实现成本**：
- 相机投影计算（viewport_to_world_2d 等）
- 屏幕坐标 → 网格坐标的数学转换
- GridMap 查询（Entity 查找）
- 测试覆盖坐标转换的正确性

### 两方案对比

| 维度 | 方案 A (SpritePickingPlugin) | 方案 B (Custom Grid Backend) |
|------|-----------------------------|------------------------------|
| **代码量** | ~30 行（plugin + observe） | ~150 行（backend + 坐标转换 + query） |
| **Bevy 版本风险** | 跟随 Bevy 版本更新 | 需要适配 Bevy 版本 |
| **Tiled 地图兼容性** | 需要调整（sprite bounds 变化） | 天然兼容（grid-based） |
| **Camera 耦合** | 无额外耦合（已由 Bevy 处理） | `Camera::viewport_to_world_2d` + GlobalTransform |
| **单位选择正确性** | Sprite BoundingBox 命中 | Grid 位置精确命中 |
| **性能开销** | CPU: AABB vs Sprite 变换计算 + 碰撞检测 | CPU: 相机投影 + GridMap 查找 |
| **hover/highlight** | Pointer<Over>/<Out> 原生支持 | 需要额外实现 |
| **测试成本** | Bevy 已有测试 | 需要坐标转换 + GridMap 集成测试 |
| **当前复杂度** | 零开发成本 | 非零（SRPG 中网格选择是正确做法） |

## 决策

**决策：方案 A（SpritePickingPlugin），分阶段实施，保留未来迁移到方案 B 的接口。**

理由：

1. **MVP 目标优先**：当前 Fre 核心需求是"让单位可点击"，不是"完美网格命中检测"。SpritePickingPlugin 一行配置即实现此目标。

2. **复杂度治理（宪法 SS1.2）**：只解决当前复杂度。当前单位是纯色 Sprite，Tiled 地图尚未集成，不存在网格精度问题。`SpritePickingMode::BoundingBox` 对方块 sprite 的命中检测 100% 正确。

3. **Bevy 0.19 原生支持**：SpritePickingPlugin 是 Bevy 官方库的一部分，跟随 Bevy 版本自动更新，零维护成本。

4. **相位隔离**：picking 机制（如何检测点击）与选择逻辑（点击后做什么）完全解耦。即使未来切换到 Grid Backend，`Selection` Resource 和 observer 系统不需修改。接口层（observer + `On<Pointer<Click>>`）不变。

5. **Camera 帧时序兼容**：上一帧 GlobalTransform 用于 picking 是 Bevy 标准模式，SRPG 中不影响正确性。

6. **UI 穿透简单**：只需在 UI 根节点加 `Pickable::IGNORE`。

7. **hover/out 事件原生**：Pointer<Over>/<Out> 事件可直接用于单位高亮（修改 Sprite.color），无需额外系统。

8. **避免重复投资**：Custom Grid Backend 的核心逻辑（屏幕→网格→单位查询）本质上与 Sprite 无关。当 Tiled 地图落地时，无论选择什么 backend，屏幕→网格的坐标转换仍需实现。这个转换放在 unit selection handler（业务层）比放在 picking backend（基础设施层）更合适，因为转换本身需要依赖 tactical domain 的网格系统。

## Module Design

### 新增模块

```
src/infra/picking/
├── mod.rs                 # 模块声明 + re-export
├── plugin.rs              # PickingPluginGroup（配置 picking 后端 + settings）
└── settings.rs            # PickingSettings / SpritePickingSettings 配置
```

### 修改文件

| 文件 | 变更 |
|------|------|
| `src/infra/picking/` | 新增 (如上) |
| `src/infra/mod.rs` | 新增 `pub mod picking;` |
| `src/infra/camera/plugin.rs` | 无变更（picking 不依赖 camera 内部） |
| `src/app/app_plugin.rs` | Phase 8 新增 `infra::picking::PickingPlugin` 注册 |
| `src/core/domains/tactical/` | 新增 `selection` 模块（unit_selection_handler observer） |
| `src/ui/screens/battle/` | 更新 selection → ViewModel 映射 |
| `docs/01-architecture/README.md` §3.4 | L2 Infra 表格新增 Picking 模块行 |
| `docs/01-architecture/README.md` §6.1 | Phase 8 新增 `infra::picking::PickingPlugin` |

### Plugin 注册位置

```rust
// Phase 8: Infrastructure (L2)
.add_plugins(RegistryPlugin)
.add_plugins(PipelinePlugin)
.add_plugins(ReplayPlugin)
.add_plugins(SavePlugin)
.add_plugins(InputPlugin)
.add_plugins(CameraPlugin)
.add_plugins(PickingPlugin)   // ← 新增：在 Camera 之后，确保 Camera/Transform 已就绪
.add_plugins(MapPlugin)
.add_plugins(LoggingPlugin)
.add_plugins(LocalizationPlugin)
```

PickingPlugin 必须在 CameraPlugin 之后注册（picking 依赖 Camera 组件）。必须在 UI Plugin（Phase 11）之前注册（UI 依赖 picking 事件）。

### PickingPlugin 内部结构

```rust
pub struct PickingPlugin;

impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        use bevy::sprite::SpritePickingPlugin;
        use bevy_picking::DefaultPickingPlugins;

        // 1. Bevy 核心 picking 管线（PointerInputPlugin + PickingPlugin + InteractionPlugin）
        //    注意：PickingPlugin 是 bevy_picking 中的系统名，与本模块名冲突
        //    实际使用 DefaultPickingPlugins 或单独显式注册
        app.add_plugins(bevy_picking::DefaultPickingPlugins);

        // 2. Sprite picking backend
        app.add_plugins(SpritePickingPlugin);

        // 3. 配置 SpritePickingMode（BoundingBox — 纯色方块不需要 Alpha 检测）
        app.insert_resource(SpritePickingSettings {
            picking_mode: SpritePickingMode::BoundingBox,
            require_markers: false,
        });

        // 4. 可选：配置全局 PickingSettings
        app.insert_resource(PickingSettings {
            // 禁用 window-level picking（减少 UI 树 picking 干扰）
            is_window_picking_enabled: false,
            ..default()
        });

        // 5. 注册 Selection Resource
        app.init_resource::<Selection>();
    }
}
```

**模块内部设计原则**：
- PickingPlugin 是一个纯组装模块 — 只做 Bevy plugin 注册和 settings 配置
- 不包含业务逻辑（业务逻辑在 Tactical domain 的 selection observer 中）
- 不创建新 picking 后端（仅注册已有后端）
- `infra/picking/` 的职责仅在 Phase 1-2；当需要 Grid Backend 时（Phase 3+），在此模块中追加

## Communication Design

### 四级通信映射

| 通信方向 | 机制 | 说明 |
|---------|------|------|
| Sprite → Picking Backend | PointerHits (Message) | SpritePickingPlugin 内部产出，消费端不感知 |
| Picking → Entity Observer | `commands.trigger(Pointer<Click>)` | Bevy 原生 Observer 事件 |
| Entity Observer → Selection | `ResMut<Selection>` | 直接 Resource 写操作 |
| Selection → UI ViewModel | Observer + Dirty flag | 通过 Projection 系统 |
| Hover → Sprite.color | `On<Pointer<Over>>` observer | 直接修改 Sprite.color |

### 事件流

```
用户点击 Sprite
    │
    ▼
PickingPipeline (PreUpdate)
    ├── PointerInputPlugin: 收集鼠标输入
    ├── PickingPlugin: RayMap 构建 + Backend 注册
    ├── SpritePickingPlugin: sprite_picking → PointerHits
    ├── hover: 合并排序 → HoverMap
    └── InteractionPlugin: pointer_events → Pointer<Click> trigger
    │
    ▼
Selection Observer (Update)
    ├── On<Pointer<Click>> 触发
    ├── Entity 识别 + BattleUnitId 查找
    ├── 写入 Selection Resource
    └── 触发 selection_changed Event (可选)
    │
    ▼
Projection System (PostUpdate)
    ├── 读取 Selection Resource
    ├── 更新 BattleHudData / SkillPanelVm
    └── 标记 Dirty<T>
    │
    ▼
UI Render (Update/PostUpdate)
    └── Widget 读取 ViewModel → 渲染
```

### Observer 注册位置

Observer 应注册在 Entity（单位 Sprite）上，而非全局：

```rust
// 单位生成时：
commands.spawn((
    Sprite { ... },
    Transform { ... },
    Pickable::default(),
    BattleUnitMarker,  // 标记 Tag Component
    // ...其他单位组件
)).observe(on_unit_click)  // ← observer 挂在每个单位实体上
  .observe(on_unit_hover);
```

事件通过 Entity hierarchy 冒泡（PointerTraversal），父实体可以统一拦截子实体的点击。

### 跨 Phase 设计

```
Phase 1 (MVP):  SpritePickingPlugin + Pickable + on_unit_click → Selection → console.log
Phase 2:         Selection → BattleHudData → SkillPanelVm UI 联动
Phase 3:         选中高亮 → Pointer<Over>/<Out> 修改 Sprite.color
Phase 4 (Future): Grid Backend 可插入替代 SpritePickingPlugin
```

核心隔离层：所有业务逻辑通过 `Selection` Resource 解耦。无论 picking 是来自 Sprite 还是 Grid，Selection Resource 接口不变。

## Critical Analysis: Camera Frame Timing

`sprite_picking` 系统在 PreUpdate 读取 GlobalTransform，而 Camera 的 `write_to_transform` 在 PostUpdate 写入 Transform。Bevy 的 `TransformPropagate` 也在 PostUpdate。

### 具体时序

```
Frame N:
  PostUpdate:
    1. TransformPropagate (Bevy core) — 将 Transform → GlobalTransform
    2. write_to_transform (Fre camera) — 写入 Transform.translation (发生在 GlobalTransform 更新之后!)
  // Camera GlobalTransform 在 frame N 结束时未反映 write_to_transform

Frame N+1:
  PreUpdate:
    1. RayMap.repopulate — 读取 Camera GlobalTransform (= Frame N 的 Transform 传播结果)
    2. sprite_picking — 使用 RayMap 射线进行碰撞检测
  // Picking 使用 Frame N 的最终位置 (不是 Frame N+1 的最新位置)
```

### 影响评估

- **正确性**：Camera 和 Sprite 的 GlobalTransform 在同一时间坐标系（都来自 Frame N 的 TransformPropagate），相对关系完全正确
- **精度**：命中检测位置偏差 = Frame N+1 的相机位移量。对于 SRPG（相机移动慢，100-200 px/s），偏差 < 3 px/frame @ 60fps
- **功能影响**：玩家在相机移动中点击的概率极低（SRPG 点击操作通常在相机静止时进行）
- **结论**：无需修改 Camera 时序。如需修正（Phase 3+），可将 `write_to_transform` 重新调度到 `TransformPropagate` set 中：

```rust
// 可选修正方案（Phase 3+）
app.add_systems(PostUpdate, (
    movement::interpolate_pose,
    bounds::clamp_position,
    shake::apply_shake,
    movement::write_to_transform.before(TransformPropagate),
));
```

## 边界定义

### 允许

- 任何 entity 通过 `.observe(|ev: On<Pointer<Click>>| ...)` 响应点击
- UI 根节点设置 `Pickable::IGNORE` 实现 UI 穿透
- 使用 `SpritePickingMode::BoundingBox`（纯色方块不需要 Alpha 检测）
- `infra/picking/` 模块内部引用 `bevy_picking` 和 `bevy_sprite` 类型
- Tactical domain 的 selection handler 读取 Selection Resource
- UI Projection 系统读取 Selection Resource 更新 ViewModel
- Phase 3+ 将 write_to_transform 重新调度到 `TransformPropagate` 之前
- Phase 4+ 在 `infra/picking/` 追加 Grid Backend（保持接口一致）

### 禁止

- infra/picking/ 模块引入任何 `core/domains/*` 的类型（picking 是基础设施，不应依赖业务域）
- picking backend 直接修改业务数据（如插入 Selection Resource 或修改 Sprite.color）
- Tactical domain 直接依赖 bevy_picking 类型（通过 EntityEvent 解耦）
- 使用 `EventWriter<Pointer<Click>>` / `EventReader<Pointer<Click>>`（必须用 Observer `On<Pointer<Click>>`）
- 在 `Pickable` 字段中硬编码 `should_block_lower: false`（对需要阻挡的元素显式设置）
- 在 `infra/picking/` 之外创建 `PickingSettings` / `SpritePickingSettings` 资源（配置集中管理）
- Camera 的 write_to_transform 在 PostUpdate 默认位置（允许但推荐在 Phase 3+ 优化时序）
- 为尚未确定的 Tiled Map 场景提前实现 Grid Backend（复杂度治理）

## Forbidden（禁止事项）

| 禁止行为 | 理由 |
|---------|------|
| infra/picking/ 引入 core::domains::* 类型 | 违反层级依赖方向（L2 Infra 不依赖 L1 Core Domains） |
| Tactical domain 直接 import bevy_picking::* 类型 | picking 是基础设施细节，业务层不应耦合 |
| 全局 EventWriter/EventReader 用于 picking 事件 | 违反 ADR-054，必须用 Observer |
| 使用 EventWriter<Pointer<Click>> 代替 .observe() | Observer 是 Bevy 0.19 标准方式 |
| 在非 UI 交互元素上不设置 Pickable::IGNORE 导致 picking 被拦截 | 全屏 UI Node 会意外阻挡 sprite picking |
| 在 entity 上添加多个 `On<Pointer<Click>>` observer 导致重复响应 | 点击事件应一次处理 |
| Selection Resource 存储 Entity（使用 BattleUnitId 等稳定 ID） | Entity 生命周期不稳定，可能被回收 |
| 绕过 Selection Resource 直接在 observer 中修改 ViewModel | 违反 Projection 架构（UI 应观察状态，而不是被直接修改） |
| picking 使用非确定性随机源 | 违反 Replay First 原则 |

## Definition / Instance Design

| 类型 | 层级 | 存储 | 可变性 | 说明 |
|------|------|------|--------|------|
| `PickingSettings` | Infra Config | Resource | 运行时配置 | Bevy 原生，在 plugin.rs 初始化 |
| `SpritePickingSettings` | Infra Config | Resource | 运行时配置 | 设置 SpritePickingMode::BoundingBox |
| `Pickable` | Infra Component | ECS Component | 每 Entity | 标记哪些 entity 可被 picking 检测 |
| `Selection` | Business State | Resource (tactical domain) | 运行时可变 | 当前选中单位 ID |
| `SelectionHighlight` | Presentation State | Component | 运行时可变 | 高亮标记（Phase 3） |
| `Pointer<Click>` | Transient | Event (Observer) | 瞬时 | Bevy 原生事件 |
| `Pointer<Over>/<Out>` | Transient | Event (Observer) | 瞬时 | hover 高亮驱动 |

**Selection Resource 建议 Schema**（由 @data-architect 完善）：

```rust
/// Selection — 当前选中状态。
///
/// 由 Tactical domain 的 unit selection observer 维护。
/// UI Projection 系统读取此 Resource 更新 ViewModel。
/// 使用 BattleUnitId 而非 Entity（Entity 生命周期不稳定）。
#[derive(Resource, Default, Debug, Clone, Reflect)]
#[reflect(Resource, Default, Debug)]
pub struct Selection {
    /// 当前选中的单位（None = 未选中）
    pub selected_unit: Option<BattleUnitId>,
    /// 最近 hover 的单位（用于 Tooltip/Preview）
    pub hovered_unit: Option<BattleUnitId>,
    /// 选择状态
    pub phase: SelectionPhase,
}

pub enum SelectionPhase {
    /// 未选择任何单位
    Idle,
    /// 已选择单位，等待行动选择
    UnitSelected,
    /// 已选择行动，等待目标选择
    ActionSelected,
    /// 技能目标选择中
    Targeting,
}
```

## 后果

### 正面

1. **零开发成本的 MVP** — SpritePickingPlugin 一行配置即可用，即刻获得单位点击能力
2. **标准 Bevy 模式** — Observer + Pickable + PointerEvent 是 Bevy 0.19 推荐的 picking 方式，无技术债
3. **Phase 隔离** — Selection Resource 作为唯一桥梁，picking 机制和业务逻辑完全解耦
4. **hover/out 原生** — 单位高亮通过 `On<Pointer<Over>>` 直接实现，无需额外系统
5. **UI 兼容明确** — 明确的 Pickable::IGNORE 策略，UI 不干扰 picking
6. **未来可迁移** — `infra/picking/` 模块作为抽象边界，Phase 4+ 无缝切换到 Grid Backend
7. **Camera 帧时序不影响功能** — 上一帧 GlobalTransform 用于 picking 在 SRPG 场景中无感知

### 负面

1. **非 SRPG 原生** — 网格选择比 Sprite BoundingBox 更符合 SRPG 语义（但当前精度足够）
2. **Camera 帧时序欠优化** — `write_to_transform` 在 `TransformPropagate` 之后运行，导致 picking 使用上一帧 GlobalTransform（Phase 3+ 可修复）
3. **UI Pickable 配置负担** — 每个 UI 组件需要明确配置 Pickable 策略（根节点 IGNORE，交互元素默认）
4. **Tiled 地图切换成本** — 当 Map 系统切换为 Tiled 时，可能需要切换到 Grid Backend（接口不变，但 backend 替换）
5. **UI backdrop 可能拦截事件** — 如果 UI 节点 `should_block_lower=true` 且全屏覆盖，Sprite picking 被阻断（已通过 Pickable::IGNORE 解决）

### 分阶段实施计划

```
Phase 1 (MVP — 当前任务):
  - 新增 infra/picking/ 模块 + PickingPlugin
  - 注册 DefaultPickingPlugins + SpritePickingPlugin
  - 设置 SpritePickingMode::BoundingBox
  - 在单位 Sprite Entity 上添加 Pickable::default()
  - 在 UI 根节点添加 Pickable::IGNORE
  - 在单位 Entity 上注册 on_unit_click observer
  - on_unit_click 写入 Selection Resource → console.log
  - 测试：点击单位 → console 输出

Phase 2 (UI 联动):
  - Selection → BattleHudData / SkillPanelVm Projection
  - 选中单位后显示 ActionMenu
  - ActionMenu 行动 → SkillPanel 显示技能

Phase 3 (Hover 高亮):
  - 单位 Entity 注册 on_unit_hover/on_unit_out observer
  - Over 时修改 Sprite.color（高亮）
  - Out 时恢复 Sprite.color（取消高亮）
  - 可选：调整 write_to_transform 时序到 TransformPropagate 之前

Phase 4 (Grid Backend 可插拔 — Future):
  - 在 infra/picking/ 追加 grid_backend.rs
  - 实现 screen → GridPos → Entity 的 picking 后端
  - 通过 feature flag 切换 Sprite vs Grid backend
  - Selection Resource 接口不变
```

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 完全不用 picking 框架，用鼠标位置 + AABB 手动检测 | 重复实现 Bevy 已提供的功能（hover/out/click/drag/bubble），增加维护成本 |
| 仅用 Grid Backend（方案 B）| 当前 Tiled 地图未落地，网格选择带来的精度收益在纯色方块阶段为零。开发成本 ~150 行，远高于方案 A 的 ~30 行 |
| 自定义 Backend + SpritePickingPlugin 同时启用 | 两者会竞争产生 PointerHits，需要复杂的合并逻辑。当前无 Grid Map 时，Sprite 检测已足够 |
| 把 picking 放到 core/domains/ 中 | Picking 是基础设施（输入检测），不包含业务规则。放在 Domains 违反 DDD 依赖方向 |
| 把 picking 放到 ui/ 中 | Picking 检测独立于 UI 渲染层（在 PreUpdate 执行），与 UI 的 Schedule（Update/PostUpdate）不一致 |
| 全用 Bevy UI 上的 Button/Interaction 组件 | 单位在 World Space (Camera2d)，不在 UI Space (Node)。UI picking 和 Sprite picking 是不同的后端 |
| 使用 `EventWriter<PointerClick>` 手动模拟点击 | 违反 ADR-054（必须用 Observer）。且需要重新实现 Bevy 已完整的 hover/click 语义 |
| 不添加 Pickable::IGNORE 在 UI 根节点，而是设置 `should_block_lower: false, is_hoverable: false` | 两种写法结果相同（Pickable::IGNORE 是 const 等价物）。IGNORE 更简洁（2 字段 vs const 常量） |

## 架构合规性自检

- [x] 符合 ECS 约束（Pickable=Component, sprite_picking=System, Pointer=Event）
- [x] 双轴边界合规：Capabilities 无 picking 规则，Domains 无 picking 机制（picking 纯基础设施）
- [x] Domain 间无直接依赖 — picking 在 Infra 层，通过 Selection Resource 桥接到业务层
- [x] infra/picking/ 是独立模块（Config + Plugin 组装），不创建 components.rs/systems.rs/utils.rs
- [x] Effect/Modifier Pipeline 没有被绕过 — picking 不涉及战斗数值
- [x] 符合"定义与实例分离"原则 — Pickable Instance 在 Entity，Selection Resource 是运行时状态
- [x] 符合"规则与内容分离"原则 — picking 行为由代码规则定义，无 RON 配置
- [x] 符合"逻辑与表现分离"原则 — picking 检测（PreUpdate）与 UI 渲染（PostUpdate）分离
- [x] 使用 trigger + Observer（符合 ADR-054）
- [x] Plugin 注册顺序符合层次要求（Phase 8，Camera 之后，UI 之前）
- [x] Camera 帧时序已分析并确认不影响功能
- [x] UI 穿透方案已明确（Pickable::IGNORE）
- [x] 分阶段实施计划已定义
- [x] 所有 Forbidden 已明确列出（8 项）
- [x] 备选方案已分析并给出放弃理由（7 项）

## 后续工作

### 需要其他 Agent 补充

1. **@domain-designer**: 补充 `docs/02-domain/domains/tactical_domain.md` 中的"单位选择"领域规则（什么是可选中的单位、选择状态机、选择→行动流程）
2. **@data-architect**: 设计 `Selection` Resource 的 Schema（在 `docs/04-data/domains/tactical_schema.md`），确保使用 BattleUnitId 而非 Entity，兼容 Replay/Save
3. **@presentation-architect**（如需）: 如果 Battlescreen UI 需要 picking 交互（如点击 UI 按钮），补充 UI picking 穿透规范
4. **@feature-developer**: Phase 1 MVP 实现（infra/picking/ 模块创建 + observer + Selection console.log）
