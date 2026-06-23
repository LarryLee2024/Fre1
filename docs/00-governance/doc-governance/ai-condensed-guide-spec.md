---
id: DOC-GOV-AI-CONDENSED
title: AI 精简版规范（CLAUDE.md 策略）
status: accepted
stability: stable
layer: governance
related:
  - ai-constitution-complete.md
tags:
  - doc-governance
  - ai
  - claude
---

# AI 精简版规范（CLAUDE.md 策略）

> 原文来源：`docs/99-history/ai_ignore_this_dir/19拆宪法.md` L734–L792
> 锚定总宪法：第二十编 AI 专属执行规范

## 问题

这个是最容易被忽略但收益最大的。不要让 Claude 每次读：

```text
500页宪法
```

## 方案：建立 CLAUDE.md 精简版

### 篇幅控制

```text
CLAUDE.md

只保留：

20页以内
```

### 内容范围

```text
架构铁律

依赖方向

禁止事项

代码风格

生成规则
```

### 四层知识体系

```text
CLAUDE.md
↓
Constitution
↓
ADR
↓
Guides
```

详细内容：

```text
CLAUDE.md          ← AI 入口，≤20 页，只放铁律
    ↓
Constitution       ← 项目总宪法，完整规则
    ↓
ADR                ← 架构决策记录
    ↓
Guides             ← 操作指南
```

## 与总宪法第二十编的对接

总宪法第二十编「AI 专属执行规范」定义 AI 的反模式黑名单、自检清单、权限边界、最高优先级条款。这些是**完整规则**。

CLAUDE.md 是这些规则的**精简入口**：

- CLAUDE.md 只放最核心的铁律（AI 每次必读）
- 完整规则留在总宪法第二十编（按需查阅）
- CLAUDE.md 通过引用指向总宪法对应编章

## 本项目现状

本项目已有 `CLAUDE.md`（项目根目录），包含：

- Build & Test 命令
- Architecture 3-Second Summary
- P0 Red Lines
- P0 Mandatory Rules
- Key References 表
- Agent Delegation 指引

符合本规范"≤20 页、只放铁律、引用详细文档"的原则。

## 收益

对于未来几十万行甚至百万行的 Bevy 项目，AI 精简版带来的长期收益巨大：减少 AI 上下文噪音，提升检索精度，降低每次会话的 token 成本。
