---
id: 10-reviews.infrastructure-completeness
title: Review — Infrastructure & Foundation Completeness Pre-Implementation
status: completed
owner: architect
created: 2026-06-16
updated: 2026-06-16
tags:
  - review
  - infrastructure
  - completeness
  - tech-debt
---

# Review Report: Infrastructure & Foundation Completeness

**Reviewer**: @architect
**Scope**: Infrastructure domain documents, foundation data schemas, and ADR-guaranteed dependencies
**Trigger**: All 19 ADRs promoted to `approved` — assessing what gaps remain before implementation begins
**Date**: 2026-06-16

---

## Executive Summary

**All 19 ADRs now `approved`**, 30 domain docs and 33 data schemas are `stable`. The core architecture is fully locked.

This review evaluates: **if implementation (Rust code in `src/`) starts now, which missing infrastructure documents would accumulate technical debt?**

**Verdict: GO — No blocking gaps. 2 preemptive actions recommended before implementation reaches Phase 8 (Infrastructure layer).**

Missing documentation exists but is **not in the critical path** of the first implementation phases (Capabilities 1-15, Business Domains 1-15). All gaps are either well-covered by ADRs already, or only become relevant late in the implementation order.

---

## 1. Assessment: What Exists

### Foundation docs (all in `docs/04-data/foundation/`)

| File | Lines | Status | Assessment |
|------|-------|--------|------------|
| `id_strategy.md` | 325 | ✅ stable | **Complete** — deep coverage of ID format, allocation, lifecycle, cross-ref validation, Registry integration |
| `save_architecture.md` | 327 | ✅ stable | **Complete** — hierarchy, version migration chain, compatibility guarantees |
| `replay_architecture.md` | 430 | ✅ stable | **Complete** — frame format, determinism rules, recording/playback flow |
| `migration_policy.md` | 48 | ⬜ pending | **Skeleton only** — all 7 sections are `TBD` |

### Infrastructure schemas (all in `docs/04-data/infrastructure/`)

| File | Status | Assessment |
|------|--------|------------|
| `registry_schema.md` | ✅ stable | Complete |
| `pipeline_schema.md` | ✅ stable | Complete |
| `replay_schema.md` | ✅ stable | Complete |

### Infrastructure ADRs (all now `approved`)

| ADR | Title | Covers |
|-----|-------|--------|
| ADR-041 | Replay Determinism | Frame format, RNG streams, sync checkpoints |
| ADR-042 | Save Persistence | Trait-based per-Feature serialization, chain migration |
| ADR-043 | Command / Input | 3-tier command architecture, 3-source unification |

---

## 2. Gaps Found

### [LOW] Gap 1: No infrastructure domain docs (`docs/02-domain/`)

Infrastructure features (registry, pipeline, replay, save, input) have **zero domain rule documents**.

**Why LOW**: The ADRs are the design decisions. Data schemas define the structures. Infrastructure features are implementation mechanisms — they enforce rules defined by domain docs, not define their own. For example, `replay_domain.md` would mostly duplicate ADR-041 content. The absence is noted but not harmful.

**Risk of deferral**: None. ADRs + data schemas provide sufficient guidance.

**Recommendation**: Create domain docs only when the infrastructure modules are implemented and rules need to be encoded as testable invariants. Do **not** pre-write them now.

---

### [LOW] Gap 2: `migration_policy.md` is a skeleton

48 lines, all TBD. This is the only foundation doc without real content.

**Why LOW**: Migration policy only matters when:
1. There is actual data to migrate (save files exist)
2. Schema versions change between releases

Neither condition exists yet. Prematurely writing a migration policy would violate the project's 「只解决当前复杂度」 principle.

**Risk of deferral**: Negligible. Deadline: before the first public release.

**Recommendation**: Defer. Revisit when save/load implementation begins.

---

### [MEDIUM] Gap 3: No `input_schema.md` or `command_schema.md`

ADR-043 is approved and defines the 3-tier command architecture (InputEvent → Command → Action). However, there is **no data schema file** in `docs/04-data/infrastructure/` for the input/command system.

The infrastructure directory only has 3 files:
```
infrastructure/
├── registry_schema.md
├── pipeline_schema.md
└── replay_schema.md
```

Missing: `input_schema.md` or `command_schema.md`.

**Why MEDIUM**: Not blocking — the ADR covers the design. But when @feature-developer starts implementing input handling (likely after core combat), they'll need to define Command enums, InputEvent types, and binding structures. Without a schema, this becomes an ad-hoc design choice that might diverge from the ADR's intent.

**Risk of deferral**: Low but non-zero. If implementation reaches input without a schema, the developer will create one inline. It will work, but may need refactoring when the formal schema is written.

**Recommendation**: Create `input_schema.md` before any input code is written — ideally during infra Phase 1 (shared/math/ids) since input is low-hanging fruit.

---

## 3. Implementation Dependency Map

This shows which infrastructure doc gaps are in the **critical path** of each implementation phase:

```
Phase 0: Shared (IDs, Math, Error, Traits)
    → Depends on: id_strategy.md ✅ (complete)
    → Blocking: nothing

Phase 1-4: Capabilities (Tag → Cue, all 15 domains)
    → Depends on: capability schemas ✅ (33 files, all stable)
    → Depends on: ADR-000/001/002 ✅ (approved)
    → Blocking: nothing

Phase 5-7: Business Domains (Tactical → Summon, all 15 domains)
    → Depends on: domain schemas ✅ (all stable)
    → Depends on: ADR-020/021/022/023/030/031/032/033 ✅ (approved)
    → Blocking: nothing

Phase 8: Infrastructure (Registry, Pipeline, Replay, Save, Input)
    → Depends on: infra schemas ⚠️ only 3 of 5 exist (missing input_schema)
    → Depends on: ADR-041/042/043 ✅ (approved)
    → Depends on: migration_policy.md ⬜ skeleton only
    → Blocking: input_schema.md if input is the first infra module built
```

**Key insight**: All 15 capabilities and 15 business domains can be fully implemented without a single infrastructure domain doc. The gaps only surface during Phase 8, which is naturally last in any implementation sequence.

---

## 4. Risk Scoring

| Gap | Severity | When It Bites | Action |
|-----|----------|---------------|--------|
| No infra domain docs | 🟢 None | Never, really — ADRs cover it | No action needed |
| `migration_policy.md` skeleton | 🟢 Low | First public release | Defer |
| No `input_schema.md` | 🟡 Medium | When input module implementation starts | Create before input work |
| Any other foundation doc | ✅ Covered | — | Already done |

---

## 5. Conclusion

### PASS — No blocking infrastructure gaps

All 19 ADRs are now `approved`. The documentation across all three trees (architecture, domain, data) is **sufficient to begin Rust implementation without accumulating architectural debt**.

**The only preemptive action recommended:**
1. **Create `docs/04-data/infrastructure/input_schema.md`** before any input handling code is written — currently the only missing data schema in a path that will be needed.

Everything else (`migration_policy.md`, infrastructure domain docs) can evolve naturally alongside implementation, per the project's complexity governance principle: 「只解决当前复杂度」.
