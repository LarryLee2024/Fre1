---
id: DOC-GOV-KNOWLEDGE-MAP
title: 知识地图（Knowledge Map）规范
status: accepted
stability: stable
layer: governance
related:
  - ai-constitution-complete.md
  - doc-governance-overview.md
tags:
  - doc-governance
  - knowledge-map
---

# 知识地图（Knowledge Map）规范

> 原文来源：`docs/99-history/ai_ignore_this_dir/19拆宪法.md` L596–L626
> 锚定总宪法：全局导航

## 问题

不要让文档互相引用成蜘蛛网。

## 方案：建立 Knowledge Map

```text
docs/knowledge-map.md
```

## 结构：按场景的阅读路径

知识地图按"场景"组织阅读路径，而非按目录罗列。

例如：

```text
新技能系统

先读：
01 architecture

再读：
ability
effect
trigger

最后读：
combat
```

新人和 AI 都能快速定位。

## 与总宪法的关系

总宪法是"规则本体"，知识地图是"导航入口"。知识地图不重复规则内容，只指明：

- 遇到某类任务，先读什么
- 再读什么
- 最后读什么

## 与本套规范的关系

本套文档治理规范本身也应有阅读路径：

```text
新人文档治理入门

先读：
doc-governance-overview.md（总览八层方法论）

再读：
doc-classification-spec.md（理解 P0–P4 分级）
yaml-frontmatter-spec.md（理解元数据规范）

最后读：
rule-id-system-spec.md（编号体系）
adr-lifecycle-spec.md（ADR 生命周期）
```

## 与 Index 的区别

- **Index**（第八层）：目录式罗列，回答"有什么"
- **Knowledge Map**：路径式引导，回答"按什么顺序读"

两者互补，缺一不可。
