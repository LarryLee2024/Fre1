---
id: 01-architecture.ADR-064
title: "ADR-064: Camera 系统架构"
status: Proposed
owner: architect
created: 2026-06-21
tags:
  - architecture
  - camera
  - infra
  - presentation
  - cross-cutting
  - state-machine
---

# ADR-064: Camera 系统架构

## 状态

**Proposed**

## 背景

当前 Camera 是一个 7 行的 `spawn_camera` 函数，位于 `src/app/scenes/test_battle/render.rs`，固定位置无任何交互能力。项目需要独立的 Camera 系统以支持：

1. WASD/方向键/边缘滚动的自由镜头移动
2. 镜头跟随（单位移动、技能演出）
3. 镜头聚焦事件（技能释放、对话触发）
4. 缩放控制
5. 震屏效果
6. 边界约束（不出地图/场景边界）
7. 外部通过 Event 而非直接修改 Transform

### 当前约束

- **Replay First**：所有核心逻辑必须可确定性重放
- **Logic/Presentation 分离**：表现层不耦合业务逻辑
- **四级通信**：Hook > Trigger > Observer > Message
- **复杂度治理**：只解决当前复杂度，不为未明确需求提前设计（宪法 SS1.2）
- **宪法 P0**：Effect/Modifier 管线不可绕过，Feature First，Definition/Instance 分离

### 已有输入对接

`src/infra/input/action.rs` 已预定义了 Camera 相关 `InputAction`：

```
CameraUp, CameraDown, CameraLeft, CameraRight, CameraZoomIn, CameraZoomOut
```

输入层（ADR-043）已准备好，Camera 系统需要消费这些 InputAction。

### Cue 领域对接

`cue_domain.md` §5.2 已指定 `Shake → Infra.presentation.camera/shake`，Camera 系统需要消费 Cue 的震屏请求。

### 历史经验

`docs/99-history/ai_ignore_this_dir/17camera.md` 提供了大型 SRPG 项目的 Camera 设计教训，关键结论：
- Camera 是独立系统，不是 Map/UI 的附属
- Event 驱动：所有 Gameplay 通过 CameraRequest 修改镜头，禁止直接改 Transform
- Camera 不应该知道 Combat/Dialogue/Quest 等业务领域
- CameraPose（Target/Current）分离 + 插值过渡
- 优先级栈解决多系统抢控制权
- Replay 兼容（CameraCommand 可记录）
- 输入解耦（Input → CameraCommand → Camera）
- Camera 是 State Machine（Idle/FreeMove/Follow/Focus 等状态）

## 引用的领域规则与架构

- `docs/01-architecture/README.md` §2.1 — DDD 纵向三层（Camera 定位在 L2 Infra/表现层）
- `docs/01-architecture/README.md` §4.2 — 四级通信机制（使用 trigger + Observer）
- `docs/01-architecture/README.md` §6 — Plugin 组合与注册顺序（Phase 8，Input 之后）
- `ADR-043` — 命令层与输入抽象（InputAction 预定义了 CameraUp/Down/Left/Right/ZoomIn/ZoomOut）
- `ADR-050` — 游戏状态机与场景架构（Camera 场景生命周期对接）
- `ADR-055` — UI 表现层架构（Camera 是独立的 Presentation 组件，不属于 UI 层）
- `docs/02-domain/capabilities/cue_domain.md` §5.2 — Cue Shake 路由到 camera/shake
- `docs/02-domain/capabilities/cue_domain.md` §3.4 — Cue 的可选性（Camera shake 必须可禁用）

### 引用说明

- **领域规则文档**：Camera 领域规则文档待 @domain-designer 补充（`docs/02-domain/infrastructure/camera_domain.md`）
- **数据 Schema**：Camera 数据 Schema 待 @data-architect 补充（`docs/04-data/infrastructure/camera_schema.md`）
- **UI 交互规则**：Camera-UI 交互规则待 @presentation-architect 补充（`docs/06-ui/` 相关）
- **内容架构**：Camera 不需要 Def 定义（无配置可加载内容）

## 决策

### 1. Camera 定位：Infra 层独立模块

Camera 位于 `src/infra/camera/`，属于 L2 Infra 层，与 registry/pipeline/replay/save/input 平级。

**理由**：
- Camera 是技术基础设施（画面呈现），不包含业务规则——没有 Formula/Condition/Effect，不能放在 `core/domains/`
- Camera 是全局系统的协调者，不是 UI 子模块——放在 `ui/` 会违反 "Camera 不应该知道业务域" 原则
- Camera 需要自己的模块边界：State Machine、Pose 数据、边界约束、事件接口——分散在 `app/scenes/` 或 `infra/input/` 会重复历史教训
- 架构总纲 `README.md` L153 已明确 Camera 属于 "L2 Infra + 横切 Content → Presentation"
- 与 infra/input/ 的关系：Input 是输入抽象（Layer 1），Camera 是表现层系统（Layer 2），Input 不依赖 Camera，Camera 消费 InputAction

**架构变更**：在 `src/infra/` 下新增 `camera/` 目录，不影响现有模块结构。

### 2. 模块结构

```
src/infra/camera/
├── mod.rs                  # 模块声明 + pub re-export
├── plugin.rs               # CameraPlugin（以下按顺序注册）
├── foundation/             # 纯类型定义（零 ECS 依赖，可独立测试）
│   ├── mod.rs
│   ├── pose.rs             # CameraPose（position, zoom, rotation）
│   ├── target.rs           # CameraTarget（WorldPos/TilePos/UnitId）
│   ├── request.rs          # CameraRequest 枚举（所有镜头请求）
│   ├── state.rs            # CameraState 枚举（Idle/FreeMove/Follow/Focus）
│   └── command.rs          # CameraCommand（可录制的命令子集）
├── systems/                # Bevy Systems
│   ├── mod.rs
│   ├── input_handler.rs    # InputAction → CameraRequest
│   ├── state_machine.rs    # CameraState 状态转换 + 请求仲裁
│   ├── movement.rs         # Pose 插值 + Transform 写入 + 边界钳位
│   ├── shake.rs            # 震屏效果
│   └── bounds.rs           # CameraBounds 聚合 + 应用
├── query.rs                # 公开查询 API（SystemParam + 纯函数）
└── tests/
    ├── mod.rs
    ├── unit/               # foundation/ 层纯函数测试
    └── integration/        # 事件驱动集成测试
```

**约定**：
- `foundation/` 是纯类型定义模块，零 ECS 依赖，可单元测试
- `systems/` 包含所有 Bevy System
- `plugin.rs` 是唯一对外入口（符合 ADR-046 模块可见性策略）
- `query.rs` 提供 CameraQuery SystemParam 和独立函数（供外部只读查询）
- 不创建 `events.rs`（CameraRequest 在 foundation/ 中定义）
- 不创建 `resources.rs` / `components.rs` 等全局技术文件（类型分布在 foundation/ 和 systems/ 中）

### 3. State Machine 设计

```rust
/// CameraState — 镜头状态机
///
/// 所有镜头行为通过状态机仲裁，禁止外部系统直接修改 Transform。
pub enum CameraState {
    /// 空闲：未触发任何主动行为，镜头静止
    Idle,
    /// 自由移动：玩家通过 WASD/方向键/边缘滚动/拖拽控制镜头
    FreeMove,
    /// 跟随：镜头跟随一个目标（单位移动等）
    Follow(CameraTarget),
    /// 聚焦：镜头动画过渡到特定位置/目标（技能释放、对话触发）
    /// 聚焦动画完成后自动回到 Idle
    Focus {
        target: CameraTarget,
        duration: f32,
        elapsed: f32,
    },
}
```

**状态转移图**：

```
                           ┌──────────────┐
                  启动 ──→  │     Idle     │
                           └──────┬───────┘
                    ┌──────────────┼──────────────┐
                    │              │              │
           用户输入开始       FollowRequest    FocusRequest
                    │              │              │
                    ▼              ▼              ▼
            ┌───────────┐  ┌───────────┐  ┌───────────┐
            │ FreeMove  │  │  Follow   │  │  Focus    │
            └─────┬─────┘  └─────┬─────┘  └─────┬─────┘
                    │              │              │
           用户输入停止       UnfollowRequest  动画完成
       (N秒无输入回到Idle)    (或长按取消)      (计时结束)
                    │              │              │
                    └──────┬───────┘              │
                           │                      │
                           └──────────┬───────────┘
                                      │
                                      ▼
                                 ┌────────┐
                                 │  Idle  │
                                 └────────┘
```

**跨状态转移规则**：

| 当前状态 | 收到 FollowRequest | 收到 FocusRequest | 用户输入 | 无条件 |
|---------|-------------------|-------------------|---------|--------|
| Idle | → Follow | → Focus | → FreeMove | 保持 Idle |
| FreeMove | → Follow | → Focus | 继续 FreeMove（刷新超时计时器） | 输入停止 N 秒 → Idle |
| Follow | → Follow（切换目标） | → Focus | → FreeMove（用户覆盖） | Unfollow → Idle |
| Focus | 排队（但当前不做，静默忽略） | 排队（但当前不做，静默忽略） | 禁止（锁定输入） | 动画完成 → Idle |

**初始实现的状态转移限制**：
- Focus 状态下不处理新的 Request（静默忽略——简化实现）
- 用户输入始终可以覆盖 Follow（FreeMove > Follow）
- FreeMove 输入停止 2 秒后自动回到 Idle

### 4. 事件接口契约

所有外部系统通过 `trigger(CameraRequest)` 与 Camera 通信。禁止直接修改 Camera Entity 的 Transform/Projection。

```rust
/// CameraRequest — 所有镜头请求的统一枚举。
///
/// 外部系统通过 commands.trigger(CameraRequest::...) 发送。
/// CameraPlugin 内的 Observer 消费此事件并更新状态机。
/// 禁止外部系统直接修改 Camera Entity。
#[derive(Event, Debug, Clone)]
pub enum CameraRequest {
    /// 移动到指定世界位置（带插值时长）
    MoveTo {
        target: CameraTarget,
        duration: f32,
    },
    /// 跟随一个目标（单位/位置）
    Follow {
        target: CameraTarget,
    },
    /// 取消跟随
    Unfollow,
    /// 设置缩放级别
    SetZoom {
        zoom: f32,
        duration: f32,
    },
    /// 震屏
    Shake {
        intensity: f32,
        duration: f32,
    },
    /// 重置到默认位置
    Reset {
        duration: f32,
    },
    /// 锁定/解锁摄像头输入（FreeMove 状态开关）
    LockInput,
    UnlockInput,
}

/// CameraTarget — 镜头目标，使用领域 ID 而非 Entity。
///
/// 使用 UnitId/TilePos 而非 Entity 的原因：
/// - Entity 是 ECS 运行时概念，可能在目标聚焦期间被销毁
/// - UnitId 是领域身份，不受 Entity 回收影响
/// - 符合 DDD 定义与实例分离原则
#[derive(Debug, Clone)]
pub enum CameraTarget {
    /// 世界坐标位置
    WorldPos(Vec2),
    /// 网格坐标位置
    TilePos(i32, i32),  // GridPos 解耦——不用 domain 类型
    /// 单位 ID
    UnitId(u64),         // UnitId 解耦——不用 domain 类型
}
```

**外部系统使用方式**：

```rust
// 单位聚焦（由 Combat/Dialogue 系统触发）
commands.trigger(CameraRequest::Focus {
    target: CameraTarget::UnitId(unit_id),
    duration: 0.5,
});

// 震屏（由 Cue 系统触发）
commands.trigger(CameraRequest::Shake {
    intensity: 5.0,
    duration: 0.3,
});

// 跟随单位（由 Tactical 系统在单位选中时触发）
commands.trigger(CameraRequest::Follow {
    target: CameraTarget::UnitId(unit_id),
});
```

**与 Input 的对接**：

```
InputMap → InputAction (CameraUp/CameraDown/...)
    ↓
input_handler System (消费 InputAction)
    ↓
CameraRequest::MoveTo / CameraRequest::SetZoom (内部 trigger)
    ↓
state_machine Observer/Sytem (仲裁 Request)
    ↓
movement System (插值 → 写入 Transform)
```

Camera 输入处理在 `input_handler.rs` 中完成，与通用 Input 系统解耦。Input 层（`infra/input/`）只产生产出 InputAction，不感知 Camera 具体实现。

### 5. 数据流：CameraPose 分离 + 插值管线

```rust
/// CameraPose — 镜头姿态值对象。
///
/// position: 世界坐标位置（2D）
/// zoom: 缩放倍数（1.0 = 默认）
/// rotation: 旋转角度（弧度，预留，初始始终为 0）
#[derive(Debug, Clone, PartialEq)]
pub struct CameraPose {
    pub position: Vec2,
    pub zoom: f32,
    pub rotation: f32,
}
```

**数据流管线**：

```
TargetPose (目标姿态，由状态机设置)
    │
    ▼
InterpolationSystem (每帧 lerp，独立于帧率)
    │  CameraPose::lerp(&self, target: &CameraPose, t: f32) → CameraPose
    │  使用共享时间资源计算 delta
    ▼
CurrentPose (当前姿态，插值结果)
    │
    ├──→ ClampSystem (应用 CameraBounds)
    │      position = position.clamp(bounds.min, bounds.max)
    │
    ├──→ ShakeSystem (叠加震屏偏移)
    │      position += shake_offset (由 Timer 驱动的随机偏移衰减)
    │
    ▼
TransformWriteSystem (帧末写入 Bevy 组件)
    │  transform.translation = current_pose.position.extend(z)
    │  projection.scale = current_pose.zoom
    │  (rotation 预留，初始不写入)
    ▼
Bevy Transform / Projection (实际渲染)
```

**关键设计点**：
- `CameraPose` 是纯值对象，不包含 ECS 引用
- `TargetPose` 和 `CurrentPose` 是分离的 ECS Component，挂在 Camera Entity 上
- 插值是确定性数学运算（不依赖随机数），Replay 兼容
- Shake 是临时叠加偏移，不修改 Target/Current Pose 本身
- Rotation 字段预留，初始不写入 Transform（未来扩展点）

### 6. 边界约束设计

Camera 不感知地图/场景的具体形状，通过 `CameraBounds` 组件解耦：

```rust
/// CameraBounds — 镜头边界约束组件。
///
/// 挂在 Camera Entity 上，由场景生命周期系统或 Map Domain 在 OnEnter 时设置。
/// Camera 不知道边界来自地图/竞技场/过场——只读取 min/max 执行钳位。
#[derive(Component, Clone, Debug)]
pub struct CameraBounds {
    pub min: Vec2,
    pub max: Vec2,
}
```

**数据流**：

```
Map Domain (场景 OnEnter)
    │  根据 GridMap 尺寸计算边界
    │
    ├──→ CameraCommands::SetBounds(...) (直接 Query Camera Entity insert CameraBounds)
    │     或
    └──→ commands.trigger(CameraRequest::SetBounds { min, max }) (未来扩展)
    │
    ▼
CameraPlugin
    │  读取 CameraBounds Component
    │
    └──→ ClampSystem: pose.position = pose.position.clamp(bounds.min, bounds.max)
```

**禁止 Camera 耦合 Map Domain**：
- Camera 不 import `tactical`/`terrain`/`map` 的任何类型
- CameraBounds 使用纯 Vec2，不包含 GridPos/TileMap 引用
- 边界数据由业务层（场景系统或 Map Domain）在恰当生命周期设置

**边界不存在时的行为**：当 Camera Entity 没有 `CameraBounds` Component 时，不执行钳位（镜头可自由移动到任何位置）。

### 7. Replay 兼容

Camera 的 Replay 支持分为两层：

**Layer 1 — CameraCommand 录制**：

```rust
/// CameraCommand — 可录制的镜头命令子集。
///
/// CameraCommand 记录了关键帧镜头操作，独立于 GameCommand 的 replay 流。
/// 回放时 CameraSystem 按照时间戳重放命令，恢复镜头状态。
/// 与 GameCommand 分离的理由：镜头操作是表现层行为，不影响业务逻辑确定性。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CameraCommand {
    MoveTo(CameraTarget),
    Follow(CameraTarget),
    Unfollow,
    SetZoom(f32),
    Shake(f32, f32),
    Reset,
    // SetBounds(Vec2, Vec2) — 边界由场景系统管理，不录制
}
```

**Layer 2 — Replay 桥接**（Phase 3 实现，当前只预留录制点）：

```
CameraPlugin 在 input_handler system 中检测到 CameraRequest 产生时：
  → 如果是可记录的命令，推送到 ReplayRecorder（如果录制中）
  → 录制格式：{ frame: u64, command: CameraCommand }

回放模式：
  → ReplayCameraSystem 从 ReplayFrame 读取 CameraCommand
  → 按照帧时间戳逐个 trigger CameraRequest
  → Camera 状态机正常处理这些 Request（不回放 CameraBounds——场景系统负责）
```

**初始实现**：Replay 录制接口预留（`CameraCommand` derive Serialize/Deserialize），回放消费不做。

### 8. 当前不做（Future 范围）

以下功能标记为 Future，不纳入初始架构：

| 功能 | 理由 | 触发条件 |
|------|------|---------|
| 多摄像机支持（CameraId） | 当前只需要一个主镜头 | MiniMap/Replay 独立相机需要时 |
| 优先级栈 | 当前 Focus 不与其他状态同时发生 | 多个系统同时请求镜头控制时 |
| 脚本驱动序列（CameraSequence） | SRPG 叙事系统尚未实现 Cinematic | Narrative/Cutscene 需要时 |
| 预测聚焦（Predictive Focus） | 精度要求不高，简化实现 | 用户反馈镜头跟随反应慢时 |
| Camera 调试面板 | DevTool 系统尚未实现 | Tools 层成熟时 |
| Focus 状态下排队后续请求 | 当前静默忽略，简化实现 | 多段聚焦动画需要时 |
| 3D/2.5D 旋转 | 当前只有 2D 视角 | 渲染层升级时 |
| CameraBounds 动态更新 | 当前在场景 OnEnter 设置一次 | 地图动态扩展/分区域切换需要时 |
| 震屏独立禁用 | Cue 的可选性由 Cue 系统管理（cue_domain.md §3.4） | 用户设置需要时 |

## Module Design

### 新增模块

```
src/infra/camera/
├── mod.rs
├── plugin.rs
├── foundation/
│   ├── mod.rs
│   ├── pose.rs
│   ├── target.rs
│   ├── request.rs
│   ├── state.rs
│   └── command.rs
├── systems/
│   ├── mod.rs
│   ├── input_handler.rs
│   ├── state_machine.rs
│   ├── movement.rs
│   ├── shake.rs
│   └── bounds.rs
├── query.rs
└── tests/
    ├── mod.rs
    ├── unit/
    └── integration/
```

### 修改文件

| 文件 | 变更 |
|------|------|
| `src/infra/mod.rs` | 新增 `pub mod camera;` |
| `src/app/app_plugin.rs` | 新增 `infra::camera::CameraPlugin` 注册（Phase 8，Input 之后） |
| `docs/01-architecture/README.md` | 更新 ADR 索引 + Camera 定位描述 |
| `docs/01-architecture/README.md` §3.4 | L2 Infra 表格新增 Camera 模块行 |
| `docs/01-architecture/README.md` §6.1 Phase 8 | 新增 `infra::camera::CameraPlugin` 注册 |
| `src/app/scenes/test_battle/render.rs` | 删除 `spawn_camera`，迁移到 CameraPlugin |

**不修改的文件**：所有 Capability Plugin、Domain Plugin、UI 层代码。

### Plugin 注册顺序

```rust
// Phase 8: Infrastructure (L2)
.add_plugins(infra::registry::RegistryPlugin)
.add_plugins(infra::pipeline::PipelinePlugin)
.add_plugins(infra::replay::ReplayPlugin)
.add_plugins(infra::save::SavePlugin)
.add_plugins(infra::input::InputPlugin)
.add_plugins(infra::camera::CameraPlugin)     // ← 新增：在 Input 之后，确保 InputAction 可用
.add_plugins(infra::localization::LocalizationPlugin)
```

`CameraPlugin` 内部注册顺序：

```rust
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        // 1. 注册类型
        app.register_type::<CameraPose>();
        app.register_type::<CameraBounds>();
        app.register_type::<CameraEvent>();

        // 2. 初始化 Camera Entity
        // 不由 CameraPlugin 直接 spawn——由场景系统在 OnEnter 时 spawn
        // CameraPlugin 提供 spawn_camera 函数供场景使用

        // 3. 注册 Observer（消费 CameraRequest）
        // 4. 注册 System（按调度顺序）
        app.add_systems(PreUpdate, (
            input_handler::handle_camera_input,
        ));
        app.add_systems(Update, (
            state_machine::process_camera_requests,
            state_machine::idle_timeout,
        ));
        app.add_systems(PostUpdate, (
            movement::interpolate_pose,
            bounds::clamp_position,
            shake::apply_shake,
            movement::write_to_transform,
        ));
    }
}
```

**Schedule 职责分配**：

| Schedule | System | 职责 |
|----------|--------|------|
| `PreUpdate` | `handle_camera_input` | 消费 InputAction，生成内部 CameraRequest |
| `Update` | `process_camera_requests` | Observer 处理外部 CameraRequest，更新 State Machine |
| `Update` | `idle_timeout` | FreeMove 空闲超时检测 → Idle |
| `PostUpdate` | `interpolate_pose` | TargetPose → CurrentPose 插值 |
| `PostUpdate` | `clamp_position` | 应用 CameraBounds 钳位 |
| `PostUpdate` | `apply_shake` | 震屏偏移叠加 |
| `PostUpdate` | `write_to_transform` | CurrentPose → Bevy Transform/Projection |

## Communication Design

| 通信方向 | 机制 | 说明 |
|---------|------|------|
| 外部系统 → Camera | `commands.trigger(CameraRequest::...)` | 单向请求，Camera 内部仲裁执行 |
| Input 层 → Camera | `Res<InputActionState>` 读取 + 内部转换 | input_handler system 读取 InputAction，转换为内部请求 |
| Cue 层 → Camera | CueTriggered（CueType::Shake）→ Camera 消费 | cue_domain.md §5.2 指定映射 |
| Camera → 其他系统 | 无反向事件（Camera 纯消费，不产生业务事件） | Camera 是表现层基础设施 |
| 外部只读查询 | `CameraQuery` SystemParam | world_to_screen, screen_to_world, visible_rect |
| GameState → Camera | OnEnter(GameState::X) 时场景系统 spawn Camera | 场景生命周期管理 Camera Entity |
| CameraBounds 设置 | 场景 OnEnter System 直接插 CameraBounds Component | 单向数据注入 |

### CameraQuery API

```rust
/// CameraQuery — 公开只读查询 API。
///
/// 外部系统通过此 API 获取镜头信息，无需直接 Query Camera Entity。
pub struct CameraQuery;  // SystemParam 包装

impl CameraQuery {
    /// 世界坐标 → 屏幕坐标
    pub fn world_to_screen(
        world_pos: Vec2,
        camera: &Camera,
        camera_transform: &GlobalTransform,
        window: &Window,
    ) -> Option<Vec2> { ... }

    /// 屏幕坐标 → 世界坐标
    pub fn screen_to_world(
        screen_pos: Vec2,
        camera: &Camera,
        camera_transform: &GlobalTransform,
        window: &Window,
    ) -> Option<Vec2> { ... }

    /// 当前可视矩形
    pub fn visible_rect(
        camera: &Camera,
        camera_transform: &GlobalTransform,
    ) -> Rect { ... }
}
```

`CameraQuery` 是纯函数集合，不依赖 Camera 内部组件。外部系统只需要 Bevy 标准的 `Camera`/`GlobalTransform`/`Window` 查询参数。

## 边界定义

### 允许

- 任何系统通过 `commands.trigger(CameraRequest::...)` 请求镜头变化
- 场景系统在 OnEnter 时 spawn Camera Entity（使用 `CameraPlugin::spawn_camera` 工厂函数）
- 场景系统在 OnEnter 时插 CameraBounds Component
- 外部系统使用 `CameraQuery` 查询镜头信息（world_to_screen 等）
- Cue 系统通过 CueTriggered 触发震屏（Camera 消费 CueType::Shake）
- Input 层产出 InputAction，Camera 系统在 PreUpdate 中消费
- Camera 内部使用内部 trigger 传递请求（不暴露内部事件细节）

### 禁止

- 外部系统直接修改 Camera Entity 的 Transform/Projection/GlobalTransform
- 外部系统通过 Query<&mut Transform, With<MainCamera>> 直接操作
- Camera 系统依赖任何 Domain（tactical/combat/narrative 等）的类型
- Camera 系统直接读取 GridPos/TileMap/MapConfig 等地图数据
- Camera 系统在 Update 之外独立驱动游戏逻辑（Camera 是纯表现层）
- Camera Entity 的 TargetPose/CurrentPose 被外部系统读取/修改
- Camera 模块中出现 GameState/OverlayState 的业务枚举名
- Focus 状态下处理新的外部 CameraRequest（初始实现）

## Forbidden（禁止事项）

| 行为 | 理由 |
|------|------|
| 外部系统直接 Query<&mut Transform, With<Camera>> 修改镜头 | 违反 Event 驱动原则，导致 Camera 状态机被绕过，不可 Replay |
| Camera 模块 import core::domains::* 的任何类型 | Camera 是 Infra 层，不应依赖业务 Domain |
| Camera 模块中出现 Combat/Dialogue/Unit 等业务词汇 | Camera 是通用系统，不应有业务知识 |
| CameraPose 中直接存 Entity | Entity 生命周期不稳定，使用 CameraTarget（UnitId/TilePos/Vec2） |
| 场景系统在 Update 中反复设置 CameraBounds | CameraBounds 应在 OnEnter 设置一次，后续不变化（当前范围） |
| Camera system 中使用 EventWriter/EventReader | 违反 ADR-054，必须使用 trigger + Observer 模式 |
| Camera system 中直接调用业务函数 | Camera 是纯消费层，不生产业务事件 |
| Camera Entity 跨 GameState 边界存活 | 符合 ADR-050 场景生命周期规范，OnExit 时 despawn |
| Camera 使用非确定性随机源用于震屏 | 违反 Replay First 原则——震屏偏移必须使用 SeededRng 或确定性算法 |
| 为未明确的 Future 功能预留在代码分支中 | 符合复杂度治理原则——只解决当前需要的状态 |

## Definition / Instance Design

| 类型 | 层级 | 存储位置 | 可变性 | 是否持久化 |
|------|------|---------|--------|-----------|
| `CameraPose` | Instance | ECS Component | 每帧可变 | 否（表现层状态） |
| `CameraState` | Instance | ECS Component（Camera Entity 标记） | 运行时可变 | 否 |
| `CameraBounds` | Instance | ECS Component | 场景生命周期可变 | 否 |
| `CameraRequest` | Transient | Event（trigger 时存在） | 瞬时 | 通过 CameraCommand 录制 |
| `CameraCommand` | Persistence | Replay 流 | 录制时不可变 | 是（Replay 文件） |
| `CameraTarget` | Transient | Request/Pose 中的值对象 | 不可变（创建后不改） | 仅包含在 CameraCommand 中 |

**说明**：Camera 没有 Definition（无配置定义的资产），所有类型是运行时 Instance 或瞬时 Event 类型。Camera 的行为完全由代码规则定义，没有通过 RON 配置化的内容。

## 后果

### 正面

1. **独立模块边界**——Camera 不是 Map/UI/Input 的附属，拥有完整的 State Machine 和 Pose 管线
2. **Event 驱动**——所有外部请求通过 CameraRequest，禁止直接修改 Transform，架构可追溯
3. **业务解耦**——Camera 不依赖任何 Domain 类型，通过 CameraTarget(Vec2/UnitId/i32) 和 CameraBounds(Vec2) 隔离
4. **Replay 就绪**——CameraCommand 序列化接口预留，关键操作可录制
5. **State Machine 仲裁**——消除多个 System 抢控制权的问题，Idle/FreeMove/Follow/Focus 职责清晰
6. **与现有架构对齐**——InputAction 已预定义、Cue 层已知 Camera 对接点、场景生命周期理解
7. **Query API**——外部系统无需直接 Query Camera Entity，通过 CameraQuery 安全读取

### 负面

1. **新增间接层**——Event 驱动 + State Machine + Pose 插值比直接修改 Transform 多三层
2. **初始实现复杂化**——7 行的 spawn_camera 变成完整的模块，约 500-800 行初始代码
3. **Focus 简化实现**——当前静默忽略队列请求，未来优先级栈需要重构
4. **手动场景集成**——每个 Scene 的 OnEnter 需要显式 spawn Camera + 设置 Bounds
5. **Replay 录制预留未实现**——CameraCommand 已定义但回放消费在 Phase 3

### 宪法同步

以下条款需要同步到 `docs/00-governance/ai-constitution-complete.md`：

- **Camera 架构定位条款**：Camera 是 Infra 层独立模块（`src/infra/camera/`），非业务 Domain
- **Camera Event 驱动条款**：所有外部镜头操作必须通过 `commands.trigger(CameraRequest::...)`，禁止直接修改 Camera Transform
- **Camera 业务解耦条款**：Camera 禁止依赖 `core::domains::*` 的任何类型
- **Camera Replay 条款**：CameraCommand 必须支持 Serialize/Deserialize

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| Camera 作为 core/domains/ 下的业务域 | Camera 不含业务规则（Formula/Condition/Effect），放在 Domains 违反 DDD 语义 |
| Camera 作为 ui/ 的子模块 | Camera 是全局系统，不是 UI 控件。放在 UI 下违背 "Camera 不知道业务域" 原则 |
| 无状态机，多个独立 System 直接修改 Transform | 历史教训：20+ System 抢控制权，导致谁改了什么不可追踪 |
| 使用 EventWriter/EventReader | 违反 ADR-054（Bevy 0.19 Observer 优先），必须使用 trigger + Observer |
| Camera 直接存 Entity 引用 | Entity 生命周期不稳定（死亡/回收/替换），使用 UnitId/Vec2/TilePos 更稳定 |
| CameraBounds 直接 import Map/Terrain 类型 | 违反解耦原则，Camera 不应知道地图结构 |
| 单体 CameraState Resource | 状态应该作为 Component 挂在 Camera Entity 上，符合 ECS 模式 |
| 延迟到有具体需求再设计（保持 7 行函数） | 输入层已预定义 Camera Action，Cue 层已知对接点，基础设施已成熟，此时引入架构成本最低 |

## 架构合规性自检

- [x] 符合 ECS 约束（Entity=ID, Component=数据, System=行为）— CameraPose/Bounds 为 Component，state_machine/movement 为 System
- [x] 双轴边界合规：Capabilities 无业务规则，Domain 无重复机制 — Camera 在 Infra 层
- [x] Domain 间无直接依赖 — Camera 不依赖任何 Domain 类型
- [x] Camera 有独立的模块边界（`infra/camera/`），integration/ 模式不适用（Infra 层不需要）
- [x] 没有创建禁止的模块（components.rs/systems.rs/utils.rs）
- [x] Effect/Modifier Pipeline 没有被绕过 — Camera 不涉及战斗数值
- [x] 符合"定义与实例分离"原则 — Camera 无 Definition，所有类型是 Instance
- [x] 符合"规则与内容分离"原则 — Camera 行为由代码规则定义，无 RON 配置
- [x] 符合"逻辑与表现分离"原则 — Camera 是表现层基础设施，不包含业务逻辑
- [x] 所有禁止事项已明确列出（10 条）
- [x] 通信机制使用 trigger + Observer（符合 ADR-054）
- [x] Plugin 注册顺序符合层次要求（Phase 8，Input 之后）
- [x] 已明确"当前不做"范围（8 项 Future 标记）
- [x] 已标注需要宪法同步的条款
