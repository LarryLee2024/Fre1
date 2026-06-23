---
id: DOC-GOV-ANTI-PATTERN
title: 反模式库规范
status: accepted
stability: stable
layer: governance
related:
  - ai-constitution-complete.md
  - rule-id-system-spec.md
tags:
  - doc-governance
  - anti-pattern
---

# 反模式库规范

> 原文来源：`docs/99-history/ai_ignore_this_dir/19拆宪法.md` L468–L510
> 锚定总宪法：第二十编 20.1 AI 反模式黑名单

## 问题

很多团队只有：

```text
正确写法
```

没有：

```text
错误写法
```

实际上 AI 最容易学的是反例。

## 方案：建立 Anti-Pattern 库

```text
anti-patterns/

entity-god-object.md
event-storm.md
service-locator.md
shared-garbage-bin.md
mega-plugin.md
mega-resource.md
```

## 每篇结构（三段式）

每篇反模式必须包含：

```text
问题

为什么错

正确替代方案
```

Claude 特别喜欢这种格式。

## 与总宪法反模式黑名单的关系

总宪法第二十编 20.1「AI 反模式黑名单」列出**生成前必须对照检查**的反模式清单；本反模式库是这些黑名单的**详细展开**，提供每条反模式的问题分析、根因和替代方案。

## 与 Rule ID 的关系

每条反模式应标注其违反的 Rule ID（见 [rule-id-system-spec.md](./rule-id-system-spec.md)），例如：

```markdown
# Entity God Object

违反：ARCH-007（实体职责单一原则）

## 问题
...

## 为什么错
...

## 正确替代方案
...
```

## 建议初始反模式清单

| 反模式文件 | 含义 | 关联 Rule ID |
|-----------|------|-------------|
| `entity-god-object.md` | 实体上帝对象，单实体承载过多组件/职责 | ARCH-007 |
| `event-storm.md` | 事件风暴，滥用 Event 导致耦合 | ECS-通信规范 |
| `service-locator.md` | 服务定位器反模式 | ARCH-依赖注入 |
| `shared-garbage-bin.md` | 共享垃圾箱，Resource 承载过多异质数据 | ECS-Resource 设计 |
| `mega-plugin.md` | 巨型 Plugin，单 Plugin 包揽过多系统 | 第七编 Plugin 边界 |
| `mega-resource.md` | 巨型 Resource，单 Resource 聚合过多状态 | ECS-Resource 设计 |
