---
id: DOC-GOV-ADR-LIFECYCLE
title: ADR 生命周期管理规范
status: accepted
stability: stable
layer: governance
related:
  - ai-constitution-complete.md
  - doc-classification-spec.md
tags:
  - doc-governance
  - adr
---

# ADR 生命周期管理规范

> 原文来源：`docs/99-history/ai_ignore_this_dir/19拆宪法.md` L511–L545
> 锚定总宪法：第十九编 19.3 架构版本管理、19.4 演进流程

## 问题

ADR 以后会爆炸。如果不增加状态管理，几年后会出现：

```text
ADR-022
ADR-041
ADR-065

三个互相矛盾
```

AI 会懵。

## 方案：ADR 状态机

### 五种状态

```text
Proposed
Accepted
Deprecated
Superseded
Rejected
```

### 状态流转

```text
Proposed ──accept──→ Accepted
   │                    │
   │                    ├──deprecate──→ Deprecated
   │                    │
   │                    └──supersede──→ Superseded (by ADR-XXX)
   │
   └──reject──→ Rejected
```

### supersedes 关系链

例如：

```text
ADR-022
Status: Superseded by ADR-065
```

被取代的 ADR 保留原文（不删除），但顶部标注 `Superseded by ADR-XXX`，指向新决策。

## 与总宪法的对接

总宪法第十九编 19.3「架构版本管理」与 19.4「演进流程」定义架构层面的版本演进；本规范定义 ADR 文档本身的生命周期。两者关系：

- 总宪法引用 ADR 时，必须引用 **Accepted** 状态的 ADR
- ADR 被 Superseded 后，总宪法引用需同步更新指向新 ADR
- 总宪法第十九编 19.1「新增模块检查清单」应核对相关 ADR 状态

## frontmatter 示例

```yaml
---
id: ADR-065
title: Map Pipeline
status: accepted
stability: stable
related:
  - ADR-041
tags:
  - map
  - content
  - architecture
supersedes:
  - ADR-041
---
```

> status 字段取值详见 [doc-classification-spec.md](./doc-classification-spec.md) 与 [yaml-frontmatter-spec.md](./yaml-frontmatter-spec.md)。
