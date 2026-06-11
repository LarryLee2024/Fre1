# Skill Module Test Review

**Module**: `src/skill/` (domain/types.rs, domain/mod.rs, domain/defaults.rs, preview.rs, slots.rs)
**Review Date**: 2026-06-10
**Reviewer**: Sisyphus (Automated)
**Test Spec Version**: 3.1 (Testing Constitution)

---

## §1 Scope

### Files Reviewed

| File | Lines | Purpose |
|------|-------|---------|
| `src/skill/mod.rs` | 30 | Plugin registration, re-exports |
| `src/skill/domain/types.rs` | 491 | SkillData, SkillCondition, SkillTargeting, SkillDef, SkillUseError |
| `src/skill/domain/mod.rs` | 399 | SkillRegistry (HashMap<String, SkillData>) + RegistryLoader impl |
| `src/skill/domain/defaults.rs` | 119 | 6 built-in skill registrations |
| `src/skill/preview.rs` | 321 | SkillExecutionContext, SkillPreview, preview_skill_effects |
| `src/skill/slots.rs` | 205 | SkillSlots, SkillCooldowns, effective_skill_range |
| `tests/feature/skill.rs` | 124 | External: skill condition checks (MP, tag, cooldown) |
| `tests/legacy/skill_system.rs` | 451 | External: cross-module skill integration tests |

### Test Count

| Location | Count |
|----------|-------|
| `domain/types.rs` inline | 18 |
| `domain/mod.rs` inline | 13 |
| `preview.rs` inline | 4 |
| `slots.rs` inline | 12 |
| `tests/feature/skill.rs` | 3 |
| `tests/legacy/skill_system.rs` | 20 |
| **Total** | **70** |

---

## §2 Review Criteria

Evaluated against `docs/test_spec.md` v3.1:

1. **Invariant Coverage** (§9): Every domain invariant mapped and tested
2. **Test Pyramid** (§6): Unit > Integration > E2E ratio
3. **Determinism** (§4): No time/sleep/rand依赖
4. **Schema Compliance** (§3): RON deserialization, type safety
5. **Standard Test Data** (§7.1): Unit_001/Unit_002/Unit_003 usage
6. **Replay Tests** (§5/§8): Mandatory replay test presence
7. **AI Self-Check** (§13.1): Annotation presence

---

## §3 Domain Invariant Coverage

### Identified Invariants (INV-SKILL-XXX)

| ID | Invariant | Source | Tested |
|----|-----------|--------|--------|
| INV-SKILL-001 | `can_use()` returns `OnCooldown` when `current_cooldown > 0` | types.rs:154-158 | ✅ types:275, mod:99, feature:116, legacy:167 |
| INV-SKILL-002 | `MpCost` condition returns `InsufficientMp` when `mp < cost` | types.rs:162-169 | ✅ types:297, mod:120, feature:77, legacy:197 |
| INV-SKILL-003 | `RequireTag` returns `MissingTag` when `!source_tags.has(tag)` | types.rs:171-175 | ✅ types:325, mod:147, feature:97, legacy:224 |
| INV-SKILL-004 | `TargetRequireTag` returns `TargetMissingTag` when target lacks tag; skips if `None` | types.rs:176-182 | ✅ types:353,381, mod:305, legacy:280,307 |
| INV-SKILL-005 | `HpBelow(pct)` returns `HpNotBelow` when `hp/max_hp >= pct` | types.rs:183-189 | ✅ types:395,406, mod:193, legacy:253,266 |
| INV-SKILL-006 | `HpAbove(pct)` returns `HpNotAbove` when `hp/max_hp < pct` | types.rs:190-196 | ✅ types:419,430, mod:280 |
| INV-SKILL-007 | Multiple conditions checked in order; first failure returned | types.rs:160-198 | ✅ types:443,457, mod:368, legacy:366 |
| INV-SKILL-008 | `SkillDef` → `SkillData` conversion preserves all fields | types.rs:126-142 | ✅ mod:218 |
| INV-SKILL-009 | `SkillTargeting.requires_target_selection()` true only for SingleEnemy/SingleAlly | types.rs:45-47 | ✅ types:483, mod:70 |
| INV-SKILL-010 | `SkillTargeting.label()` returns correct Chinese labels | types.rs:33-42 | ✅ types:473, mod:338 |
| INV-SKILL-011 | `SkillSlots.default_attack()` returns `BASIC_ATTACK_ID` for empty list | slots.rs:21-26 | ✅ slots:97,103, legacy:139 |
| INV-SKILL-012 | `SkillSlots.special_skill()` returns second element or `None` | slots.rs:29-31 | ✅ slots:109,115,121, legacy:189 |
| INV-SKILL-013 | `SkillCooldowns.set(0)` removes the entry | slots.rs:57-63 | ✅ slots:180 (implicit via tick to 0) |
| INV-SKILL-014 | `SkillCooldowns.tick()` decrements all, removes zeroed entries | slots.rs:67-72 | ✅ slots:187, legacy:170,386 |
| INV-SKILL-015 | `SkillCooldowns.clear()` removes all entries | slots.rs:75-77 | ✅ slots:197, legacy:420 |
| INV-SKILL-016 | `effective_skill_range` uses `skill.range` if > 0, else `base_attack_range` | slots.rs:81-87 | ✅ slots:136,153, legacy:321,327,333 |
| INV-SKILL-017 | `preview_skill_effects` returns `SkillPreview` with correct `skill_id`/`skill_name` | preview.rs:125-129 | ✅ preview:161 |
| INV-SKILL-018 | Damage preview computes `amount = attack - defense` | preview.rs:118-122 | ✅ preview:161 (10-3=7) |
| INV-SKILL-019 | Lethal flag set when `damage >= target HP` | preview.rs:118-122 | ✅ preview:207 |
| INV-SKILL-020 | Heal preview returns correct amount | preview.rs:118-122 | ✅ preview:247 |
| INV-SKILL-021 | Heal capped at `max_hp - current_hp` | preview.rs:118-122 | ✅ preview:285 |
| INV-SKILL-022 | `register_defaults` is idempotent (no-op if non-empty) | defaults.rs:9-11 | ❌ No dedicated test |
| INV-SKILL-023 | 6 built-in skills registered (basic_attack, charge, pierce, fireball, heal, cleanse_skill) | defaults.rs:8-118 | ❌ No dedicated test |

### Coverage Summary

- **Total Invariants**: 23
- **Covered**: 21 (91.3%)
- **Missing**: 2 (INV-SKILL-022, INV-SKILL-023)

---

## §4 Test Pyramid

| Level | Count | Percentage |
|-------|-------|------------|
| Unit (inline) | 47 | 67.1% |
| Integration (legacy) | 20 | 28.6% |
| Feature (external) | 3 | 4.3% |
| E2E / Replay | 0 | 0% |
| **Total** | **70** | |

### Assessment

- **Inline/External ratio**: 47:23 = 2.04:1 — within acceptable range (§6 recommends > 1:1)
- **Missing Replay Tests**: 0/70 — **CRITICAL GAP** per §5/§8
- **Legacy tests at `tests/legacy/skill_system.rs`**: 20 tests covering cross-module integration (attribute→skill, cooldown→slots, tag→condition). Good coverage but not marked as Replay.

---

## §5 Determinism

| Check | Status |
|-------|--------|
| No `std::thread::sleep` | ✅ Pass |
| No `SystemTime::now()` | ✅ Pass |
| No `rand::random()` | ✅ Pass |
| No file I/O in tests | ✅ Pass |
| All test data hardcoded | ✅ Pass |
| `Entity::from_bits()` used correctly | ✅ Pass |

**Verdict**: All 70 tests are deterministic.

---

## §6 Schema Compliance

### RON Deserialization

| Test | File | Status |
|------|------|--------|
| `ron_反序列化_技能定义` | mod.rs:250 | ✅ Full SkillDef with effects, tags, conditions |
| `ron_反序列化_旧配置无version字段` | types.rs:222 | ✅ `#[serde(default)]` on version field |
| `skill_def_转换为_skill_data` | mod.rs:218 | ✅ TagName→GameplayTag conversion |

### Type Safety

- `SkillConditionDef` → `SkillCondition` conversion via `From` impl (types.rs:78-88)
- `SkillDef` → `SkillData` conversion via `From` impl (types.rs:126-142)
- `EffectDef` enum variants properly handled in preview (preview.rs:70-83)

**Verdict**: Schema compliance is solid.

---

## §7 Missing Tests

### P0 — Missing Replay Tests

Per §5/§8, Replay Tests are mandatory and highest priority.

| Skill ID | Name | Status |
|----------|------|--------|
| basic_attack | 普通攻击 | ❌ No replay test |
| charge | 冲锋 | ❌ No replay test |
| pierce | 穿刺 | ❌ No replay test |
| fireball | 火球 | ❌ No replay test |
| heal | 治疗 | ❌ No replay test |
| cleanse_skill | 净化 | ❌ No replay test |

**Recommendation**: Create `tests/replay/skill_replay.rs` with deterministic replay tests for all 6 built-in skills.

### P1 — Missing Invariant Tests

| ID | Invariant | Gap |
|----|-----------|-----|
| INV-SKILL-022 | `register_defaults` idempotency | No test verifying second call is no-op |
| INV-SKILL-023 | All 6 built-in skills registered | No test verifying registry contains exactly 6 entries with correct IDs |

**Recommendation**: Add to `tests/feature/skill.rs`:
```rust
#[test]
fn 内置技能_注册表包含6个技能() {
    let mut reg = SkillRegistry::default();
    reg.register_defaults();
    assert_eq!(reg.skills.len(), 6);
    assert!(reg.get("basic_attack").is_some());
    assert!(reg.get("charge").is_some());
    assert!(reg.get("pierce").is_some());
    assert!(reg.get("fireball").is_some());
    assert!(reg.get("heal").is_some());
    assert!(reg.get("cleanse_skill").is_some());
}

#[test]
fn 内置技能_重复注册幂等() {
    let mut reg = SkillRegistry::default();
    reg.register_defaults();
    let count_before = reg.skills.len();
    reg.register_defaults();
    assert_eq!(reg.skills.len(), count_before);
}
```

### P2 — Missing Standard Test Data

Per §7.1, tests should use Unit_001/Unit_002/Unit_003 fixtures. Current tests use:
- `UnitBuilder::warrior()` (legacy)
- Custom `mage_attrs()` / `warrior_attrs()` helpers
- Inline `make_attrs()` / `make_source_attrs()` / `make_target_attrs()`

**No tests use the standard Unit_001/Unit_002/Unit_003 fixtures.**

### P3 — Missing Edge Cases

| Edge Case | Status |
|-----------|--------|
| `can_use` with `MaxHp == 0` (division by zero guard) | ❌ Not tested |
| `SkillCooldowns.set()` with `turns == 0` removes entry | ❌ Not explicitly tested |
| `SkillSlots` with 3+ skills (iteration beyond special) | ❌ Not tested |
| `preview_skill_effects` with empty effects vec | ❌ Not tested |
| `effective_skill_range` with `range == 0` and `base_attack_range == 0` | ❌ Not tested |

---

## §8 Code Quality Issues

### Duplicate Test Logic

**`domain/mod.rs` vs `domain/types.rs`**: Both files contain `can_use` condition tests with overlapping coverage:

| Test | mod.rs | types.rs | Overlap |
|------|--------|----------|---------|
| 冷却中不可使用 | `条件_冷却中不可使用` | `can_use_冷却中返回错误` | Exact |
| MP不足不可使用 | `条件_mp不足不可使用` | `can_use_mp不足返回错误` | Exact |
| 缺少标签不可使用 | `条件_缺少标签不可使用` | `can_use_缺少标签返回错误` | Exact |
| 满足条件可使用 | `条件_满足条件可使用` | `can_use_mp足够成功` | Near |
| HP低于阈值 | `条件_hp低于阈值` | `can_use_hp低于阈值成功` + `can_use_hp不低于阈值返回错误` | Near |
| HP高于阈值 | `条件_hp高于阈值` | `can_use_hp高于阈值成功` + `can_use_hp不高于阈值返回错误` | Near |
| 目标缺少标签 | `条件_目标缺少标签` | `can_use_目标缺少标签返回错误` + `can_use_目标拥有标签成功` + `can_use_无目标标签检查跳过` | Partial |
| 多条件全满足 | `条件_多个条件全满足` | `can_use_多条件全部满足` + `can_use_多条件之一不满足` | Partial |
| targeting label | `目标类型_label` | `targeting_label` | Exact |
| targeting requires_target | `目标类型_需要选择目标` | `targeting_requires_target_selection` | Exact |

**10 tests are duplicated or near-duplicated** across the two inline modules. Recommend consolidating into `types.rs` only (since that's where the implementation lives).

### Helper Inconsistency

- `types.rs` uses `make_attrs(mp, hp, vitality)` — 3 params
- `mod.rs` uses `make_attrs(hp, max_hp, mp)` — 3 params, different order/semantics
- `preview.rs` uses `make_source_attrs(atk)` + `make_target_attrs(def, hp)` — 2 helpers
- `legacy/skill_system.rs` uses `warrior_attrs()` + `mage_attrs()` — fixture-based
- `feature/skill.rs` uses `low_mp_warrior_attrs()` + `mage_only_skill()` + `expensive_skill()` + `cooldown_skill()` — fixture-based

**Recommendation**: Standardize on `UnitBuilder::warrior()` / `UnitBuilder::mage()` from `tests/common/fixtures.rs`.

---

## §9 Issue Statistics

| Priority | Count | Description |
|----------|-------|-------------|
| P0 | 6 | Missing Replay Tests for all 6 built-in skills |
| P1 | 2 | Missing invariant tests (idempotency, registry count) |
| P2 | 1 | No standard Unit_001/Unit_002/Unit_003 usage |
| P3 | 5 | Missing edge cases (MaxHp=0, empty effects, etc.) |
| Quality | 10 | Duplicate test logic across mod.rs and types.rs |
| **Total** | **24** | |

---

## §10 Priority Recommendations

### Immediate (P0)
1. Create `tests/replay/skill_replay.rs` with deterministic replay tests for all 6 built-in skills
2. Each replay test must: load skill → construct identical context → apply → assert identical output

### Short-term (P1)
3. Add `register_defaults` idempotency test
4. Add registry count verification test
5. Deduplicate 10 overlapping tests between `mod.rs` and `types.rs` (keep in `types.rs`)

### Medium-term (P2)
6. Migrate all test helpers to use `UnitBuilder::warrior()` / `UnitBuilder::mage()`
7. Add standard Unit_001/Unit_002/Unit_003 fixture usage

### Long-term (P3)
8. Add edge case tests (MaxHp=0, empty effects, range=0+base=0, etc.)
9. Add AI Self-Check annotations per §13.1

---

## §11 Summary

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Total Tests | 70 | — | — |
| Invariant Coverage | 91.3% (21/23) | 100% | ⚠️ 2 gaps |
| Replay Tests | 0 | ≥ 1 per skill | ❌ Missing |
| Determinism | 100% | 100% | ✅ |
| Schema Compliance | Solid | Solid | ✅ |
| Test Pyramid | 67% unit / 29% int / 4% feature | Unit > Int > E2E | ⚠️ No E2E |
| Standard Fixtures | 0% | 100% | ❌ Missing |
| Duplicate Tests | 10 | 0 | ⚠️ Cleanup needed |

### Overall Score: **3.8 / 5.0**

**Strengths**:
- Excellent condition checking coverage (all 5 `SkillCondition` variants fully tested)
- Good cross-module integration via legacy tests (attribute→skill, cooldown→slots, tag→condition)
- Deterministic, well-structured test helpers
- RON deserialization + version migration covered

**Weaknesses**:
- Zero Replay Tests (§5/§8 critical gap)
- Duplicate test logic across `mod.rs` and `types.rs`
- No standard Unit_001/Unit_002/Unit_003 fixtures
- Missing idempotency + registry count invariant tests

---

## §12 Self-Check

- [x] All `#[test]` functions counted (70 total)
- [x] All domain invariants identified from source (23 total)
- [x] Coverage matrix complete (21/23 covered)
- [x] Test pyramid calculated (67/29/4%)
- [x] Determinism verified (no sleep/time/rand)
- [x] Schema compliance checked (RON deserialization)
- [x] Missing tests documented with specific file/line references
- [x] Duplicate tests identified with overlap matrix
- [x] Issue priorities assigned (P0-P3)
- [x] Recommendations actionable and specific
