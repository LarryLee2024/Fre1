# docs/11-refactor — 技术债扫描与重构

本目录存放技术债扫描报告和重构计划。已完成的扫描归档至 `done/`。

## 目录结构

```
11-refactor/
├── README.md                          # 本文件
└── done/                              # 已完成的扫描报告（共 6 份）
    ├── debt-inventory-2026-06-17.md   # 首次全量扫描（433 warnings baseline）
    ├── tech-debt-batch3-4-2026-06-18.md # Batch 3+4 领域扫描（6 域）
    ├── tech-debt-scan-2026-06-19.md   # ErrorContext 接入审查 + 架构扫描（6 项全 Resolved）
    ├── tech-debt-scan-2026-06-22.md   # 全量扫描（Leak-005 Fixed，Drift/Open 为预留）
    ├── clippy-debt-2026-06-28.md      # Clippy 技术债扫描（722→44 warnings，真实技术债为 0）
    └── repetitive-patterns-analysis-2026-06-25.md # 重复性代码模式分析（Error thiserror 改造完成）
```

## 技术债生命周期

1. **扫描** → refactor-guardian 执行扫描，输出报告至 `11-refactor/` 根目录
2. **修复** → feature-developer 按优先级修复
3. **归档** → 所有债务 Resolved 后，移动至 `done/`

## 当前状态

- 所有已识别的技术债已 Resolved 或标记为预留
- Clippy 技术债：真实技术债为 0，剩余 44 warnings 全部为架构设计模式
- 重复性代码模式：Error thiserror 改造、Mod.rs 可见性统一、测试 Fixtures 提取均已完成
- 下次扫描建议间隔：新增 2+ 域或重大架构变更后
