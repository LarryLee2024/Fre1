---
id: DOC-GOV-GLOSSARY
title: 术语表（Vocabulary）规范
status: accepted
stability: stable
layer: governance
related:
  - ai-constitution-complete.md
tags:
  - doc-governance
  - glossary
  - vocabulary
---

# 术语表（Vocabulary）规范

> 原文来源：`docs/99-history/ai_ignore_this_dir/19拆宪法.md` L546–L595
> 锚定总宪法：全局术语统一

## 问题

这是超大型项目特别容易忽视的。例如项目中：

```text
Ability
Skill
Spell

Effect
Modifier

Tag
Trait

Command
Request

Event
Message
```

以后一定有人混用。

## 方案：建立 Vocabulary（术语表）

单独目录：

```text
docs/glossary/
```

## 术语定义格式

每个术语条目应包含：

- 术语名（中英文）
- 定义
- 与相关术语的关系（包含、同义、区分）

例如：

```text
Spell
=
法术

Ability
=
所有可执行能力

Spell ∈ Ability
```

## 易混术语对照（初始清单）

以下五组术语必须在本项目中有明确定义，禁止混用：

| 术语组 | 区分要点 |
|--------|---------|
| Ability / Skill / Spell | Ability 是所有可执行能力的总称；Spell ∈ Ability（法术是能力的子集）；Skill 视语境另定 |
| Effect / Modifier | Effect 是效果的抽象定义；Modifier 是属性修改的具体操作（Add/Mul/Set） |
| Tag / Trait | Tag 是无数据标签；Trait 是带数据的特征定义 |
| Command / Request | Command 是命令层指令（可回放）；Request 是 UI 发起的请求 |
| Event / Message | Event 是领域事件（Observer 体系）；Message 是跨系统消息 |

## 收益

AI 检索准确率会暴涨——术语统一后，Claude/CodeGraph 能精准定位概念，而非在多个近义词间猜测。
