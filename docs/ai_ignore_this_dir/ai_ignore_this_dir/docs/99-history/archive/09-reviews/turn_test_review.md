# Turn Module Test Review

**Module**: `src/turn/` (order.rs, state.rs)
**Review Date**: 2026-06-10
**Reviewer**: Sisyphus (Automated)
**Test Spec Version**: 3.1 (Testing Constitution)

---

## §1 Scope

### Files Reviewed

| File | Lines | Purpose |
|------|-------|---------|
| `src/turn/mod.rs` | 41 | Plugin registration, resource init, system registration |
| `src/turn/order.rs` | 411 | TurnOrder, TurnState, AiTimer, NeedsResolve, messages, init/turn_end systems |
| `src/turn/state.rs` | 42 | AppState, TurnPhase (SubState), GameSet |
| `tests/legacy/turn_flow.rs` | 378 | External: cross-module turn flow integration tests |

### Test Count

| Location | Count |
|----------|-------|
| `order.rs` inline | 12 |
| `state.rs` inline | 0 |
| `tests/legacy/turn_flow.rs` | 15 |
| **Total** | **27** |

---

## §2 Review Criteria

Evaluated against `docs/test_spec.md` v3.1:

1. **Invariant Coverage** (§9): Every domain invariant mapped and tested
2. **Test Pyramid** (§6): Unit > Integration > E2E ratio
3. **Determinism** (§4): No time/sleep/rand依赖
4. **Schema Compliance** (§3): State machine correctness, message contracts
5. **Standard Test Data** (§7.1): Unit_001/Unit_002/Unit_003 usage
6. **Replay Tests** (§5/§8): Mandatory replay test presence
7. **AI Self-Check** (§13.1): Annotation presence

---

## §3 Domain Invariant Coverage

### Identified Invariants (INV-TURN-XXX)

| ID | Invariant | Source | Tested |
|----|-----------|--------|--------|
| INV-TURN-001 | `TurnOrder::build()` sorts by Initiative descending | order.rs:38-41 | ✅ inline:211, legacy:52 |
| INV-TURN-002 | `TurnOrder::build()` is stable (preserves order for equal initiative) | order.rs:40 | ✅ inline:221, legacy:61 |
| INV-TURN-003 | `TurnOrder::build()` handles empty input | order.rs:38-41 | ✅ inline:230, legacy:69 |
| INV-TURN-004 | `TurnOrder::current_unit()` returns entity at `current_index` | order.rs:45-47 | ✅ inline:236, legacy:75 |
| INV-TURN-005 | `TurnOrder::advance()` increments index, returns `None` when exhausted | order.rs:50-53 | ✅ inline:236, legacy:75 |
| INV-TURN-006 | `TurnOrder::current_faction()` returns faction of current unit | order.rs:56-60 | ❌ No dedicated test |
| INV-TURN-007 | `turn_end_on_enter()` increments `turn_number` | order.rs:167 | ✅ inline:268, legacy:96 |
| INV-TURN-008 | `turn_end_on_enter()` resets all unit `acted` flags | order.rs:171-173 | ✅ inline:377, legacy:253,283 |
| INV-TURN-009 | `turn_end_on_enter()` sets `needs_resolve = true` | order.rs:176 | ✅ inline:293, legacy:116 |
| INV-TURN-010 | `turn_end_on_enter()` rebuilds queue filtering `Dead` units | order.rs:179-184 | ✅ legacy:314,352 |
| INV-TURN-011 | `turn_end_on_enter()` updates `current_faction` to first queue unit | order.rs:187-191 | ✅ legacy:150 |
| INV-TURN-012 | `turn_end_on_enter()` resets `AiTimer` | order.rs:194 | ✅ legacy:193 |
| INV-TURN-013 | `turn_end_on_enter()` transitions to `TurnPhase::SelectUnit` | order.rs:203 | ✅ inline:314, legacy:131 |
| INV-TURN-014 | `TurnStarted` message contains correct turn number | order.rs:116-118 | ✅ inline:337 |
| INV-TURN-015 | `TurnEnded` message contains correct turn number | order.rs:156-158 | ✅ inline:343 |
| INV-TURN-016 | `ForceEndTurn` message is consumable without error | order.rs:164 | ✅ inline:349,355 |
| INV-TURN-017 | `AppState` defaults to `MainMenu` | state.rs:8-9 | ❌ No test |
| INV-TURN-018 | `TurnPhase` defaults to `SelectUnit` | state.rs:19-20 | ❌ No test |
| INV-TURN-019 | `TurnPhase` is `SubState` of `AppState::InGame` | state.rs:15-16 | ❌ No test (Bevy framework guarantee) |
| INV-TURN-020 | `GameSet` variants: Camera, Map, Unit, Ui | state.rs:37-41 | ❌ No test (Bevy framework guarantee) |

### Coverage Summary

- **Total Invariants**: 20
- **Covered**: 14 (70.0%)
- **Missing**: 6 (INV-TURN-006, 017, 018, 019, 020)

Note: INV-TURN-019 and INV-TURN-020 are Bevy framework guarantees (SubState derive, SystemSet derive). Testing these is optional per §1.1 (Bevy engine functionality verification is out of scope).

**Effective Coverage**: 14/18 = 77.8% (excluding 2 framework invariants)

---

## §4 Test Pyramid

| Level | Count | Percentage |
|-------|-------|------------|
| Unit (inline) | 12 | 44.4% |
| Integration (legacy) | 15 | 55.6% |
| Feature (external) | 0 | 0% |
| E2E / Replay | 0 | 0% |
| **Total** | **27** | |

### Assessment

- **Inline/External ratio**: 12:15 = 0.8:1 — **inverted** (§6 recommends > 1:1)
- **Missing Replay Tests**: 0/27 — **CRITICAL GAP** per §5/§8
- **Missing Feature Tests**: No `tests/feature/turn.rs` exists
- **Integration-heavy**: 55.6% integration is high; inline unit tests should dominate

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

**Note**: `AiTimer` uses `Timer::from_seconds(0.8, ...)` and one test (`legacy:193`) calls `timer.tick(Duration::from_secs(5))` — this is deterministic (controlled duration, not wall clock).

**Verdict**: All 27 tests are deterministic.

---

## §6 Schema Compliance

### State Machine

| Transition | From | To | Trigger | Tested |
|------------|------|----|---------|--------|
| Game start | — | `TurnPhase::SelectUnit` | `OnEnter(AppState::InGame)` | ❌ No dedicated test |
| Turn end | Any | `TurnPhase::TurnEnd` | Queue exhausted / ForceEndTurn | ✅ (implicit in turn_end tests) |
| After turn end | `TurnPhase::TurnEnd` | `TurnPhase::SelectUnit` | `turn_end_on_enter` | ✅ inline:314, legacy:131 |

### Message Contracts

| Message | Field | Type | Tested |
|---------|-------|------|--------|
| `TurnStarted` | `turn` | `u32` | ✅ inline:337 |
| `TurnEnded` | `turn` | `u32` | ✅ inline:343 |
| `ForceEndTurn` | (unit struct) | — | ✅ inline:349 |

### Resource Defaults

| Resource | Default | Tested |
|----------|---------|--------|
| `TurnState` | `current_faction: Player, turn_number: 1` | ❌ No explicit default test |
| `TurnOrder` | `queue: [], current_index: 0, turn_number: 0` | ❌ No explicit default test |
| `AiTimer` | `timer: 0.8s Once` | ❌ No explicit default test |
| `NeedsResolve` | `false` | ❌ No explicit default test |

---

## §7 Missing Tests

### P0 — Missing Replay Tests

Per §5/§8, Replay Tests are mandatory and highest priority.

| Scenario | Status |
|----------|--------|
| Basic turn flow (3 units, 2 rounds) | ❌ No replay test |
| ForceEndTurn mid-turn | ❌ No replay test |
| Unit death during turn rebuilds queue | ❌ No replay test |
| All enemies dead → only allies in queue | ❌ No replay test |

**Recommendation**: Create `tests/replay/turn_replay.rs` with deterministic replay tests for core turn scenarios.

### P1 — Missing Invariant Tests

| ID | Invariant | Gap |
|----|-----------|-----|
| INV-TURN-006 | `TurnOrder::current_faction()` | No test verifying faction lookup from queue |
| INV-TURN-017 | `AppState` defaults to `MainMenu` | No test (optional per §1.1) |
| INV-TURN-018 | `TurnPhase` defaults to `SelectUnit` | No test |

**Recommendation**: Add to `tests/feature/turn.rs`:
```rust
#[test]
fn turn_order_current_faction_returns_first_unit_faction() {
    // Spawn Player unit with high initiative, Enemy with low
    // Verify current_faction() returns Player
}

#[test]
fn app_state_defaults_to_main_menu() {
    assert_eq!(AppState::default(), AppState::MainMenu);
}

#[test]
fn turn_phase_defaults_to_select_unit() {
    assert_eq!(TurnPhase::default(), TurnPhase::SelectUnit);
}
```

### P2 — Missing Standard Test Data

Per §7.1, tests should use Unit_001/Unit_002/Unit_003 fixtures. Current tests use:
- Custom `spawn_unit(app, faction, initiative)` helper
- `Attributes::default()` with manual `set_base()` calls
- `Entity::from_bits()` for pure unit tests

**No tests use the standard Unit_001/Unit_002/Unit_003 fixtures.**

### P3 — Missing Edge Cases

| Edge Case | Status |
|-----------|--------|
| `TurnOrder::build()` with single unit | ❌ Not tested |
| `TurnOrder::advance()` on empty queue | ❌ Not tested |
| `turn_end_on_enter()` with zero units alive | ❌ Not tested |
| `TurnPhase` state transitions (SelectUnit → MoveUnit → ...) | ❌ Not tested |
| `ForceEndTurn` message during non-TurnEnd phase | ❌ Not tested |

---

## §8 Code Quality Issues

### Duplicate Test Logic

**`order.rs` inline vs `legacy/turn_flow.rs`**: 8 tests are exact or near-exact duplicates:

| Test | order.rs | legacy/turn_flow.rs | Overlap |
|------|----------|---------------------|---------|
| 行动队列_按initiative降序排列 | `行动队列_按initiative降序排列` | `行动队列_按initiative降序排列` | Exact |
| 行动队列_相同initiative稳定排序 | `行动队列_相同initiative稳定排序` | `行动队列_相同initiative稳定排序` | Exact |
| 行动队列_空队列 | `行动队列_空队列` | `行动队列_空队列` | Exact |
| 行动队列_current_unit和advance | `行动队列_current_unit和advance` | `行动队列_current_unit和advance` | Exact |
| 回合结束_重建队列并增加回合数 | `回合结束_重建队列并增加回合数` | `回合结束_重建队列并增加回合数` | Near |
| 回合结束_needs_resolve标记设置 | `回合结束_needs_resolve标记设置` | `回合结束_needs_resolve标记设置` | Near |
| 回合结束_总是切换到_select_unit | `回合结束_进入后总是切换到_select_unit()` | `回合结束_总是切换到_select_unit()` | Near |
| 回合结束_重置单位行动状态 | `回合结束_重置单位行动状态` | `回合结束_重置所有单位acted状态` | Near |

**8 tests are duplicated.** Recommend keeping inline tests for pure unit logic (TurnOrder::build, advance) and legacy tests for integration (turn_end_on_enter with App).

### `state.rs` Has Zero Tests

The 42-line `state.rs` file defines `AppState`, `TurnPhase`, and `GameSet` but contains no inline tests. While these are mostly type definitions, the default values and SubState relationship should be verified.

### `current_faction` Unimplemented in Tests

`TurnOrder::current_faction()` (order.rs:56-60) is a public API method that queries the ECS to get the faction of the current unit. No test covers this method, despite it being used in the turn flow.

---

## §9 Issue Statistics

| Priority | Count | Description |
|----------|-------|-------------|
| P0 | 4 | Missing Replay Tests for core turn scenarios |
| P1 | 3 | Missing invariant tests (current_faction, defaults) |
| P2 | 1 | No standard Unit_001/Unit_002/Unit_003 usage |
| P3 | 5 | Missing edge cases (empty queue, zero units, etc.) |
| Quality | 8 | Duplicate tests across inline and legacy |
| Quality | 1 | state.rs has zero tests |
| **Total** | **22** | |

---

## §10 Priority Recommendations

### Immediate (P0)
1. Create `tests/replay/turn_replay.rs` with deterministic replay tests:
   - Basic 3-unit turn flow over 2 rounds
   - ForceEndTurn mid-turn
   - Unit death queue rebuild
   - All enemies dead scenario

### Short-term (P1)
2. Add `TurnOrder::current_faction()` test
3. Add default value tests for `AppState`, `TurnPhase`, `TurnState`, `TurnOrder`, `AiTimer`, `NeedsResolve`
4. Deduplicate 8 overlapping tests (keep pure unit tests inline, integration tests in legacy)

### Medium-term (P2)
5. Migrate `spawn_unit` helper to use `UnitBuilder::warrior()` / `UnitBuilder::mage()` fixtures
6. Add standard Unit_001/Unit_002/Unit_003 fixture usage

### Long-term (P3)
7. Add edge case tests (empty queue advance, zero units alive, single unit, phase transitions)
8. Add AI Self-Check annotations per §13.1
9. Add `tests/feature/turn.rs` for feature-level tests

---

## §11 Summary

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Total Tests | 27 | — | — |
| Invariant Coverage | 77.8% (14/18 effective) | 100% | ⚠️ 4 gaps |
| Replay Tests | 0 | ≥ 1 per scenario | ❌ Missing |
| Determinism | 100% | 100% | ✅ |
| Schema Compliance | Good | Good | ✅ |
| Test Pyramid | 44% unit / 56% int / 0% feature | 70/20/8/2 | ⚠️ Inverted |
| Standard Fixtures | 0% | 100% | ❌ Missing |
| Duplicate Tests | 8 | 0 | ⚠️ Cleanup needed |

### Overall Score: **3.5 / 5.0**

**Strengths**:
- Excellent turn_end_on_enter coverage (7 tests from different angles)
- Good integration tests covering multi-unit scenarios (faction switch, death filtering, acted reset)
- Deterministic, well-structured test helpers
- Message contracts verified (TurnStarted, TurnEnded, ForceEndTurn)

**Weaknesses**:
- Zero Replay Tests (§5/§8 critical gap)
- 8 duplicate tests across inline and legacy
- `current_faction()` untested public API
- `state.rs` has zero tests
- Inverted test pyramid (integration > unit)
- No standard Unit_001/Unit_002/Unit_003 fixtures

---

## §12 Self-Check

- [x] All `#[test]` functions counted (27 total)
- [x] All domain invariants identified from source (20 total)
- [x] Coverage matrix complete (14/18 effective covered)
- [x] Test pyramid calculated (44/56/0/0%)
- [x] Determinism verified (no sleep/time/rand)
- [x] Schema compliance checked (state machine, messages)
- [x] Missing tests documented with specific file/line references
- [x] Duplicate tests identified with overlap matrix
- [x] Issue priorities assigned (P0-P3)
- [x] Recommendations actionable and specific
