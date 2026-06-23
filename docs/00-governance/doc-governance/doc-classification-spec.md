---
id: DOC-GOV-CLASSIFICATION
title: 文档分级与稳定性规范
status: accepted
stability: stable
layer: governance
related:
  - ai-constitution-complete.md
  - yaml-frontmatter-spec.md
tags:
  - doc-governance
  - classification
---

# 文档分级与稳定性规范

> 原文来源：`docs/99-history/ai_ignore_this_dir/19拆宪法.md` L373–L422, L627–L655
> 锚定总宪法：第一编 1.3 强制等级说明（P0/P1 对齐）

## 一、文档分级（P0–P4）

很多大型项目最后失败不是因为文档少，而是：

```text
规则
建议
经验

混在一起
```

导致没人知道什么必须遵守。

### 五级分级体系

```text
P0 = Constitution
P1 = ADR
P2 = Standard
P3 = Guide
P4 = Notes
```

对应目录结构：

```text
docs/

00-governance/      # P0

adr/                # P1

standards/          # P2

guides/             # P3

notes/              # P4
```

这样 AI 和人都知道：

```text
Constitution > ADR > Standard > Guide
```

### 与总宪法强制等级的对应

总宪法第一编 1.3 节定义的强制等级（P0 铁律 / P1 强制 / P2 建议）适用于**规则条款**；本规范定义的 P0–P4 适用于**文档载体**。两者关系：

| 文档分级 | 承载的规则强制等级 | 示例 |
|---------|------------------|------|
| P0 Constitution | P0 铁律 + P1 强制 | 项目总宪法 |
| P1 ADR | P1 强制（决策一经接受） | ADR-065 Map Pipeline |
| P2 Standard | P1/P2 | ECS 通信规范、YAML 元数据规范 |
| P3 Guide | P2/P3 | 如何新增一个 Ability |
| P4 Notes | P3/P4 | 经验记录、调研笔记 |

---

## 二、Stability Level（稳定性分级）

并非所有规范都一样稳定。建议：

```text
Stable
Evolving
Experimental
```

例如：

```text
Tag System
Stable

BSN
Experimental

Mod API
Evolving
```

这样未来重构时不会误伤。

### 三级稳定性定义

| 级别 | 含义 | 重构风险 | 示例 |
|------|------|---------|------|
| `Stable` | 已固化，破坏性变更需重大版本升级 | 低 | Tag System |
| `Evolving` | 演进中，可能有小幅调整 | 中 | Mod API |
| `Experimental` | 实验性，随时可能重构或移除 | 高 | BSN |

### 与 YAML frontmatter 的关系

稳定性级别写入 frontmatter 的 `stability` 字段，详见 [yaml-frontmatter-spec.md](./yaml-frontmatter-spec.md)。

```yaml
---
id: CAP-EFFECT
status: stable
stability: stable
---
```

> 注意：`status`（文档生命周期状态：accepted/deprecated 等）与 `stability`（内容稳定性）是两个独立维度，不可混淆。
