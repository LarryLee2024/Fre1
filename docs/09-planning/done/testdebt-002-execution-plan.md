---
id: 09-planning.testdebt-002-execution-plan
title: TestDebt-002 执行计划 — combat integration facade 测试补全
status: done
owner: test-guardian
created: 2026-06-20
scope: combat/integration/ 下 facade 测试标准化与覆盖补全
---

# TestDebt-002 执行计划 — combat integration facade 测试补全

> **关联债务**: `docs/11-refactor/tech-debt-scan-2026-06-19.md` Debt-023
> **严重程度**: Medium
> **前置条件**: Content-002 已完成并提交 (`0489eb8`)

---

## 1. 现状分析

### 1.1 facade 测试覆盖矩阵

| Facade | facade_test.rs | 行数 | 状态 | 问题 |
|--------|---------------|------|------|------|
| ability | ✅ | 7 | ⚠️ Stub | 仅编译测试 |
| aggregator | ✅ | 55 | ✅ OK | 覆盖 pipeline/modifier/aggregation |
| condition | ✅ | 24 | ✅ OK | 覆盖 immunity check |
| effect | ❌ | 0 | 🔴 缺失 | 测试全在 `mod.rs` (239 行) |
| event | ✅ | 14 | ⚠️ Stub | 仅枚举转换测试 |
| execution | ✅ | 51 | ✅ OK | 覆盖 damage context/build/execute |
| gameplay_context | ✅ | 55 | ✅ OK | 覆盖 context 查询 |
| targeting | ✅ | 39 | ✅ OK | 覆盖 range/line of sight |
| trigger | ✅ | 14 | ⚠️ Stub | 仅枚举转换测试 |

### 1.2 标准模式（来自 execution/aggregator/gameplay_context）

```
integration/{name}/tests/
├── mod.rs           # 仅声明: mod facade_test;
└── facade_test.rs   # 所有实际测试
```

**effect facade 违反此模式**：`tests/mod.rs` 包含 239 行测试代码，`facade_test.rs` 不存在。

---

## 2. 修复范围

### Task A: 结构标准化 — effect facade 测试迁移
- 将 `effect/tests/mod.rs` 中的 239 行测试代码完整迁移至 `effect/tests/facade_test.rs`
- 重写 `effect/tests/mod.rs` 为标准模式（仅 `mod facade_test;`）
- **约束**: 不修改任何测试逻辑，纯文件结构迁移

### Task B: 覆盖补全 — ability facade 测试扩展
目标 API 覆盖:
- `CombatAbilityFacade::empty_container()` — 创建空容器
- `CombatAbilityFacade::try_activate_ability()` — 技能激活成功/失败路径
- `CombatAbilityFacade::complete_and_cooldown()` — 完成技能并进入冷却
- `CombatAbilityFacade::tick_all_cooldowns()` — 推进冷却并返回到期列表

### Task C: 覆盖补全 — trigger facade 测试扩展
目标 API 覆盖:
- `CombatTriggerFacade::create_trigger_entry()` — 创建触发器条目
- `CombatTriggerFacade::can_trigger_check()` — 触发条件评估
- `CombatTriggerFacade::evaluate_triggers()` — 批量触发器评估
- `CombatTriggerFacade::empty_container()` — 创建空容器

### Task D: 覆盖补全 — event facade 测试扩展
目标 API 覆盖:
- `CombatEventFacade::publish()` — 发布普通优先级事件
- `CombatEventFacade::publish_priority()` — 发布高优先级事件
- `CombatEventTag::to_event_tag()` — 枚举映射完整性

---

## 3. 验证标准

- [x] `cargo nextest run` 全部通过（基准 1485/1485）
- [x] 新增测试不得降低现有测试通过率
- [x] effect 迁移后测试数量不变（验证零回归）
- [x] 每个 facade_test.rs 文件头添加模块文档注释
- [x] 遵循 `docs/05-testing/test-spec.md` 测试规范

> **完成日期**: 2026-06-20 — 4 个 Task 全部完成，effect/ability/trigger/event facade 测试均已就位

---

## 4. 执行顺序

```
Task A (effect 结构迁移)
    ↓
Task B (ability 覆盖补全) ─┐
Task C (trigger 覆盖补全) ─┼→ 并行执行
Task D (event 覆盖补全) ───┘
    ↓
验证 (cargo nextest run)
    ↓
更新债务扫描文档
```

---

## 5. 风险评估

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| effect 迁移遗漏导入 | 低 | 编译失败 | 迁移后即时编译验证 |
| ability/trigger 内部 API 变更 | 低 | 测试失效 | 优先使用 facade 公开 API |
| 测试依赖 Bevy ECS 初始化 | 中 | 测试无法独立运行 | 使用纯单元测试，避免 SystemParam 测试 |

---

## 6. 交付物

1. `src/core/domains/combat/integration/effect/tests/facade_test.rs` — 新建
2. `src/core/domains/combat/integration/effect/tests/mod.rs` — 重写（3 行）
3. `src/core/domains/combat/integration/ability/tests/facade_test.rs` — 扩展
4. `src/core/domains/combat/integration/trigger/tests/facade_test.rs` — 扩展
5. `src/core/domains/combat/integration/event/tests/facade_test.rs` — 扩展
6. `docs/11-refactor/tech-debt-scan-2026-06-19.md` — 更新 TestDebt-002 状态
