---
name: camera-schema
description: Camera 镜头系统的完整数据架构设计，包括 CameraPose、CameraState、CameraRequest、CameraCommand 及 Transient 类型的四层分离
metadata:
  type: reference
---

# Camera Schema Design (2026-06-21)

**File**: `/Users/lf380/Code/Bevy/Fre/docs/04-data/infrastructure/camera_schema.md`

## Key Design Decisions

### Layer Classification
Camera 无 Definition 层——没有可配置资产。所有类型分布在 Instance / Transient / Persistence 三层：

| Layer | Types |
|-------|-------|
| Instance (ECS Component) | CameraPose (TargetPose+CurrentPose), CameraState, CameraBounds, CameraShake, CameraInputBlock, IdleTimeout |
| Transient | CameraTarget (值嵌入), CameraRequest (Event), CameraStateVm (UI投影) |
| Persistence | CameraCommand (嵌入 ReplayFrame) |

### Cross-layer Value Pattern
CameraTarget 是一种"跨层值类型"——不自立为 Component/Resource，而是嵌入 CameraRequest (Transient)、CameraState (Instance)、CameraCommand (Persistence)。这种模式避免了值类型的层归属争议。

### Replay Boundary
- 用户输入 (WASD/Zoom) 不录制——表现层交互不影响业务确定性
- CameraBounds 不录制——场景 OnEnter 时重新设置
- LockInput/UnlockInput 不录制——回放时由状态机隐式管理
- 震屏使用 SeededRng，种子从 ReplayContext 分配

### Data Law Compliance
DL001 (Def-Instance Separation): Camera 无 Def，所有 Instance 类型不承担存档职责
DL008 (Stacking): CameraInputBlock.block_count 是输入堆叠计数，非游戏机制堆叠
DL009 (Cue): Camera 震屏消费 Cue::Shake 事件，不直接响应 Effect
DL010 (Replay Priority): CameraCommand serialize/deserialize 就绪，SeededRng 震屏
DL012 (Domain Isolation): Camera 不引用任何 core::domains::* 类型，通过 CameraTarget(Vec2/i32/u64) 解耦

### State Transition Matrix
Documented in schema §3.5. Key invariants: Focus 状态忽略所有外部请求（初始实现），Focus 时输入锁定，FreeMove 2 秒超时回 Idle，用户输入始终覆盖 Follow。
