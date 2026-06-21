# docs/11-refactor — 技术债扫描与重构

本目录存放技术债扫描报告和重构计划。已完成的扫描归档至 `done/`。

## 目录结构

```
11-refactor/
├── README.md                                   # 本文件
└── done/                                       # 已完成的扫描报告（共 11 份）
    ├── communication-debt-2026-06-21.md        # 通信系统技术债（阶段 1 Deferred，阶段 2-5 已完成）
    ├── rust-features-debt-2026-06-21.md        # 21个Rust特性吸收激进重构
    ├── debt-inventory-2026-06-17.md            # 首次全量扫描（433 warnings baseline）
    ├── tech-debt-batch3-4-2026-06-18.md        # Batch 3+4 领域扫描（6 域）
    ├── tech-debt-scan-2026-06-19.md            # ErrorContext 接入审查 + 架构扫描
    ├── tech-debt-scan-2026-06-22.md            # 全量扫描
    ├── clippy-debt-2026-06-28.md               # Clippy 技术债扫描
    ├── repetitive-patterns-analysis-2026-06-25.md # 重复性代码模式分析
    ├── error-system-refactoring-2026-06-28.md  # 错误处理系统激进重构
    ├── logging-system-refactoring-2026-06-28.md # 日志/可观测系统激进重构
    └── 37-principles-implementation-2026-07-01.md # 37条经验吸收激进重构
```

## 技术债生命周期

1. **扫描** → refactor-guardian 执行扫描，输出报告至 `11-refactor/` 根目录
2. **修复** → feature-developer 按优先级修复
3. **归档** → 所有债务 Resolved 后，移动至 `done/`

## 当前状态

- **已完成**: 通信系统技术债全部完成（阶段 1 Event History Deferred，阶段 2-5 Resolved）
- 错误处理系统重构：全部 9 阶段完成（P0-P4）
- 日志/可观测系统重构：全部 8 阶段完成（P0-P3）
- 37条经验吸收：全部 78 项完成（P0-P3）
