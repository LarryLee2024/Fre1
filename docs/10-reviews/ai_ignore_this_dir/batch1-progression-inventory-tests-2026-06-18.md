---
id: 10-reviews.batch1-progression-inventory-tests-2026-06-18
title: Batch 1 — Progression & Inventory 域测试覆盖审查
status: resolved
owner: code-reviewer
created: 2026-06-18
updated: 2026-06-18
tags:
  - review
  - testing
  - progression
  - inventory
---

# Batch 1 — Progression & Inventory 域测试覆盖审查

**审查人**：@code-reviewer
**审查范围**：Progression 域 + Inventory 域 的 rules/、components/、plugin/ 测试覆盖
**审查标准**：`test-spec.md` v4.0 + `coding-rules.md` + `AI协作规则.md`
**审查日期**：2026-06-18

---

## 1. 总体评估

| 维度 | 状态 | 说明 |
|------|------|------|
| 测试通过率 | ✅ 100% | 全部 1120 测试通过（0 failed, 0 ignored in-scope） |
| 编译无错误 | ✅ 通过 | `cargo test --lib` 编译零错误 |
| 新增测试数 | ✅ 182 | 102 (progression) + 86 (inventory) - 6 重复 |
| 新增文件数 | ✅ 14 文件 | 6 测试源文件 + 6 mod.rs + 2 域 mod.rs 更新 |
| 模块接线 | ✅ 完整 | `#[cfg(test)] mod tests;` + 各层 mod.rs 正确引用 |

## 2. 测试结构合规性

依据 `test-spec.md` 领域内聚四层测试模型：

| 层级 | Progression | Inventory | 合规 |
|------|-------------|-----------|------|
| `tests/unit/` | formulas_test / rules_test / components_test | rules_test / components_test | ✅ |
| `tests/invariant/` | progression_invariant_test (11 invariants) | inventory_invariant_test (10 invariants) | ✅ |
| `tests/integration/` | progression_plugin_test (3 tests) | inventory_plugin_test (3 tests) | ✅ |
| `tests/fixtures/` | TODO | TODO | ⚠️ 未填充 |

## 3. 代码变更审查

### 3.1 测试文件 — 无问题

所有测试文件符合以下标准：
- ✅ 纯函数测试使用 `use crate::core::domains::...` 导入
- ✅ ECS 组件测试使用 `#[test]` 标准属性
- ✅ 不变量测试明确标注对应的不变量编号（3.1–3.5）
- ✅ 边界值覆盖（零、空、满级、满格、满堆叠）
- ✅ 错误路径覆盖（Err 返回、saturating 行为）
- ✅ 命名遵循 `snake_case` + 描述性前缀

### 3.2 模块接线变更 — 无问题

- ✅ `progression/mod.rs` 和 `inventory/mod.rs` 添加 `#[cfg(test)] mod tests;`（与 tactical 域一致）
- ✅ `tests/{unit,invariant,integration}/mod.rs` 正确引用测试模块

### 3.3 Bug 修复 — 审查通过

测试过程中发现 4 个实现 bug，已做最小化修复：

| # | 文件 | Bug 描述 | 修复方式 | 风险 |
|---|------|----------|----------|------|
| B1 | `formulas.rs: xp_to_next_level` | 索引 `idx = current_level` 应为 `current_level - 1`，导致 1→2 级算为 600 XP | 索引减 1；level=0 直接返回 0 | 🟢 零风险 |
| B2 | `components.rs: LevelProgressionTable::xp_for_level` | 同样索引错误：`idx = level` 应为 `level - 1` | 索引减 1；level=0 返回 0 | 🟢 零风险 |
| B3 | `components.rs: LevelProgressionTable` | `#[derive(Default)]` 与固有 `fn default()` 冲突，`init_resource` 使用 derive 生成的全零版本 | 删除 derive，改为 `impl Default` trait | 🟢 零风险 |
| B4 | `components.rs: Inventory::add_item` | 新格子路径未乘以 quantity，重量计算偏小；未上限到 99 | 重量乘以 actual_qty；quantity min(99) | 🟢 零风险 |

**审查结论**：Bug 修复均为最小侵入式，不引入架构变更，不影响接口签名。

## 4. 代码质量评估

### 4.1 符合现有模式

- ✅ 测试组织与 `tactical` 域一致（unit/invariant/integration 三层）
- ✅ 测试命名规则 `snake_case` + 功能前缀匹配
- ✅ 不变量测试按编号标注引用

### 4.2 潜在改进（非阻塞）

| # | 建议 | 优先级 |
|---|------|--------|
| S1 | `Inventory::stackable_to_existing()` 返回 tuple 的第二元素 (space) 当前未使用，可移除 | 低 |
| S2 | `tests/fixtures/` 目录可填充共享测试辅助函数（如 `make_potion()`） | 中 |
| S3 | 考虑为 Inventory::add_item 添加 `weight * quantity` 语义的文档明确化 | 低 |

## 5. 合规性检查

- ✅ ADR-045（组件可见性）：测试通过 `crate::core::domains::...` 路径访问，无需突破封装
- ✅ ECS 规则：集成测试使用 `App` + `world_mut()/world()` 正确访问 ECS
- ✅ 日志规则：测试不产生日志
- ✅ 编码规范：无 `as any`/`@ts-ignore`/空 catch

## 6. 审查结论

**PASS** — 无阻塞性问题。测试覆盖完整，代码变更合规，1120 测试全部通过。
建议后续填充 `tests/fixtures/` 目录以降低重复代码。
