# docs/09-planning — 规划文档

本目录存放项目执行规划文档。已完成的规划归档至 `done/`。

## 目录结构

```
09-planning/
├── README.md                                     # 本文件
└── done/                                         # 全部已归档（无活跃规划）
```

## 活跃规划

| 文档 | 创建日期 | 负责人 | 状态 |
|------|----------|--------|------|
| `ui-domain-integration-plan.md` | 2026-06-21 | feature-developer | 🟡 进行中 |

## 规划文档生命周期

1. **创建** → 文件置于 `09-planning/` 根目录
2. **执行中** → 按规划推进，更新任务状态
3. **完成** → 移动至 `done/`，更新本 README

---

## 已完成归档

| 文档 | 完成日期 | 说明 |
|------|----------|------|
| `ui-development-plan.md` | 2026-06-21 | UI 层开发实施计划（6 阶段全部完成） |

### Bevy 0.19 迁移总纲

`bevy-0.19-migration-v3-aggressive.md` — v3.3 ✅ 最终状态：

| 维度 | 状态 |
|------|------|
| 宪法 v5.2 | ✅ 引擎版本/通信机制/ECS 规则全部对齐 |
| 架构 ADR-002 v2 | ✅ 通信优先级更新 (Observer 优先) |
| 测试规范 | ✅ Observer/Delayed 测试规范已添加 |
| 文档对齐 | ✅ 宪法/架构/领域/数据/测试/规则 全部对齐 0.19 |
| cargo check + nextest | ✅ 全绿（1530/1530） |
| cargo clippy | ✅ 30 warnings（全部为设计模式，非债务） |
| 阻塞项（外部） | 📦 User Settings / Reflect 深层递归 / BSN / Relationship / 冒烟测试 — 等待 0.19 稳定版 |

### 旧版迁移方案（9 个）

| 文档 | 原版本 | 说明 |
|------|--------|------|
| `bevy-0.19-migration-plan-overview.md` | v2.0 | 被 v3.x 取代 |
| `bevy-0.19-migration-compatible.md` | v1.0 Phase A | 被 v3.x 取代 |
| `bevy-0.19-migration-domain-checklist.md` | v1.0 | 被 v3.x 取代 |
| `bevy-0.19-migration-features.md` | v1.0 Phase B+C | 被 v3.x 取代 |
| `bevy-0.19-migration-future.md` | 架构展望 | 内容已并入宪法 v5.2 |
| `new_bevy-0.19-migration-master-plan.md` | v3.0 总纲 | 被 v3.x 取代 |
| `new_bevy-0.19-module-checklist.md` | v3.0 | 被 v3.x 取代 |
| `new_bevy-0.19-phase1-aggressive.md` | v3.0 Phase 1 | 被 v3.x 吸收 |
| `new_bevy-0.19-phase2-deep-refactor.md` | v3.0 Phase 2 | 被 v3.x 吸收 |

### Bevy 0.19 迁移技术知识库（11 篇）

文档 `docs/03-technical/bevy-0.19-migration/00-*.md` → `10-srpg-architecture-impact.md`，迁移完成后归档。

### 其他已归档规划

| 文档 | 完成日期 | 说明 |
|------|----------|------|
| `localization-implementation-plan.md` | 2026-06-19 | Localization 基础设施实施 |
| `phase-e-game-flow-execution.md` | 2026-06-25 | Phase E 游戏流程集成 |
| `logging-architecture-plan.md` | 2026-06-26 | 日志架构实施 |
| `consolidated-execution-plan.md` | — | 综合执行计划 |
| `doc-conflict-evaluation.md` | — | 文档冲突评估 |
| `feature-developer-implementation-roadmap.md` | — | 功能实现路线图 |
| `Fre 项目领域文件清单与设计排序分析.md` | — | 领域文件分析 |
| `infrastructure-integration-plan.md` | — | 基础设施集成 |
| `integration-facade-plan.md` | — | Integration Facade 计划 |
| `Phase-C-D-execution-plan.md` | — | Phase C-D 执行 |
| `Phase-post-M1-execution-plan.md` | — | M1 后执行计划 |
| `spell-formulas-refactor-plan.md` | — | 法术公式重构 |
| `testdebt-002-execution-plan.md` | — | 测试债务处理 |
