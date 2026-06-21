---
name: adr-064-camera-architecture
description: ADR-064 定义了 Camera 系统架构定位、State Machine、事件契约、Pose 数据流设计
metadata:
  type: reference
---

# ADR-064 Camera 系统架构

**文件位置**: `docs/01-architecture/40-cross-cutting/ADR-064-camera-architecture.md`

## 核心决策

1. **定位**: Camera 是 `src/infra/camera/` 下的 Infra 层独立模块，非业务 Domain，非 UI 子模块
2. **模块结构**: foundation/(纯类型) + systems/ + query.rs + plugin.rs，不创建全局 components.rs/resources.rs
3. **State Machine**: Idle ↔ FreeMove ↔ Follow ↔ Focus 四状态，Focus 状态下不接受新请求（初始简化）
4. **事件驱动**: 外部系统通过 `commands.trigger(CameraRequest::...)` 修改镜头，禁止直接操作 Transform/Pose
5. **目标解耦**: CameraTarget 使用 UnitId/TilePos/Vec2，禁止直接存 Entity
6. **Pose 分离**: TargetPose → Interpolation → CurrentPose → Clamp + Shake → TransformWrite
7. **边界解耦**: CameraBounds(Vec2/Vec2) 组件由场景系统设置，Camera 不依赖 Map/Terrain 类型
8. **Replay 就绪**: CameraCommand 定义 + Serialize 接口预留，回放消费 Phase 3 实现
9. **Plugin 注册**: Phase 8, Input 之后, Localization 之前
10. **当前不做**: 多摄像机、优先级栈、脚本序列、预测聚焦、调试面板

## 关联文档

- 领域规则: 待 @domain-designer 补充 (`docs/02-domain/infrastructure/camera_domain.md`)
- 数据 Schema: 待 @data-architect 补充 (`docs/04-data/infrastructure/camera_schema.md`)
- UI 交互: 待 @presentation-architect 补充 (`docs/06-ui/`)
- 规划跟踪: `docs/09-planning/camera-architecture-plan.md`
