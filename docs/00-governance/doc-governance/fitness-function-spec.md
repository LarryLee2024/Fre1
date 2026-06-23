---
id: DOC-GOV-FITNESS-FUNCTION
title: 架构适应度函数（Fitness Function）规范
status: accepted
stability: stable
layer: governance
related:
  - ai-constitution-complete.md
tags:
  - doc-governance
  - fitness-function
  - ci
---

# 架构适应度函数（Fitness Function）规范

> 原文来源：`docs/99-history/ai_ignore_this_dir/19拆宪法.md` L656–L701
> 锚定总宪法：第十九编 19.2 自动化门禁、第二编 2.9 依赖方向总规则

## 定位

这是很多百万行项目才会做的。不要靠 Code Review，让 CI 自动检查。

## 方案：dependency_checker 提升到 P0 基础设施

```text
tools/dependency_checker
```

自动检查：

```text
Domain -> Domain

是否出现

Core -> Infra

是否出现

UI -> Domain

是否出现
```

## 检查项

| 检查项 | 规则 | 对应总宪法 |
|--------|------|-----------|
| Domain → Domain | 禁止 Domain 直接依赖 Domain | 第三编 3.5 边界铁则 |
| Core → Infra | 禁止 Core 依赖 Infra（高层依赖低层，Core 是 L1，Infra 是 L2，方向反了） | 第二编 2.9 依赖方向总规则 |
| UI → Domain | 禁止 UI 直接依赖 Domain（须经 Projection） | 第九编 UI 宪法 |
| Shared ← 任意 | 任意层可依赖 Shared（L0） | 第二编 2.4 |
| App → 任意 | App 可依赖所有层（唯一装配点） | 第二编 2.2 |

> 注：原文 "Core -> Infra 是否出现" 指的是依赖方向违规。按本项目架构，L0 Shared ← L1 Core ← L2 Infrastructure，即 Core 依赖 Infra 是允许的（高层依赖低层）。但若出现 Infra → Core 反向依赖则违规。此处保留原文检查意图：检测非预期依赖方向。

## 与总宪法 19.2 自动化门禁的对接

总宪法第十九编 19.2「自动化门禁」定义 CI 门禁标准；本规范明确 dependency_checker 作为门禁的核心组件。

建议：

```text
dependency_checker 提升到 P0 基础设施级别
```

即：CI 必须运行 dependency_checker，违规直接阻断合并，不允许人工豁免绕过。

## 与 Rule ID 的关系

每条依赖检查规则应关联 Rule ID（见 [rule-id-system-spec.md](./rule-id-system-spec.md)），违规输出格式：

```text
违反 ARCH-014：Domain禁止直接依赖Domain
  src/core/domains/combat/mod.rs → src/core/domains/spell/mod.rs
```
