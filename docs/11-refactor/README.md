# docs/11-refactor — 技术债扫描与重构

本目录存放技术债扫描报告和重构计划。已完成的扫描归档至 `done/`。

## 目录结构

```
11-refactor/
├── README.md                                   # 本文件
├── content-compatibility-report.md             # Content 架构与 Screen Spec 兼容性报告
├── overlap-review-report.md                    # Shared/Infra vs Core 重复评审报告
├── schema-compatibility-report.md              # Data Schema 与 Screen Spec 兼容性报告
├── ui-doc-gaps.md                              # UI 文档债务扫描报告
└── done/                                       # 已完成的扫描报告（共 19 份）
    ├── comment-convention-debt-2026-06-21.md    # 注释规范技术债
    ├── communication-debt-2026-06-21.md        # 通信系统技术债
    ├── macro-governance-debt-2026-06-21.md     # 宏治理激进重构
    ├── rng-debt-2026-06-21.md                  # RNG 系统技术债
    ├── rust-features-debt-2026-06-21.md        # 21个Rust特性吸收激进重构
    ├── shared-infra-gaps-2026-06-21.md         # Shared/Infra 层差距分析
    ├── debt-inventory-2026-06-17.md            # 首次全量扫描（433 warnings baseline）
    ├── tech-debt-batch3-4-2026-06-18.md        # Batch 3+4 领域扫描（6 域）
    ├── tech-debt-scan-2026-06-19.md            # ErrorContext 接入审查 + 架构扫描
    ├── tech-debt-scan-2026-06-22.md            # 全量扫描
    ├── clippy-debt-2026-06-28.md               # Clippy 技术债扫描
    ├── repetitive-patterns-analysis-2026-06-25.md # 重复性代码模式分析
    ├── error-system-refactoring-2026-06-28.md  # 错误处理系统激进重构
    ├── logging-system-refactoring-2026-06-28.md # 日志/可观测系统激进重构
    ├── 37-principles-implementation-2026-07-01.md # 37条经验吸收激进重构
    ├── explain-research-report.md              # 解释器研究报告
    ├── id-system-refactoring-2026-06-20.md     # ID 系统重构计划
    ├── localization-debt-2026-06-21.md         # Localization 技术债
    └── ui-screen-spec-execution-plan.md        # UI Screen Specification 重构执行计划 v3.0
```

## 技术债生命周期

1. **扫描** → refactor-guardian 执行扫描，输出报告至 `11-refactor/` 根目录
2. **修复** → feature-developer 按优先级修复
3. **归档** → 所有债务 Resolved 后，移动至 `done/`

## 当前状态

- **总扫描次数**: 23 次（19 份已完成 + 4 份活跃扫描）
- **已完成**（`done/`）: 19 份扫描报告归档
- **活跃扫描**（根目录）:
  - `content-compatibility-report.md` — active — Content 架构与 Screen Spec 兼容性分析（95% 兼容，P0 widget-id-map def_registry 列待补充）
  - `overlap-review-report.md` — Shared/Infra vs Core 模块重复评审
  - `schema-compatibility-report.md` — partial — Data Schema 与 Screen Spec 兼容性分析（P0 BuffSlot 已修复，schema 文档协调待完成）
  - `overlap-review-report.md` — active — Shared/Infra vs Core 重复评审（0 处重复，3 处越界，P1 业务常量待迁移）
  - `ui-doc-gaps.md` — active — UI 文档债务扫描（21 项，4 P0 中 3 已修复，DoD 勾选待完善）

### 关键完成项

| 扫描 | 状态 | 详情 |
|------|------|------|
| 通信系统技术债 | ✅ 已完成 | 阶段 1 Event History Deferred，阶段 2-5 Resolved |
| 宏治理激进重构 | ✅ 已完成 | 全部 4 阶段完成 |
| RNG 系统 | ✅ 已完成 | core→infra 依赖已修复 |
| Rust 特性吸收 | ✅ 已完成 | Phase 0+D 完成（Phase C1-C7 代码改造 Deferred） |
| Shared/Infra 差距分析 | ✅ 已完成 | 全部 10 个 Phase 完成（1791 测试通过） |
| 错误处理系统重构 | ✅ 已完成 | 全部 9 阶段完成（P0-P4） |
| 日志/可观测系统重构 | ✅ 已完成 | 全部 8 阶段完成（P0-P3） |
| 37条经验吸收 | ✅ 已完成 | 全部 78 项完成（P0-P3） |
| 注释规范技术债 | ✅ 已完成 | 废话注释 82→0，公开 API 缺文档 884→0，Trait 存在理由 14 个补充等 |
| UI Screen Spec 重构 | ✅ 已完成 | ADR-066 批准，6/6 Screen Spec 完成，4 reference 文件，宪法修订 |
