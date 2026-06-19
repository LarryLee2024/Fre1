# docs/09-planning — 规划文档

本目录存放项目执行规划文档。已完成的规划归档至 `done/`。

## 目录结构

```
09-planning/
├── README.md                          # 本文件
│
├── bevy-0.19-migration-plan-overview.md     # Bevy 迁移总纲 v2.0（激进：5周+4-6 Agent并行）
├── bevy-0.19-migration-compatible.md        # Phase A：核心系统并行重写（Observer/Delayed 全面接管）
├── bevy-0.19-migration-features.md          # Phase B+C：架构现代化+收尾（BSN/Resource→Entity/Relationship）
├── bevy-0.19-migration-future.md            # Post-Migration：迁移完成后的架构展望与维护规则
├── bevy-0.19-migration-domain-checklist.md  # 各领域迁移检查清单（134 项，激进版）
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
| `bevy-0.19-migration-plan-overview.md` | 🔴 激进 v2.0 | 全面采用 0.19 ECS 模型，3 阶段并行，4–6 Agent，5 周 |
| `bevy-0.19-migration-compatible.md` | 🔴 激进 v2.0 | Phase A：Observer/Delayed 全面接管，~210 文件并行重写 |
| `bevy-0.19-migration-features.md` | 🔴 激进 v2.0 | Phase B+C：BSN/Resource→Entity/Relationship/性能 |
| `bevy-0.19-migration-future.md` | 🔴 激进 v2.0 | 迁移完成后的架构展望 + 5 条宪法级新规则 |
| `bevy-0.19-migration-domain-checklist.md` | 🔴 激进 v2.0 | 134 项检查清单，4 阶段分区 |

### 已完成归档

- `done/` 中的所有文档均已全部完成
- Phase E（游戏流程集成）已于 2026-06-25 完成并归档
- Localization 基础设施实施已于 2026-06-19 完成并归档
- 日志架构实施已于 2026-06-26 完成并归档
