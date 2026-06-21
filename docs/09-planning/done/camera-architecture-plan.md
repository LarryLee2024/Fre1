---
id: 09-planning.camera-architecture-plan
title: Camera 架构补充规划
status: completed
owner: architect
created: 2026-06-21
updated: 2026-06-21
tags:
  - planning
  - camera
  - architecture
---

# Camera 架构补充规划

> 基于 `docs/99-history/ai_ignore_this_dir/17camera.md` 历史经验 + 当前项目现状分析

---

## 1. 现状评估

| 维度 | 现状 | 差距 |
|------|------|------|
| Camera 代码 | `src/app/scenes/test_battle/render.rs:116` — 7 行 `spawn_camera`，固定位置的 Camera2d | 无模块，无独立组织 |
| 架构文档 | `docs/01-architecture/README.md` L153 提到 "Presentation (UI, VFX, SFX, Camera)" | 只有一句话，无正式定位 |
| 领域规则 | 31 个领域中无 Camera | 缺少领域规则 |
| 数据 Schema | 无 | 缺少 Pose/Command 定义 |
| 宪法 | 无 Camera 条款 | 需补充 |
| Cue 领域 | `cue_domain.md` 提到 `Shake → Infra.presentation.camera/shake` | 但 camera 模块不存在 |
| UI 文档 | 不涉及 Camera | 缺少交互规则 |

### 现有 Camera 代码

```rust
// src/app/scenes/test_battle/render.rs
pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera::default(),
        Transform::from_xyz(240.0, 240.0, 10.0),
        GlobalTransform::default(),
    ));
}
```

无：镜头跟随、WASD/边缘滚动、缩放、聚焦事件、震屏、边界限制、Event 机制。

---

## 2. 历史文档核心建议摘要

`17camera.md` 提出的关键设计教训：

1. **Camera 是独立系统**，不是 Map/UI 的附属功能
2. **Event 驱动** — 所有 Gameplay 通过 `CameraRequest` 修改镜头，禁止直接改 Transform
3. **Camera 不应该知道 Combat/Dialogue/Quest** — 统一接口
4. **CameraPose（Target/Current）分离** — 插值过渡而非直接跳跃
5. **优先级栈** — 避免多系统抢控制权
6. **Replay 兼容** — CameraCommand 可记录
7. **输入解耦** — Input → CameraCommand → Camera
8. **Camera 是 State Machine** — 而非一堆系统集合

---

## 3. 行动方案

### Phase 1 — 架构决策（当前 sprint）

| 步骤 | 负责 Agent | 产出 | 依赖 |
|------|-----------|------|------|
| 1.1 | @architect | ADR: Camera 架构定位+State Machine+事件契约 | 无 |
| 1.2 | @domain-designer | Camera 领域规则文档 | ADR 完成后 |
| 1.3 | @data-architect | Camera 数据 Schema | 领域规则完成后 |
| 1.4 | @presentation-architect | Camera-UI 交互规则 | ADR 完成后 |

### Phase 1 进展
- 1.1 ✅ ADR-064 Camera 架构已完成并写入 docs/01-architecture/40-cross-cutting/ADR-064-camera-architecture.md
- 1.2 ✅ Camera 领域规则已完成并写入 docs/02-domain/infrastructure/camera_domain.md + docs/02-domain/README.md 索引更新
- 1.3 ✅ Camera 数据 Schema 已完成并写入 docs/04-data/infrastructure/camera_schema.md + docs/04-data/README.md 索引更新
- 1.4 ✅ Camera-UI 交互规则已完成并写入 docs/06-ui/04-data-flow/camera-ui-interaction.md + docs/06-ui/README.md 索引更新

### Phase 2 进展
- 2.1 ✅ Camera 基础模块代码已实现：src/infra/camera/（23 个文件，15/15 测试通过）
- 2.2 ✅ 宪法已更新：docs/00-governance/ai-constitution-complete.md（6 处新增 Camera 条款）

| 步骤 | 负责 Agent | 产出 | 依赖 |
|------|-----------|------|------|
| 2.1 | @feature-developer | Camera 基础模块代码 | 全部 Phase 1 产出 |
| 2.2 | — | 更新宪法+索引文档 | Phase 1 产出 |

### Phase 3 — 扩展（有具体需求时）

- 多摄像机支持（CameraId）
- 优先级栈（CameraRequest.priority）
- 脚本驱动序列
- 预测聚焦
- 调试面板

---

## 4. 关键决策点

**Camera 定位**：现有架构已暗示 Camera 属于 `Infra.presentation`，但历史文档建议作为独立 Domain。需要 Architect 裁决。

我倾向 **Infra 层独立模块**（非业务 Domain），理由：
- Camera 不包含业务规则，是技术编排
- 但需要自己的模块边界（不是散落在 ui/ 下的若干系统）
- 内部 State Machine 设计属于架构决策

---

## 5. 需要更新的文档清单

| 文档 | 改动内容 | 时机 |
|------|---------|------|
| `docs/01-architecture/README.md` | 明确 Camera 架构定位，添加 ADR 引用 | Phase 1.1 |
| `docs/01-architecture/40-cross-cutting/ADR-???-camera.md` | 新建 ADR | Phase 1.1 |
| `docs/02-domain/README.md` | 索引中添加 Camera | Phase 1.2 |
| `docs/02-domain/capabilities/camera_domain.md` | 新建 Camera 领域规则 | Phase 1.2 |
| `docs/04-data/README.md` | 索引中添加 Camera Schema | Phase 1.3 |
| `docs/04-data/infrastructure/camera_schema.md` | 新建 Camera Schema | Phase 1.3 |
| `docs/06-ui/README.md` | 补充 Camera/UI 交互规则 | Phase 1.4 |
| `docs/00-governance/ai-constitution-complete.md` | 补充 Camera 架构条款 | Phase 2 |
| `docs/09-planning/camera-architecture-plan.md` | 本文件，完成后归档 | 全部完成后 |

---

## 6. 不做的范围（Phase 3 再考虑）

- 多摄像机系统（MiniMap/Replay/Debug 独立相机）
- 脚本驱动序列（Script-driven CameraSequence）
- 预测聚焦（Predictive Focus）
- Camera 调试面板
- 3D/2.5D 切换
