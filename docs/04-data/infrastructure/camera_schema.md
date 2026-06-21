---
id: infrastructure.camera.schema.v1
title: Camera Schema — 镜头系统数据架构
status: draft
owner: data-architect
created: 2026-06-21
updated: 2026-06-21
layer: instance
replay-safe: true
---

# Camera Schema — 镜头系统数据架构

> **领域归属**: Infrastructure — Camera | **定义依据**: `docs/01-architecture/40-cross-cutting/ADR-064-camera-architecture.md`, `docs/02-domain/infrastructure/camera_domain.md` | **依赖 Schema**: 无业务 Schema 依赖（仅引用 Bevy 内置类型 Vec2/f32/u64）

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `CameraPose` | Instance | 镜头姿态值对象，作为 ECS Component 存储（TargetPose/CurrentPose 各一个实例） |
| `CameraState` | Instance | 镜头状态机枚举，ECS Component |
| `CameraTarget` | Transient | 镜头目标值对象，嵌入在 Request/State/Command 中，不自立存在 |
| `CameraRequest` | Transient | 外部镜头请求事件枚举 |
| `CameraCommand` | Persistence | 可录制的镜头命令子集，存于 Replay 流 |
| `CameraBounds` | Instance | 镜头边界约束 ECS Component |
| `CameraShake` | Instance | 震屏效果临时状态 ECS Component |
| `CameraInputBlock` | Instance | 输入阻塞堆叠计数 ECS Component |
| `IdleTimeout` | Instance | FreeMove 空闲超时计时器 ECS Component |
| `CameraStateVm` | Transient | UI 只读投影值对象（非 ECS Component，由查询 API 构建） |

**说明**: Camera 系统没有 Definition 层——所有行为由代码规则定义，没有通过 RON 配置化的资产。这是 Infra 层基础设施的正常特征（与 input/registry 等同类组件一致）。

---

## 2. Type Map — 类型图谱总览

```
┌──────────────────────────────────────────────────────────────────┐
│  Transient Layer                                                 │
│  (瞬时事件 / 值对象 / 投影)                                       │
│                                                                  │
│  CameraTarget ─── 嵌入 ──→ CameraRequest                         │
│  CameraTarget ─── 嵌入 ──→ CameraCommand                         │
│  CameraTarget ─── 嵌入 ──→ CameraState (Follow/Focus)            │
│  CameraRequest ── trigger ──→ CameraPlugin Observer              │
│  CameraStateVm ── 查询构建 ──→ CameraQuery API                   │
├──────────────────────────────────────────────────────────────────┤
│  Instance Layer                                                   │
│  (ECS Components, 挂在 Camera Entity 上)                         │
│                                                                  │
│  CameraPose(TargetPose) ── 插值 ──→ CameraPose(CurrentPose)     │
│  CameraState ──→ 状态机仲裁                                       │
│  CameraBounds ──→ ClampSystem 钳位                               │
│  CameraShake ──→ 震屏叠加                                        │
│  CameraInputBlock ──→ 输入过滤                                   │
│  IdleTimeout ──→ 空闲超时检测                                    │
├──────────────────────────────────────────────────────────────────┤
│  Persistence Layer                                                │
│  (Replay 录制持久化)                                              │
│                                                                  │
│  CameraCommand ─── 录制 ──→ ReplayFrame                          │
│  CameraCommand ─── 回放 ──→ CameraRequest (Phase 3)              │
├──────────────────────────────────────────────────────────────────┤
│  Definition Layer                                                 │
│  (空白 — Camera 无配置化内容)                                     │
└──────────────────────────────────────────────────────────────────┘
```

---

## 3. Schema Design

### 3.1 CameraPose — 镜头姿态值对象（Instance 层）

```rust
/// 镜头姿态值对象。
///
/// 表示镜头在二维世界空间中的完整姿态——位置、缩放、旋转。
/// 在 Camera Entity 上以两个独立 Component 存在：
///   - TargetPose：状态机设置的目标姿态
///   - CurrentPose：每秒插值逼近 TargetPose 的当前姿态
///
/// Replay 影响：CameraPose 不直接持久化。目标姿态通过 CameraCommand 间接记录，
///             回放时由状态机 + 插值管线自动重建。震屏偏移叠加在 CurrentPose
///             上但不修改 CurrentPose 本身。
/// Save 影响：不存档。Camera 是表现层状态，场景 OnEnter 时重新初始化。
#[derive(Component, Debug, Clone, PartialEq)]
pub struct CameraPose {
    /// 世界坐标位置（二维）。
    /// 单位：游戏世界坐标单位。
    /// 范围：受 CameraBounds 约束（如果设置）。
    /// 不变性：此字段通过插值管线每帧更新，但禁止外部系统直接修改。
    pub position: Vec2,

    /// 缩放倍数。
    /// 1.0 = 默认缩放（每世界单位 = 每屏幕像素的基准比例）。
    /// 范围：[0.5, 3.0]，超出自动钳位。
    /// 步进：用户缩放按 0.5x / 1x / 2x / 3x 档位调整。
    /// Replay: 通过 CameraCommand::SetZoom 录制。
    pub zoom: f32,

    /// 旋转角度（弧度）。
    /// 当前阶段（2D）此字段为预留——始终为 0，不写入 Transform。
    /// 未来 2.5D / 3D 扩展时启用。
    /// 范围：[-PI, PI]（未来启用时）。
    pub rotation: f32,
}
```

**默认值**:
```rust
impl Default for CameraPose {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            rotation: 0.0,
        }
    }
}
```

**构造约束**:
| 字段 | 约束 | 越界处理 |
|------|------|----------|
| `zoom` | [0.5, 3.0] | SetZoom 请求时隐式钳位，不 panic |
| `rotation` | 始终为 0（当前阶段） | 非零值在写入 Transform 前被忽略 |

---

### 3.2 CameraTarget — 镜头目标值对象（Transient 层）

```rust
/// 镜头目标——目标位置的抽象表示。
///
/// 使用领域 ID 而非 ECS Entity 的原因：
/// - Entity 是 ECS 运行时概念，可能在聚焦期间被销毁/回收
/// - UnitId 是领域身份，不受 Entity 生命周期影响
/// - 符合 Definition/Instance 分离原则
///
/// 此类型不自立为 ECS Component 或 Resource，而是作为值对象嵌入
/// CameraRequest、CameraState、CameraCommand 中。
/// 系统内部在状态转移时将 CameraTarget 解析为 Vec2 存入 TargetPose。
///
/// Replay 影响：直接嵌入 CameraCommand，序列化后参与回放。
/// Save 影响：不独立存档。
#[derive(Debug, Clone, PartialEq)]
pub enum CameraTarget {
    /// 绝对世界坐标位置。
    /// 使用场景：镜头移动到指定地点（如地图标记点、初始位置）。
    /// 准确度：此位置的精确世界坐标，不受网格对齐影响。
    WorldPos(Vec2),

    /// 网格坐标位置（行, 列）。
    /// 使用场景：镜头聚焦到某个网格格点（如技能目标格、移动目标格）。
    /// 转换：系统内部将 TilePos 转换为世界坐标 (row * tile_size, col * tile_size)。
    /// 约束：不验证 TilePos 是否在网格范围内——由发送方负责。
    /// 与 WorldPos 的关系：TilePos 是逻辑位置，WorldPos 是物理位置。
    TilePos(i32, i32),

    /// 单位 ID。
    /// 使用场景：镜头跟随某个单位（移动中单位、选中的单位、对话中的单位）。
    /// 解析：系统内部通过 UnitId 查询单位当前位置（通过 CameraQuery 或直接查询）。
    /// 约束：UnitId 对应的单位可能不存在（已阵亡/已移除），此时 Follow 回退到当前位。
    /// 与 Entity 的关系：UnitId 是领域身份，不受 ECS Entity 回收影响。
    UnitId(u64),
}
```

**使用场景矩阵**:

| 变体 | CameraRequest 中使用 | CameraState 中使用 | CameraCommand 中使用 |
|------|---------------------|-------------------|---------------------|
| `WorldPos(Vec2)` | MoveTo, Reset | Focus | MoveTo, Reset |
| `TilePos(i32, i32)` | MoveTo, Follow | Follow, Focus | MoveTo, Follow |
| `UnitId(u64)` | MoveTo, Follow | Follow, Focus | MoveTo, Follow |

---

### 3.3 CameraRequest — 镜头请求事件枚举（Transient 层）

```rust
/// 所有镜头请求的统一枚举——Camera 系统的唯一外部修改入口。
///
/// 外部系统通过 `commands.trigger(CameraRequest::...)` 发送此事件。
/// CameraPlugin 内的 Observer 消费此事件并更新状态机。
/// 禁止外部系统直接修改 Camera Entity 的 Transform/Projection。
///
/// 此类型的本质是 Event，在 trigger 时瞬时存在，不持久化。
/// 可录制的请求同时以 CameraCommand 形式录制到 Replay 流。
///
/// Replay 影响：请求本身不持久化。可录制子集通过 CameraCommand 录制。
/// Save 影响：不存档。
#[derive(Event, Debug, Clone, PartialEq)]
pub enum CameraRequest {
    /// 移动到指定世界位置。
    ///
    /// 参数：
    ///   target: 目标位置（WorldPos/TilePos/UnitId）
    ///   duration: 插值过渡时长（秒）。0 = 瞬移。
    ///
    /// 谁触发：Tactical（单位选中后移动）、Narrative（对话聚焦）、
    ///         Cue（Effect 演出定位）、任何需要重新定位镜头的系统。
    /// 状态机行为：不改变 CameraState。仅设置 TargetPose。
    /// 录制：是 → CameraCommand::MoveTo（限外部发起，不录制用户输入产生的请求）
    MoveTo {
        /// 镜头目标位置。
        target: CameraTarget,
        /// 过渡动画时长（秒）。
        /// 0 = 立即跳转（无插值过渡）。
        /// >0 = 插值过渡，此期间 CurrentPose 逐渐逼近 TargetPose。
        /// 约束：>= 0。负值视为 0。
        duration: f32,
    },

    /// 跟随一个目标。
    ///
    /// 参数：
    ///   target: 跟随目标（UnitId / TilePos / WorldPos）
    ///
    /// 谁触发：Tactical（单位选中时）、Combat（技能释放时追踪目标）。
    /// 状态机行为：切换到 Follow 状态。如果之前是 FreeMove 则终止输入。
    /// 约束：Focus 状态下静默忽略。
    /// 录制：是 → CameraCommand::Follow
    Follow {
        /// 跟随目标。
        target: CameraTarget,
    },

    /// 取消跟随。
    ///
    /// 谁触发：Tactical（取消单位选中时）、Combat（技能执行完毕时）。
    /// 状态机行为：从 Follow 回到 Idle。非 Follow 状态下静默忽略。
    /// 录制：是 → CameraCommand::Unfollow
    Unfollow,

    /// 设置缩放级别。
    ///
    /// 参数：
    ///   zoom: 目标缩放倍数 [0.5, 3.0]
    ///   duration: 过渡时长（秒）
    ///
    /// 谁触发：Input（用户缩放）、Narrative（剧情强调）、任何需要调整视野的系统。
    /// 状态机行为：不改变 CameraState。直接更新 TargetPose.zoom。
    /// 约束：zoom 值超出 [0.5, 3.0] 时隐式钳位。
    /// 录制：是 → CameraCommand::SetZoom（限外部发起，不录制用户缩放）
    SetZoom {
        /// 目标缩放倍数。
        /// 范围：[0.5, 3.0]，超出自动钳位。
        zoom: f32,
        /// 过渡时长（秒）。
        /// 0 = 立即跳转。
        duration: f32,
    },

    /// 震屏效果。
    ///
    /// 参数：
    ///   intensity: 震动强度 [1.0, 20.0]
    ///   duration: 震动时长（秒）[0, 5.0]
    ///
    /// 谁触发：Cue 系统（Effect 触发震屏，如大范围技能、单位死亡）。
    /// 状态机行为：不改变 CameraState。创建/覆盖 CameraShake Component。
    /// 约束：intensity 超出 [1.0, 20.0] 时钳位。duration 超出 [0, 5.0] 时钳位。
    /// 确定性：震屏偏移使用 SeededRng，确保 Replay 一致。
    /// 录制：是 → CameraCommand::Shake
    Shake {
        /// 震动强度。
        /// 1.0 = 轻微抖动，5.0 = 中等震屏，10.0+ = 强烈震屏。
        /// 范围：[1.0, 20.0]，超出自动钳位。
        intensity: f32,
        /// 震动持续时间（秒）。
        /// 范围：[0, 5.0]，超出自动钳位。0 表示无震屏（静默忽略）。
        duration: f32,
    },

    /// 重置镜头到默认位置。
    ///
    /// 参数：
    ///   duration: 过渡时长（秒）
    ///
    /// 谁触发：场景系统（场景切换时）、UI（重置视角按钮）。
    /// 状态机行为：不改变 CameraState。设置 TargetPose 为默认姿态（由 spawn_camera 时定义）。
    /// 默认姿态由场景在 spawn_camera 时通过 default_pose 参数指定。
    /// 录制：是 → CameraCommand::Reset
    Reset {
        /// 过渡时长（秒）。
        /// 0 = 立即跳转。
        duration: f32,
    },

    /// 锁定用户镜头控制（FreeMove 输入）。
    ///
    /// 谁触发：Narrative（剧情演出时）、Combat（技能动画时）。
    /// 效果：设置 input_locked 标记，Camera 忽略所有 FreeMove 相关输入。
    /// 堆叠：通过 CameraInputBlock block_count 支持堆叠锁定。
    /// 不录制：锁定/解锁是瞬时控制状态，不影响 Replay 确定性。
    LockInput,

    /// 解锁用户镜头控制。
    ///
    /// 谁触发：Narrative（剧情演出结束时）、Combat（技能动画结束时）。
    /// 效果：减少 CameraInputBlock block_count。计数归零时恢复输入响应。
    /// 不录制：解锁/锁定是瞬时控制状态。
    UnlockInput,
}
```

---

### 3.4 CameraCommand — 可录制镜头命令（Persistence 层）

```rust
/// 可录制的镜头命令子集——Replay 流的数据载体。
///
/// CameraCommand 记录了关键帧镜头操作，作为 ReplayFrame 的一部分持久化。
/// 独立于 GameCommand 的 replay 流的理由：
///   镜头操作是表现层行为，不影响业务逻辑确定性。
///   但在回放时恢复镜头状态可提升观看体验。
///
/// 录制规则：
///   - 仅录制由外部系统触发的 CameraRequest（非用户输入产生的内部请求）
///   - 用户输入（WASD/缩放）不录制——这些是表现层交互，不影响业务确定性
///   - CameraBounds 设置不录制——由场景系统在回放时重新设置
///
/// 回放规则（Phase 3 实现）：
///   - ReplayCameraSystem 从 ReplayFrame 读取 CameraCommand
///   - 按帧时间戳逐个 trigger CameraRequest
///   - Camera 状态机正常处理这些 Request（与运行时行为一致）
///
/// Save 影响：此类型是 Replay 持久化的核心，必须带版本号。
/// Replay 影响：所有变体必须序列化/反序列化确定。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CameraCommand {
    /// 对应 CameraRequest::MoveTo。仅录制外部系统发起的 MoveTo。
    MoveTo(CameraTarget),

    /// 对应 CameraRequest::Follow。
    Follow(CameraTarget),

    /// 对应 CameraRequest::Unfollow。
    Unfollow,

    /// 对应 CameraRequest::SetZoom。仅录制外部系统发起的 SetZoom。
    SetZoom(f32),

    /// 对应 CameraRequest::Shake。
    /// 第一个 f32 = intensity，第二个 f32 = duration。
    Shake(f32, f32),

    /// 对应 CameraRequest::Reset。
    Reset,
}
```

**录制过滤规则**:

| CameraRequest 来源 | 是否录制 | 理由 |
|-------------------|---------|------|
| 外部系统（Tactical/Combat/Narrative） | 是 | 业务驱动的镜头行为，回放应恢复 |
| 用户输入（WASD/方向键/缩放） | 否 | 表现层交互，不影响业务确定性 |
| Cue 系统 Shake | 是 | Effect 触发的表现行为，回放应恢复 |
| CameraBounds 设置 | 否 | 场景生命周期事件，回放时重新设置 |
| LockInput / UnlockInput | 否 | 瞬时控制状态，回放时由状态机隐式管理 |

**版本兼容性策略**:
```
schema_version: u32 = 1

v1（当前）: 初始版本，6 个命令变体
v2（未来）: 新增命令变体 → 使用 serde tag 枚举兼容
v3（未来）: 参数类型变更 → 通过 Migration 链转换
```

---

### 3.5 CameraState — 镜头状态机枚举（Instance 层）

```rust
/// 镜头状态机——所有镜头行为通过此状态机仲裁。
///
/// 同一时刻有且仅有一个活跃的 CameraState 值。
/// 挂在 Camera Entity 上作为 ECS Component。
///
/// Replay 影响：状态本身不持久化。状态转移由 CameraCommand 回放间接驱动。
/// Save 影响：不存档。场景 OnEnter 时初始化为 Idle。
#[derive(Component, Debug, Clone, PartialEq)]
pub enum CameraState {
    /// 空闲：未触发任何主动行为，镜头静止。
    /// 进入动作：设置 TargetPose = CurrentPose（停止移动）。
    /// 退出动作：记录退出时间点（用于插值平滑）。
    /// 默认状态：Camera Entity 初始状态。
    Idle,

    /// 自由移动：玩家通过 WASD/方向键/边缘滚动/拖拽控制镜头。
    /// 进入动作：启动 IdleTimeout 计时器（2 秒）。
    /// 保持动作：每帧根据 InputAction 增量更新 TargetPose，刷新超时计时器。
    /// 退出动作：停止 IdleTimeout 计时器。
    /// 超时：输入停止 2 秒自动回到 Idle。
    FreeMove,

    /// 跟随：镜头跟随一个 CameraTarget（单位移动等）。
    /// 进入动作：设置 TargetPose = 目标位置。
    /// 保持动作：每帧根据目标当前位置更新 TargetPose。
    /// 退出动作：清除 Follow 目标引用。
    /// 输入覆盖：用户输入时切换为 FreeMove。
    Follow(CameraTarget),

    /// 聚焦：镜头动画过渡到特定 CameraTarget。
    /// 进入动作：设置 TargetPose = 目标位置，锁定输入，记录持续时间。
    /// 保持动作：每帧推进 elapsed，更新 TargetPose。
    /// 退出动作：解锁输入。
    /// 约束：聚焦期间不处理新的外部 CameraRequest（初始实现）。
    /// 自动完成：elapsed >= duration 时回到 Idle。
    Focus {
        /// 聚焦目标。
        target: CameraTarget,
        /// 聚焦动画总时长（秒）。
        duration: f32,
        /// 已过时间（秒）。每帧由 state_machine system 推进。
        /// 初始值：0。
        /// 完成条件：elapsed >= duration。
        elapsed: f32,
    },
}
```

**状态转移合法性矩阵**:

| 当前状态 ↓ \ 事件 → | FollowRequest | FocusRequest | UnfollowRequest | 用户输入 | 输入停止超时 | Focus 动画完成 |
|---|---|---|---|---|---|---|
| **Idle** | -> Follow | -> Focus | (静默忽略) | -> FreeMove | (静默忽略) | (静默忽略) |
| **FreeMove** | -> Follow | -> Focus | (静默忽略) | 保持 FreeMove,刷新超时 | -> Idle | (静默忽略) |
| **Follow** | -> Follow(切换目标) | -> Focus | -> Idle | -> FreeMove | (静默忽略) | (静默忽略) |
| **Focus** | 静默忽略 | 静默忽略 | (静默忽略) | 禁止(输入锁定) | (静默忽略) | -> Idle |

**不变量**:
1. 同一 Camera Entity 上有且仅有一个活跃的 CameraState 值
2. Focus 状态下输入锁定——所有 FreeMove 相关 InputAction 被忽略
3. Focus 状态下不处理新的外部 CameraRequest（初始实现）
4. FreeMove 输入停止 2 秒后自动回到 Idle
5. 用户输入始终可以覆盖 Follow（FreeMove > Follow 优先级）

---

### 3.6 CameraBounds — 镜头边界约束组件（Instance 层）

```rust
/// 镜头边界约束——限制镜头在世界空间中的移动范围。
///
/// 挂在 Camera Entity 上作为 ECS Component。
/// 由场景系统在 OnEnter 时插入（根据 GridMap 尺寸计算边界），
/// Camera 系统只读取不创建/修改。
///
/// 解耦设计：
///   - Camera 不感知 Map Domain / Terrain Domain 的任何类型
///   - CameraBounds 使用纯 Vec2，不包含 GridPos/TileMap 引用
///   - 边界数据由业务层（场景系统或 Map Domain）在恰当生命周期设置
///
/// 边界不存在时的行为：
///   当 Camera Entity 没有 CameraBounds Component 时不执行钳位，
///   镜头可自由移动到任何位置（无边界世界）。
///
/// Replay 影响：不在 Replay 流中录制。回放时由场景系统重新设置。
/// Save 影响：不存档。场景 OnEnter 时重新计算并插入。
#[derive(Component, Clone, Debug, PartialEq)]
pub struct CameraBounds {
    /// 边界最小值（世界坐标，左下角）。
    /// 镜头 position.x >= min.x 且 position.y >= min.y。
    pub min: Vec2,

    /// 边界最大值（世界坐标，右上角）。
    /// 镜头 position.x <= max.x 且 position.y <= max.y。
    pub max: Vec2,
}
```

**构造约束**:

| 规则 | 描述 | 越界处理 |
|------|------|----------|
| min.x <= max.x | X 轴最小值不大于最大值 | 视为配置错误，警告日志 + 跳过钳位 |
| min.y <= max.y | Y 轴最小值不大于最大值 | 视为配置错误，警告日志 + 跳过钳位 |

---

### 3.7 CameraShake — 震屏效果状态组件（Instance 层）

```rust
/// 震屏效果状态——临时 Component，震屏期间存在于 Camera Entity 上。
///
/// 由 CameraRequest::Shake 触发时创建，震屏结束时自动移除。
/// 震屏偏移在 ClampSystem 之后、TransformWrite 之前叠加到位置。
///
/// 确定性要求：
///   震屏偏移计算必须使用 SeededRng（从 Replay 上下文获取种子），
///   禁止使用非确定性随机源，确保 Replay 回放时震屏结果一致。
///
/// 堆叠策略（初始实现）：
///   同时收到多个 Shake 请求时，强度取最大值，时长重置。
///   不叠加多个震屏效果。
///
/// Replay 影响：不直接持久化。震屏通过 CameraCommand::Shake 录制，
///             回放时由 ShakeSystem 从种子重新生成偏移序列。
/// Save 影响：不存档。表现层瞬时状态。
#[derive(Component, Clone, Debug)]
pub struct CameraShake {
    /// 震动强度。
    /// 决定偏移幅度：offset_magnitude = intensity * base_scale。
    /// 范围：[1.0, 20.0]，超出在 CameraRequest 解析时已钳位。
    pub intensity: f32,

    /// 震动总时长（秒）。
    /// 范围：[0, 5.0]。
    pub duration: f32,

    /// 已过时间（秒）。每帧由 ShakeSystem 推进。
    /// 约束：elapsed <= duration。
    pub elapsed: f32,

    /// 当前帧的偏移量（每帧更新）。
    /// 由确定性算法从 SeededRng 生成，随 elapsed 衰减。
    /// 叠加公式：display_position = current_pose.position + current_offset。
    pub current_offset: Vec2,

    /// 确定性 RNG 种子偏移（由 replay 系统分配）。
    /// 种子计算方法：replay_initial_seed + seed_offset。
    /// 确保回放时产生与录制时一致的偏移序列。
    pub seed_offset: u64,
}
```

**默认值**:
```rust
impl CameraShake {
    /// 创建新的震屏状态。seed_offset 由 ReplayContext 分配。
    pub fn new(intensity: f32, duration: f32, seed_offset: u64) -> Self {
        Self {
            intensity: intensity.clamp(1.0, 20.0),
            duration: duration.clamp(0.0, 5.0),
            elapsed: 0.0,
            current_offset: Vec2::ZERO,
            seed_offset,
        }
    }
}
```

---

### 3.8 CameraInputBlock — 输入阻塞堆叠组件（Instance 层）

```rust
/// 输入阻塞堆叠计数器——支持多个系统同时锁定用户镜头控制。
///
/// 工作原理：
///   - CameraRequest::LockInput → block_count += 1
///   - CameraRequest::UnlockInput → block_count = block_count.saturating_sub(1)
///   - block_count > 0 时，所有 FreeMove 相关输入被忽略
///   - block_count == 0 时，正常响应 FreeMove 输入
///
/// 堆叠语义：多个系统分别 LockInput，各自 UnlockInput。
/// 只有所有锁都解除后（count == 0），输入才恢复。
/// 这避免了"一个系统 Lock 后崩溃，另一个系统永远无法 Unlock"的风险。
///
/// Replay 影响：不直接录制。回放时由状态机隐式管理。
/// Save 影响：不存档。场景 OnEnter 时初始化为 0。
#[derive(Component, Default, Debug, Clone, PartialEq)]
pub struct CameraInputBlock {
    /// 当前阻塞计数。
    /// 0 = 输入正常。>0 = 输入被阻塞。
    /// 最大值：u32::MAX，达到后 saturating_add 不再增长。
    pub block_count: u32,
}
```

---

### 3.9 IdleTimeout — 空闲超时计时器（Instance 层）

```rust
/// FreeMove 空闲超时计时器。
///
/// 仅在 CameraState == FreeMove 时存在。当用户停止输入后开始计时，
/// 超过 timeout_duration 秒无输入时自动回到 Idle。
///
/// 每帧由 idle_timeout system 检查：
///   1. 如果有新的用户输入：重置 elapsed 为 0
///   2. 如果没有输入：elapsed += delta_seconds
///   3. 如果 elapsed >= timeout_duration：CameraState -> Idle
///
/// Replay 影响：此计时器在 Replay 回放时不运行（回放不涉及用户输入）。
///             FreeMove 状态在回放中从不出现。
/// Save 影响：不存档。场景 OnEnter 时初始化为默认值。
#[derive(Component, Clone, Debug, PartialEq)]
pub struct IdleTimeout {
    /// 已过时间（秒）。从上一次用户输入开始计算。
    pub elapsed: f32,

    /// 超时时长（秒）。超过此值无输入则回到 Idle。
    /// 默认值：2.0 秒。
    pub timeout_duration: f32,
}

impl Default for IdleTimeout {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            timeout_duration: 2.0,
        }
    }
}
```

---

### 3.10 CameraStateVm — UI 只读投影（Transient 层）

```rust
/// Camera 状态 UI 投影——供 UI 层只读查询的系统当前状态。
///
/// 此类型不是 ECS Component，而是由 CameraQuery API 在 UI 帧末构建的瞬时值对象。
/// UI 层通过 CameraQuery::state_vm() 获取此对象，用于显示当前镜头状态信息
/// （如 HUD 上的缩放指示、锁定提示等）。
///
/// 与 Instance 的分离：
///   - UI 不直接读取 CameraPose/CameraState 等 ECS Component
///   - CameraStateVm 是受控的公开视图，不暴露内部状态细节
///   - 内部状态变化不影响 UI 层读取到的快照一致性
///
/// 后续扩展（Future）：可添加更多只读字段（如当前世界位置、目标类型等）。
///
/// Replay 影响：不涉及 Replay。
/// Save 影响：不存档。
#[derive(Debug, Clone, PartialEq)]
pub struct CameraStateVm {
    /// 当前缩放倍数（CurrentPose.zoom 的副本）。
    /// UI 用途：HUD 缩放指示器（"当前缩放: 2.0x"）。
    pub current_zoom: f32,

    /// 当前状态的字符串描述。
    /// UI 用途：调试信息、状态指示器。
    /// 值域：取值为 CameraState 变体名的 SnakeCase 字符串。
    ///   - "idle"
    ///   - "free_move"
    ///   - "follow"
    ///   - "focus"
    /// 说明：此字段使用 String 而非 enum，是为了避免 UI 层对 Camera 类型的编译依赖。
    ///       UI 层只需做字符串匹配，不引用 CameraState 枚举。
    pub current_state: String,

    /// 当前是否锁定输入。
    /// UI 用途：显示输入锁定指示（"当前输入已锁定"）。
    /// 对应 CameraInputBlock.block_count > 0。
    pub input_locked: bool,

    /// 当前是否正在震屏。
    /// UI 用途：震屏指示（可选显示）。
    /// 对应 CameraShake Component 是否存在。
    pub is_shaking: bool,

    /// 是否有边界约束。
    /// UI 用途：调试信息。
    /// 对应 CameraBounds Component 是否存在。
    pub has_bounds: bool,
}
```

---

## 4. Layer Analysis

| 数据结构 | Layer | 存储方式 | 持久化 | 生命周期 | Replay 安全 |
|----------|-------|---------|--------|---------|-------------|
| `CameraPose` (TargetPose) | Instance | ECS Component | 否 | 场景生存期 | 间接（通过 CameraCommand 重建） |
| `CameraPose` (CurrentPose) | Instance | ECS Component | 否 | 场景生存期 | 间接（通过插值重建） |
| `CameraState` | Instance | ECS Component | 否 | 场景生存期 | 间接（通过 CameraCommand 重建） |
| `CameraTarget` | Transient | 嵌入值对象 | 否（仅嵌入） | 瞬时 | 序列化要求 |
| `CameraRequest` | Transient | Event (trigger) | 否 | 瞬时 | 不可直接回放（通过 CameraCommand） |
| `CameraCommand` | Persistence | ReplayFrame 嵌入 | 是（Replay 文件） | Replay 文件生存期 | 是 |
| `CameraBounds` | Instance | ECS Component | 否 | 场景生存期 | 否（场景系统管理） |
| `CameraShake` | Instance | ECS Component（临时） | 否 | 震屏期间 | 间接（通过种子重建） |
| `CameraInputBlock` | Instance | ECS Component | 否 | 场景生存期 | 否（输入状态不录制） |
| `IdleTimeout` | Instance | ECS Component（临时） | 否 | FreeMove 期间 | 否（回放无用户输入） |
| `CameraStateVm` | Transient | 值对象（查询构建） | 否 | 帧瞬时 | 不涉及 |

**无污染确认**: 所有类型明确归属单一层，没有跨层污染。
- CameraPose 是纯 Instance（ECS Component），不承担配置/存档职责
- CameraTarget 是值类型，不自立为 Component/Resource，仅嵌入其他类型
- CameraRequest 是纯 Transient Event
- CameraCommand 是纯 Persistence（Replay 持久化）
- CameraBounds/CameraShake/CameraInputBlock/IdleTimeout 是纯 ECS Component
- CameraStateVm 是纯 Transient 投影值

---

## 5. Dependency Analysis

### 5.1 类型内部引用关系

```
CameraRequest
  ├── CameraTarget (Transient → 值嵌入)
  └── f32 (duration, zoom, intensity)

CameraState
  └── CameraTarget (Instance → 值嵌入)

CameraCommand
  ├── CameraTarget (Persistence → 值嵌入)
  └── f32 (zoom, intensity, duration)

CameraShake
  └── Vec2 (Bevy), f32, u64 (seed_offset)

CameraBounds
  └── Vec2 (Bevy)

CameraStateVm
  ├── f32 (current_zoom)
  └── String (current_state, 状态描述)

CameraPose
  └── Vec2 (Bevy), f32 (zoom, rotation)
```

### 5.2 跨 Schema 引用

| Camera 类型 | 引用的外部 Schema | 引用方式 | 方向 |
|------------|------------------|---------|------|
| `CameraTarget::UnitId(u64)` | Domain ID（不属于任何 Schema 的值） | 纯 u64 值，通过 Event 传递 | Camera -> Domain（值引用） |
| `CameraRequest` | 无 | Camera 消费事件 | 外部 -> Camera |
| `CameraCommand` | `ReplaySchema` → `ReplayFrame` | CameraCommand 嵌入 ReplayFrame | Camera -> Replay |
| `CameraQuery` | Bevy `Camera`, `GlobalTransform`, `Window` | 函数参数引用 | Camera -> Bevy |
| `CameraShake.seed_offset` | `ReplaySchema` → `ReplayContext` | Seed 由 Replay 系统分配 | Camera -> Replay |

**约束**: Camera 不引用任何业务 Domain Schema（Tactical/Combat/Narrative 等），符合 Infra 层定位。

---

## 6. Validation Rules

### 6.1 构造时校验

| # | 规则 | 作用域 | 校验逻辑 | 失败处理 |
|---|------|--------|----------|----------|
| V1 | CameraPose.zoom 范围 | `CameraPose` | zoom ∈ [0.5, 3.0] | 隐式钳位 + debug 日志 |
| V2 | CameraPose.rotation 预留 | `CameraPose` | rotation == 0.0 | 非零不写入 Transform，记录 warn 日志 |
| V3 | CameraBounds.min <= max | `CameraBounds` | min.x <= max.x && min.y <= max.y | 跳过钳位 + warn 日志 |
| V4 | Shake intensity 范围 | `CameraRequest::Shake` | intensity ∈ [1.0, 20.0] | 隐式钳位 + debug 日志 |
| V5 | Shake duration 范围 | `CameraRequest::Shake` | duration ∈ [0, 5.0] | 隐式钳位 + debug 日志 |
| V6 | SetZoom zoom 范围 | `CameraRequest::SetZoom` | zoom ∈ [0.5, 3.0] | 隐式钳位 + debug 日志 |
| V7 | MoveTo duration 非负 | `CameraRequest::MoveTo` | duration >= 0 | 负值视为 0 |
| V8 | Reset duration 非负 | `CameraRequest::Reset` | duration >= 0 | 负值视为 0 |
| V9 | Focus duration 正数 | `CameraState::Focus` | duration > 0 | 视为瞬时完成（duration = 0 → 立即完成） |
| V10 | Focus elapsed 不超 | `CameraState::Focus` | elapsed <= duration | state_machine 在 elapsed >= duration 时触发完成 |

### 6.2 运行时不变量

| # | 不变量 | 检查时机 | 违反后果 |
|---|--------|----------|----------|
| I1 | 同一 Camera Entity 只有一个 CameraState | 每次状态转移后 | 状态冲突属系统 Bug |
| I2 | Focus 状态下输入锁定 | Update 每帧 | 输入渗透属体验缺陷 |
| I3 | FreeMove 输入停止 2s 回 Idle | Update 每帧 | 镜头悬空属体验缺陷 |
| I4 | Camera 不依赖任何 Domain 类型 | 编译时 | 架构违规 |
| I5 | 震屏使用 SeededRng | 震屏创建时 | 回放不一致属系统 Bug |
| I6 | Camera Entity 不跨场景存活 | GameState OnExit | 状态污染属系统 Bug |
| I7 | CameraBounds 场景内不可变 | 运行时 | 镜头跳动属体验缺陷 |
| I8 | Focus 状态下不处理新外部 CameraRequest | 状态转移时 | 状态混乱属初始实现限制 |

---

## 7. Replay Compatibility

### 7.1 确定性分析

| Camera 操作 | 确定性 | 说明 |
|------------|--------|------|
| TargetPose → CurrentPose 插值 | 确定 | 纯数学 lerp(t)，不依赖随机数 |
| CameraBounds 钳位 | 确定 | position.clamp(min, max) 纯数学 |
| 状态转移仲裁 | 确定 | 输入决定论 + 计时器决定 |
| 用户输入（WASD/缩放） | 不确定 | **不录制到 Replay**，表现层交互 |
| 震屏偏移生成 | 确定 | 使用 SeededRng（种子从 ReplayContext 获取） |
| Focus 动画计时 | 确定 | 基于帧计数/DeltaTime（GameTime） |
| FreeMove 超时 | 确定 | 基于帧计数计时器 |

### 7.2 Replay 录制映射

| CameraCommand 变体 | 对应 CameraRequest | 录制条件 |
|-------------------|-------------------|----------|
| `MoveTo(CameraTarget)` | `CameraRequest::MoveTo` | 仅外部系统发起（排除用户输入） |
| `Follow(CameraTarget)` | `CameraRequest::Follow` | 所有外部 Follow |
| `Unfollow` | `CameraRequest::Unfollow` | 所有 Unfollow |
| `SetZoom(f32)` | `CameraRequest::SetZoom` | 仅外部系统发起（排除用户缩放输入） |
| `Shake(f32, f32)` | `CameraRequest::Shake` | 所有 Shake |
| `Reset` | `CameraRequest::Reset` | 所有 Reset |

**不录制列表**:
| 操作 | 不录制理由 |
|------|-----------|
| 用户输入 Movement/Zoom | 表现层交互，不影响业务确定性 |
| LockInput / UnlockInput | 输入锁在回放模式由状态机隐式管理 |
| CameraBounds 设置 | 场景生命周期事件，回放时重新设置 |

### 7.3 Replay 回放架构

```
录制路径（运行时）：
  CameraRequest (外部触发) → [录制检测] → CameraCommand → ReplayFrame

回放路径（Phase 3）：
  ReplayFrame → [ReplayCameraSystem] → CameraRequest (trigger) → 状态机正常处理
```

### 7.4 震屏确定性

```
震屏偏移生成算法：
  1. ShakeSystem 通过 ReplayContext 获取当前帧的 RNG 种子偏移
  2. 种子 = replay_initial_seed + CameraShake.seed_offset + frame_number
  3. 使用确定性 PRNG 从种子生成归一化偏移方向
  4. 偏移幅度 = intensity * (1.0 - elapsed/duration) * base_scale
  5. 每帧偏移量在 -amplitude 到 +amplitude 之间波动

保证：同一种子序列产生完全相同的震屏偏移序列。
```

---

## 8. Save Compatibility

### 8.1 存档策略

Camera 不参与游戏存档（Save）的原因：

| 理由 | 说明 |
|------|------|
| 表现层状态 | Camera 是纯表现层基础设施（镜头位姿是渲染状态，非游戏状态） |
| 场景生命周期 | Camera Entity 随场景创建/销毁（OnEnter spawn, OnExit despawn） |
| 无业务规则 | Camera 不包含游戏进度、角色状态、任务进度等可持久化数据 |
| 运行时重建 | 读档后出发场景时，场景 OnEnter System 重新 spawn Camera |

### 8.2 Persistence 层范围

Camera 的唯一持久化需求是 Replay 录制：

| 数据 | 持久化位置 | 格式 | 版本号 |
|------|-----------|------|--------|
| `CameraCommand` | ReplayFrame 嵌入 | 嵌入 ReplayLog 二进制流 | 跟随 ReplaySchema 版本 |
| `CameraCommand` 枚举 | serde (de)serialization | derive(Serialize, Deserialize) | serde tag 模式 |

### 8.3 CameraCommand 版本兼容

```rust
// 版本兼容策略：serde tag 枚举 + schema_version 字段

// v1 格式（当前）：
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CameraCommand {
    MoveTo(CameraTarget),
    Follow(CameraTarget),
    Unfollow,
    SetZoom(f32),
    Shake(f32, f32),
    Reset,
}

// v2（未来扩展示例：新增命令变体）：
// 新增 Focus(CameraTarget, f32) 变体
// 旧 Replay 文件中的未知变体 → 跳过（记录警告）
```

---

## 9. Migration Strategy

### 9.1 版本演进

| 版本 | 变更类型 | 变更内容 | 迁移策略 |
|------|---------|---------|----------|
| v1 | 初始 | 6 个 CameraCommand 变体 | — |
| v2 (未来) | 新增变体 | 新增 CameraCommand 变体 | serde tag 枚举新增 + 未知变体跳过 |
| v3 (未来) | 参数扩展 | 现有变体参数变更（如 Shake 增加参数） | 旧格式用 Migration 函数转换 |
| v4 (未来) | 字段扩展 | CameraStateVm 新增字段 | 新字段使用 Option<T>，旧代码默认值填充 |

### 9.2 迁移兼容规则

```
当前版本 v1 → 未来 v2：
  - 新增枚举变体：serde tag 格式确保旧文件加载时新变体被识别为 Unknown
  - 未知变体处理：跳过（不崩溃），记录 warn 日志

当前版本 v1 → 未来 v3：
  - 参数变化：提供 Migration v1→v2 + v2→v3 链式迁移
  - 不支持直接 v1→v3 迁移

所有迁移：
  - 迁移失败时保留原始 Replay 数据并报错
  - 迁移操作独立可测试
```

---

## 10. Future Extension

| 扩展点 | 影响的数据类型 | 预留设计 | 触发条件 |
|--------|---------------|---------|---------|
| 多摄像机支持 | CameraPose → 需 CameraId 标识 | CameraPose 无全局单例假设 | MiniMap/Replay 独立相机需要时 |
| 优先级栈 | CameraState → 添加 Request 队列 | Focus 初始实现静默忽略，预留队列插入点 | 多个系统同时请求镜头控制时 |
| 脚本驱动序列 | CameraCommand → 新增 CameraSequence | CameraCommand 枚举预留 tag 扩展 | Narrative/Cutscene 需要时 |
| 预测聚焦 | CameraState → Focus 中新增预测字段 | CurrentPose 插值算法可独立优化 | 用户反馈镜头跟随反应慢时 |
| 调试面板 | CameraStateVm → 新增调试字段 | CameraStateVm 是查询构建的投影值，字段可加 | Tools 层成熟时 |
| Focus 队列 | CameraState → Focus 中新增 pending 队列 | CameraState 枚举变体向前兼容 | 多段聚焦动画需要时 |
| 3D/2.5D 旋转 | CameraPose.rotation 启用 | rotation 字段已预留 | 渲染层升级时 |
| CameraBounds 动态更新 | CameraBounds → 运行时更新支持 | CameraBounds 现有字段可扩展 | 地图动态扩展需要时 |
| 回放消费 | CameraCommand → 回放路径实现 | CameraCommand 序列化已预留 | Phase 3 Replay 系统完善时 |

---

## 11. Risks

| 风险 | 影响 | 可能性 | 缓解措施 |
|------|------|--------|----------|
| 状态机与 CameraCommand 不一致 | 运行时状态与录制状态偏差导致回放异常 | 中 | 录制检测时验证当前状态与命令兼容性；单元测试覆盖所有状态转移 |
| 震屏种子分配冲突 | 多个 CameraShake 使用相同种子导致回放偏差 | 低 | seed_offset 由 ReplayContext 统一分配，保证全局唯一 |
| FreeMove 在回放模式下意外触发 | 回放时用户输入干扰镜头回放 | 低 | 回放模式禁用用户输入处理；Focus 状态天然锁定输入 |
| CameraTarget::UnitId 对应单位不存在 | Follow/Focus 时目标丢失 | 中 | 目标解析失败时回退到当前位置，不崩溃 |
| CameraBounds min > max 数据错误 | 镜头钳位失效 | 低 | 构造时校验 + 跳过钳位 + warn 日志 |
| 多系统同时 LockInput | 输入锁定计数溢出 | 低 | 使用 saturating_add/saturating_sub，不会溢出崩溃 |
| CameraCommand 版本不兼容 | 旧 Replay 文件无法在新版本播放 | 中 | serde tag 枚举 + 未知变体跳过；版本迁移链 |

---

## 12. Constitution Check

### 12.1 Data Laws 合规性自检

| # | 条款 | 合规 | 说明 |
|---|------|------|------|
| 001 | Def-Instance 分离 | ✅ | Camera 无 Definition 层。所有 Instance 类型（Pose/State/Bounds/Shake/InputBlock/Timeout）不承担配置和存档职责 |
| 002 | Rule-Content 分离 | ✅ | Camera 无可配置内容（无 RON 资产）。所有行为由代码规则定义（`camera_domain.md` §5 流程定义） |
| 003 | 配置只引用 ID | ✅ N/A | Camera 无 Definition 配置，无 ID 引用 |
| 004 | Ability 不拥有行为 | ✅ N/A | Camera 不属于 Ability 系统 |
| 005 | Effect 是唯一业务执行入口 | ✅ | Camera 不从 Ability/Trigger/Modifier 管线直接访问数据。仅通过与 Cue 层的 Event 通信（CueTriggered）消费震屏请求 |
| 006 | Modifier 不拥有业务逻辑 | ✅ N/A | Camera 不属于 Modifier 系统 |
| 007 | Duration 属于 Effect | ✅ N/A | Camera 的 duration 字段（MoveTo/SetZoom/Reset/Focus/Shake）是镜头过渡时长，与 Effect Duration 语义不同 |
| 008 | 堆叠归属 Stacking | ✅ | CameraInputBlock 的 block_count 堆叠是输入锁定计数，非游戏机制堆叠，无需归属 Stacking 系统。CameraShake 不叠加多个震屏（初始实现取最大值+重置时长） |
| 009 | 表现必须经过 Cue | ✅ | Camera 震屏消费 Cue::Shake 事件（通过 CueTriggered → CameraRequest::Shake 链路），不直接响应 Effect |
| 010 | Replay 优先 | ✅ | CameraCommand 序列化/反序列化就绪。震屏使用 SeededRng。用户输入不录制。所有时间基于帧计时，无 wall-clock 依赖 |
| 011 | Schema 版本化 | ✅ | CameraCommand 使用 serde tag 枚举格式，支持前向/后向兼容。schema_version 字段预留 |
| 012 | 域间禁止直接数据引用 | ✅ | Camera 不引用任何 `core::domains::*` 类型。通过 CameraTarget(Vec2/i32/u64) 和 CameraBounds(Vec2) 解耦 |
| 013 | 用户可见文本使用 LocalizationKey | ✅ | Camera 是表现层基础设施，不产出自有用户可见文本（无 name/desc/flavor 等文本字段）。CameraStateVm 的 current_state 字符串用于 UI 状态匹配，非用户可见文本 |
| 014 | LocalizationKey 以 en-US 模板 | ✅ N/A | Camera 不定义 LocalizationKey |

### 12.2 四层分离验证

- [x] **Definition Layer**: Camera 无 Definition 类型（零配置内容）
- [x] **Instance Layer**: CameraPose, CameraState, CameraBounds, CameraShake, CameraInputBlock, IdleTimeout — 均为纯 ECS Component
- [x] **Persistence Layer**: CameraCommand — 仅嵌入 ReplayFrame，不独立存档
- [x] **Transient Layer**: CameraTarget, CameraRequest, CameraStateVm — 瞬时值/事件

### 12.3 宪法关键条款对齐

| 条款 | 合规 | 说明 |
|------|------|------|
| P0 Replay First | ✅ | CameraCommand + SeededRng 震屏 + 无 wall-clock |
| P0 Feature First | ✅ | Camera 模块按业务语义组织（pose/target/request/state/command） |
| P0 Definition/Instance 分离 | ✅ | 无 Definition，所有类型为 Instance/Transient/Persistence |
| P0 分领域错误枚举 | ✅ | Camera 不产出自有错误枚举（不包含业务逻辑，不产生业务错误） |
| P0 四级通信 | ✅ | 使用 trigger + Observer（符合 ADR-054） |
| SS1.2 复杂度治理 | ✅ | 明确标记 8 项 Future 范围（多摄像机、优先级栈、Focus 队列等），不为未明确需求预设计 |
| 输入解耦 | ✅ | Camera 消费 InputAction 枚举，不直接读取原始按键绑定 |
| Cue 层对接 | ✅ | Camera 通过 CueTriggered 消费 Shake 请求（cue_domain.md §5.2） |
| 场景生命周期 | ✅ | Camera Entity 随 GameState OnEnter/OnExit 创建/销毁 |

---

## 附录 A: 类型字段影响矩阵

| 类型 | 字段 | 影响 Replay | 影响 Save | UI 可见 | 说明 |
|------|------|-------------|-----------|---------|------|
| `CameraPose` | `position` | 间接 | 否 | 通过 CameraStateVm | 插值重建 |
| `CameraPose` | `zoom` | 间接 | 否 | 通过 CameraStateVm | 录制在 CameraCommand |
| `CameraPose` | `rotation` | 否 | 否 | 否 | 预留字段 |
| `CameraState` | 所有变体 | 间接 | 否 | 通过 CameraStateVm | 通过 CameraCommand 重建 |
| `CameraBounds` | `min, max` | 否 | 否 | 否 | 场景生命周期管理 |
| `CameraShake` | `intensity, duration, elapsed, offset, seed` | 间接 | 否 | 通过 CameraStateVm | 通过种子重建 |
| `CameraInputBlock` | `block_count` | 否 | 否 | 通过 CameraStateVm | 回放无用户输入 |
| `CameraCommand` | 所有变体 | 直接 | 否（仅 Replay） | 否 | 核心 Replay 数据 |
| `CameraTarget` | 所有变体 | 嵌入 | 否（仅嵌入） | 否 | 值类型不自立存在 |
| `CameraStateVm` | 所有字段 | 否 | 否 | 直接 | UI 投影值 |
