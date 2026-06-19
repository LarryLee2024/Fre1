---
id: 09-planning.spell-formulas-refactor-plan
title: Spell Formulas 纯函数重构计划
status: active
owner: feature-developer
created: 2026-06-20
scope: spell/rules/formulas.rs 冗余消除与参数化
---

# Spell Formulas 纯函数重构计划

> **关联债务**: `docs/11-refactor/tech-debt-scan-2026-06-19.md` spell rules/formulas 冗余
> **严重程度**: P3

---

## 1. 问题分析

### 1.1 `proficiency_bonus_for_level` — 完全重复

- **位置**: `spell/rules/formulas.rs:72`
- **问题**: 硬编码 match 与 `LevelProgressionTable::proficiency_bonus()` 完全等价
- **调用方**: `spell/tests/unit/mod.rs`, `spell/tests/invariant/mod.rs`
- **方案**: 委托给 `LevelProgressionTable::default().proficiency_bonus(level as u32)`

### 1.2 `calc_concentration_dc` — 硬编码 10

- **位置**: `spell/rules/formulas.rs:36`
- **问题**: 硬编码 `10u32`，而 `SpellConfig::concentration_base_dc` 已 RON 配置化
- **调用方**: `spell/rules/rules.rs::concentration_save`, 测试
- **方案**: 函数签名改为接受 `base_dc: u32` 参数

---

## 2. 变更清单

| 文件 | 变更 |
|------|------|
| `spell/rules/formulas.rs` | `proficiency_bonus_for_level` 委托给 `LevelProgressionTable`；`calc_concentration_dc` 接受 `base_dc` |
| `spell/rules/rules.rs` | `concentration_save` 接受 `base_dc` 参数 |
| `spell/tests/unit/mod.rs` | 更新测试调用，传递 base_dc |
| `spell/tests/invariant/mod.rs` | 更新测试调用，传递 base_dc |
| `docs/11-refactor/tech-debt-scan-2026-06-19.md` | 标记 spell formulas 为 Resolved |

---

## 3. 验证标准

- [ ] `cargo nextest run` 1513/1513 passed
- [ ] 无新编译错误/警告
