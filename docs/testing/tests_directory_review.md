# tests/ Directory Comprehensive Test Review

**Review Date**: 2026-06-11
**Reviewer**: Sisyphus (Automated)
**Test Spec Version**: 3.1 (Testing Constitution)
**Scope**: All files under `D:\Code\Bevy\Fre\tests\`

---

## §1 Scope

### Directory Structure

```
tests/
├── common/                    # Shared test infrastructure
│   ├── mod.rs
│   ├── app_builder.rs         # Minimal/combat/equipment/full battle App builders
│   ├── assertions.rs          # Custom assertion macros
│   ├── combat_helpers.rs      # deal_damage, deal_heal, tick, get_hp
│   └── fixtures.rs            # UnitBuilder (warrior/mage/goblin) + legacy helpers
├── feature/                   # Feature-level integration tests
│   ├── buff.rs                # Buff lifecycle via Effect Pipeline
│   ├── consumable.rs          # Consumable use: HP restore, buff grant, quantity
│   ├── death.rs               # Death hook, lethal damage, dead unit exclusion
│   ├── equipment.rs           # Equip/unequip, requirements, auto-unequip, trait lifecycle
│   ├── inventory.rs           # Container transfer, capacity limits, pure function
│   ├── skill.rs               # Skill condition checks (MP, tag, cooldown)
│   ├── traits.rs              # Trait grant tag, modify attribute, lifecycle
│   └── turn.rs                # Turn end, reset acted, ForceEndTurn
├── golden/                    # Snapshot regression tests
│   ├── golden_battle.rs       # insta snapshot: damage, heal, death
│   └── snapshots/             # 6 snapshot files (3 old + 3 new format)
├── legacy/                    # Legacy integration tests
│   ├── buff_damage.rs         # Buff → attribute → damage calculation
│   ├── buff_lifecycle.rs      # Buff apply → tick → expire → remove
│   ├── combat_pipeline.rs     # EffectHandlerRegistry generate/preview
│   ├── edge_cases.rs          # HP full heal, modifier stacking, empty ops
│   ├── skill_system.rs        # Skill slots, cooldowns, conditions cross-module
│   ├── terrain_combat.rs      # Terrain grid, pathfinding, range
│   └── turn_flow.rs           # Turn end, queue rebuild, faction switch
├── rule/                      # Property-based tests (proptest)
│   ├── rules.rs               # Damage formula, attributes, tags, container stacking
│   └── rules.proptest-regressions
├── scenario/                  # BDD-style scenario tests
│   └── scenarios.rs           # Fireball vs knight, poison, terrain, death
├── system/                    # System-level tests
│   └── systems.rs             # execute_effects, resolve, equip, use_item, transfer, traits
├── feature.rs                 # Entry point: includes feature/ + common/
├── golden.rs                  # Entry point: includes golden/ + common/
├── legacy_buff.rs             # Entry point: includes legacy buff tests
├── legacy_combat.rs           # Entry point: includes legacy combat tests
├── legacy_edge.rs             # Entry point: includes legacy edge tests
├── legacy_turn.rs             # Entry point: includes legacy turn tests
├── rule.rs                    # Entry point: includes rule/ + common/
├── scenario.rs                # Entry point: includes scenario/ + common/
└── system.rs                  # Entry point: includes system/ + common/
```

### Test Count Summary

| Directory | Files | Tests | Type |
|-----------|-------|-------|------|
| `common/` | 5 | 0 | Helpers only |
| `feature/` | 8 | 40 | Integration |
| `golden/` | 1 | 3 | Snapshot/Replay |
| `legacy/` | 7 | 104 | Integration |
| `rule/` | 1 | 13 | Property-based |
| `scenario/` | 1 | 4 | BDD/Scenario |
| `system/` | 1 | 10 | System-level |
| **Total** | **24** | **174** | |

---

## §2 Review Criteria

Evaluated against `docs/test_spec.md` v3.1:

1. **Test Pyramid** (§4): 70% Unit / 20% Integration / 8% Replay / 2% E2E
2. **Test Categories** (§5): Unit, Integration, Replay, Regression, E2E
3. **Determinism** (§6): No random, time, network dependencies
4. **Standard Test Data** (§7.1): Unit_001/Unit_002/Unit_003
5. **Replay Tests** (§5/§8): Mandatory for all battle scenarios
6. **Coverage Strategy** (§9): 100% core domain rules
7. **Error Testing** (§10): Invalid input, boundary values
8. **Regression Rules** (§11): Bug → test first
9. **AI Self-Check** (§13.1): Annotation presence

---

## §3 Detailed Review Results

### §3.1 Test Infrastructure (`common/`)

**Assessment: EXCELLENT**

| Component | Status | Notes |
|-----------|--------|-------|
| `UnitBuilder` | ✅ | Fluent API, 3 templates (warrior/mage/goblin) |
| `app_builder.rs` | ✅ | 4 App levels: minimal, combat, equipment, full_battle |
| `assertions.rs` | ✅ | `assert_attr_eq!`, `assert_has_buff!`, `assert_has_tag!` |
| `combat_helpers.rs` | ✅ | `deal_damage`, `deal_heal`, `tick`, `get_hp`, `get_mp` |
| `fixtures.rs` | ✅ | Legacy `warrior_attrs()` / `mage_attrs()` preserved |

**Issues**:
- `UnitBuilder::warrior()` stats don't match §7.1 Unit_001 (Might=5 → Attack=10 vs expected ATK=30)
- `UnitBuilder::mage()` stats don't match §7.1 Unit_002 (Intelligence=8 → MaxMp=40 vs expected ATK=40)

### §3.2 Feature Tests (`feature/`)

**Assessment: STRONG** — 40 tests covering all major business flows

| File | Tests | Coverage | Issues |
|------|-------|----------|--------|
| `buff.rs` | 4 | Poison lifecycle, attack buff, cleanse debuff/buff selectivity | Good |
| `consumable.rs` | 3 | HP restore, buff grant, quantity consumption | Good |
| `death.rs` | 4 | Dead hook, lethal damage, dead exclusion from resolve | Good |
| `equipment.rs` | 8 | Equip/unequip, requirements, auto-unequip, trait lifecycle, multi-slot, PersistentTags | Excellent |
| `inventory.rs` | 5 | Transfer, capacity full, pure function (3 variants) | Good |
| `skill.rs` | 3 | MP insufficient, missing tag, cooldown | Good |
| `traits.rs` | 10 | Grant tag, modify attribute, lifecycle, multi-source, non-passive | Excellent |
| `turn.rs` | 3 | Turn end, reset acted, ForceEndTurn | Good |

**Issues**:
- `feature/buff.rs`: Uses custom `warrior_attrs()` instead of `UnitBuilder::warrior().attrs()`
- `feature/skill.rs`: Only 3 tests — missing Heal skill, AoE targeting, multi-condition combinations
- `feature/turn.rs`: Only 3 tests — missing queue rebuild, faction switch, Dead unit filtering

### §3.3 Golden Tests (`golden/`)

**Assessment: GOOD** — 3 snapshot tests with insta

| Test | Status | Notes |
|------|--------|-------|
| `基础战斗_战士攻击哥布林` | ✅ | Snapshot with entity redaction |
| `治疗战斗_角色受伤后治疗` | ✅ | Multi-step: damage → heal |
| `致命伤害_角色死亡` | ✅ | Lethal damage → death |

**Issues**:
- Only 3 golden scenarios — missing buff interaction, equipment interaction, terrain combat
- 6 snapshot files (3 old format `golden_battle__*.snap` + 3 new format `golden__golden_battle__*.snap`) — potential stale snapshots
- Per §5/§8, these are closest to Replay Tests but lack the full YAML schema (Scenario/Initial State/Actions/Expected State)

### §3.4 Legacy Tests (`legacy/`)

**Assessment: EXCELLENT** — 104 tests, most thorough category

| File | Tests | Coverage | Issues |
|------|-------|----------|--------|
| `buff_damage.rs` | 12 | Buff apply/remove, attribute modification, damage calculation combo | Excellent |
| `buff_lifecycle.rs` | 9 | Apply → tick → expire, stun, DoT, HoT, cleanse, shared tags, same-source refresh | Excellent |
| `combat_pipeline.rs` | 19 | EffectHandlerRegistry generate/preview, all handler types, multi-effect skills | Excellent |
| `edge_cases.rs` | 10 | HP full heal, modifier stacking, empty ops, type mismatch, HP=0 boundary, tag idempotency | Good |
| `skill_system.rs` | 20 | Skill slots, cooldowns, conditions cross-module | Excellent |
| `terrain_combat.rs` | 19 | Terrain grid, pathfinding, occupancy, range | Excellent |
| `turn_flow.rs` | 15 | Turn end, queue rebuild, faction switch, acted reset, death filtering | Excellent |

**Issues**:
- `combat_pipeline.rs`: Local `goblin_attrs()` differs from `fixtures::goblin_attrs()` (line 18 comment acknowledges this)
- Several legacy tests duplicate feature tests (buff damage, turn flow)

### §3.5 Rule Tests (`rule/`)

**Assessment: EXCELLENT** — 13 proptest property-based tests

| Category | Tests | Properties Verified |
|----------|-------|---------------------|
| Damage formula | 4 | damage ≥ 1, ignore_def monotonic, multiplier monotonic, terrain monotonic |
| Attributes | 2 | set_base/get roundtrip, fill_vital_resources consistency |
| Tags | 4 | add/has, add/remove, idempotent add, independence |
| Container stacking | 3 | within stack_size, merge consistency, capacity limit |

**Issues**:
- `rules.proptest-regressions` file exists — indicates past test failures (regression tracking working correctly)

### §3.6 Scenario Tests (`scenario/`)

**Assessment: GOOD** — 4 BDD-style scenarios

| Scenario | Style | Notes |
|----------|-------|-------|
| 火球vs骑士 | Given-When-Then | Skill damage + buff application |
| 毒伤战斗 | Given-When-Then | DoT over 4 rounds |
| 地形优势 | Given-When-Then | Damage formula + actual HP verification |
| 击杀触发死亡 | Given-When-Then | Lethal → Dead hook → BattleRecord |

**Issues**:
- Only 4 scenarios — missing equipment interaction, inventory transfer, turn flow scenarios
- No YAML replay files per §8 schema

### §3.7 System Tests (`system/`)

**Assessment: GOOD** — 10 tests verifying individual system behavior

| System | Tests | Notes |
|--------|-------|-------|
| `execute_effects` | 2 | Damage reduces HP, heal restores HP |
| `resolve_status_effects` | 2 | Buff tick decrements, buff expiry removes |
| `equip_item_system` | 2 | Equip occupies slot, unequip clears slot |
| `use_item_system` | 2 | Consumable reduces count, non-consumable blocked |
| `transfer_item_system` | 2 | Transfer success, capacity full failure |
| Trait triggers | 2 | OnAttack/OnKill effects enqueued |

**Issues**:
- Some system tests duplicate feature tests (equip, use_item, transfer)

---

## §4 Test Pyramid Analysis

### Current Distribution

| Category | Count | Percentage | Target |
|----------|-------|------------|--------|
| Unit (proptest) | 13 | 7.5% | 70% |
| Integration (legacy + feature + system) | 154 | 88.5% | 20% |
| Replay/Snapshot (golden) | 3 | 1.7% | 8% |
| BDD/Scenario | 4 | 2.3% | 2% |
| **Total** | **174** | | |

### Assessment

**⚠️ CRITICAL: Test pyramid is severely inverted.**

- **Unit tests**: 7.5% (target: 70%) — only proptest rules cover pure logic
- **Integration tests**: 88.5% (target: 20%) — dominant category, nearly all tests use App
- **Replay tests**: 1.7% (target: 8%) — golden snapshots are closest but not full replay schema
- **E2E tests**: 0% (target: 2%) — missing

**Root cause**: Most tests that should be unit tests (pure function verification) are written as integration tests (spawn App, add plugins, run systems). Examples:
- `legacy/combat_pipeline.rs`: Tests `calculate_damage_from_effect` (pure function) but wraps in App context
- `legacy/buff_damage.rs`: Tests `apply_buff` / `remove_buff` (pure functions) but uses `warrior_attrs()` fixture
- `feature/skill.rs`: Tests `SkillData::can_use` (pure function) but could be standalone

---

## §5 Determinism Verification

| Check | Status | Evidence |
|-------|--------|----------|
| No `std::thread::sleep` | ✅ | Grep: 0 matches |
| No `SystemTime::now()` | ✅ | Grep: 0 matches |
| No `rand::random()` | ✅ | Grep: 0 matches |
| No network calls | ✅ | Grep: 0 matches |
| No file I/O in tests | ✅ | All registries use `register_defaults()`, not file loading |
| `Entity::from_bits()` correct | ✅ | Used consistently for mock entities |
| `insta` snapshots with redaction | ✅ | Entity IDs redacted in golden tests |

**Verdict**: All 174 tests are deterministic.

---

## §6 Standard Test Data Compliance (§7.1)

### Required Standard Units

| Unit | HP | MaxHP | ATK | DEF | SPD | Level |
|------|-----|-------|-----|-----|-----|-------|
| Unit_001 (Warrior) | 100 | 100 | 30 | 10 | 10 | 1 |
| Unit_002 (Mage) | 80 | 80 | 40 | 5 | 12 | 1 |
| Unit_003 (Tank) | 150 | 150 | 20 | 20 | 5 | 1 |

### Actual `UnitBuilder` Stats

| Builder | HP | MaxHP | ATK | DEF | SPD |
|---------|-----|-------|-----|-----|-----|
| `warrior()` | 30 | 30 | 10 | 5 | — |
| `mage()` | 40 | 40 | 4 | 3 | — |
| `goblin()` | 20 | 20 | 6 | 3 | — |

### Assessment

**❌ CRITICAL: Zero tests use Unit_001/Unit_002/Unit_003 standard fixtures.**

- `UnitBuilder::warrior()` is a "基础战士" but with completely different stats (HP=30 vs 100, ATK=10 vs 30)
- No `Unit_002` or `Unit_003` equivalents exist
- All tests use custom attribute values that don't match §7.1

**Impact**: Tests verify implementation behavior, not domain rules as defined in §7.1.

---

## §7 Replay Test Compliance (§5/§8)

### Required Schema (§8)

```yaml
- Scenario: [name]
- Initial State: [unit states]
- Actions: [sequence]
- Expected State: [result]
- Expected Messages: [events]
- Expected Winner: [outcome]
```

### Current Coverage

| Scenario | Golden Snapshot | YAML Replay | Status |
|----------|-----------------|-------------|--------|
| Basic attack | ✅ | ❌ | Partial |
| Heal after damage | ✅ | ❌ | Partial |
| Lethal damage | ✅ | ❌ | Partial |
| Buff interaction | ❌ | ❌ | Missing |
| Equipment interaction | ❌ | ❌ | Missing |
| Terrain combat | ❌ | ❌ | Missing |
| Full turn flow | ❌ | ❌ | Missing |

### Assessment

**❌ CRITICAL: Zero YAML replay tests per §8 schema.**

Golden snapshots are close but:
- Use `insta` snapshot format, not YAML
- Lack explicit Action sequence
- Lack Expected Winner field
- Lack Expected Messages field

---

## §8 Coverage Strategy (§9)

### Required 100% Coverage: Core Domain Rules

| Domain | Module | Tests | Coverage | Status |
|--------|--------|-------|----------|--------|
| Damage | `core/effect` | 19+ (combat_pipeline) | High | ✅ |
| Heal | `core/effect` | 5+ (combat_pipeline, edge_cases) | High | ✅ |
| Death | `character/dead` | 4 (feature/death) + 2 (scenario) | Good | ✅ |
| Buff | `buff/` | 21+ (feature + legacy) | Excellent | ✅ |
| Turn | `turn/` | 18+ (feature + legacy) | Good | ✅ |
| Equipment | `equipment/` | 10+ (feature + system) | Good | ✅ |
| Modifier | `core/attribute` | 13+ (rule + legacy) | Good | ✅ |
| Skill | `skill/` | 23+ (feature + legacy) | Good | ✅ |
| Inventory | `inventory/` | 9+ (feature + system) | Good | ✅ |
| Map/Pathfinding | `map/` | 19 (legacy/terrain_combat) | Excellent | ✅ |
| Trait | `character/trait` | 12+ (feature + system) | Good | ✅ |

### Assessment

**✅ Domain rule coverage is strong.** All 11 core domains have dedicated tests.

---

## §9 Error Testing (§10)

| Error Type | Tests | Examples |
|------------|-------|----------|
| Invalid Input | 3 | `transfer_item` with non-existent ID (NotFound), full container (Full), zero HP |
| Boundary Values | 8 | HP=0, HP=MaxHp, empty queue, empty buffs, empty container |
| Missing Data | 2 | Non-existent skill, non-existent buff |
| Type Mismatch | 1 | Damage handler receives Heal def (edge_cases.rs) |

**Assessment**: Error testing is present but not systematic. No dedicated error test file.

---

## §10 Code Quality Issues

### §10.1 Massive Test Duplication

| Test Pattern | Locations | Count |
|--------------|-----------|-------|
| `buff damage apply/remove` | `feature/buff.rs`, `legacy/buff_damage.rs`, `legacy/buff_lifecycle.rs` | 3 |
| `turn end → queue rebuild` | `feature/turn.rs`, `legacy/turn_flow.rs`, `src/turn/order.rs` (inline) | 3 |
| `skill condition checks` | `feature/skill.rs`, `legacy/skill_system.rs`, `src/skill/domain/mod.rs` (inline), `src/skill/domain/types.rs` (inline) | 4 |
| `equip/unequip flow` | `feature/equipment.rs`, `system/systems.rs` | 2 |
| `transfer items` | `feature/inventory.rs`, `system/systems.rs` | 2 |
| `use consumable` | `feature/consumable.rs`, `system/systems.rs` | 2 |

**Estimated duplication: ~30-40% of tests have near-identical counterparts.**

### §10.2 Inconsistent Test Helpers

| Helper | Locations | Issue |
|--------|-----------|-------|
| `warrior_attrs()` | `fixtures.rs`, `feature/buff.rs`, `feature/death.rs`, `legacy/combat_pipeline.rs` | Multiple local definitions |
| `goblin_attrs()` | `fixtures.rs`, `legacy/combat_pipeline.rs` | Local version differs |
| `spawn_unit()` | `feature/buff.rs`, `feature/death.rs`, `feature/turn.rs`, `golden/golden_battle.rs`, `scenario/scenarios.rs` | 5 different implementations |
| `tick()` / `trigger_resolve()` | `common/combat_helpers.rs`, `feature/buff.rs`, `scenario/scenarios.rs` | Duplicate helpers |

### §10.3 Stale Snapshots

Golden snapshot directory contains both old and new format:
- `golden_battle__*.snap` (old, likely from before module path change)
- `golden__golden_battle__*.snap` (current)

The old snapshots may be stale and should be cleaned up.

### §10.4 proptest Regression File

`rule/rules.proptest-regressions` exists, indicating past test failures. This is **good** — it means regression tracking is working. However, the file should be reviewed to ensure all regressions are fixed.

---

## §11 Issue Statistics

| Priority | Count | Description |
|----------|-------|-------------|
| P0 | 2 | No YAML replay tests (§5/§8); No standard Unit_001/Unit_002/Unit_003 (§7.1) |
| P1 | 2 | Severely inverted test pyramid (88.5% integration vs 7.5% unit); 3 stale snapshot files |
| P2 | 4 | ~35% test duplication (44 duplicate definitions); inconsistent helpers; missing skill/turn/golden tests; inconsistent test approaches |
| P3 | 4 | Missing E2E tests; missing error test file; proptest regression review; missing scenario tests |
| **Total** | **12** | |

---

## §12 Priority Recommendations

### Immediate (P0)

1. **Create YAML replay tests** per §8 schema for at least 5 core scenarios:
   - Basic attack (warrior vs goblin)
   - Skill usage (fireball with burn)
   - Buff interaction (poison → resolve → expire)
   - Equipment change (equip → stat change → unequip)
   - Turn flow (multi-unit, multi-round)

2. **Create Unit_001/Unit_002/Unit_003 fixtures** in `tests/common/fixtures.rs`:
   ```rust
   pub fn unit_001() -> UnitBuilder { /* HP=100, ATK=30, DEF=10, SPD=10 */ }
   pub fn unit_002() -> UnitBuilder { /* HP=80, ATK=40, DEF=5, SPD=12 */ }
   pub fn unit_003() -> UnitBuilder { /* HP=150, ATK=20, DEF=20, SPD=5 */ }
   ```

### Short-term (P1)

3. **Delete stale snapshot files**: Remove 3 old-format `.snap` files from `tests/golden/snapshots/`
4. **Refactor integration tests to pure unit tests** where possible:
   - `calculate_damage_from_effect` tests → remove App wrapper
   - `apply_buff` / `remove_buff` tests → remove App wrapper
   - `SkillData::can_use` tests → remove App wrapper
   - Target: unit test ratio ≥ 50%

### Medium-term (P2)

5. **Consolidate duplicated helpers**: Move common helpers to `tests/common/`:
   - `spawn_unit` → `common/fixtures.rs`
   - `test_buff_registry` / `test_terrain_registry` → `common/app_builder.rs`
   - `enqueue_damage` / `enqueue_apply_buff` → `common/combat_helpers.rs`
   - `make_buff_data` / `make_poison` → `common/fixtures.rs`
6. **Standardize helper signatures**: Unify `spawn_unit`, `put_item_in_backpack`, `enqueue_damage`
7. **Add missing golden scenarios**: buff, equipment, terrain, full turn flow

### Long-term (P3)

8. **Create dedicated error test file**: `tests/error/` with systematic boundary testing
9. **Review proptest regressions**: Ensure all are fixed
10. **Add AI Self-Check annotations** per §13.1
11. **Migrate entry points**: Remove `#[path]` attributes, use standard directory-based module resolution

---

## §13 Summary

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Total Tests | 174 | — | — |
| Unit Tests | 13 (7.5%) | 70% | ❌ Critical |
| Integration Tests | 154 (88.5%) | 20% | ❌ Critical |
| Replay Tests | 3 (1.7%) | 8% | ❌ Missing |
| E2E Tests | 0 (0%) | 2% | ❌ Missing |
| Determinism | 100% | 100% | ✅ |
| Standard Fixtures | 0% | 100% | ❌ Critical |
| YAML Replays | 0 | ≥ 5 | ❌ Critical |
| Domain Coverage | 11/11 | 100% | ✅ |
| Test Duplication | ~35% (44 defs) | 0% | ❌ High |
| Stale Snapshots | 3 files | 0 | ⚠️ Should delete |
| Helper Inconsistencies | 13 patterns | 0 | ⚠️ Should consolidate |

### Overall Score: **3.0 / 5.0** (downgraded from 3.2 due to additional findings)

**Strengths**:
- Excellent domain rule coverage (all 11 core domains tested)
- Strong integration test depth (104 legacy tests cover edge cases thoroughly)
- Good test infrastructure (`common/` provides reusable builders, assertions, helpers)
- Deterministic across all 174 tests
- Property-based testing (proptest) for formulas
- Golden snapshot regression tracking
- 3 pure function tests in inventory.rs demonstrate correct unit test pattern

**Weaknesses**:
- **Severely inverted test pyramid** (88.5% integration vs 7.5% unit)
- **Zero YAML replay tests** per §8 schema
- **Zero standard Unit_001/Unit_002/Unit_003 usage** (§7.1)
- **~35% test duplication** (44 duplicate helper definitions across 13 patterns)
- **No E2E tests** for core main flow
- **Inconsistent test helpers** (7 different `spawn_unit` implementations, inconsistent signatures)
- **3 stale snapshot files** that should be deleted
- **Inconsistent test approaches** (mixed pure function + App in same test)

---

## §14 Additional Findings (Detailed Re-Review)

### §14.1 Stale Snapshot Files

The `tests/golden/snapshots/` directory contains **6 files** — 3 old format + 3 new format:

| Old Format (Stale) | New Format (Current) |
|---------------------|----------------------|
| `golden_battle__基础战斗_战士攻击哥布林.snap` | `golden__golden_battle__基础战斗_战士攻击哥布林.snap` |
| `golden_battle__治疗战斗_角色受伤后治疗.snap` | `golden__golden_battle__治疗战斗_角色受伤后治疗.snap` |
| `golden_battle__致命伤害_角色死亡.snap` | `golden__golden_battle__致命伤害_角色死亡.snap` |

**Issues with old format**:
- `source: tests/golden_battle.rs` (old module path)
- `assertion_line: 104` (hardcoded line number)
- Missing `breakdown: ~` field (struct changed)
- Entity IDs not redacted (4294967294, 4294967295)

**Action**: Delete the 3 old-format snapshot files.

### §14.2 Duplicate Helper Functions (Quantified)

| Helper | Definitions | Locations |
|--------|-------------|-----------|
| `spawn_unit` | **7** | feature/buff.rs, feature/death.rs, feature/turn.rs, golden/golden_battle.rs, scenario/scenarios.rs, legacy/turn_flow.rs, common/fixtures.rs (UnitBuilder::spawn) |
| `test_buff_registry` | **5** | feature/buff.rs, feature/death.rs, golden/golden_battle.rs, scenario/scenarios.rs, system/systems.rs |
| `test_terrain_registry` | **4** | feature/buff.rs, feature/death.rs, golden/golden_battle.rs, scenario/scenarios.rs |
| `tick` | **4** | common/combat_helpers.rs, feature/buff.rs, scenario/scenarios.rs, system/systems.rs (imports from common) |
| `warrior_attrs` | **3** | common/fixtures.rs, feature/buff.rs, feature/death.rs |
| `enqueue_damage` | **3** | feature/death.rs, golden/golden_battle.rs, scenario/scenarios.rs (as `enqueue_skill_damage`) |
| `make_buff_data` | **4** | feature/buff.rs, feature/death.rs, legacy/buff_lifecycle.rs, legacy/edge_cases.rs |
| `make_poison` | **2** | feature/buff.rs, feature/death.rs |
| `spawn_container` | **2** | feature/inventory.rs, system/systems.rs |
| `put_item_in_container` | **2** | feature/inventory.rs, system/systems.rs |
| `put_item_in_backpack` | **2** | feature/equipment.rs, system/systems.rs |
| `enqueue_apply_buff` | **2** | feature/buff.rs, scenario/scenarios.rs |
| `trigger_resolve` | **2** | feature/buff.rs, scenario/scenarios.rs |

**Total duplicate definitions**: **44** (across 13 helper patterns)

### §14.3 Inconsistent Helper Signatures

| Helper | Signature A | Signature B |
|--------|-------------|-------------|
| `spawn_unit` | `(app, builder: UnitBuilder, name: &str) -> Entity` | `(app, faction: Faction, initiative: f32) -> Entity` |
| `put_item_in_backpack` | `(app, entity, def_id) -> u64` (no count) | `(app, entity, def_id, count) -> u64` |
| `enqueue_damage` | `(app, source, target, amount)` | `(app, source, target, amount, is_skill: bool)` |

### §14.4 Entry Point Structure

All entry point files use `#[path]` attributes:
```rust
#[path = "feature/buff.rs"]
mod buff;
#[path = "common/mod.rs"]
mod common;
```

This is **unusual** — standard Rust test structure uses directory-based module resolution. The `#[path]` approach works but makes the module structure fragile.

### §14.5 Pure Function Tests (Unit Test Candidates)

Only **3 tests** in `feature/inventory.rs` are true unit tests (no App wrapper):
- `纯函数transfer_item_成功转移` (line 177)
- `纯函数transfer_item_目标满返回full` (line 217)
- `纯函数transfer_item_不存在返回not_found` (line 261)

These demonstrate the correct pattern for unit testing pure functions.

### §14.6 Inconsistent Test Approaches

| Test | Approach | Issue |
|------|----------|-------|
| `scenario/scenarios.rs::地形优势_森林地形减少伤害` | Mixed: pure function + App | Same test mixes `calculate_damage_from_effect` (pure) with App-based verification |
| `legacy/buff_damage.rs` | Direct `buffs.tick()` | Tests raw `ActiveBuffs::tick()` instead of `resolve_status_effects` system |
| `legacy/combat_pipeline.rs` | Direct function calls | Tests `calculate_damage_from_effect` and `EffectHandlerRegistry` without App |

### §14.7 Missing Test Coverage

| Module | Missing Tests | Priority |
|--------|---------------|----------|
| `feature/skill.rs` | Heal skill, AoE targeting, multi-condition combinations | P2 |
| `feature/turn.rs` | Queue rebuild, faction switch, Dead unit filtering | P2 |
| `golden/` | Buff interaction, equipment interaction, terrain combat, full turn flow | P1 |
| `scenario/` | Equipment interaction, inventory transfer, turn flow scenarios | P2 |
| `system/systems.rs` | Buff interaction with damage, skill execution pipeline | P2 |

### §14.8 proptest Regression File

`rule/rules.proptest-regressions` exists with content:
```
source tests/rule/rules.rs:63: test rule_tests::terrain_defense_reduces_damage failed by shrinking to (10.0, 10.0, 10.0, 1.0, 0.0, 1)
```

This indicates a past regression was caught and fixed. The regression tracking is working correctly.

---

## §15 Self-Check

- [x] All `#[test]` functions counted (174 total)
- [x] All test files reviewed (24 files)
- [x] Test pyramid calculated (7.5/88.5/1.7/0%)
- [x] Determinism verified (no sleep/time/rand)
- [x] Standard fixture compliance checked (0%)
- [x] Replay test compliance checked (0 YAML replays)
- [x] Domain coverage mapped (11/11)
- [x] Duplication patterns identified (44 duplicate definitions)
- [x] Code quality issues documented
- [x] Issue priorities assigned (P0-P3)
- [x] Recommendations actionable and specific
- [x] Stale snapshots identified (3 files)
- [x] Helper signature inconsistencies documented
- [x] Pure function test candidates identified
- [x] Missing test coverage mapped
