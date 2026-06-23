---
id: DOC-GOV-OVERVIEW
title: 文档治理总览与拆分方法论
status: accepted
stability: stable
layer: governance
related:
  - ai-constitution-complete.md
  - doc-classification-spec.md
  - rule-id-system-spec.md
  - anti-pattern-library-spec.md
  - adr-lifecycle-spec.md
  - glossary-spec.md
  - knowledge-map-spec.md
  - fitness-function-spec.md
  - rule-retirement-spec.md
  - ai-condensed-guide-spec.md
  - yaml-frontmatter-spec.md
tags:
  - doc-governance
  - constitution
  - architecture
---

# 文档治理总览与拆分方法论

> 原文来源：`docs/99-history/ai_ignore_this_dir/19拆宪法.md` L1–L369
> 锚定总宪法：第一编总则、第二编架构体系、第六编 ECS、第九编 UI、第十编数据、第十一编可观测性、第十二编测试

对于接近「项目宪法 + 架构规范 + AI规则库」级别的文档（最终会到 500~1000 页），**单文件不是最佳方案**。

## 核心判断

如果是目标 **50万~100万行 Bevy SRPG** 的项目，应把总宪法压缩到 **20~40页**，剩下全部拆出去。这样 Claude Code、Claude-Mem、Repomix、CodeGraph 检索效率最高，长期维护成本也最低。当文档到达临界点后，再继续往单文件塞内容，后面会越来越难维护。

---

## 第一层：保留一个总宪法（单文件）

保留：

```text
00-project-constitution.md
```

只放：

* 项目目标
* 核心哲学
* P0铁律
* 架构总览
* 依赖方向
* 违反规则处理方式

控制在：

```text
50~100页以内
```

类似：

```text
第一编 总则
第二编 架构总览
第三编 核心铁律
第四编 AI执行规范
```

作用：**项目最高法律**，而不是细节手册。

---

## 第二层：拆成法律体系

类似现实世界：

```text
宪法
↓
民法
↓
行政法
↓
实施细则
```

项目也一样。例如：

```text
docs/
├── 00-governance/
│
├── 01-architecture/
│
├── 02-domain/
│
├── 03-content/
│
├── 04-ui/
│
├── 05-ecs/
│
├── 06-testing/
│
├── 07-modding/
│
└── 08-ai/
```

---

## 第三层：按主题拆

例如 ECS 宪法，不要放在总宪法里。拆成：

```text
05-ecs/
├── ecs-constitution.md
├── communication-rules.md
├── observer-rules.md
├── component-design.md
├── command-layer.md
├── replay-rules.md
└── state-management.md
```

---

## 第四层：Capabilities 单独成册

Tag / Attribute / Modifier / Aggregator / Ability / Trigger ... 已经足够形成一本书。

建议：

```text
02-domain/
└── capabilities/
    ├── tag.md
    ├── attribute.md
    ├── modifier.md
    ├── aggregator.md
    ├── gameplay_context.md
    ├── spec.md
    ├── ability.md
    ├── trigger.md
    ├── condition.md
    ├── targeting.md
    ├── execution.md
    ├── effect.md
    ├── stacking.md
    ├── event.md
    └── cue.md
```

这样 Claude Code 检索时也更精准。

---

## 第五层：Domain 单独成册

例如：

```text
02-domain/
└── domains/
    ├── combat.md
    ├── spell.md
    ├── progression.md
    ├── inventory.md
    ├── quest.md
    ├── narrative.md
    └── ...
```

不要放总宪法。否则：

```text
Combat修改一次

整个宪法都变
```

Git Diff 非常痛苦。

---

## 第六层：ADR 独立

已有 ADR-064 / ADR-065 / ADR-066，不要写进总宪法。改成：

```text
docs/adr/

ADR-001-ddd.md
ADR-002-event-system.md
ADR-003-ui-factory.md
ADR-064-camera.md
ADR-065-map-pipeline.md
ADR-066-screen-spec.md
```

然后总宪法：

```text
Camera 采用 ADR-064
Map 采用 ADR-065
```

一句话即可。否则 ADR内容 + 宪法内容双份维护。

> ADR 生命周期管理详见 [adr-lifecycle-spec.md](./adr-lifecycle-spec.md)

---

## 第七层：AI Rules 独立

这是很多人漏掉的。文档里如果有大量 Claude / GPT / Agent / Code Review 相关规则，建议独立：

```text
08-ai/
├── ai-constitution.md
├── code-review-checklist.md
├── architecture-review-checklist.md
├── generation-rules.md
├── prompt-templates.md
└── forbidden-patterns.md
```

这样 Claude-Mem、Repomix 最容易吃进去。

---

## 第八层：最关键的 Index

大型项目最重要的不是文档，而是：**索引**。

例如：

```text
docs/

README.md
```

内容：

```text
项目知识地图

00-governance/
    项目宪法

01-architecture/
    总体架构

02-domain/
    Gameplay规则

03-content/
    数据驱动

04-ui/
    UI体系

05-ecs/
    ECS规范

06-testing/
    测试体系

07-modding/
    Mod体系

08-ai/
    AI规则
```

Claude Code 最喜欢这种结构。

> 知识地图规范详见 [knowledge-map-spec.md](./knowledge-map-spec.md)

---

## 最终形态建议

不是：

```text
00-project-constitution.md

300页
500页
800页
```

而是：

```text
00-governance/
    project-constitution.md      ← 最高法律

01-architecture/
    architecture-overview.md
    dependency-rules.md
    plugin-boundary.md

02-domain/
    capabilities/
    domains/

03-content/
    content-pipeline.md
    localization.md

04-ui/
    ui-constitution.md
    screen-spec.md
    widget-system.md

05-ecs/
    ecs-constitution.md
    communication.md
    replay.md

06-testing/
    testing-pyramid.md
    invariant-testing.md

07-modding/
    mod-api.md
    mod-loader.md

08-ai/
    ai-constitution.md
    review-checklist.md

adr/
    ADR-001 ~ ADR-999
```

对于目标 **50万~100万行 Bevy SRPG** 的项目，把总宪法压缩到 **20~40页**，剩下全部拆出去。这样 Claude Code、Claude-Mem、Repomix、CodeGraph 检索效率最高，长期维护成本也最低。

---

## 本套规范索引

本目录（`doc-governance/`）下其余规范文件：

| 文件 | 主题 | 原文行号 |
|------|------|---------|
| [doc-classification-spec.md](./doc-classification-spec.md) | 文档分级 P0–P4 + Stability Level | L373–L422, L627–L655 |
| [rule-id-system-spec.md](./rule-id-system-spec.md) | Rule ID 编号体系 | L423–L467 |
| [anti-pattern-library-spec.md](./anti-pattern-library-spec.md) | 反模式库规范 | L468–L510 |
| [adr-lifecycle-spec.md](./adr-lifecycle-spec.md) | ADR 生命周期管理 | L511–L545 |
| [glossary-spec.md](./glossary-spec.md) | 术语表规范 | L546–L595 |
| [knowledge-map-spec.md](./knowledge-map-spec.md) | 知识地图规范 | L596–L626 |
| [fitness-function-spec.md](./fitness-function-spec.md) | 架构适应度函数 | L656–L701 |
| [rule-retirement-spec.md](./rule-retirement-spec.md) | 规则退役机制 | L702–L733 |
| [ai-condensed-guide-spec.md](./ai-condensed-guide-spec.md) | AI 精简版规范 | L734–L792 |
| [yaml-frontmatter-spec.md](./yaml-frontmatter-spec.md) | YAML 元数据规范 | L794–L1103 |

---

## 方法论总结

> 原文来源：`19拆宪法.md` L792

对于未来几十万行甚至百万行的 Bevy 项目，"术语表（Glossary）+ Rule ID + Anti-Pattern库 + Fitness Function + AI精简版"带来的长期收益，实际上比继续细分目录还大。

因为真正让大型知识库失控的，往往不是文件太大，而是：

- **规则无法引用** → 由 [rule-id-system-spec.md](./rule-id-system-spec.md) 解决
- **术语不统一** → 由 [glossary-spec.md](./glossary-spec.md) 解决
- **错误模式没有沉淀** → 由 [anti-pattern-library-spec.md](./anti-pattern-library-spec.md) 解决
- **AI上下文噪音过多** → 由 [ai-condensed-guide-spec.md](./ai-condensed-guide-spec.md) 解决
