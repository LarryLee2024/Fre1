---
name: picking-architecture-refactor-june-2026
description: Picking 系统架构评审与重构图: infra→ui 迁移, Entity→BattleUnitId, 事件流4层管道, 三态分离
metadata:
  type: project
  updated: 2026-06-23
---

# Picking 架构激进重构 (2026-06-23)

**范围**: 全局 Picking/Selection 架构从 ADR-067 升级到 ADR-PICK-000 宪法级 ADR。

## 核心变更
1. **归属迁层**: `src/infra/picking/` → `src/ui/picking/` (Infra → Presentation Layer Input Adapter)
2. **Entity→BattleUnitId**: Selection 存储 `Option<Entity>` → `Option<BattleUnitId>` (领域 ID)
3. **事件流4层管道**: Pointer Event → PickIntent → Domain Event → Selection State Machine
4. **三态分离**: Hovered(帧级) / Focused(会话) / Selected(行动级) 独立 Resource
5. **PickContext 注入**: 不同模式(Normal/Attack/Skill/Move)下同一点击不同行为
6. **Preview/Commit 分离**: Hover=Preview(无副作用), Click=Commit(可触发状态变更)
7. **Camera 合规**: 镜头跟随通过 CameraRequest::Follow, 不再直接写 TargetPose
8. **清理**: 删除 println! 调试, 全局 debug observer 移至 dev feature

**Why**: ADR-067 将 Picking 定位在 Infra 层, 违反运行时三层分类(应为 Presentation Layer)。Selection 存储 Entity 违反 DDD 身份原则。事件流绕过 Domain Event 直接写 Selection 违反四级通信规范。

**How to apply**: 5 份重构文档在 `docs/11-refactor/` 根目录: ADR-PICK-000(宪法), refactor-picking-layer(分层), refactor-event-flow(事件流), refactor-selection(状态), refactor-remove-debug(清理)。执行顺序: ADR-PICK-000 先行 → 按文档顺序逐步骤。

**关联文档**:
- ADR-067 → 被 ADR-PICK-000 部分取代 (技术分析保留, 架构决策更新)
- ADR-055 / docs/06-ui/architecture.md → 对齐验证 (Picking 在 L3 层)
- docs/01-architecture/README.md §2.4 → 运行时三层定义
