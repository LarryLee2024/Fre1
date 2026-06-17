---
id: 02-domain.GAS.README
title: GAS Domain Rules
status: proposed
owner: domain-designer
created: 2026-06-16
updated: 2026-06-16
tags:
  - domain
  - GAS
---

# GAS 领域规则

Version: 1.0

SRPG-GAS（Gameplay Ability System）是项目的战斗核心领域，管理技能释放、效果执行、属性计算、Buff 管理等全部战斗逻辑。本目录包含 14 个子领域的详细规则文档和 1 个全景概览文档。

---

## 文档索引

### 全景文档

| 文档 | 说明 |
|------|------|
| `GAS_domain_overview.md` | **GAS 领域全景**：统一术语、架构图、数据流、业务流程、实现路径、依赖矩阵、宪法合规摘要 |

### 子领域文档

| # | 文档 | 领域 | 版本 | 状态 |
|---|------|------|------|------|
| 1 | `ability/ability-rules.md` | Ability 能力管理 | v1.0 | Proposed |
| 2 | `attribute-modifier/attribute-modifier-rules.md` | AttributeModifier 属性修饰管线 | v1.1 | Proposed |
| 3 | `condition/condition-rules.md` | Condition 条件系统 | v1.0 | Proposed |
| 4 | `cost/cost-rules.md` | Cost 消耗系统 | v1.0 | Proposed |
| 5 | `cue/cue-rules.md` | Cue 表现事件总线 | v1.0 | Proposed |
| 6 | `duration/duration-rules.md` | Duration 持续策略 | v1.0 | Proposed |
| 7 | `effect/effect-rules.md` | Effect 效果管线 | v1.1 | Proposed |
| 8 | `execution/execution-rules.md` | Execution 执行算式 | v1.0 | Proposed |
| 9 | `formula/formula-rules.md` | Formula 公式系统 | v1.0 | Proposed |
| 10 | `requirement/requirement-rules.md` | Requirement 释放前提 | v1.0 | Proposed |
| 11 | `stack-policy/stack-policy-rules.md` | StackPolicy 叠层策略 | v2.0 | **Complete** ✅ |
| 12 | `tag/tag-rules.md` | Tag 标签系统 | v1.0 | Proposed |
| 13 | `targeting/targeting-rules.md` | Targeting 目标选择 | v1.0 | Proposed |
| 14 | `trigger/trigger-rules.md` | Trigger 触发器系统 | v1.1 | Proposed |

### 架构文档

| 文档 | 说明 |
|------|------|
| `docs/01-architecture/01-battle-gas/gas-architecture.md` | GAS 系统架构设计（ADR）：模块划分、通信设计、边界定义、Forbidden、数据存储、扩展性、安全架构 |
| `docs/01-architecture/01-battle-gas/skill-buff-abstraction.md` | Effect Executor 统一抽象（500 技能收敛为 20-30 个 Executor） |

---

## GAS 核心链路速查

```
Character → Ability → Targeting → Effect → Buff → Modifier → Tag
```

**五阶段管线**：Requirement → Cost → Targeting → Effect → Settlement

**Effect Pipeline**：Generate（纯函数）→ Modify（纯函数）→ Execute（有副作用）

---

## 阅读建议

1. **首次阅读**：先读 `GAS_domain_overview.md` 获取全景视图
2. **深入特定领域**：按需阅读对应的 `*-rules.md` 文档
3. **架构设计**：参阅 `docs/01-architecture/01-battle-gas/gas-architecture.md`
4. **实现参考**：参阅 `docs/01-architecture/01-battle-gas/skill-buff-abstraction.md`
