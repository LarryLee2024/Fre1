# docs/09-planning — 规划文档

本目录存放项目执行规划文档。已完成的规划归档至 `done/`。

## 目录结构

```
09-planning/
├── README.md                          # 本文件
│
├── bevy-0.19-migration-v3-aggressive.md     # Bevy 迁移总纲 v3.0（激进：基于实际代码扫描的精准方案）
│
└── done/                              # 已完成的规划
    ├── consolidated-execution-plan.md # 全规划整合 + 骨架域填充路线图（所有 Batch 1-4 + Phase E/F ✅）
    ├── localization-implementation-plan.md # 国际化基础设施实施计划（全部阶段 ✅）
    ├── logging-architecture-plan.md # 日志架构实施计划（全部 Phase 1-5 ✅）
    ├── infrastructure-integration-plan.md # 基础设施接入规划（所有 P0/P1 项 ✅）
    ├── feature-developer-implementation-roadmap.md # 原路线图（Phase A~H）
    ├── Phase-C-D-execution-plan.md    # M1 之前的并行执行计划
    ├── Phase-post-M1-execution-plan.md # M1 之后的执行跟踪
    ├── integration-facade-plan.md     # Tactical Anti-Corruption Layer 计划
    ├── Fre 项目领域文件清单与设计排序分析.md # 30 领域依赖排序
    ├── spell-formulas-refactor-plan.md # 法术公式重构
    ├── testdebt-002-execution-plan.md  # 测试债务修复计划
    └── doc-conflict-evaluation.md     # 文档冲突修复记录
    └── phase-e-game-flow-execution.md  # Phase E 游戏流程集成（全部完成）
```

## 规划文档生命周期

1. **创建** → 文件置于 `09-planning/` 根目录
2. **执行中** → 按规划推进，更新任务状态
3. **完成** → 移动至 `done/`，更新本 README

## 状态说明

### 活跃规划文档（进行中）

| 文档 | 状态 | 说明 |
|------|------|------|
| `bevy-0.19-migration-v3-aggressive.md` | ✅ v3.1 活跃 | 已完成 ~70%：宪法/架构/规则文档全部对齐，Cutscene Delayed 迁移完成，30+ 类型 Reflect，DiagnosticsOverlay 注册。剩余深层 Reflect 递归 + User Settings 等待 0.19 稳定版 |

### 已归档（旧版迁移方案）

旧 v1.0–v2.0 迁移方案（位于 `ignore_this_dir/`）已被 v3.0 取代：
- 实际代码扫描发现大量迁移工作已在 `0.19.0-rc.3` 使用过程中完成
- v3.0 基于当前实际状态，聚焦剩余工作 + 文档对齐
- 旧方案不再维护

### 已完成归档

- `done/` 中的所有文档均已全部完成
- Phase E（游戏流程集成）已于 2026-06-25 完成并归档
- Localization 基础设施实施已于 2026-06-19 完成并归档
- 日志架构实施已于 2026-06-26 完成并归档
