# docs/10-reviews — 审查报告

本目录存放代码审查、架构审查、一致性审查等报告。所有报告均已完成，归档于 `done/`。

## 目录结构

```
10-reviews/
├── README.md                          # 本文件
└── done/                              # 所有审查报告（共 28 份）
    ├── comprehensive-review-2026-06-19.md      # 综合审查（2 Critical + 5 High + 4 Medium + 2 Low）
    ├── cross-reference-audit-2026-06-19.md     # 文件引用审计（CRITICAL/HIGH/MEDIUM 全部已修复）
    ├── code-review-batch3-4-2026-06-18.md      # Batch 3+4 代码审查
    ├── code-reviewer-test-rename-2026-06-17.md # 测试命名规范审查
    ├── batch1-progression-inventory-tests-2026-06-18.md # Batch 1 测试审查
    ├── pipeline-combat-code-review-2026-06-20.md # Pipeline-Combat 代码审查
    ├── terrain-registry-test-review.md         # 地形注册表测试审查
    ├── input-review.md                         # 输入系统审查
    ├── pipeline-review.md                      # Pipeline 审查
    ├── tactical-review.md                      # 战术域审查
    ├── replay-bridge-review.md                 # Replay 桥接层审查
    ├── save-bridge-review.md                   # Save 桥接层审查
    ├── test-guardian-audit-2026-06-15.md       # 测试守护审计
    ├── phase-c3-d2-capabilities-review.md      # Phase C3/D2 能力审查
    ├── feature-developer-capabilities-alignment.md  # 能力对齐审查
    ├── feature-developer-code-alignment-overview.md # 代码对齐总览
    ├── feature-developer-domains-alignment.md       # 域对齐审查
    ├── feature-developer-infrastructure-alignment.md # 基础设施对齐审查
    ├── architecture-vs-governance-review.md    # 架构 vs 治理审查
    ├── cross-layer-consistency-review.md       # 跨层一致性审查
    ├── data-vs-governance-review.md            # 数据 vs 治理审查
    ├── domain-vs-governance-review.md          # 域 vs 治理审查
    ├── infrastructure-completeness-review.md   # 基础设施完整性审查
    ├── .qoder-agents-vs-constitution-review.md # Agent vs 宪法审查
    ├── agent-prerequisite-constraints-review.md # Agent 前置约束审查
    ├── qoder-agents-constitution-v5-review.md  # Agent 宪法 v5 审查
    ├── qoder-agents-vs-constitution-review.md  # Agent vs 宪法审查
    └── trae-rules-vs-constitution-review.md    # Trae 规则 vs 宪法审查
```

## 审查文档生命周期

1. **创建** → 文件置于 `10-reviews/` 根目录
2. **执行中** → 按审查清单逐项检查
3. **完成** → 移动至 `done/`，更新本 README

## 状态说明

- `done/` 中的所有审查报告均已全部完成
- 当前无活跃审查
