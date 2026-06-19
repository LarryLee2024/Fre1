# docs/11-refactor — 技术债扫描与重构

本目录存放技术债扫描报告和重构计划。已完成的扫描归档至 `done/`。

## 目录结构

```
11-refactor/
├── README.md                          # 本文件
└── done/                              # 已完成的扫描报告
    ├── debt-inventory-2026-06-17.md   # 首次全量扫描（433 warnings baseline）
    ├── tech-debt-batch3-4-2026-06-18.md # Batch 3+4 领域扫描（6 域）
    ├── tech-debt-scan-2026-06-19.md   # ErrorContext 接入审查 + 架构扫描（6 项全 Resolved）
    └── tech-debt-scan-2026-06-22.md   # 全量扫描（Leak-005 Fixed，Drift/Open 为预留）
```

## 技术债生命周期

1. **扫描** → refactor-guardian 执行扫描，输出报告至 `11-refactor/` 根目录
2. **修复** → feature-developer 按优先级修复
3. **归档** → 所有债务 Resolved 后，移动至 `done/`

## 当前状态

- 所有已识别的技术债已 Resolved 或标记为预留（dead code ~30+ items，按 ADR-045 §6.2 视为预留非债务）
- 重复性代码模式分析已完成（2026-06-25），识别出 Error thiserror 改造、Mod.rs 可见性统一、测试 Fixtures 提取三项待执行
- 下次扫描建议间隔：新增 2+ 域或重大架构变更后
