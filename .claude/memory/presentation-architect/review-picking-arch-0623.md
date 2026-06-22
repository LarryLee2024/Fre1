---
name: review-picking-arch-0623
description: ADR-PICK-000 picking directory structure review — 5 violations found, corrected structure applied to architecture.md
metadata:
  type: reference
---

## ADR-PICK-000 目录结构审查（2026-06-23）

### 发现的问题
1. `ui/picking/selection/` 自相矛盾 — ADR 明确说 Selection 与 Picking 分离，却把 selection/ 放在 picking/ 内部
2. `ui/picking/battle/` 领域碎片化 — 与 `ui/screens/battle/` 职责重叠，违反 Cohesion
3. `ui/picking/selection/focus_manager.rs` 与已有 `ui/focus/` 模块冲突
4. `ui/picking/domain_bridge/` 将跨层桥梁埋入子模块
5. `ui/picking/selection/highlight_system.rs` 无定义数据源，可能绕过 Projection

### 处理
- 已更新 `docs/06-ui/01-architecture/architecture.md`：
  - Section 3: picking/ + selection/ 加入目录结构树
  - New Section 3.7: Picking/Selection 子层架构
  - Section 4.2: 添加 Picking 作为 UiIntent 补充的输入路径说明
  - Section 5.1: 通信表新增 4 条 picking/selection 条目
  - Section 8.2: 插件注册顺序新增 PickingUiPlugin + SelectionPlugin
- Projections 列表新增 `selection.rs`
- ViewModels 列表新增 `selection.rs`

### 核心变更
- `picking/` 职责收缩到只产生 PickIntent
- `selection/` 提到 picking/ 同级（包含 bridge, state, pick_context）
- BattleScreen 负责 PickContext 设置
- SelectionState → SelectionProjection → SelectionVm → Dirty 管道强制

### 相关文档
- `docs/01-architecture/40-cross-cutting/ADR-PICK-000-picking-architecture-constitution.md` — 需更新目录结构
