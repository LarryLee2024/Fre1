---
id: 06-ui.camera-ui-interaction
title: Camera-UI Interaction Design — 镜头与表现层交互规则
status: draft
owner: presentation-architect
created: 2026-06-21
tags:
  - ui
  - camera
  - interaction
  - dataflow
  - infra
  - cross-cutting
  - presentation
---

# Camera-UI Interaction Design — 镜头与表现层交互规则

> **职责**: @presentation-architect | **上游**: ADR-064 (Camera 架构), ADR-055 (UI 表现层架构)
> **依赖方向**: UI → Infra/Camera (允许), Infra/Camera → UI (禁止)
> **通信机制**: CameraQuery (只读), CameraRequest (Event trigger), CameraInputBlock (共享 Resource)

> **SSPEC参考**: docs/06-ui/07-specs/ — AI-Consumable Screen Specification 标准。新增 Screen 必须先写 SSPEC，见 ADR-066。

---

## 1. 背景与定位

### 1.1 Camera 与 UI 的关系

Camera (`src/infra/camera/`, L2 Infra) 和 UI (`src/ui/`, L3 Presentation) 是两个独立的 Presentation 组件，通过**定义好的 API 契约**交互：

```
  Camera (L2 Infra)                   UI (L3 Presentation)
┌──────────────────────┐     ┌────────────────────────────┐
│  CameraQuery         │ ←── │  读取 Camera 状态          │
│  (SystemParam + 纯函数) │     │  (坐标转换/可视区域查询)     │
│                      │     │                            │
│  CameraState         │ ←── │  读取镜头状态              │
│  (Component)         │     │  (缩放级别/镜头模式显示)     │
│                      │     │                            │
│  CameraRequest       │ ──→ │  UI 发送镜头控制请求        │
│  (commands.trigger)  │     │  (聚焦/跟随/锁定输入)       │
│                      │     │                            │
│  CameraInputBlock    │ ←── │  UI 设置输入阻塞标志        │
│  (Resource)          │     │  (面板打开时阻止镜头移动)     │
└──────────────────────┘     └────────────────────────────┘
```

**核心约束**（来自 ADR-064 §禁止 和 ADR-055 §1）：
- UI 可以依赖 Infra/Camera（通过 CameraQuery、CameraRequest type import）— 符合 ADR-055 §1 的 `UI → Infra/Input (通过接口，允许)`
- Camera 不依赖任何 UI 类型 — Camera 是通用基础设施，不知道 Screen/Widget/Overlay 的存在
- Camera 不依赖任何 Domain 类型 — 使用纯 Vec2/CameraTarget(UnitId/TilePos/Vec2) 隔离
- UI 不直接修改 Camera Transform — 必须通过 CameraRequest

### 1.2 术语定义

| 术语 | 定义 |
|------|------|
| CameraQuery | `infra::camera::query::CameraQuery` SystemParam，提供坐标转换和可见区域查询 |
| CameraRequest | `infra::camera::foundation::request::CameraRequest` Event，UI 通过 trigger() 发送 |
| CameraInputBlock | `ui::application::camera_block::CameraInputBlock` Resource，UI 设置阻塞标志 |
| CameraStateVm | `ui::view_models::camera_state::CameraStateVm` Resource，UI 读取的镜头状态投影 |
| World Pos | 游戏世界坐标 (Vec2)，单位为像素/世界单位 |
| Screen Pos | 屏幕像素坐标 (Vec2)，原点在屏幕左上角 |

### 1.3 调度时序

Camera 系统和 UI 系统的调度时序决定了数据的一致性：

```
Phase 8 (Infra):
  PreUpdate:  Camera input_handler (消费 InputAction)
  Update:     Camera state_machine (处理 CameraRequest)
  PostUpdate: Camera movement (插值 → 写入 Transform)

Phase 11 (UI):
  Update:     UI Systems (读取 CameraQuery, 触发 CameraRequest)
```

**关键时序保证**：当 UI 在 Phase 11 读取 CameraQuery 时，Camera 已经完成 Phase 8 的全部处理（包括状态机转换和 Transform 写入）。因此 UI 读取的 Camera 状态总是最新帧的。

---

## 2. 数据流模式

### 2.1 Camera → UI: 读取 Camera 状态

UI 通过 CameraQuery 读取 Camera 状态。CameraQuery 是纯函数集合，封装了对 Bevy Camera/GlobalTransform/Window 的查询。

**允许的调用点**：
- Widget 系统函数（坐标转换）
- Overlay 系统函数（伤害数字定位）
- Screen 系统函数（镜头状态显示）

**禁止的调用点**：
- Projection 层（Camera 状态是帧相关的，导致 Projection 失去纯函数性质）
- Primitives 层（原语不应感知 Camera）

**访问模式**：

```rust
// ✅ 允许：Widget 系统使用 CameraQuery
fn position_unit_tooltip(
    tooltip_query: Query<&TooltipVm>,
    camera_q: CameraQuery,  // SystemParam 包装
    camera: Query<(&Camera, &GlobalTransform)>,
    window: Query<&Window>,
    mut style_query: Query<&mut Style, With<TooltipWidget>>,
) {
    let Ok((cam, cam_transform)) = camera.single() else { return };
    let Ok(window) = window.single() else { return };

    for tooltip_vm in &tooltip_query {
        if let Some(world_pos) = tooltip_vm.world_position {
            // 使用 CameraQuery 的纯函数进行转换
            if let Some(screen_pos) = CameraQuery::world_to_screen(
                world_pos, cam, cam_transform, &window
            ) {
                // 在屏幕位置定位 Tooltip
                for mut style in &mut style_query {
                    style.left = Val::Px(screen_pos.x + TOOLTIP_OFFSET_X);
                    style.top = Val::Px(screen_pos.y + TOOLTIP_OFFSET_Y);
                }
            }
        }
    }
}
```

### 2.2 UI → Camera: 控制 Camera

UI 通过 `commands.trigger(CameraRequest::...)` 控制 Camera。Camera 是 Infra 层组件，因此**不经过 UiCommand → GameCommand 链条**。

**原则**：
- UI 直接 trigger CameraRequest（Camera 是 Infra，不是 Domain）
- CameraRequest 不经过 UiCommand 转换器
- CameraRequest 不进入 Replay（表现层操作，不影响业务确定性）
- 与 Domain 操作并行的场景（如单位选中 + 镜头聚焦），UI Screen 系统同时发出 Domain Command 和 CameraRequest

**允许的调用点**：
- Screen 系统函数（聚焦/跟随等业务驱动的镜头操作）
- Overlay 系统函数（不适用，Overlay 不控制镜头）
- Widget → UiAction → Screen 处理链（最终由 Screen 系统触发）

**禁止的调用点**：
- Widget 系统函数直接触发 CameraRequest（Widget 只发射 UiAction，不由 Widget 决定镜头行为）
- Projection 层（Projection 不应产生副作用）

```rust
// ✅ 允许：Screen 系统协调 Domain Command 和 CameraRequest
fn on_unit_selected(
    trigger: Trigger<UiActionSelectCharacter>,
    state: Res<BattleScreenState>,
    commands: Commands,
) {
    let unit_id = trigger.unit_id;

    // 1. Domain 操作
    commands.trigger(UiCommand::SelectTarget(unit_id));

    // 2. Camera 操作（并行，不依赖 Domain 响应）
    commands.trigger(CameraRequest::Follow {
        target: CameraTarget::UnitId(unit_id.0),
    });
}
```

**注意**：CameraRequest 是 Event，使用 `commands.trigger()` 发送，与 Bevy 的 Observer 模式一致（符合 ADR-054）。

### 2.3 UI → Camera: 输入阻塞

当 UI 面板/Modal/Popup 打开时，Camera 的 FreeMove 输入（WASD/边缘滚动）应被阻止。

**实现方式**：共享 Resource `CameraInputBlock`

```rust
// src/ui/application/camera_block.rs
// UI 层维护的 Resource，Camera 层读取
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct CameraInputBlock {
    /// true = 阻止 Camera 自由移动输入
    pub blocked: bool,
    /// 堆叠计数器：多个阻塞源同时存在时，全部解除后才解除阻塞
    block_count: u32,
}

impl CameraInputBlock {
    pub fn block(&mut self) {
        self.block_count += 1;
        self.blocked = true;
    }

    pub fn unblock(&mut self) {
        self.block_count = self.block_count.saturating_sub(1);
        if self.block_count == 0 {
            self.blocked = false;
        }
    }
}
```

**Camera 侧的消费**（在 `input_handler.rs` 中）：

```rust
// src/infra/camera/systems/input_handler.rs
pub fn handle_camera_input(
    input_block: Option<Res<ui::application::camera_block::CameraInputBlock>>,
    // ... 其他参数
) {
    // 检查输入阻塞标志
    if let Some(block) = input_block {
        if block.blocked {
            return; // 跳过所有 Camera 自由移动输入
        }
    }
    // 正常处理 WASD/边缘滚动
}
```

**注意**：CameraPlugin 通过 `Option<Res<>>` 读取 CameraInputBlock —— 如果 UI 层没有注册此 Resource，Camera 仍能正常工作（退化为不阻塞）。这保证 Camera 不硬依赖 UI 类型。

### 2.4 Camera 状态在 UI 的反映

Camera 状态（缩放级别、镜头模式）在 UI 中的显示通过**一个专用的 CameraStateVm** 实现。

```rust
// src/ui/view_models/camera_state.rs
#[derive(Resource, Clone, Reflect, Default)]
#[reflect(Resource)]
pub struct CameraStateVm {
    /// 当前缩放级别 (1.0 = 默认)
    pub zoom_level: f32,
    /// 镜头状态描述
    pub state_key: &'static str,  // "idle", "free_move", "follow", "focus"
    /// 是否正在震屏
    pub is_shaking: bool,
}
```

**更新机制**：CameraStateVm 由 CameraPlugin 直接写入（**注意**：这是唯一一个 Camera → UI 的数据通道，写入通过一个注册在 CameraPlugin 中的 system 完成）：

```rust
// src/infra/camera/systems/ui_state.rs
// Camera 系统直接写入 UI 的 ViewModel
// 这是 ADR-055 §5.1 "唯一例外" 模式的扩展：
//   Camera 不依赖 UI 类型，但写入一个 UI 层预定义类型的 Resource
pub fn update_camera_state_vm(
    camera_query: Query<(&CameraPose, &CameraState)>,
    mut camera_state_vm: ResMut<crate::ui::view_models::camera_state::CameraStateVm>,
) {
    let Ok((pose, state)) = camera_query.single() else { return };
    camera_state_vm.zoom_level = pose.zoom;
    camera_state_vm.state_key = state.as_ref();
    camera_state_vm.is_shaking = false; // 当前 Future
}
```

**架构权衡**：此模式是 [Camera → UI] 方向唯一的直接写入通道。与标准的 Domain → Projection → ViewModel 不同，CameraStateVm 由 Camera 系统直接写入，因为 Camera 状态的读取是纯技术性的（不涉及业务规则），不需要 Projection 中间层。

---

## 3. 七种交互场景详细设计

### 场景 1: Tooltip/Popup 定位

**需求**：Widget 需要知道目标单位的世界坐标在屏幕上的位置，以正确定位 Tooltip。

**数据流**：

```
Widget 触发 Tooltip
  → Widget 持有 UnitId
  → TooltipVm.world_position = Some(unit_world_pos)  // 来自投影的数据
  → TooltipService::show(tooltip_vm)
  → TooltipOverlay System
  → CameraQuery::world_to_screen(world_pos, camera, transform, window)
  → 计算屏幕坐标 → 定位 Tooltip Node
```

**关键问题**：Widget 如何获取 `unit_world_pos`？

**解答**：`unit_world_pos` 来自投影层。CharacterPanelVm 或 UnitVm 中需要包含单位的**世界坐标**字段：

```rust
// 在 CharacterPanelVm 中新增：
pub struct CharacterPanelVm {
    // ...现有字段
    pub world_position: Vec2,  // 新增：单位的世界坐标
}

// 在 BattleProjection 中投影：
fn project_unit_position(
    trigger: Trigger<TurnStarted>,
    unit_positions: Query<&GridPosition>,
    transform_query: Query<(&Transform, &GlobalTransform)>,
    mut store: ResMut<UiStore>,
) {
    // 从 Domain 读取单位位置，投影到 ViewModel
    // world_position 在 Projection 阶段计算（使用已知的 grid→world 映射）
    // 不需要 CameraQuery（Camera 状态无关）
    store.character_panel.world_position = computed_world_pos;
    store.character_panel.mark_dirty();
}
```

**注意**：单位的世界坐标是**地图逻辑坐标**，与 Camera 状态无关。它可以通过 GridPosition + Map 配置直接计算，不需要 CameraQuery。CameraQuery 只在 **world_to_screen** 转换时需要，这个转换发生在 TooltipOverlay 系统（渲染时），不是 Projection。

**TooltipVm 扩展**：

```rust
// src/ui/widgets/tooltip/components.rs
pub struct TooltipVm {
    pub content_key: UiTextKey,
    pub params: HashMap<String, String>,
    pub position_hint: TooltipPosition,

    /// 世界坐标（可选）。如果有值，Overlay 系统会自动转换为屏幕坐标定位。
    /// 为什么放在 TooltipVm 而不是让 Overlay 自己去查？
    ///   因为 TooltipService 的调用者（Widget/Screen）才知道 target 是谁，
    ///   Overlay 是通用层，不应该知道如何获取特定目标的世界坐标。
    pub world_position: Option<Vec2>,

    pub max_width: Val,
}
```

**完整交互流程**：

```
1. 用户悬停/聚焦单位图标
2. Widget 获取单位 ID (CharacterId)
3. Widget 读取 ViewModel 中该单位的 world_position
4. Widget 调用 TooltipService::show(TooltipVm {
       content_key: SKILL_DESC_KEY,
       world_position: Some(char_vm.world_position),
       position_hint: TooltipPosition::Bottom,
       ..default()
   })
5. TooltipOverlay System 每帧处理：
   a. 读取当前 CameraState（通过 CameraQuery）
   b. 如果 tooltip_vm.world_position 有值：
      → CameraQuery::world_to_screen(...) → screen_pos
      → 根据 screen_pos + position_hint 计算最终位置
      → 更新 Tooltip Node 的 Style.left/top
   c. 如果超出屏幕边界 → 翻转方向
6. Tooltip 在 TooltipLayer 中渲染，不受 Screen 切换影响
```

---

### 场景 2: UI 是否拦截 Camera 输入

**需求**：当 UI 面板打开时，Camera 的 FreeMove 输入（WASD/方向键/边缘滚动）应被阻止。

**数据流**：

```
UI 面板打开 → Event/Popup/Modal
  → UI System 调用 CameraInputBlock::block()
  → CameraInputBlock.blocked = true

Camera input_handler（每帧 PreUpdate）
  → 读取 CameraInputBlock
  → blocked == true → 跳过所有自由移动输入

UI 面板关闭 → CameraInputBlock::unblock()
  → block_count == 0 → blocked = false
  → Camera input_handler 恢复正常
```

**触发规则**：

| UI 元素 | 是否阻塞 | 理由 |
|---------|---------|------|
| BattleScreen（战斗 HUD） | 否 | HUD 不影响地图操作 |
| Modal（确认对话框） | 是 | Modal 阻塞下层交互 |
| Fullscreen Popup（设置/商店） | 是 | 全屏覆盖 |
| Tooltip | 否 | Tooltip 很小，不阻塞 |
| Notification Banner | 否 | 通知不阻塞操作 |
| Context Menu | 是 | 菜单需要专注选择 |
| Skill Panel（展开状态） | 否 | 战斗操作需要同时看地图 |
| Settings Screen | 是 | 全屏设置 |
| Inventory Screen | 是 | 全屏背包 |
| DamageTextOverlay | 否 | 纯表现，无交互 |

**特别注意**：CameraInputBlock 使用**计数堆叠**（而非简单的 bool flag），确保多个阻塞源同时打开时，只有全部关闭后才解除阻塞。

**代码位置**：

| 职责 | 文件 |
|------|------|
| Resource 定义 | `src/ui/application/camera_block.rs` |
| 设置阻塞（UI 侧） | `src/ui/application/camera_block.rs` (as part of UiApplicationPlugin) |
| 读取阻塞（Camera 侧） | `src/infra/camera/systems/input_handler.rs` |
| Modal/Popup 自动阻塞 | `src/ui/overlay/services.rs` (ModalService 内部调用 block/unblock) |

---

### 场景 3: Minimap 点击导航

**需求**：点击 Minimap 上的位置 → Camera 移动到对应的世界坐标。

**数据流**：

```
用户点击 Minimap
  → MinimapWidget 获取点击在 minimap 上的局部坐标 (minimap_x, minimap_y)
  → MinimapWidget 知道 minimap 的像素尺寸 (minimap_w, minimap_h)
  → MinimapWidget 知道世界地图边界 (world_min, world_max)  — 来自 ViewModel
  → 计算世界坐标:
      fraction_x = minimap_x / minimap_w
      fraction_y = minimap_y / minimap_h
      world_x = lerp(world_min.x, world_max.x, fraction_x)
      world_y = lerp(world_min.y, world_max.y, fraction_y)
  → Widget 发射 UiAction::MinimapClick(world_x, world_y)
  → Screen System 处理 UiAction
  → commands.trigger(CameraRequest::MoveTo {
      target: CameraTarget::WorldPos(Vec2::new(world_x, world_y)),
      duration: 0.3,
  })
```

**关键说明**：

- Minimap 不需要 `CameraQuery::screen_to_world` 转换。Minimap 上的点击→世界坐标转换通过最小图比例计算，与 Camera 当前状态无关。
- Minimap 需要从 ViewModel 获取世界地图边界 (`world_min`, `world_max`)，这由 Projection 从 Map/Terrain Domain 投影而来。
- CameraRequest 是直接 trigger 的，不经过 UiCommand 转换器（Camera 是 Infra）。

**Minimap ViewModel**：

```rust
// UiStore.minimap 在后续迭代中新增
pub struct MinimapVm {
    pub world_bounds: (Vec2, Vec2),    // (min, max) 世界边界
    pub camera_visible_rect: Rect,     // 当前相机可视区域（用于 Minimap 上显示视口框）
    pub units: Vec<MinimapUnitVm>,     // 单位标记
}
```

**camera_visible_rect 的更新**：这需要 CameraQuery。在 Overlay 或 Widget 系统中每帧读取：

```rust
fn update_minimap_camera_rect(
    camera_query: CameraQuery,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut minimap_vm: ResMut<MinimapVm>,
) {
    let Ok((cam, transform)) = camera.single() else { return };
    minimap_vm.camera_visible_rect = CameraQuery::visible_rect(cam, transform);
}
```

**为什么 camera_visible_rect 不放在 CameraStateVm 里？** 因为它是 Minimap 特有的数据需求，没必要在全局 CameraStateVm 中增加字段。Minimap 自身每帧同步即可。

---

### 场景 4: 单位选中聚焦

**需求**：玩家选中一个单位 → Camera 聚焦到该单位位置。

**数据流**：

```
玩家点击单位图标/地图上的单位
  → Widget 发射 UiAction::SelectCharacter(character_id)
  → Screen System (如 BattleScreen) 处理:
      a. UiCommand::SelectTarget(character_id) → Domain 操作
      b. CameraRequest::Follow {
             target: CameraTarget::UnitId(character_id.0),
         }
  → Camera 状态机: Idle/FreeMove → Follow(CameraTarget::UnitId(...))
  → Camera 每帧获取 UnitId 对应的世界坐标 → 更新 TargetPose
```

**关键问题**：Camera 如何将 `UnitId` 解析为世界坐标？

**解答**：ADR-064 指定 CameraTarget::UnitId 作为目标类型。Camera 状态机内部需要将 UnitId 解析为世界坐标。这需要一个**坐标解析函数**，由业务侧注册到 Camera 系统中。

```rust
// src/infra/camera/foundation/target.rs
pub enum CameraTarget {
    WorldPos(Vec2),
    TilePos(i32, i32),
    UnitId(u64),
}

// src/infra/camera/systems/state_machine.rs
// UnitId → WorldPos 的解析器（注册为 Resource）
pub type UnitPositionResolver = fn(u64) -> Option<Vec2>;

// CameraPlugin 初始化时由业务侧设置
pub fn setup_unit_position_resolver(
    mut resolver: ResMut<UnitPositionResolver>,
    // 可选：读取 Domain 数据来构建解析器
) {
    // 由业务侧（如 Combat/Tactical Plugin）在初始化时注册
    // Camera 本身不感知 Domain 类型
}
```

**交互模式对比**：

| 方式 | 适用场景 | 是否经过 UiCommand |
|------|---------|-------------------|
| UI → CameraRequest::Follow | 单位选中、自动跟随 | 否（直接 trigger） |
| UI → UiCommand → GameCommand | 施放技能、移动单位 | 是 |
| UI → 同时触发两者（协调模式） | 选中单位 + 聚焦镜头 | 部分协调 |

**协调模式**：单位选中是 Domain 操作（SelectTarget）和 Camera 操作（Follow）同时发生。在 Screen 系统中协调：

```rust
fn on_unit_selected(
    trigger: Trigger<UiActionSelectCharacter>,
    mut commands: Commands,
    screen_state: Res<BattleScreenState>,
) {
    let unit_id = trigger.character_id;

    // 1. Domain 操作
    commands.trigger(UiCommand::SelectTarget(unit_id.0 as u32));

    // 2. Camera 操作（不等待 Domain 响应，立即执行）
    commands.trigger(CameraRequest::Follow {
        target: CameraTarget::UnitId(unit_id.0),
    });
}
```

**两个操作是并发的**——Camera 不需要等待 Domain 处理完成。因为选中操作是瞬时的（不需要服务器确认），Camera 可以立即开始移动。

---

### 场景 5: 战斗日志点击跳转

**需求**：点击战斗日志中的条目 → Camera 聚焦到该条目关联的单位位置。

**数据流**：

```
用户点击战斗日志条目
  → 日志条目标记了关联单位 ID (attacker_id / target_id)
  → Widget 发射 UiAction::FocusLogEntry(unit_id)
  → Screen System (BattleScreen) 处理:
      → commands.trigger(CameraRequest::Focus {
          target: CameraTarget::UnitId(unit_id),
          duration: 0.5,
      })
  → Camera 状态机: any → Focus { target: UnitId, duration: 0.5, elapsed: 0 }
  → Focus 动画完成后 → Idle
```

**特点**：
- 使用 `CameraRequest::Focus`（带动画过渡）而非 `Follow`（持续跟随）
- Focus 完成后自动回到 Idle，不持续跟踪单位
- 与场景 4 的核心区别：Focus 是一次性动画，Follow 是持续行为

**数据源**：战斗日志条目需要携带关联单位 ID。日志数据由 BattleProjection 投影到 ViewModel：

```rust
pub struct BattleLogEntryVm {
    pub timestamp: u32,
    pub message_key: UiTextKey,
    pub params: HashMap<String, String>,
    pub associated_unit_id: Option<u64>,  // 关联单位 ID，可为 None
}
```

**交互与场景 4 的差异**：

| 特征 | 单位选中 (场景 4) | 日志点击 (场景 5) |
|------|-------------------|------------------|
| Camera 行为 | Follow（持续跟随） | Focus（一次性动画） |
| 是否同时触发 Domain 操作 | 是 (SelectTarget) | 否（纯表现） |
| 动画时长 | 插值移动（无固定时长） | 0.5s 动画 |
| 结束后行为 | 继续跟随 | 回到 Idle |

---

### 场景 6: Camera 状态在 UI 的反映

**需求**：UI 中某些元素需要显示 Camera 的当前状态（如缩放级别、镜头模式名称）。

**数据流**（详见 §2.4）：

```
Camera 系统（Phase 8 PostUpdate）
  → update_camera_state_vm system
  → 写入 CameraStateVm.zoom_level, .state_key

UI 系统（Phase 11 Update）
  → ZoomIndicator Widget 读取 CameraStateVm
  → 显示放大镜图标 + 缩放百分比

UI 系统（Phase 11 Update）
  → Widget 根据 CameraStateVm.state_key 决定是否显示提示
  → 例如 Focus 状态下显示 "聚焦中..." 提示
```

**ZoomIndicator Widget**：
- 属于 UI Overlay，放在 TooltipLayer 或独立的小状态指示层
- 显示当前缩放百分比（如 "100%"、"150%"）
- 可选：显示镜头模式图标（自由移动/跟随/聚焦）
- 仅在战斗场景中有意义

**约束**：
- CameraStateVm 不持久化、不进入 Replay、不进入 Save
- CameraStateVm 的更新不经过 Projection（由 Camera 系统直接写入）
- CameraStateVm 只在战斗场景（GameState::Combat）中注册和使用

---

### 场景 7: UI Overlay 与 Camera 的 z-order

**需求**：确保 UI 元素始终在 Camera 渲染内容之上。

**现状分析**：Bevy 默认行为中：
- Camera 渲染到 RenderTarget（默认是整个窗口）
- UI Node 在单独的 UI Pass 中渲染，默认覆盖 Camera 渲染层
- 只要 UI Node 的 Style.position_type 正确设置且无特殊 RenderTarget，UI 始终在 Camera 之上

**决策**：**无需额外处理**。Bevy 的默认渲染管线已经保证 UI 在 Camera 输出之上。

**确认事项**：
- UI Root 节点使用默认的 `TargetCamera` 组件（指向 Main Camera） — 确保 UI 正确覆盖
- 如果未来引入多 Camera（如 Minimap 独立 Camera），需要为 Minimap Camera 配置单独的 UI 层
- DamageTextOverlay 作为 ScreenLayer 的子节点，仍然在 Camera 渲染之上（因为它是 UI Node）

---

## 4. 新增/修改的文件

### 4.1 新增文件

| 文件 | 职责 | 所有者 |
|------|------|--------|
| `src/ui/application/camera_block.rs` | CameraInputBlock Resource 定义 + block/unblock 方法 | UI 层 |
| `src/ui/view_models/camera_state.rs` | CameraStateVm Resource 定义 | UI 层 |
| `docs/06-ui/04-data-flow/camera-ui-interaction.md` | 本文档 | Presentation Architect |

### 4.2 修改文件

| 文件 | 变更 | 原因 |
|------|------|------|
| `src/ui/application/mod.rs` | 新增 `pub mod camera_block;` | 注册 CameraInputBlock 模块 |
| `src/ui/view_models/mod.rs` | 新增 `pub mod camera_state;` | 注册 CameraStateVm 模块 |
| `src/ui/plugin.rs` | 注册 `CameraStateVm` Resource + `CameraInputBlock` Resource | UI Plugin 初始化 |
| `src/infra/camera/systems/input_handler.rs` | 添加 `Option<Res<CameraInputBlock>>` 读取 | Camera 读取 UI 阻塞标志 |
| `src/infra/camera/systems/state_machine.rs` | 注册 `UnitPositionResolver` Resource | Camera 解析 UnitId → WorldPos |
| `src/infra/camera/plugin.rs` | 注册 `update_camera_state_vm` system | 将 Camera 状态写入 UI ViewModel |
| `src/ui/projections/battle.rs` | 投影单位世界坐标到 ViewModel | 为 Tooltip 定位提供 world_position |
| `src/ui/overlay/tooltip.rs` | 添加 CameraQuery 坐标转换逻辑 | Tooltip 世界空间定位 |
| `src/ui/overlay/damage_text.rs` | 添加 CameraQuery 坐标转换逻辑 | 伤害数字世界→屏幕转换 |
| `src/ui/view_models/character_panel.rs` | 新增 `world_position: Vec2` 字段 | 为 Tooltip 定位提供数据源 |
| `src/ui/screens/battle/systems.rs` | 添加 CameraRequest trigger（单位选中、日志点击） | 实现场景 4/5 |

### 4.3 不修改的文件

| 文件 | 原因 |
|------|------|
| `src/ui/widgets/*` | Widget 不直接触发 CameraRequest，也不直接查 CameraQuery（通过 TooltipService 或 Screen System 间接） |
| `src/core/domains/*` | Domain 不感知 Camera — Camera 是纯表现层 |
| `src/ui/ui_binding.rs` | 不新增 Camera 相关 UiBinding 变体（CameraStateVm 变化不频繁，不需要 Dirty<>) |
| `src/ui/projections/*`（除 battle.rs 新增 world_position 字段外） | Projection 不依赖 CameraQuery |

---

## 5. 架构规则补充

### 5.1 新增规则

| # | 规则 | 说明 |
|---|------|------|
| CAM-UI-01 | UI 直接 trigger CameraRequest，不经过 UiCommand 转换器 | Camera 是 Infra，不是 Domain |
| CAM-UI-02 | Projection 不读取 CameraQuery | Camera 状态帧相关，破坏纯函数性质 |
| CAM-UI-03 | CameraStateVm 由 Camera 系统直接写入，不经过 Projection | 唯一一个 Camera → UI 直接写入通道 |
| CAM-UI-04 | Widget 不直接触发 CameraRequest | Widget 只发射 UiAction，由 Screen 系统协调 |
| CAM-UI-05 | Widget 不直接查询 CameraQuery | Widget 通过 TooltipVm.world_position 间接使用 |
| CAM-UI-06 | CameraInputBlock 使用计数堆叠 | 多个阻塞源同时存在时，全部解除后才解除阻塞 |
| CAM-UI-07 | CameraRequest 不进入 Replay 录制 | 表现层操作不影响业务确定性 |
| CAM-UI-08 | Overlay（Tooltip/DamageText）负责 world→screen 转换 | 转换发生在渲染时，使用 CameraQuery |

### 5.2 Projection 纯函数的例外说明

Projection 层根据 ADR-055 必须是纯函数。Camera 相关的世界中立数据（如 GridPosition → world_position）可以在 Projection 中计算，因为这是确定性的 Map/Gird 映射。

**Projection 中合法计算的 Camera 相关数据**：
- `GridPosition(x, y)` → `world_position(x * tile_size, y * tile_size)` — 纯数学映射，无 Camera 依赖

**Projection 中禁止计算的 Camera 相关数据**：
- `world_position → screen_position` — 依赖 Camera 的 Transform 和 Window，帧相关，不是纯函数

### 5.3 验证规则

| # | 规则 | 校验逻辑 |
|---|------|----------|
| CC-VAL-01 | UI 代码不直接 import Camera Transform/Projection | CI 审查：Widget/Screen 中禁止 Query<&mut Transform, With\<Camera\>> |
| CC-VAL-02 | Camera 代码不 import UI 类型 | CI 审查：`src/infra/camera/` 中禁止 `use crate::ui::*` |
| CC-VAL-03 | CameraInputBlock 被 UI 设置，被 Camera 读取 | 代码审查：write 端在 UI，read 端在 Camera |
| CC-VAL-04 | CameraStateVm 被 Camera 写入，被 UI 读取 | 代码审查：write 端在 Camera，read 端在 UI |
| CC-VAL-05 | CameraRequest 不在 Widget 中直接 trigger | 代码审查：Widget 系统函数中禁止 `commands.trigger(CameraRequest::*)` |
| CC-VAL-06 | Camera 不依赖 Domain 类型（UnitId 是 u64 别名） | CI 审查：`src/infra/camera/` 中禁止 `use crate::core::*` |

---

## 6. 通信全景视图

```
Camera (L2 Infra)                               UI (L3 Presentation)
─────────────────                               ────────────────────

    CameraState (Component) ─────────→ update_camera_state_vm
                                             │
                                             ▼
                                        CameraStateVm (Resource)
                                             │
                                    ┌────────┼────────┐
                                    ▼        ▼        ▼
                              ZoomIndicator  其他 Widget  ...

    CameraQuery::world_to_screen() ←──────── TooltipOverlay
    CameraQuery::world_to_screen() ←──────── DamageTextOverlay
    CameraQuery::visible_rect()    ←──────── MinimapWidget

    CameraRequest::Follow   ←──────── on_unit_selected Screen System
    CameraRequest::Focus    ←──────── on_log_entry_click Screen System
    CameraRequest::MoveTo   ←──────── on_minimap_click Screen System

    CameraInputBlock ←───────────────── ModalService.block()
                                        PopupService.block()
                                        FullscreenScreen.block()
```

---

## 7. 决策记录

| 决策 | 选择 | 替代方案 | 理由 |
|------|------|---------|------|
| world→screen 转换位置 | Overlay/Widget System（渲染时） | Projection 层 | Camera 状态帧相关，Projection 必须纯函数 |
| UI → Camera 通信方式 | 直接 trigger CameraRequest | 经过 UiCommand → GameCommand | Camera 是 Infra，不是 Domain |
| Camera 状态显示 | CameraStateVm（Camera 直接写入） | Projection 投影 | Camera 没有 Domain Event，不需要中间层 |
| 输入阻塞机制 | CameraInputBlock Resource | Camera 查询 UI 状态（被禁止） | Camera 不依赖 UI，通过共享 Resource 解耦 |
| 计数堆叠 | 使用 u32 block_count | bool flag | 防止多个阻塞源嵌套导致状态错误 |
| Minimap 坐标转换 | 比例计算（map_size → world） | CameraQuery::screen_to_world | Minimap 有自己的坐标系，与相机当前状态无关 |
| UnitId → WorldPos 解析 | UnitPositionResolver（注册的 fn） | Camera 直接 Query Domain 组件 | ADR-064 禁止 Camera 依赖 Domain |
| CameraRequest 是否进入 Replay | 否 | 是 | 表现层操作不影响业务确定性 |

---

*本文档由 @presentation-architect 维护。Camera-UI 交互规则的变更需要 Presentation Architect 审查。CR 修改也必须通知 Camera 模块的架构师。*
