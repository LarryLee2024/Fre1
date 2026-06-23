---
id: DOC-GOV-RULE-ID
title: Rule ID 编号体系规范
status: accepted
stability: stable
layer: governance
related:
  - ai-constitution-complete.md
  - anti-pattern-library-spec.md
  - rule-retirement-spec.md
tags:
  - doc-governance
  - rule-id
---

# Rule ID 编号体系规范

> 原文来源：`docs/99-history/ai_ignore_this_dir/19拆宪法.md` L423–L467
> 锚定总宪法：第二十一编红线禁止事项总览（规则编号化）

## 问题

很多规则是：

```text
禁止XXX
禁止YYY
```

以后会越来越难引用。

## 方案：全部编号

### 前缀按领域划分

```text
ARCH-001
ARCH-002
ARCH-003

ECS-001
ECS-002

UI-001
UI-002
```

### 前缀命名约定

| 前缀 | 领域 | 对应总宪法编章 |
|------|------|---------------|
| `ARCH` | 架构总则 / 依赖方向 / 模块边界 | 第一、二、七编 |
| `CAP` | Capabilities 能力层 | 第三编 3.2–3.3 |
| `DOM` | Domains 业务域 | 第三编 3.4 |
| `ECS` | ECS 宪法 | 第六编 |
| `SRPG` | SRPG 核心系统专项 | 第八编 |
| `UI` | UI 系统 | 第九编 |
| `DATA` | 数据驱动与存档 | 第十编 |
| `OBS` | 可观测性 | 第十一编 |
| `TEST` | 测试与确定性 | 第十二编 |
| `PERF` | 性能 | 第十四编 |
| `AI` | AI 执行规范 | 第二十编 |
| `L10N` | 国际化 | 第二十二编 |

## 代码中引用

```rust
// ARCH-014
// Domain禁止直接依赖Domain
```

## Review 时引用

```text
违反 ECS-023
违反 UI-008
```

效率极高。

## 与反模式库的关系

反模式库中每条反模式应关联其违反的 Rule ID，详见 [anti-pattern-library-spec.md](./anti-pattern-library-spec.md)。

## 与规则退役的关系

退役流程以 Rule ID 为单位进行追踪，详见 [rule-retirement-spec.md](./rule-retirement-spec.md)。
