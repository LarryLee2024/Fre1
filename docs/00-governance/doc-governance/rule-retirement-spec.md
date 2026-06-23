---
id: DOC-GOV-RULE-RETIREMENT
title: 规则退役（Rule Retirement）机制
status: accepted
stability: stable
layer: governance
related:
  - ai-constitution-complete.md
  - rule-id-system-spec.md
tags:
  - doc-governance
  - retirement
  - maintenance
---

# 规则退役（Rule Retirement）机制

> 原文来源：`docs/99-history/ai_ignore_this_dir/19拆宪法.md` L702–L733
> 锚定总宪法：第十七编长期维护与运营

## 问题

大部分文档只规定怎么加，不规定怎么删。否则文档会无限膨胀。

## 方案：Rule Retirement（规则退役）

### 三阶段退役流程

以 Rule ID 为单位追踪（见 [rule-id-system-spec.md](./rule-id-system-spec.md)）：

```text
连续12个月无人引用

进入审查

连续18个月无人引用

标记废弃

连续24个月无人引用

允许删除
```

### 阶段定义

| 阶段 | 触发条件 | 动作 | 状态标记 |
|------|---------|------|---------|
| 审查期 | 连续 12 个月无人引用 | 进入人工/自动审查，确认是否仍需要 | `status: under-review` |
| 废弃期 | 连续 18 个月无人引用 | 标记为 Deprecated，保留原文但提示不再适用 | `status: deprecated` |
| 可删除 | 连续 24 个月无人引用 | 允许从文档库删除（Git 历史保留） | 删除 |

### "引用"的判定

引用计数来源：

- 代码注释中的 Rule ID 引用（如 `// ARCH-014`）
- 其他文档中的 Rule ID 引用
- ADR 中的 related/supersedes 字段
- Review 记录中的违规引用

## 与总宪法第十七编的对接

总宪法第十七编「长期维护与运营」定义核心维护原则；本规范是其"退出机制"的细化。没有退役机制的文档体系会无限膨胀，最终压垮维护能力。

## 与 ADR 生命周期的区别

- **ADR 生命周期**（[adr-lifecycle-spec.md](./adr-lifecycle-spec.md)）：管理**架构决策**的状态流转
- **规则退役**：管理**规则条款**的存废

ADR 被 Superseded 是主动决策；规则退役是被动触发（无人引用）。
