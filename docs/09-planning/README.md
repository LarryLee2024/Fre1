# docs/09-planning — 规划文档

本目录存放项目执行规划文档。已完成的规划归档至 `done/`。

## 目录结构

```
09-planning/
├── README.md                          # 本文件
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

- `done/` 中的所有文档均已全部完成
- 所有活跃规划文档已全部完成并归档
- Phase E（游戏流程集成）已于 2026-06-25 完成并归档
- Localization 基础设施实施已于 2026-06-19 完成并归档
- 日志架构实施已于 2026-06-26 完成并归档
