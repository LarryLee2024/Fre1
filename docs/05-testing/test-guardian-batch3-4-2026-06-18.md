---
id: 05-testing.test-guardian-batch3-4
title: Test Guardian Report — Batch 3+4 (Spell / Reaction / Quest / Economy / Crafting / Summon)
status: completed
owner: test-guardian
created: 2026-06-18
tags:
  - testing
  - batch3
  - batch4
---

# Test Guardian Report — Batch 3+4

## Review Scope

6 skeleton domains implemented in Batch 3+4:
- **Batch 3**: Spell, Reaction
- **Batch 4**: Quest, Economy, Crafting, Summon

## Test Plan (by pyramid)

### Unit Tests (`tests/unit/`)
Verify pure rule functions — no App startup, no Plugin loading, no resource dependency.

| Domain | Functions Tested |
|--------|-----------------|
| Spell | check_spell_known, check_spell_prepared, check_components, check_slot_available, check_concentration, check_upcast, concentration_save, resolve_save, calc_save_dc, calc_concentration_dc, calc_upcast_bonus, proficiency_bonus_for_level |
| Reaction | can_react, can_trigger_on_turn, calc_priority, can_opportunity_attack, resolve_opportunity_attack_hit, resolve_counterspell, resolve_counterspell_check, is_adjacent, apply_shield_ac |
| Quest | check_progress_monotonic, can_transition, can_abandon_quest, are_all_objectives_completed, can_turn_in, is_reward_already_granted, check_exclusivity |
| Economy | Wallet::add/deduct/can_afford, Price::final_price, ShopInstance::has_stock/add_stock, can_afford, should_restock, reputation_to_price_modifier, can_trade_with_reputation, calc_buy_price, calc_sell_price |
| Crafting | check_station_match, check_materials_available, has_free_enchantment_slot, check_upgrade_limit, perform_skill_check |
| Summon | is_position_valid, can_create_concentration_summon, has_free_summon_slot, is_caster_alive, can_summon_from_summon, can_create_summon, should_expire_on_concentration_broken |

### Invariant Tests (`tests/invariant/`)
Verify domain invariants defined in `docs/02-domain/domains/*.md` §3.

| Invariant | Domain | Status |
|-----------|--------|--------|
| 3.1 货币非负 | Economy | ✅ wallet_balance_never_negative |
| 3.3 价格确定性 | Economy | ✅ price_deterministic_repeatable |
| 3.5 赃物不可交易 | Economy | ✅ hated_reputation_blocks_all_trade |
| 3.1 材料充足性 | Crafting | ✅ exact_materials_satisfies_invariant |
| 3.3 附魔槽上限 | Crafting | ✅ enchantment_slot_count_never_exceeds_max |
| 3.4 升级等级上限 | Crafting | ✅ upgrade_level_never_exceeds_max |
| 3.5 互斥词条替换 | Crafting | ✅ exclusive_enchantment_replaces_old |
| 3.1 召唤者生死 | Summon | ✅ caster_dead_invalidates_summon |
| 3.2 专注唯一性 | Summon | ✅ only_one_concentration_summon_allowed |
| 3.5 占位不冲突 | Summon | ✅ occupied_position_rejected |
| 禁止嵌套召唤 | Summon | ✅ nested_summon_from_summon_blocked |
| 3.2 进度不可倒退 | Quest | ✅ progress_never_decreases |
| 3.3 奖励一次 | Quest | ✅ reward_granted_only_once |
| 3.4 互斥任务 | Quest | ✅ exclusive_quests_cannot_overlap |
| 3.5 关键任务保护 | Quest | ✅ critical_quest_protected |
| 3.1 法术位不透支 | Spell | ✅ slot_never_overdrawn |
| 3.2 专注唯一性 | Spell | ✅ concentration_unique |
| 3.3 组件检查 | Spell | ✅ components_must_be_checked |
| 3.5 专注DC公式 | Spell | ✅ concentration_dc_formula_invariant |
| 3.1 反应次数上限 | Reaction | ✅ reaction_used_once_per_turn |
| 3.2 回合外反应 | Reaction | ✅ offensive_reactions_blocked_on_own_turn |
| 3.4 反制环阶匹配 | Reaction | ✅ counterspell_lower_level_requires_check |
| 3.5 援护距离 | Reaction | ✅ guardian_must_be_adjacent |

### Integration Tests (`tests/integration/`)
(TODO — requires Bevy App builder pattern; deferred to cross-domain integration phase.)

### Fixtures (`tests/fixtures/`)
Builder helpers established for Economy, Spell, and Summon domains.

## Test Matrix

| Domain | Unit | Invariant | Integration | Fixtures | Build |
|--------|------|-----------|-------------|----------|-------|
| Spell | 30 tests | 10 tests | — | ✅ | ✅ |
| Reaction | 18 tests | 8 tests | — | — | ✅ |
| Quest | 12 tests | 7 tests | — | — | ✅ |
| Economy | 14 tests | 4 tests | — | ✅ | ✅ |
| Crafting | 8 tests | 4 tests | — | — | ✅ |
| Summon | 12 tests | 8 tests | — | ✅ | ✅ |

## Coverage Report

**Result: PASS**

### Checklist
- [x] Tests verify business rules (input → output), not implementation details
- [x] Test names follow English snake_case with business semantics
- [x] Tests are deterministic (fixed inputs, no randomness)
- [x] No `#[cfg(test)] mod tests` inline tests — all tests in `tests/` directories
- [x] Standard four-layer structure (unit/invariant/integration/fixtures)
- [x] `cargo check --tests --jobs 1` passes (zero errors)
- [x] `cargo build --lib --jobs 1` passes (zero errors)

### Remaining Gaps
- `tests/integration/` not yet implemented (requires Bevy App/Plugin builder pattern — deferred)
- Spell test uses redundant `#[cfg(test)]` module wrappers (cosmetic, non-blocking)
