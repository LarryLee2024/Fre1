---
id: DOC-GOV-YAML-FRONTMATTER
title: YAML Front Matter 元数据规范
status: accepted
stability: stable
layer: governance
related:
  - ai-constitution-complete.md
  - doc-classification-spec.md
tags:
  - doc-governance
  - yaml
  - metadata
---

# YAML Front Matter 元数据规范

> 原文来源：`docs/99-history/ai_ignore_this_dir/19拆宪法.md` L794–L1103
> 锚定总宪法：全局（元数据规范适用于所有编章产出物）

## 总体原则

**不是所有文档都写 YAML Front Matter。**

很多团队最后失败的原因是：

```yaml
---
id: xxx
title: xxx
owner: xxx
status: xxx
created: xxx
updated: xxx
tags: [...]
---
```

然后 500 个文件全都要维护。最后：

```text
元数据比正文还难维护
```

---

## 一、分级处理

### 第一类：必须写 YAML（治理类文档）

```text
00-governance/
adr/
standards/
```

必须有。例如：

```yaml
---
id: ADR-065
title: Map Pipeline
status: accepted
owner: architect
created: 2026-06-22
updated: 2026-06-22
tags:
  - map
  - content
  - architecture
supersedes:
  - ADR-041
---
```

因为需要：

* 搜索
* 索引
* 生命周期管理
* AI检索
* ADR关系图

### 第二类：建议写极简 YAML（Capability / Domain）

```text
capabilities/
domains/
```

不要写一堆。只保留：

```yaml
---
id: DOM-COMBAT
status: stable
---
```

最多：

```yaml
---
id: DOM-COMBAT
title: Combat Domain
status: stable
depends_on:
  - CAP-ATTRIBUTE
  - CAP-EFFECT
---
```

结束。

### 第三类：不要 YAML（Guide / Tutorial / Note / Example）

```text
guides/
tutorials/
notes/
examples/
```

直接正文。例如：

```markdown
# 如何新增一个 Ability
...
```

即可。

---

## 二、对 AI 最有价值的字段

如果只能保留几个字段，推荐：

```yaml
---
id:
title:
status:
depends_on:
---
```

其它基本都可以不要。例如：

```yaml
---
id: CAP-EFFECT
title: Effect Capability
status: stable
depends_on:
  - CAP-TARGETING
  - CAP-EXECUTION
---
```

这已经足够 Claude、CodeGraph、Repomix 使用。

---

## 三、不要写的字段

很多人喜欢：

```yaml
author:
reviewer:
approver:
last_editor:
department:
priority:
severity:
version:
category:
```

对于个人项目：**全部是噪音**。未来只有一个作者，没有价值。

---

## 四、推荐增加的字段

对于架构项目，比 owner 更有价值：

```yaml
---
id: DOM-COMBAT
status: stable

layer: domain

stability: stable

depends_on:
  - CAP-ATTRIBUTE
  - CAP-EFFECT

related:
  - ADR-012
  - ADR-034
---
```

这几个字段能直接构建：

```text
知识图谱
依赖图
架构图
```

### 字段说明

| 字段 | 含义 | 价值 |
|------|------|------|
| `id` | 文档唯一标识 | 检索、引用 |
| `title` | 标题 | 检索 |
| `status` | 文档生命周期状态（accepted/deprecated/under-review） | 生命周期管理 |
| `layer` | 架构层级（governance/architecture/domain/content/ui/ecs/testing/ai） | 架构图 |
| `stability` | 内容稳定性（stable/evolving/experimental） | 重构风险评估 |
| `depends_on` | 依赖的其他文档 id | 依赖图 |
| `related` | 相关文档 id | 知识图谱 |
| `tags` | 主题标签 | 检索 |
| `supersedes` | 取代的 ADR id（仅 ADR） | ADR 关系链 |

> `status` 与 `stability` 是两个独立维度，详见 [doc-classification-spec.md](./doc-classification-spec.md)。

---

## 五、最终模板

### 治理类文档（governance / adr / standards）

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
---
```

### Capability

```yaml
---
id: CAP-EFFECT

status: stable

depends_on:
  - CAP-TARGETING
  - CAP-EXECUTION
---
```

### Domain

```yaml
---
id: DOM-COMBAT

status: stable

depends_on:
  - CAP-EFFECT
  - CAP-ATTRIBUTE
---
```

### Guide

```markdown
# How to Create New Ability
```

不写 YAML。

---

## 六、汇总

对于 **Claude Code + Claude-Mem + Repomix + CodeGraph 驱动的 50万~100万行项目**，控制策略：

```text
治理文档     100% YAML
ADR         100% YAML

Capability  轻量 YAML
Domain      轻量 YAML

Guide       无 YAML
Tutorial    无 YAML
Example     无 YAML
Note        无 YAML
```

这样长期维护成本最低，同时 AI 检索效果最好。
