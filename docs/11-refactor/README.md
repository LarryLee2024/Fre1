# docs/11-refactor — 技术债扫描与重构

本目录存放技术债扫描报告和重构计划。已完成的扫描归档至 `done/`。

## 目录结构

```
11-refactor/
├── README.md                          # 本文件
└── done/                              # 已完成的扫描报告（共 8 份）
    ├── debt-inventory-2026-06-17.md   # 首次全量扫描（433 warnings baseline）
    ├── tech-debt-batch3-4-2026-06-18.md # Batch 3+4 领域扫描（6 域）
    ├── tech-debt-scan-2026-06-19.md   # ErrorContext 接入审查 + 架构扫描（6 项全 Resolved）
    ├── tech-debt-scan-2026-06-22.md   # 全量扫描（Leak-005 Fixed，Drift/Open 为预留）
    ├── clippy-debt-2026-06-28.md      # Clippy 技术债扫描（722→44 warnings，真实技术债为 0）
    ├── repetitive-patterns-analysis-2026-06-25.md # 重复性代码模式分析（Error thiserror 改造完成）
    ├── error-system-refactoring-2026-06-28.md # 错误处理系统激进重构（9 阶段，P0-P4 优先级）
    └── logging-system-refactoring-2026-06-28.md # 日志/可观测系统激进重构（8 阶段，P0-P3 优先级）
```

## 技术债生命周期

1. **扫描** → refactor-guardian 执行扫描，输出报告至 `11-refactor/` 根目录
2. **修复** → feature-developer 按优先级修复
3. **归档** → 所有债务 Resolved 后，移动至 `done/`

## 当前状态

- **无进行中**: 所有扫描和重构计划已归档
- 错误处理系统重构：全部 9 阶段完成（P0-P4）
- 日志/可观测系统重构：全部 8 阶段完成（P0-P3），吸收了 `14可观测.md` 的核心观点
- 所有已识别的技术债已 Resolved 或标记为已知债务
