---
id: 02-domain.camera
title: Camera（镜头）领域规则 v1.0
status: draft
owner: domain-designer
created: 2026-06-21
tags:
  - domain
  - camera
  - infrastructure
  - state-machine
---

# Camera（镜头）领域规则 v1.0

> **架构依据**: ADR-064-Camera-Architecture — Camera 是 Infra 层独立模块，非业务 Domain
> **模块位置**: `src/infra/camera/` — 与 registry/pipeline/replay/save/input 平级
> **状态机**: Idle / FreeMove / Follow / Focus 四状态
> **驱动模式**: Event 驱动（CameraRequest 统一枚举），禁止直接修改 Transform

---

## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| CameraPose | 镜头姿态值对象，包含 position (Vec2)、zoom (f32)、rotation (f32) | 负责：镜头位置/缩放/旋转的数学表示；不负责：ECS 引用、渲染写入 |
| TargetPose | 镜头目标姿态，由状态机设置，描述镜头最终要到达的位姿 | 负责：存储状态机仲裁后的目标姿态；不负责：插值计算、实际写入 |
| CurrentPose | 镜头当前姿态，每秒插值接近 TargetPose | 负责：存储当前实际姿态（插值结果）；不负责：目标设定、边界钳位 |
| CameraState | 镜头状态机枚举：Idle / FreeMove / Follow / Focus | 负责：镜头行为的仲裁与控制权管理；不负责：具体姿态计算 |
| CameraTarget | 镜头目标值对象：WorldPos(Vec2) / TilePos(i32,i32) / UnitId(u64) | 负责：目标位置的抽象表示；不负责：Entity 引用、生命周期管理 |
| CameraRequest | 外部系统通过 trigger() 发送的镜头请求统一枚举 | 负责：所有外部系统与 Camera 的通信契约；不负责：请求的执行顺序仲裁 |
| CameraCommand | CameraCommand 可录制的镜头命令子集，支持 Serialize/Deserialize | 负责：Replay 录制与回放的数据载体；不负责：运行时状态管理 |
| CameraBounds | 镜头边界约束组件 (Vec2 min, Vec2 max)，挂在 Camera Entity 上 | 负责：镜头在世界空间中的移动范围限制；不负责：地形/地图数据的读取 |
| CameraQuery | 外部只读查询 API：world_to_screen / screen_to_world / visible_rect | 负责：坐标系转换和可视区域计算；不负责：状态修改 |
| CameraShake | 震屏效果状态，包含强度、时长、计时器和偏移量 | 负责：震屏偏移的生成与衰减；不负责：Pose 插值、边界钳位 |
| IdleTimeout | FreeMove 状态下的空闲超时计时器，N 秒无输入自动回到 Idle | 负责：FreeMove 自动退回 Idle 的计时；不负责：输入处理 |

### 1.1 术语与项目词汇对齐

- **Unit** / **Tile** / **WorldPos**：CameraTarget 使用领域 ID（UnitId / TilePos / Vec2）而非 ECS Entity，与项目"Definition/Instance 分离"原则一致（Entity 是 ECS 实现细节，UnitId 是领域身份）
- **GameState**：Camera Entity 生命周期绑定 GameState 场景——OnEnter 时 spawn，OnExit 时 despawn（符合 ADR-050）
- **InputAction**：Camera 消费 InputAction（CameraUp/Down/Left/Right/ZoomIn/ZoomOut），Input 层只产出不感知 Camera 具体实现
- **Cue**：Camera 通过 cue_domain.md §5.2 定义的 Shake 路由消费震屏请求
- **Replay**：CameraCommand 作为 Replay 流的一部分录制/回放（独立于 GameCommand）

---

## 2. 状态机

### 2.1 状态定义

```
CameraState（镜头状态机）
 │
 ├── Idle ─────── 空闲：未触发任何主动行为，镜头静止
 │                 默认状态，等帧插值到当前位置保持静止
 │
 ├── FreeMove ─── 自由移动：玩家通过 WASD/方向键/边缘滚动/拖拽控制镜头
 │                 输入期间持续移动，输入停止 N 秒后回 Idle
 │
 ├── Follow ───── 跟随：镜头跟随一个 CameraTarget（单位移动等）
 │                 每帧将 TargetPose 设置为目标位置
 │
 └── Focus ────── 聚焦：镜头动画过渡到特定 CameraTarget
                    target（目标）、duration（时长）、elapsed（已过时间）
                    计时完成自动回到 Idle
```

### 2.2 状态转移规则

```
                           ┌──────────────┐
                  启动 ──→ │     Idle     │
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

### 2.3 状态转移矩阵

| 当前状态 ↓ \ 事件 → | FollowRequest | FocusRequest | UnfollowRequest | 用户输入 | 输入停止超时 | Focus 动画完成 |
|---|---|---|---|---|---|---|
| **Idle** | → Follow | → Focus | (无意义，静默忽略) | → FreeMove | (无意义) | (无意义) |
| **FreeMove** | → Follow | → Focus | (无意义) | 保持 FreeMove（刷新超时计时器） | → Idle | (无意义) |
| **Follow** | → Follow（切换目标） | → Focus | → Idle | → FreeMove（用户覆盖） | (无意义，Follow 不超时) | (无意义) |
| **Focus** | 静默忽略（初始实现） | 静默忽略（初始实现） | (无意义) | 禁止（输入锁定） | (无意义) | → Idle |

### 2.4 状态进入/退出动作

| 状态 | 进入动作 | 退出动作 | 保持动作 |
|------|---------|---------|---------|
| Idle | 设置 TargetPose = CurrentPose（停止移动） | 记录退出时间点 | 无（CurrentPose 插值稳定） |
| FreeMove | 启动 IdleTimeout 计时器 | 停止 IdleTimeout 计时器 | 每帧根据 InputAction 增量更新 TargetPose，刷新超时计时器 |
| Follow | 设置 TargetPose = 目标位置 | 清除 Follow 目标引用 | 每帧根据目标当前位置更新 TargetPose |
| Focus | 设置 TargetPose = 目标位置，锁定输入，记录持续时间 | 解锁输入 | 每帧推进 elapsed，更新 TargetPose |

### 2.5 状态不变量

- 同一时刻 CameraState 有且仅有一个活跃状态值
- Focus 状态下输入锁定：所有 FreeMove 相关的 InputAction 被忽略
- Focus 状态下不处理新的外部 CameraRequest（初始实现）
- FreeMove 输入停止 2 秒后自动回到 Idle
- 用户输入始终可以覆盖 Follow（FreeMove > Follow 优先级）

---

## 3. 不变量（Invariants）

### 3.1 CameraRequest 是唯一外部修改入口
- **条件**：任何外部系统需要修改镜头行为时
- **不变量**：所有镜头行为变化必须通过 `commands.trigger(CameraRequest::...)` 发起，禁止绕过 CameraRequest 直接修改 Camera Entity
- **违反后果类型**：🔴 程序错误
- **违反后果**：绕过 CameraRequest 的修改导致状态机不一致，无法被 Replay 录制，属系统 Bug

### 3.2 CameraPose 分离不可绕过
- **条件**：任何 Pose 相关操作
- **不变量**：所有 Pose 修改必须经过 TargetPose → Interpolation → CurrentPose → Clamp + Shake → Transform 管线，禁止直接修改 Transform/Projection
- **违反后果类型**：🔴 程序错误
- **违反后果**：直接修改 Transform 导致 CameraShake/Clamp 被跳过，画面抖动与逻辑状态不一致，属系统 Bug

### 3.3 Camera 对 Domain 零依赖
- **条件**：编译时
- **不变量**：Camera 模块禁止 import `core::domains::*` 的任何类型
- **违反后果类型**：🔴 架构违规
- **违反后果**：引入 Domain 依赖违反 Infra 层定位，导致循环依赖和架构腐化

### 3.4 CameraState 单一性
- **条件**：任何时刻
- **不变量**：同一 Camera Entity 上同时只有一个 CameraState 值
- **违反后果类型**：🔴 程序错误
- **违反后果**：状态冲突导致多个 System 抢控制权，属系统 Bug

### 3.5 Focus 状态输入锁定
- **条件**：CameraState == Focus
- **不变量**：FreeMove 相关的 InputAction 被忽略，Camera 拒绝处理用户移动/缩放输入
- **违反后果类型**：🟡 体验缺陷
- **违反后果**：聚焦动画期间用户可干预镜头导致聚焦效果中断

### 3.6 Camera Entity 不跨场景存活
- **条件**：GameState 场景切换时
- **不变量**：Camera Entity 在当前场景 OnExit 时被 despawn，OnEnter 时重新 spawn
- **违反后果类型**：🔴 程序错误
- **违反后果**：跨场景残存的 Camera Entity 导致状态污染、Transform 引用失效，属系统 Bug

### 3.7 CameraBounds 不可变性（当前范围）
- **条件**：场景生命周期内
- **不变量**：CameraBounds 在 OnEnter 设置后不发生变化（当前实现不支持动态更新）
- **违反后果类型**：🟡 体验缺陷
- **违反后果**：场景运行时 CmaeraBounds 被修改会导致镜头突然跳动

### 3.8 震屏使用确定性随机
- **条件**：震屏效果执行时
- **不变量**：震屏偏移计算必须使用 SeededRng 或确定性算法，禁止使用非确定性随机源
- **违反后果类型**：🔴 程序错误
- **违反后果**：非确定性随机导致 Replay 回放时震屏结果不一致，违反 Replay First 原则

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：外部系统直接 `Query<&mut Transform, With<MainCamera>>` 修改镜头 — 理由：违反 Event 驱动原则，导致 Camera 状态机被绕过，不可 Replay
- 🟥 禁止：Camera 模块 import `core::domains::*` 的任何类型 — 理由：Camera 是 Infra 层组件，不应依赖业务 Domain
- 🟥 禁止：Camera 模块中出现 Combat/Dialogue/Tactical/Quest 等业务词汇 — 理由：Camera 是通用系统，不应有业务知识，通过 CameraTarget(Vec2/TilePos/UnitId) 解耦
- 🟥 禁止：Camera TargetPose/CurrentPose 被外部系统读取/修改 — 理由：Pose Component 是 Camera 内部状态，外部应通过 CameraQuery 只读访问
- 🟥 禁止：Camera 使用 Entity 引用作为目标 — 理由：Entity 生命周期不稳定（死亡/回收/替换），必须使用 CameraTarget（UnitId / TilePos / Vec2）
- 🟥 禁止：Camera 系统使用 EventWriter/EventReader — 理由：违反 ADR-054，必须使用 trigger + Observer 模式
- 🟥 禁止：Camera system 中直接调用业务函数或发布业务事件 — 理由：Camera 是纯消费层，不生产业务事件
- 🟥 禁止：Camera Entity 跨 GameState 边界存活 — 理由：不符合 ADR-050 场景生命周期规范，OnExit 时必须 despawn
- 🟥 禁止：Camera 使用非确定性随机源用于震屏 — 理由：违反 Replay First 原则，震屏偏移必须使用 SeededRng 或确定性算法
- 🟥 禁止：CameraBounds 中引用 GridPos/TileMap/MapConfig 等地图数据 — 理由：Camera 不应知道地图结构，边界数据由场景系统在 OnEnter 时以 Vec2 形式注入
- 🟥 禁止：FreeMove 状态下忽略所有输入（LockInput 时除外）— 理由：输入锁定必须显式通过 CameraRequest::LockInput 触发，禁止隐式锁定
- 🟥 禁止：Focus 状态下处理新的外部 CameraRequest — 理由：初始实现简化，Focus 队列会在未来引入优先级栈时再实现

---

## 5. 流程定义

### 5.1 Pose 插值管线

- **输入**：TargetPose（由状态机在前一帧设置）、CurrentPose（上一帧插值结果）、DeltaTime
- **处理**：
  1. `InterpolationSystem` 在 PostUpdate 调度
  2. 对 CurrentPose 执行帧率无关的线性插值：`CurrentPose = CurrentPose.lerp(TargetPose, t)`
  3. t 的计算基于配置的插值速度和 DeltaTime，确保不同帧率下的平滑度一致
  4. 插值是确定性数学运算（不依赖随机数）
- **输出**：更新后的 CurrentPose（目标未变化时稳定到精确值）
- **失败处理**：DeltaTime 异常过大（如断点续连）时，限制单帧最大插值量，避免镜头瞬移

### 5.2 边界钳位流程

- **输入**：CurrentPose（插值后）、CameraBounds（如存在）
- **处理**：
  1. `ClampSystem` 在 PostUpdate 调度（InterpolationSystem 之后）
  2. 检查 Camera Entity 是否有 CameraBounds Component
  3. 如果有：`pose.position = pose.position.clamp(bounds.min, bounds.max)`
  4. 如果没有：跳过钳位，允许镜头移动到任何位置
- **输出**：钳位后的 CurrentPose
- **失败处理**：CameraBounds 的 min > max 时视为配置错误，按非钳位处理并记录警告

### 5.3 震屏流程

- **输入**：CameraRequest::Shake { intensity, duration }、CurrentPose（钳位后）
- **处理**：
  1. `ShakeSystem` 在 PostUpdate 调度（ClampSystem 之后）
  2. 收到 CameraRequest::Shake 时，创建/覆盖 CameraShake Component（Timer + 强度 + 偏移量）
  3. 每帧检查 CameraShake 是否存在：
     a. 如果存在且在计时器有效期内，使用 SeededRng 生成确定性随机偏移
     b. 偏移量随计时器衰减（线性或指数衰减）
     c. 叠加到 CurrentPose.position：`display_position = current_pose.position + shake_offset`
  4. 计时器到期后移除 CameraShake Component
- **输出**：叠加震屏偏移后的显示位置（用于 Transform 写入）
- **失败处理**：同时收到多个 Shake 请求时，强度取最大值，时长重置（初始实现不做叠加）

### 5.4 Transform 写入流程

- **输入**：CurrentPose（已钳位 + 震屏偏移叠加）
- **处理**：
  1. `TransformWriteSystem` 在 PostUpdate 调度（最后一步）
  2. 将 CurrentPose.position 写入 `Transform.translation.xy`（z 值固定为场景深度常数）
  3. 将 CurrentPose.zoom 写入 `OrthographicProjection.scale`
  4. rotation 字段预留，不写入（保持 0）
- **输出**：Bevy Transform 和 Projection 组件更新
- **失败处理**：Camera Entity 缺少 Camera/GlobalTransform 等必要组件时跳过写入，记录警告

### 5.5 状态转移处理

- **输入**：CameraRequest 事件（通过 trigger 发送）
- **处理**：
  1. `process_camera_requests` Observer 在 Update 调度
  2. 根据当前 CameraState 和请求类型查转移矩阵（§2.3）
  3. 如果转移允许：
     a. 执行退出动作
     b. 更新 CameraState 为新状态
     c. 执行进入动作
  4. 如果转移禁止：静默忽略（不报错，不产生日志）
  5. Focus 状态下忽略所有非 Reset/UnlockInput 请求
- **输出**：更新后的 CameraState 和对应的 TargetPose 设置
- **失败处理**：无效状态转移静默忽略，不产生错误事件

### 5.6 空闲超时检测

- **输入**：IdleTimeout 计时器、用户输入状态
- **处理**：
  1. `idle_timeout` System 在 Update 调度
  2. 仅在 CameraState == FreeMove 时运行
  3. 每帧检查自上次输入以来的经过时间
  4. 如果超过 2 秒无输入：CameraState → Idle，TargetPose 固定到当前位置
- **输出**：CameraState 从 FreeMove 变为 Idle
- **失败处理**：不适用

### 5.7 Camera Entity 场景生命周期

- **输入**：GameState 场景 OnEnter 事件
- **处理**：
  1. 工厂函数 `CameraPlugin::spawn_camera` 供场景 OnEnter System 调用
  2. 创建 Camera Entity 并附加：
     a. Camera2d / Camera3d 组件
     b. CameraPose（初始位置、zoom=1.0、rotation=0）
     c. CameraState::Idle
     d. TargetPose = CurrentPose = CameraPose
     e. MainCamera 标记组件（用于外部查询识别）
  3. 场景 OnEnter System 可选插入 CameraBounds Component
  4. 场景 OnExit System 调用 `despawn_camera` 移除 Camera Entity
- **输出**：可用的 Camera Entity
- **失败处理**：场景 OnExit 时 Camera Entity 已不存在，跳过 despawn

---

## 6. 事件契约

### 6.1 CameraRequest 事件契约

Camera 是纯粹的事件消费者，不发布业务事件。`CameraRequest` 作为唯一外部接口，每个变体的语义如下：

| 请求 | 触发条件 | 预期行为 | 副作用 | 前提约束 |
|------|---------|---------|--------|---------|
| `MoveTo { target, duration }` | 外部系统需要镜头移动到指定位置 | 设置 TargetPose 为目标位置；如果目标与当前位置距离较大且 duration > 0，则状态不变（Follow/Focus 状态下行为由状态机仲裁） | 更新 TargetPose；不改变 CameraState（仅在 Idle 时） | 无 |
| `Follow { target }` | 外部系统需要镜头跟随某目标 | 设置 CameraState = Follow；目标类型决定追踪方式：UnitId 则追踪单位当前位置，WorldPos/TilePos 则追踪固定位置 | CameraState 切换到 Follow；如果之前是 FreeMove 则终止输入 | Focus 状态下静默忽略 |
| `Unfollow` | 外部系统取消跟随 | 如果 CameraState == Follow，设置 CameraState = Idle；TargetPose 固定到当前位置 | CameraState 回到 Idle | Follow 状态下才能取消 |
| `SetZoom { zoom, duration }` | 外部系统需要调整缩放 | 设置 TargetPose.zoom 为目标值；自动钳位到系统允许的缩放范围 [0.5, 3.0] | 更新 TargetPose.zoom | 无 |
| `Shake { intensity, duration }` | Cue 系统/外部触发震屏 | 创建/覆盖 CameraShake Component；强度钳位到 [1.0, 20.0] 范围内 | 震屏叠加到镜头位置 | 无 |
| `Reset { duration }` | 外部系统需要镜头回到默认位置 | 设置 TargetPose = default_pose（由场景系统在 spawn 时定义）；不改变 CameraState | 更新 TargetPose | 无 |
| `LockInput` | 外部系统需要锁定用户镜头控制（如剧情演出） | 设置 input_locked 标记；Camera 忽略所有 FreeMove 相关输入 | 输入锁定 | 无 |
| `UnlockInput` | 外部系统需要解锁用户镜头控制 | 清除 input_locked 标记；恢复 FreeMove 输入响应 | 输入解锁 | 无 |

### 6.2 事件订阅者关系

```
CameraRequest（外部系统 trigger）
    │
    ├──↦ 来自 Tactical：单位选中时 Follow、单位移动时 MoveTo
    ├──↦ 来自 Combat：技能执行时 Focus、单位死亡时轻微 Shake
    ├──↦ 来自 Cue：Effect 触发震屏请求（CueType::Shake → Shake）
    ├──↦ 来自 Input：用户输入 → 内部转换为 MoveTo/FreeMove
    ├──↦ 来自 Narrative：对话触发时 Focus、剧情对话结束时 UnlockInput
    └──↦ 来自 System：游戏状态切换时的 Reset

Camera 系统（消费后）
    │
    └── 不产生反向事件（Camera 是纯消费层，无业务事件输出）
```

### 6.3 CameraQuery 只读 API

| 方法 | 输入 | 输出 | 用途 |
|------|------|------|------|
| `world_to_screen` | world_pos: Vec2, camera: &Camera, transform: &GlobalTransform, window: &Window | Option<Vec2> | 将世界坐标转换为屏幕像素坐标 |
| `screen_to_world` | screen_pos: Vec2, camera: &Camera, transform: &GlobalTransform, window: &Window | Option<Vec2> | 将屏幕像素坐标转换为世界坐标 |
| `visible_rect` | camera: &Camera, transform: &GlobalTransform | Rect | 获取当前视野矩形（世界空间） |

**使用约束**：
- CameraQuery 是纯函数集合，不依赖 Camera 内部组件
- 外部系统只需要 Bevy 标准 Camera/GlobalTransform/Window 查询参数
- 禁止外部系统通过 CameraQuery 返回的数据反向推导 Camera 内部状态

---

## 7. 与 Input 层的关系

### 7.1 InputAction → CameraRequest 映射

```
InputAction              CameraRequest（内部生成）
──────────               ────────────────
CameraUp                 TargetPose.position += up_vector * speed * dt
CameraDown               TargetPose.position += down_vector * speed * dt
CameraLeft               TargetPose.position += left_vector * speed * dt
CameraRight              TargetPose.position += right_vector * speed * dt
CameraZoomIn             TargetPose.zoom *= zoom_factor（最小 0.5）
CameraZoomOut            TargetPose.zoom /= zoom_factor（最大 3.0）
```

### 7.2 映射规则

1. **输入处理时机**：`input_handler` System 在 PreUpdate 调度，先于状态机运行
2. **触发条件**：
   - CameraState 为 Idle 时，任何 FreeMove 相关的 InputAction 触发状态切换 FreeMove
   - CameraState 为 FreeMove 时，输入增量更新 TargetPose，刷新 IdleTimeout 计时器
   - CameraState 为 Follow 时，用户输入触发生效（不做突发跳转），状态切换为 FreeMove
   - CameraState 为 Focus 时，所有 FreeMove 输入被忽略（输入锁定）
   - input_locked 标记为 true 时，所有 FreeMove 输入被忽略
3. **移动速度**：移动速度由 Camera 系统内部配置（pixels_per_second），与 Input 层无关
4. **缩放步进**：缩放分档 0.5x/1x/2x/3x，每档步进系数固定
5. **内部转换**：InputAction 不产生公开的 CameraRequest（这是内部输入处理，不是外部请求）
6. **实施方式**：`input_handler` System 直接修改 TargetPose（作为 Camera 内部系统）；

### 7.3 解耦边界

- Input 层（`infra/input/`）只产出 `InputAction` 枚举，不感知 Camera 的存在
- Camera 通过 `InputActionState` Resource 读取当前输入状态
- Camera 不依赖 Input 层的按键绑定细节（仅消费 InputAction 分类结果）

---

## 8. 与 Cue 层的关系

### 8.1 CueType::Shake → CameraRequest::Shake 映射

```
Cue 系统（cue_domain.md §5.2 指定）
    │
    ├── CueTriggered { cue_type: CueType::Shake, context: { intensity, duration } }
    │
    ▼
Camera 系统消费
    │
    ├── 在 Observer 中监听 CueTriggered（仅关注 CueType::Shake）
    ├── 提取 intensity 和 duration 参数
    ├── 调用 commands.trigger(CameraRequest::Shake { intensity, duration })
    │
    ▼
CameraShake System 执行震屏效果
```

### 8.2 映射规则

1. Camera 仅消费 CueType::Shake，忽略其他 CueType（VFX/SFX/Animation/Popup）
2. Camera 不直接 import Cue 域的任何类型——通过 Event 机制解耦（Camera 是 Infra 层，Cue 是 Capabilities 层）
3. Shake 参数映射规则：
   - intensity（强度）：从 CueContext 提取，值域 [0, 20]，超出自动钳位
   - duration（时长）：从 CueContext 提取，值域 [0, 5] 秒，超出自动钳位
4. Cue 的可选性由 Cue 系统管理（cue_domain.md §3.4），Camera 不处理震屏禁用逻辑

---

## 9. 与 GameState 场景的关系

### 9.1 Camera Entity 生命周期

```
GameState::Battle (OnEnter)
    │
    ├── 场景 OnEnter System 调用 CameraPlugin::spawn_camera(commands, default_pose)
    │     ├── 生成 Camera Entity + Camera2d + MainCamera 标记
    │     ├── 设置 CameraState = Idle + TargetPose = CurrentPose = default_pose
    │     └── 返回 Camera Entity ID（供后续查询）
    │
    ├── [可选] 场景 OnEnter System 插入 CameraBounds
    │     ├── 根据 GridMap 尺寸计算边界：min = (0,0), max = (width, height)
    │     └── 使用 commands.entity(camera_id).insert(CameraBounds { min, max })
    │
    ├── ... 场景运行期间 Camera 状态机正常运行 ...
    │
    └── GameState::Battle (OnExit)
          │
          └── 场景 OnExit System 调用 despawn_camera(commands)
                └── commands.entity(camera_id).despawn()
```

### 9.2 生命周期规则

1. Camera Entity 不跨场景存活——每个 GameState 场景在 OnEnter 创建、OnExit 销毁
2. 场景系统负责 spawn/despawn Camera，CameraPlugin 不管理场景生命周期
3. CameraPlugin 提供 `spawn_camera` 和 `despawn_camera` 工厂函数供场景调用
4. CameraBounds 由场景系统在 spawn 后插入，Camera 系统只读取不创建/修改
5. 默认位置（default_pose）由场景系统根据场景特性设置：
   - 战斗场景：地图中心位置
   - 探索场景：玩家单位位置或地图中心
   - 对话场景：对话单位位置

### 9.3 Camera Entity 组件清单

| 组件 | 来源 | 生命周期 | 用途 |
|------|------|---------|------|
| Camera2d (或 Camera3d) | bevy | 场景完整生命周期 | 渲染管线入口 |
| MainCamera (标记组件) | CameraPlugin | 场景完整生命周期 | 外部系统识别主镜头 |
| CameraPose (CurrentPose) | CameraPlugin | 场景完整生命周期 | 当前姿态（插值结果） |
| CameraPose (TargetPose) | CameraPlugin | 场景完整生命周期 | 目标姿态（状态机设置） |
| CameraState | CameraPlugin | 场景完整生命周期 | 状态机当前状态 |
| IdleTimeout | CameraPlugin | FreeMove 持续期间 | 空闲超时计时器 |
| CameraBounds | 场景 OnEnter System | 场景完整生命周期 | 边界约束（可选） |
| CameraShake | Camera 内部 | 震屏持续期间 | 震屏效果状态 |
| Transform / GlobalTransform | bevy | 场景完整生命周期 | 渲染位置（由 Camera 系统写入） |
| OrthographicProjection | bevy | 场景完整生命周期 | 投影参数（由 Camera 系统写入） |

---

## 10. Replay 规则

### 10.1 可录制操作

| 操作 | 录制为 CameraCommand | 过滤规则 |
|------|---------------------|---------|
| 外部 CameraRequest::MoveTo | CameraCommand::MoveTo | 不录制由用户输入产生的内部 MoveTo 请求 |
| 外部 CameraRequest::Follow | CameraCommand::Follow | 录制 |
| 外部 CameraRequest::Unfollow | CameraCommand::Unfollow | 录制 |
| 外部 CameraRequest::SetZoom | CameraCommand::SetZoom | 不录制用户缩放操作产生的请求 |
| 外部 CameraRequest::Shake | CameraCommand::Shake | 录制 |
| 外部 CameraRequest::Reset | CameraCommand::Reset | 录制 |
| 用户输入 CameraUp/Down/Left/Right | 不录制 | 用户输入为表现层交互，不影响业务逻辑确定性 |
| CameraBounds 设置 | 不录制 | 边界由场景系统管理，不在 Replay 流中录制 |

### 10.2 录制格式

```yaml
CameraCommand 录制格式（Replay 流的一部分）：
  frame: u64          # 录制帧号
  command: CameraCommand  # 可录制的镜头命令
```

### 10.3 回放行为

1. 回放模式下，ReplayCameraSystem 从 ReplayFrame 读取 CameraCommand
2. 按帧时间戳逐个 trigger CameraRequest
3. Camera 状态机正常处理这些 Request（与运行时行为一致）
4. CameraBounds 不由 Replay 管理——场景系统在回放时重新设置

### 10.4 Replay 约束

- CameraCommand 必须 derive Serialize/Deserialize（Replay 文件持久化）
- 用户输入不录制（用户交互是表现层行为，不影响业务确定性）
- 震屏使用 SeededRng 确保回放结果与录制一致
- Replay 回放消费为 Phase 3 实现目标，初始实现只预留录制接口

---

## 11. 边界约束规则

### 11.1 CameraBounds 设置规则

| 规则 | 描述 | 依据 |
|------|------|------|
| 设置时机 | 场景 OnEnter System 在 spawn Camera 后设置 | ADR-064 §6 |
| 设置频率 | 仅在场景 OnEnter 时设置一次（当前不支持动态更新） | ADR-064 "当前不做" |
| 设置方式 | 通过 Query Camera Entity 插入 CameraBounds Component | ADR-064 §6 |
| 数据来源 | Map Domain / Terrain Domain 根据 GridMap 尺寸计算 | 业务层负责 |
| 数据格式 | Vec2 min, Vec2 max（世界坐标），Camera 不感知 GridPos | ADR-064 §6 |

### 11.2 边界不存在时的行为

- Camera Entity 没有 CameraBounds Component 时不执行钳位
- 镜头可自由移动到任何位置（无边界世界）

### 11.3 实施方式

当前 CameraBounds 通过 `Query Camera Entity + insert CameraBounds` 设置，不经过 Event 路由。这是 Camera 系统中唯一不通过 CameraRequest 的对外接口，原因是：
- CameraBounds 属于配置性数据注入（场景生命周期设置）
- 不在运行时频繁变化（当前范围一次性设置）
- 不需要经过状态机仲裁

### 11.4 边界钳位规则

- 钳位在 ClampSystem 中执行（PostUpdate，InterpolationSystem 之后）
- 钳位作用域：仅限 `position`，zoom 和 rotation 不受边界约束
- 钳位计算：`position.x = position.x.clamp(bounds.min.x, bounds.max.x)` + `position.y` 同理
- 边界值不合法时的行为（min > max）：警告日志 + 跳过钳位

---

## 12. 对齐校验

### 12.1 与已有架构的对齐

- ✅ Camera 定位在 `infra/camera/`，不依赖 `core/domains/*`，符合 Infra 层定义
- ✅ Event 驱动：所有外部请求通过 `commands.trigger(CameraRequest::...)`，禁止直接修改 Transform
- ✅ 业务解耦：CameraTarget 使用 Vec2/i32/UnitId，不引用 Domain 类型
- ✅ CameraBounds 使用 Vec2，不引用 GridPos/TileMap 等地图类型
- ✅ 四级通信：Cue 层通过 Event -> Camera 消费（Hook > Trigger > Observer > Message 层级兼容）
- ✅ Input 解耦：Input 层产出 InputAction，Camera 在 PreUpdate 中消费
- ✅ Replay 就绪：CameraCommand 序列化接口预留，关键操作可录制
- ✅ 状态机仲裁：Idle/FreeMove/Follow/Focus 职责清晰，消除多系统抢控制权问题
- ✅ 不存在禁止的模块模式（components.rs/systems.rs/utils.rs 等全局文件）
- ✅ 不含业务规则（Formula/Condition/Effect）——Camera 是纯表现层基础设施
- ✅ 符合"定义与实例分离"原则——Camera 不含 Definition 类型
- ✅ 符合"规则与内容分离"原则——Camera 行为由代码规则定义，不含 RON 配置
- ✅ Plugin 注册顺序符合层次要求（Phase 8，Input 之后）

### 12.2 与 ADR-064 决策对应

| ADR-064 决策 | 本文档覆盖节 |
|-------------|------------|
| Camera 定位 Infra 层独立模块 | §1, §3.3 |
| State Machine 四状态 | §2 |
| Event 接口契约 | §6 |
| CameraPose 分离 + 插值管线 | §5.1, §5.2, §5.3, §5.4 |
| CameraBounds 解耦设计 | §11 |
| Replay 兼容 | §10 |
| Input 层对接 | §7 |
| Cue 层对接 | §8 |
| GameState 场景生命周期 | §9 |
| CameraQuery 查询 API | §6.3 |
| 当前不做（Future 标记） | §13 |

---

## 13. 当前范围与 Future

以下功能标记为 Future，不纳入当前领域规则，待对应架构决策后补充：

| 功能 | 说明 | 触发条件 |
|------|------|---------|
| 多摄像机支持（CameraId） | 当前只需要一个主镜头 | MiniMap/Replay 独立相机需要时 |
| 优先级栈 | 当前 Focus 不与其他状态同时发生 | 多个系统同时请求镜头控制时 |
| 脚本驱动序列（CameraSequence） | SRPG 叙事系统尚未实现 Cinematic | Narrative/Cutscene 需要时 |
| 预测聚焦（Predictive Focus） | 精度要求不高，简化实现 | 用户反馈镜头跟随反应慢时 |
| 调试面板 | DevTool 系统尚未实现 | Tools 层成熟时 |
| Focus 状态下排队后续请求 | 当前静默忽略，简化实现 | 多段聚焦动画需要时 |
| 3D/2.5D 旋转 | 当前只有 2D 视角 | 渲染层升级时 |
| CameraBounds 动态更新 | 当前在场景 OnEnter 设置一次 | 地图动态扩展/分区域切换需要时 |
| 震屏独立禁用 | Cue 的可选性由 Cue 系统管理 | 用户设置需要时 |
| Replay 回放消费 | 当前只预留录制接口 | Phase 3 Replay 系统完善时 |

---

## 14. 领域事件

Camera 领域不产生业务事件（Camera 是纯消费层、表现层基础设施）。此处列出 Camera 消费的外部事件和内部处理映射：

| 事件来源 | 事件类型 | Camera 消费行为 | 通信机制 |
|---------|---------|----------------|---------|
| 任何外部系统 | CameraRequest::MoveTo | 设置 TargetPose | trigger + Observer |
| 任何外部系统 | CameraRequest::Follow | 切换到 Follow 状态 | trigger + Observer |
| 任何外部系统 | CameraRequest::Unfollow | 切换到 Idle 状态 | trigger + Observer |
| 任何外部系统 | CameraRequest::SetZoom | 更新 TargetPose.zoom | trigger + Observer |
| Cue 系统 | CueTriggered (CueType::Shake) | 生成 CameraRequest::Shake | trigger + Observer |
| Input 层 | InputAction (CameraUp/Down/etc.) | PreUpdate 消费，生成内部移动指令 | 直接查询 InputActionState |
| 任何外部系统 | CameraRequest::Shake | 创建 CameraShake Component | trigger + Observer |
| 任何外部系统 | CameraRequest::Reset | TargetPose 恢复到默认值 | trigger + Observer |
| 任何外部系统 | CameraRequest::LockInput | 设置 input_locked 标记 | trigger + Observer |
| 任何外部系统 | CameraRequest::UnlockInput | 清除 input_locked 标记 | trigger + Observer |
| 任何外部系统 | CameraRequest::LockInput / UnlockInput | 锁定/解锁用户输入 | trigger + Observer |
| GameState 场景 | OnEnter(GameState::X) | 场景 OnEnter System 调用 spawn_camera | 直接调用工厂函数 |
| GameState 场景 | OnExit(GameState::X) | 场景 OnExit System 调用 despawn_camera | 直接调用工厂函数 |
| GameState 场景 | OnEnter 后 | 场景 OnEnter System 可选插入 CameraBounds | 直接 insert Component |

---

## 15. 占位符说明

以下内容留空，待对应角色补充：

| 占位项 | 负责角色 | 预期位置 | 触发条件 |
|--------|---------|---------|---------|
| Camera 数据 Schema | @data-architect | `docs/04-data/infrastructure/camera_schema.md` | 本 domain 规则定稿后 |
| Camera-UI 交互规则 | @presentation-architect | `docs/06-ui/` 相关 | 本 domain 规则定稿后 |
| Camera Def 配置 | 不适用（Camera 不含可配置内容） | 无 | 无 |

---

## 16. 自检清单

- [x] 所有术语有唯一定义，与项目已有术语一致（Unit, Tile, GameState, InputAction, Cue, Replay）
- [x] 业务规则无"可能"、"也许"等模糊表述
- [x] 已检查 `docs/02-domain/` 下相关文档（cue_domain.md §3.4, §5.2），无冲突
- [x] 已检查 `docs/01-architecture/40-cross-cutting/ADR-064-camera-architecture.md`，完全对齐
- [x] 未涉及代码实现细节（函数名、trait 名等仅作为概念说明）
- [x] 领域模型完整覆盖：状态机转移矩阵、Pose 插值管线、边界钳位、震屏、场景生命周期、Replay
- [x] 所有不变量和约束条件已识别（8 条不变量）
- [x] 禁止事项已明确列出（12 条禁止）
- [x] 四状态状态机（Idle/FreeMove/Follow/Focus）定义清晰，转移矩阵完整
- [x] 每个操作有完整的流程定义（Pose 插值、边界钳位、震屏、Transform 写入、状态转移、空闲超时、场景生命周期）
- [x] Camera 不含业务 Domain 依赖，所有外部接口通过 CameraTarget(Vec2/TilePos/UnitId) 和 CameraBounds(Vec2) 解耦
- [x] 与 Input 层、Cue 层、GameState 层的交互边界已明确定义
- [x] Replay 录制格式（CameraCommand）和规则已定义
- [x] 当前不做（Future 范围）已明确列出（10 项）
