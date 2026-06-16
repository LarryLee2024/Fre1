---
id: 10-reviews.architecture-vs-governance
title: Review — Architecture Documents vs Governance Compliance
status: completed
owner: code-reviewer
created: 2026-06-16
updated: 2026-06-16
tags:
  - review
  - architecture
  - governance
  - compliance
---

# Code Review Report: Architecture (`docs/01-architecture/`) vs Governance

**Reviewer**: @code-reviewer
**Scope**: `docs/01-architecture/` (1 README + 20 ADR files across 5 directories)
**Standards**: `docs/00-governance/ai-constitution-complete.md` (v5.0) + `docs/00-governance/Fre项目架构设计.md`
**Date**: 2026-06-16

---

## ✅ Checks Passed

| Check | Status |
|-------|--------|
| ADR-000: Feature Module Map — clear mapping to 30 domains + 33 schemas | ✅ Pass |
| ADR-001: Plugin Composition — explicit registration order with Phases | ✅ Pass |
| ADR-002: ECS Communication — Hook/Trigger/Observer/Message 4-tier matrix | ✅ Pass |
| ADR-010: Ability Pipeline — 6-phase pipeline with `.chain()` + Data Law references | ✅ Pass |
| ADR-011: Modifier Pipeline — 4-phase pipeline + aggregator strategies | ✅ Pass |
| ADR-012: Stacking/Trigger/Cue — 3-way separation with clear scope boundaries | ✅ Pass |
| ADR-013: Registry/Hotreload — 2-tier architecture + snapshot mechanism | ✅ Pass |
| ADR-020: Combat Pipeline — 7-phase pipeline + preview/execute separation | ✅ Pass |
| ADR-021: Turn State Machine — dual-layer (battle phase + turn substate) | ✅ Pass |
| ADR-022: Grid/Terrain/Faction — clear ECS mapping (Resource, Tag, Component) | ✅ Pass |
| ADR-023: Spell/Reaction — spell reuses Ability Pipeline, reaction as event chain | ✅ Pass |
| ADR-030: Progression/Inventory — XP through Modifier Pipeline + equipment Modifier chain | ✅ Pass |
| ADR-031: Party/Camp/Rest — Party Resource + Formation + CampPhase FSM | ✅ Pass |
| ADR-032: Economy/Crafting — currency/transaction/crafting via Ability Pipeline | ✅ Pass |
| ADR-033: Narrative/Quest — passive event listeners + config-driven dialogues | ✅ Pass |
| ADR-040: Data Flow Ownership — clear ownership matrix + lawful modification paths | ✅ Pass |
| ADR-041: Replay Determinism — 4-stream RNG + ReplayFrame + SyncCheckpoint | ✅ Pass |
| ADR-042: Save/Persistence — per-Feature SaveLoad trait + chain migration | ✅ Pass |
| ADR-043: Command/Input — 3-tier command architecture + 3-source unification | ✅ Pass |
| All ADRs reference upstream domain docs and data schemas | ✅ Pass |
| All ADRs include Design, Forbidden, Boundary, Consequences sections | ✅ Pass |
| ADR cross-referencing (ADR-010 → ADR-011 → ADR-020) is present | ✅ Pass |

---

## ❌ Issues Found

### [CRITICAL] 1. Layer Model Mismatch: "七层" vs "DDD三层 + 横切四层"

**Location**: `docs/01-architecture/README.md` §2 (七层架构总图) vs `docs/00-governance/Fre项目架构设计.md` §三 (源码架构总览)

**Constitutional Rule**: 宪法 §2.1 定义了 DDD 纵向三层（Shared/Core/Infrastructure）+ 横切四层（App/Content/Tools/Modding）的双轴架构体系。`Fre项目架构设计.md` §三 详细给出了 `src/shared/`, `src/core/capabilities/`, `src/core/domains/`, `src/infra/`, `src/app/`, `src/content/`, `src/tools/`, `src/modding/` 的完整目录结构。

**Violation**: Architecture README §2 defines a completely different "七层" model:
- Layer 1: Tactical Foundation (Grid, Terrain, Faction, TurnPhase)
- Layer 2: Capability System (Tag, Attribute, Modifier, etc.)
- Layer 3: Combat Execution (Combat, Spell, Reaction)
- Layer 4: Progression & Economy
- Layer 5: Party & Camp
- Layer 6: Narrative & Content
- Layer 7: Infrastructure & Cross-cutting

The constitution places ALL of Layer 1-6 inside `src/core/` (as capabilities + domains), while the architecture README places them as flat `src/<feature>/` directories. These two models produce **completely different directory structures** — one nests features under `src/core/capabilities/<feature>/` and `src/core/domains/<feature>/`, the other places them directly under `src/<feature>/`.

**Why This Is Critical**: At implementation time, a developer cannot tell whether to put `combat/` under `src/core/domains/combat/` (per constitution) or `src/combat/` (per architecture README). This directly blocks all feature development.

**Recommendation**: The architecture documents must reconcile with the constitution. Specifically:
- Either update the constitution's directory structure to match the flat 7-layer approach (if that is the intended design)
- Or update the architecture README to nest features within `src/core/capabilities/` and `src/core/domains/` as the constitution specifies
- This decision needs an ADR with explicit superseding of the constitution's directory layout

**Severity**: CRITICAL — P0 (宪法 §18.2: "双轴边界严重突破")

---

### [CRITICAL] 2. `common/` Directory Banned by Constitution

**Location**: `docs/01-architecture/ADR-000-feature-module-map.md` §Layer 7 table — `common/` listed as a Layer 7 feature

**Constitutional Rule**:
- 宪法 §16.1 🟥: "绝对禁止创建 `utils.rs`、`helpers.rs`、`common.rs` 垃圾桶文件"
- 宪法 §21 红线总览 #1 🟥: "禁止创建 `utils.rs`、`helpers.rs`、`common.rs` 垃圾桶文件"
- `Fre项目架构设计.md` §16.1 相同规则

**Violation**: ADR-000 lists `common/` as a Layer 7 infrastructure feature: "纯工具函数，零业务逻辑." This directly creates the `common/` directory that the constitution explicitly bans.

**Why This Is Critical**: The constitution's prohibition of `common/` directories is a non-negotiable red-line rule. Having architecture documents promote its creation would cause immediate compliance failures at implementation time. `common/` directories become "junk drawers" where unrelated utility code accumulates, eventually growing into a maintenance burden.

**Recommendation**: Remove `common/` from ADR-000. Distribute utility functions to the `shared/` module (per constitution §2.4), or place them in the Feature they serve. If `shared/` is not part of the 7-layer model, clearly define where "zero-business-logic utility code" lives.

**Severity**: CRITICAL — P0 (红线禁令)

---

### [HIGH] 3. ADR Status vs Actual State Mismatch

**Location**: `docs/01-architecture/README.md` §9 (架构决策索引) — full table vs actual file states

**Constitutional Rule**: 宪法 §19.3 要求架构版本管理和状态追踪。

**Violation**: The README §9 "Architecture Decision Index" table lists ALL 20 ADRs as "⬜ Pending" status, but the file headers of all ADRs consistently say `status: proposed`. The Appendix B "文件状态追踪" also lists the same ADRs as "⬜ pending — architect". This means:
- There is inconsistency between `pending` and `proposed` — are they synonyms or different states?
- The upstream session context notes all ADRs were "written as proposed status" — but the README says pending
- No clear definition of what status transitions apply

**Recommendation**: Define a clear status lifecycle for ADRs (draft → proposed → approved → superseded) and apply it consistently across all files. Either update the README tables to match the file headers (`proposed`), or update file headers to `pending`.

**Severity**: HIGH — P1 (跨文件状态不一致)

---

### [HIGH] 4. README Directory Structure vs ADR-000 Directory Structure Conflict

**Location**: `docs/01-architecture/README.md` §8 (目录结构总览) vs `docs/01-architecture/ADR-000-feature-module-map.md` §Module Design

**Constitutional Rule**: 宪法 §16.1: "单一文件单一主题...优先按业务主题拆分文件"

**Violation**: The README §8 shows the source directory structure as:
```
src/
├── common/       (Layer 7)
├── input/        (Layer 7)
├── registry/     (Layer 7)
├── pipeline/     (Layer 7)
├── replay/       (Layer 7)
├── save/         (Layer 7)
├── grid_map/     (Layer 1)
├── terrain/      (Layer 1)
...
```

But ADR-000 §Module Design defines a different internal structure per Feature:
```
src/<feature>/
├── mod.rs
├── plugin.rs
├── components.rs
├── systems.rs
├── events.rs
├── resources.rs
├── api.rs
└── internal/
```

The README's directory structure doesn't show the per-Feature internal structure consistently. Additionally, the README uses `systems.rs` (singular) at the Feature level, which matches the per-feature `systems.rs` file pattern — this is acceptable per the constitution since it's per-Feature, not global.

**Recommendation**: Add a note in README §8 referencing ADR-000's per-Feature internal structure. The README directory should either remove per-Feature internal files or show them consistently.

**Severity**: HIGH — P1

---

### [MEDIUM] 5. Layer 7 Domain Documents Missing

**Location**: `docs/01-architecture/README.md` §3.7 — Layer 7 Infrastructure features

**Constitutional Rule**: 宪法 §2.6-2.7 define clear boundaries for cross-cutting layers.

**Violation**: Layer 7 features `registry/`, `pipeline/`, `replay/`, `save/`, `input/`, `common/` are listed with "--" for their corresponding domain document and data schema (partially). Several items have no domain document reference at all. Given that 30 domain documents exist in `docs/02-domain/`, the absence of domain documents for infrastructure Layer 7 features means:
- `replay/` has no `replay_domain.md` 
- `save/` has no `save_domain.md`
- `input/` has no `input_domain.md`
- `pipeline/` has no `pipeline_domain.md`
- `registry/` has no `registry_domain.md`
- `common/` has no domain doc (it should not exist anyway per Issue #2)

While some have data schemas in `docs/04-data/infrastructure/`, the domain rules for these cross-cutting features are not defined.

**Recommendation**: Define domain documents for replay, save, input, pipeline, and registry in `docs/02-domain/`, or add ADRs in architecture that serve the same normative purpose with explicit references.

**Severity**: MEDIUM — P2

---

### [MEDIUM] 6. Constitution Reference Inconsistency

**Location**: Various ADR files reference `.trae/rules/` documents

**Constitutional Rule**: 宪法 §1.5 P0 顶层铁则: Feature First, Data Driven First, Replay First.

**Violation**: The architecture ADRs reference `.trae/rules/` documents ("架构规则.md", "ECS规则.md", "SRPG专项规则.md", "编码规则.md") as constitutional authorities. However:
- The constitution (v5.0) itself is at `docs/00-governance/ai-constitution-complete.md`
- `Fre项目架构设计.md` is at `docs/00-governance/Fre项目架构设计.md`
- The `.trae/rules/` files are described as supplementary "项目规则集" in the project README

This creates a dual-authority problem: when the `.trae/rules/` files and the constitution/架构设计 disagree (as they do on the layer model), ADRs reference the `.trae/rules/` files, which creates implicit authority for the wrong source.

**Recommendation**: Update ADR reference sections to cite `docs/00-governance/` documents as primary authority, with `.trae/rules/` as secondary. Add a note about conflict resolution: when governance documents disagree, the constitution (`ai-constitution-complete.md` v5.0) takes precedence.

**Severity**: MEDIUM — P2

---

### [LOW] 7. Minor Status Tracking Inconsistencies

**Location**: `docs/01-architecture/README.md` §Appendix B — File Status Tracking

**Constitutional Rule**: 宪法 §19.3 要求架构版本和状态管理。

**Violation**: 
- Appendix B has two columns "status" and "完成日期" but no "version" column
- The file metadata header uses `status: stable`, `status: proposed` — but the appendix uses "✅ stable" and "⬜ pending" — two different status taxonomies in the same document
- ADR-033 filename has `narrative-quest.md` in the appendix but the actual file is `ADR-033-narrative-quest-summon.md`

**Recommendation**: Fix the ADR-033 filename reference. Standardize status values between the header metadata and the appendix table.

**Severity**: LOW — P3

---

## 📋 Summary

| Severity | Count | Issues |
|----------|-------|--------|
| **CRITICAL** | 2 | Layer model mismatch (7-layer vs DDD三层+横切四层), `common/` directory banned |
| **HIGH** | 2 | ADR status inconsistency, README vs ADR-000 directory conflict |
| **MEDIUM** | 2 | Missing Layer 7 domain docs, constitution reference inconsistency |
| **LOW** | 1 | Minor status tracking inconsistencies |

---

## 🎯 Conclusion

### FAIL — Critical issues found

The architecture documents (`docs/01-architecture/`) have two blocking issues that must be resolved:

1. **Layer model mismatch** between the architecture's "7-layer" model and the constitution/架构设计's "DDD三层 + 横切四层" model. This produces fundamentally incompatible directory structures. Until this is resolved, developers cannot know where to place new modules.

2. **`common/` directory** listed in ADR-000 directly violates the constitution's red-line prohibition on `common/`, `utils/`, and `helpers/` directories.

### Required Actions

- **P0**: Reconcil the 7-layer vs DDD三层+横切四层 model — either via an explicit ADR superseding the constitution's directory layout, or by restructuring the architecture docs to nest features under `src/core/capabilities/` and `src/core/domains/`
- **P0**: Remove `common/` from ADR-000 and distribute utility code to `shared/` or equivalent
- **P1**: Standardize ADR status values across all files and the README appendix
- **P2**: Add domain documents for Layer 7 features (replay, save, input, pipeline, registry)
- **P2**: Fix constitution reference hierarchy — primary = `docs/00-governance/`, secondary = `.trae/rules/`

### Next Steps

After fixing Critical issues → call **@code-reviewer** for re-review of architecture docs.
For data ownership issues → call **@data-architect**.
