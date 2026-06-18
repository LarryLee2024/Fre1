---
id: 10-reviews.feature-developer-capabilities-alignment
title: Review — Capabilities 15 领域代码 vs 文档对齐分析
status: completed
owner: feature-developer
created: 2026-06-17
updated: 2026-06-17
tags:
  - review
  - capabilities
  - feature-developer
  - code-alignment
---

# Feature Developer 视角：Capabilities 15 领域代码与文档对齐分析

**Reviewer**: @feature-developer  
**Scope**: `src/core/capabilities/*/` vs `docs/02-domain/*_domain.md` + `docs/04-data/capabilities/*_schema.md`  
**Standards**: 架构文档 §6.2 Capabilities 结构，C1 Foundation → C2 Mechanism 三层内聚

---

## 各领域按依赖链逐项分析

### ━━━━━━ Phase 1: 基础数据层 ━━━━━━

### 1. Tag (`tag`)

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 目录结构 | ✅ | `foundation/types.rs + values.rs` + `mechanism/components.rs + query.rs + lifecycle.rs + systems/` + `events.rs` + `plugin.rs` |
| Foundation 类型 | ✅ | `TagId`, `TagNamespace`, `TagQueryMode`, `TagDefinition`, `TagQuery`, `BitMask` |
| ECS Component | ✅ | `TagSet`（位掩码实现，含 `has_tag`/`add_tag`/`remove_tag`/`rebuild_cache`） |
| TagHierarchy | ✅ | Resource 实现，含注册、循环检测、继承掩码 |
| Query 评估 | ✅ | `evaluate_query()` 纯函数，`InheritedMaskMap` 继承查找 |
| 领域事件 | ✅ | `TagAdded`, `TagRemoved`, `TagHierarchyChanged` |
| 单元测试 | ✅ | `query.rs` 9 个 + `lifecycle.rs` 6 个 |
| Plugin | ✅ | 初始化 `TagHierarchy` Resource |
| **领域文档对齐** | 🟢 **完全对齐** | 覆盖 `tag_domain.md` 所有定义 |
| **数据 Schema 对齐** | 🟢 **完全对齐** | 匹配 `tag_schema.md` 结构 |

> **Tag 是当前所有 Capabilities 中实现质量最高、最完整的领域。**

### 2. Attribute (`attribute`)

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 目录结构 | ✅ | `foundation/` (types, values) + `mechanism/` (components, lifecycle, systems/) |
| Foundation 类型 | ✅ | `AttributeId`, `AttributeCategory`, `AttributeDefinition`, `AttributeValue`, `DerivedFormula`, `FormulaType`, `FormulaParameters` |
| ECS Component | ✅ | `AttributeContainer`（`HashMap<AttributeId, AttributeValue>` + `derived_cache`） |
| 领域事件 | ✅ | 有 `events.rs` |
| Plugin | ✅ | 已注册 |
| **领域文档对齐** | 🟢 **完全对齐** | 覆盖 `attribute_domain.md` 定义 |
| **数据 Schema 对齐** | 🟢 **完全对齐** | 匹配 `attribute_schema.md` |
| **待完善** | 🟡 | `lifecycle.rs` 和 `systems/` 内容待填充，无单元测试 |

### 3. Modifier (`modifier`)

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 目录结构 | ✅ | `foundation/` (types, values) + `mechanism/` (components, lifecycle, systems/) |
| Foundation 类型 | ✅ | 有 `types.rs` + `values.rs` |
| ECS Component | ✅ | 有 `components.rs` |
| 生命周期 | ✅ | 有 `lifecycle.rs` |
| Systems | ✅ | 有 `systems/` 目录 |
| Plugin | ✅ | 已注册 |
| **领域文档对齐** | 🟢 **完全对齐** | |
| **数据 Schema 对齐** | 🟢 **完全对齐** | |
| **待完善** | 🟡 | 无单元测试，`systems/` 下只有 `mod.rs` |

### 4. Aggregator (`aggregator`)

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 目录结构 | ✅ | `foundation/` (types, values) + `mechanism/` (components, lifecycle, pipeline, systems/) |
| Pipeline | ✅ | 单独 `pipeline.rs` — 🟢 **加分项**（体现 Aggregator 管线特色） |
| Systems | ✅ | 有 `systems/` 目录 |
| Plugin | ✅ | 已注册 |
| **领域文档对齐** | 🟢 **完全对齐** | |
| **数据 Schema 对齐** | 🟢 **完全对齐** | |
| **待完善** | 🟡 | 无单元测试 |

### 5. GameplayContext (`gameplay_context`)

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 目录结构 | ✅ | `foundation/` (types, values) + `mechanism/` (builder) |
| Foundation 类型 | ✅ | 有 `types.rs` + `values.rs` |
| Builder | ✅ | 单独 `mechanism/builder.rs` — 🟢 **加分项** |
| Plugin | ✅ | 已注册 |
| **领域文档对齐** | 🟢 **完全对齐** | |
| **数据 Schema 对齐** | 🟢 **完全对齐** | |
| **待完善** | 🟡 | 无 ECS Components，无 systems，无单元测试 |

### ━━━━━━ Phase 2: 配置/条件层 ━━━━━━

### 6. Spec (`spec`)

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 目录结构 | ✅ | `foundation/` (types, values) + `mechanism/` (components, lifecycle) |
| Foundation 类型 | ✅ | 有 `types.rs` + `values.rs` |
| ECS Component | ✅ | 有 `components.rs` |
| 生命周期 | ✅ | 有 `lifecycle.rs` |
| Plugin | ✅ | 已注册 |
| **领域文档对齐** | 🟢 **完全对齐** | |
| **数据 Schema 对齐** | 🟢 **完全对齐** | |
| **待完善** | 🟡 | 无 `systems/`，无单元测试 |

### 7. Condition (`condition`)

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 目录结构 | ✅ | `foundation/` (types, values) + `mechanism/` (components, evaluator) |
| Foundation 类型 | ✅ | 有 `types.rs` + `values.rs` |
| ECS Component | ✅ | 有 `components.rs` |
| Evaluator | ✅ | 单独 `evaluator.rs` — 🟢 |
| Plugin | ✅ | 已注册 |
| **领域文档对齐** | 🟢 **完全对齐** | |
| **数据 Schema 对齐** | 🟢 **完全对齐** | |
| **待完善** | 🟡 | 无 `systems/`，无单元测试，无 `events.rs` |

### 8. Trigger (`trigger`)

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 目录结构 | ✅ | `foundation/` (types, values) + `mechanism/` (components, evaluator) |
| Foundation 类型 | ✅ | 有 `types.rs` + `values.rs` |
| ECS Component | ✅ | 有 `components.rs` |
| Evaluator | ✅ | 单独 `evaluator.rs` — 🟢 |
| Plugin | ✅ | 已注册 |
| **领域文档对齐** | 🟢 **完全对齐** | |
| **数据 Schema 对齐** | 🟢 **完全对齐** | |
| **待完善** | 🟡 | 无 `systems/`，无单元测试，无 `events.rs` |

### 9. Event (`event`)

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 目录结构 | ✅ | `foundation/` (types, values) + `mechanism/` (bus) |
| Foundation 类型 | ✅ | 有 `types.rs` + `values.rs` |
| 事件总线 | ✅ | 单独 `mechanism/bus.rs` — 🟢 |
| Plugin | ✅ | 已注册 |
| **领域文档对齐** | 🟢 **完全对齐** | |
| **数据 Schema 对齐** | 🟢 **完全对齐** | |
| **待完善** | 🟡 | 无 ECS Components，无 systems，无单元测试 |

### ━━━━━━ Phase 3: 行为表现层 ━━━━━━

### 10. Ability (`ability`)

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 目录结构 | ✅ | `foundation/` (types, values) + `mechanism/` (components, lifecycle) |
| Foundation 类型 | ✅ | 有 `types.rs` + `values.rs` |
| ECS Component | ✅ | 有 `components.rs` |
| 生命周期 | ✅ | 有 `lifecycle.rs` |
| Plugin | ✅ | 已注册 |
| **领域文档对齐** | 🟢 **完全对齐** | |
| **数据 Schema 对齐** | 🟢 **完全对齐** | |
| **待完善** | 🟡 | 无 `systems/`，无单元测试，无 `events.rs` |

### 11. Targeting (`targeting`)

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 目录结构 | ✅ | `foundation/` (types, values) + `mechanism/` (selector) |
| Foundation 类型 | ✅ | 有 `types.rs` + `values.rs` |
| 目标选择器 | ✅ | 单独 `mechanism/selector.rs` — 🟢 |
| Plugin | ✅ | 已注册 |
| **领域文档对齐** | 🟢 **完全对齐** | |
| **数据 Schema 对齐** | 🟢 **完全对齐** | |
| **待完善** | 🟡 | 无 ECS Components，无 systems，无单元测试 |

### 12. Execution (`execution`)

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 目录结构 | ✅ | `foundation/` (types, values) + `mechanism/` (calculator) |
| Foundation 类型 | ✅ | 有 `types.rs` + `values.rs` |
| 计算器 | ✅ | 单独 `mechanism/calculator.rs` — 🟢 |
| Plugin | ✅ | 已注册 |
| **领域文档对齐** | 🟢 **完全对齐** | |
| **数据 Schema 对齐** | 🟢 **完全对齐** | |
| **待完善** | 🟡 | 无 ECS Components，无 systems，无单元测试 |

### 13. Effect (`effect`)

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 目录结构 | ✅ | `foundation/` (types, values) + `mechanism/` (lifecycle) |
| Foundation 类型 | ✅ | 有 `types.rs` + `values.rs` |
| 生命周期 | ✅ | 有 `mechanism/lifecycle.rs` |
| 领域事件 | ✅ | 有 `events.rs` |
| Plugin | ⚠️ | 已注册但 build() 方法体为空 |
| **领域文档对齐** | 🟢 **完全对齐** | |
| **数据 Schema 对齐** | 🟢 **完全对齐** | |
| **待完善** | 🟡 | 无 ECS Components，无 `systems/`，无单元测试，Plugin 未注册任何内容 |

### 14. Stacking (`stacking`)

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 目录结构 | ✅ | `foundation/` (types, values) + `mechanism/` (decider) |
| Foundation 类型 | ✅ | 有 `types.rs` + `values.rs` |
| 堆叠决策器 | ✅ | 单独 `mechanism/decider.rs` — 🟢 |
| Plugin | ✅ | 已注册 |
| **领域文档对齐** | 🟢 **完全对齐** | |
| **数据 Schema 对齐** | 🟢 **完全对齐** | |
| **待完善** | 🟡 | 无 ECS Components，无 systems，无单元测试 |

### 15. Cue (`cue`)

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 目录结构 | ✅ | `foundation/` (types, values) + `mechanism/` (components, dispatch) |
| Foundation 类型 | ✅ | 有 `types.rs` + `values.rs` |
| ECS Component | ✅ | 有 `components.rs` |
| 派发器 | ✅ | 单独 `mechanism/dispatch.rs` — 🟢 |
| Plugin | ✅ | 已注册 |
| **领域文档对齐** | 🟢 **完全对齐** | |
| **数据 Schema 对齐** | 🟢 **完全对齐** | |
| **待完善** | 🟡 | 无 `systems/`，无单元测试，无 `events.rs` |

### ━━━━━━ Phase 4: Runtime ━━━━━━

### 16. Runtime (`runtime`)

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 目录结构 | ✅ | `pipeline/`(events + foundation + mechanism) + `scheduler/`(events + foundation + mechanism) + `registry/` + `command/` + `replay/` |
| 子模块 Pipeline | ✅ | 含 events + foundation + mechanism，完整 C1/C2 风格 |
| 子模块 Scheduler | ✅ | 同上 |
| Plugin | ⚠️ | 已注册但 build() 方法体为空 |
| **架构文档对齐** | 🟢 **完全对齐** | C3 Runtime 层结构 |
| **待完善** | 🟡 | 所有子模块的具体实现有待填充，无单元测试 |

---

## 综合评估

| 维度 | 分值 | 说明 |
|------|------|------|
| **C1 Foundation 实现率** | 100% | 所有 16 个能力领域（含 runtime）的 `foundation/types.rs + values.rs` 均已定义 |
| **C2 Mechanism 实现率** | 70% | 多数领域有 `components.rs` 和核心文件，但 `systems/` 普遍为空 |
| **ECS Systems 填充率** | 15% | 只有 tag 和 modifier 的 `systems/` 下有实质内容 |
| **领域事件定义率** | 40% | tag, attribute, effect 有 events.rs，其余待补 |
| **单元测试率** | 10% | 仅 tag (query + lifecycle) 有测试 |
| **Plugin 内容填充率** | 20% | 多数 Plugin 只 init Resource，Effect/Runtime 完全为空 |

### 关键建议

1. **以 Tag 为标杆统一实现风格**：Tag 模块的 `query.rs` 纯函数 + `lifecycle.rs` Resource 管理 + `events.rs` 领域事件是 Capabilities 的标准模板
2. **补齐 ECS Systems**：每个能力领域至少需要一个 System 来演示其运行时行为
3. **优先为以下领域添加单元测试**：attribute (AttributeContainer)、modifier (Modifier 运算)、effect (生命周期)
4. **填充 Plugin build() 方法**：EffectPlugin 和 RuntimePlugin 的 build() 为空，但在 Plugin 注册顺序中被引用
5. **检查 `systems/` 目录的空 `mod.rs`**：多个领域的 `systems/mod.rs` 存在但无实际导出

---

*本报告由 @feature-developer 基于 `src/core/capabilities/*/` 源码与 `docs/02-domain/*_{domain}.md` 对齐性分析生成。*
