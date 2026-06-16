---
id: 10-reviews.cross-layer-consistency
title: Review — Cross-Layer Consistency (Architecture × Domain × Data)
status: completed
owner: code-reviewer
created: 2026-06-16
updated: 2026-06-16
tags:
  - review
  - cross-layer
  - consistency
  - governance
---

# Code Review Report: Cross-Layer Consistency

**Reviewer**: @code-reviewer
**Scope**: Consistency between `docs/01-architecture/`, `docs/02-domain/`, `docs/04-data/`
**Standards**: `docs/00-governance/ai-constitution-complete.md` (v5.0) + `docs/00-governance/Fre项目架构设计.md`
**Date**: 2026-06-16

This report focuses on **gaps between the three document trees** — inconsistencies where one tree says one thing and another says something different, or where information expected in one tree is missing from another.

---

## ✅ Checked and Consistent

| Cross-Layer Check | Status |
|-------------------|--------|
| Domain-to-Data mapping table in domain README §4 (30 domains → 30+ schemas) | ✅ Consistent |
| Architecture README §3 Feature Module Map references correct domain docs and schemas | ✅ Consistent |
| Domain docs reference upstream architecture (七层架构) | ✅ Consistent |
| Data schemas reference their defining domain document | ✅ Consistent |
| Data Law 005 (Effect唯一入口) aligns with ADR-010 (Ability Pipeline) and domain docs | ✅ Consistent |
| Data Law 009 (Cue桥梁) aligns with ADR-012 (Cue separation) and cue_domain | ✅ Consistent |
| Data Law 012 (域间禁止直接数据引用) aligns with ADR-040 (Data Ownership) | ✅ Consistent |
| ADR cross-references between pipelines (ADR-010→ADR-011→ADR-020) are present | ✅ Consistent |
| All three READMEs reference each other as upstream/downstream inputs | ✅ Consistent |
| 15 Capabilities + 15 Business Domains partition is consistent across all three trees | ✅ Consistent |

---

## ❌ Inconsistencies Found

### [CRITICAL] 1. Layer Model: Three Different Structures Compete

**Location**: All three trees + constitution + architecture design doc

**Constitutional Rule**: 宪法 §2.1 (DDD三层+横切四层), `Fre项目架构设计.md` §三 (shared/core/infra/app/content/tools/modding)

**Violation**: There are at least **three competing directory structure models** across the governance + architecture documents:

**Model A — Constitution v5.0 / Fre项目架构设计.md (the "canonical" model)**:
```
src/
├── shared/       (L0: 底层原子层)
├── core/         (L1: 领域规则层)
│   ├── capabilities/  (15 domains)
│   ├── domains/       (15 domains)
│   └── mod_api/
├── infra/        (L2: 技术实现层)
├── app/          (横切1)
├── content/      (横切2)
├── tools/        (横切3)
└── modding/      (横切4)
```

**Model B — Architecture README §8 (the "7-layer" model)**:
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
├── faction/      (Layer 1)
├── turn_phase/   (Layer 1)
├── tag/          (Layer 2)
├── attribute/    (Layer 2)
├── modifier/     (Layer 2)
... (flat, no shared/core/infra nesting)
```

**Model C — Domain README §2 dependency graphs (implicit)**: 
The dependency graphs show capability and domain relationships but don't commit to a directory structure. They describe logical dependencies only.

**Why This Is Critical**: A developer working on the `tag` feature needs to know:
- Model A says: put it at `src/core/capabilities/tag/`
- Model B says: put it at `src/tag/` (flat, Layer 2)
- These produce **different module paths** that affect Rust `use` statements, Plugin registration, import paths, and module visibility

Additionally:
- The constitution says `app/` is a top-level module (composition root)
- The architecture README has no `app/` — its §8 directory starts with `common/`, `input/`, etc.
- The architecture README §6 Plugin registration shows an inline `App::new().add_plugins(...)` pattern in `lib.rs` — but where is `app/`'s Plugin composition root?

**Recommendation**: This is the single most important decision to resolve before any code is written. Options:
1. **Adopt Model A** (Constitutional model): all features nest under `src/core/capabilities/` or `src/core/domains/`
2. **Adopt Model B** (Architecture README model): flat `src/<feature>/` with features distributed across conceptual layers
3. **Hybrid**: Flat `src/<feature>/` at the filesystem level, but Feature Cargo.toml dependencies enforce layer boundaries

Whichever is chosen, update ALL three trees to agree. A single `docs/01-architecture/` update superseding the constitution's directory layout would work, but it must be done explicitly via ADR.

**Severity**: CRITICAL — P0 (§18.2: "双轴边界严重突破")

---

### [CRITICAL] 2. Cross-Layer Feature Coverage Gap

**Location**: All three trees combined

**Constitutional Rule**: 宪法 §3.1 (Capabilities/Domains 双轴架构)

**Violation**: Counting features across all three trees reveals a mismatch:

| What's counted | Architecture README | Domain README | Data README |
|---------------|-------------------|---------------|-------------|
| Capabilities | 15 (Layer 2) | 15 (capabilities) | 15 (capabilities/) |
| Business Domains | 14 (Layer 3-6: combat, spell, reaction, progression, inventory, economy, crafting, summon, party, camp_rest, narrative, quest + terrain + faction) | 15 (terrain is separate from tactical, faction is separate) | 15 (domains/) |
| Layer 1 Foundation | 4 (grid_map, terrain, faction, turn_phase, movement) | 2 (tactical, terrain... no separate grid_map or turn_phase domain) | 2 (tactical_schema) |
| Layer 7 Infrastructure | 6 (common, input, registry, pipeline, replay, save) | 0 (not covered) | 3 (infrastructure/) |

Key gaps:

1. **Architecture has features with NO domain doc**: `grid_map/`, `turn_phase/`, `movement/`, `common/`, `input/`, `registry/` have no corresponding domain rule document. The domain README lists only 30 files, but the architecture needs at least 36+ features.

2. **Architecture splits `tactical_domain` into 4 features** (grid_map, terrain, faction, turn_phase, movement) but domain README lists only `tactical_domain.md`, `terrain_domain.md`, `faction_domain.md`. The `grid_map`, `turn_phase`, and `movement` features have no dedicated domain document.

3. **Data schemas cover 33 items** but architecture needs schemas for `common/`, `input/`, and potentially more.

**Why This Is Critical**: When a developer needs to implement `turn_phase`, they have:
- Architecture: yes, it's a Layer 1 feature
- Domain: no domain rules defined for turn phases (only in `tactical_domain.md` mixed with grid/movement)
- Data: `tactical_schema.md` presumably covers it, but not explicitly
- The `movement` feature has no independent domain rules at all

**Recommendation**: Either:
- Consolidate architecture features to exactly match the 30 domain documents (merge grid_map/turn_phase/movement into `tactical` capability)
- Or expand domain documents to cover all architecture features (split `tactical_domain.md` into `grid_map_domain.md`, `turn_phase_domain.md`, `movement_domain.md`)
- Same for Layer 7: create domain docs for replay, save, input, pipeline, registry

**Severity**: CRITICAL — P0 (架构违规 — "Feature 无对应领域规则")

---

### [HIGH] 3. Domain Filename vs Architecture Module Name Mismatch

**Location**: `docs/02-domain/` filenames vs `docs/01-architecture/ADR-000` module names

**Constitutional Rule**: 宪法 §16.1 — "单一文件单一主题...优先按业务主题拆分文件"

**Violation**: The architecture ADR-000 assigns Feature module names that don't always match domain filenames:

| Architecture Feature | Domain Doc Filename | Match? |
|--------------------|-------------------|--------|
| `grid_map/` | `tactical_domain.md` | ❌ different name |
| `turn_phase/` | `tactical_domain.md` (shared) | ❌ different + shared |
| `movement/` | `tactical_domain.md` (shared) | ❌ different + shared |
| `terrain/` | `terrain_domain.md` | ✅ match |
| `faction/` | `faction_domain.md` | ✅ match |
| `tag/` | `tag_domain.md` | ✅ match |
| `gameplay_context/` | `gameplay_context_domain.md` | ✅ match |
| `combat/` | `combat_domain.md` | ✅ match |
| `reaction/` | `reaction_domain.md` | ✅ match |

The `tactical_domain.md` file is expected to serve as the domain rule document for **three** architecture features (grid_map, turn_phase, movement). This is a 1:N mapping between domain documents and architecture features, which violates the "单一文件单一主题" principle. Either the domain doc is too broad (covering grid + turn + movement), or the architecture features are too granular (splitting tactical into sub-features that should be one).

**Recommendation**: Choose one approach:
- **Merge approach**: Consolidate `grid_map/`, `turn_phase/`, `movement/` into a single `tactical/` architecture feature that maps 1:1 with `tactical_domain.md`
- **Split approach**: Split `tactical_domain.md` into `grid_map_domain.md`, `turn_phase_domain.md`, `movement_domain.md` that map 1:1 with architecture features
- The same analysis applies to `terrain_domain.md` if the architecture splits terrain into multiple features

**Severity**: HIGH — P1

---

### [HIGH] 4. Metadata Format Inconsistency Across All Three Trees

**Location**: All three document trees — file metadata headers

**Constitutional Rule**: 宪法 §1.4 — 所有文档必须符合项目规范。

**Violation**: Each document tree uses a different metadata format:

| Feature | Architecture ADRs | Domain Docs | Data Schemas |
|---------|------------------|-------------|-------------|
| YAML front matter | ✅ Yes (`---`) | ❌ No (inline) | ✅ Yes (`---`) |
| `id` field | ✅ Present | ❌ Missing | ✅ Present |
| `title` field | ✅ Present | ❌ Missing | ✅ Present |
| `status` field | ✅ `proposed` | ❌ `Draft` (inline) | ✅ `draft` |
| `owner` field | ✅ `architect` | ❌ Missing | ✅ `data-architect` |
| `created` field | ✅ Date | ❌ Missing | ✅ Date |
| `updated` field | ✅ Date | ❌ Missing | ✅ Date |
| Tags | ✅ Present (in README) | ❌ Missing | ✅ Present (in README) |

The domain files are the only ones without standard metadata. The inconsistency means:
- Automated tooling cannot parse metadata from all three trees uniformly
- Cross-referencing "who owns what" requires opening domain files to find `Applies To`
- Status tracking requires two places (README + file itself)

**Recommendation**: Standardize all documents to use the same YAML front matter format. The data schema format is the most complete and should be the template. Update all 30 domain files.

**Severity**: HIGH — P1

---

### [MEDIUM] 5. missing `src/app/` in Architecture README Directory

**Location**: `docs/01-architecture/README.md` §8 vs `Fre项目架构设计.md` §十

**Constitutional Rule**: `Fre项目架构设计.md` §十 — App 层是横切1：启动装配层（Composition Root）。

**Violation**: The architecture design doc (`Fre项目架构设计.md` §十) clearly defines `src/app/` as a top-level module for composition root responsibilities:
```
src/app/
├── app_plugin.rs
├── game_app.rs
├── editor_app.rs
├── server_app.rs
├── headless_app.rs
├── state/
├── bootstrap/
└── schedule/
```

However, the Architecture README §8 directory structure shows **no `app/` directory at all**. The closest equivalent is `lib.rs` at the root level, which "负责 App 构建 + Plugin 注册". No mention of `app_plugin.rs`, `game_app.rs`, or the `state/`, `bootstrap/`, `schedule/` sub-modules.

Additionally, the architecture design doc defines `src/content/`, `src/tools/`, `src/modding/` as cross-cutting layers, but the Architecture README §8 directory structure does not include any of these.

**Why This Is Medium**: The Architecture README is supposed to be the "最高准则" (§10), but it omits entire cross-cutting layers that the constitution and architecture design doc require. This means:
- Developers following the README won't create `app/`, `content/`, `tools/`, or `modding/`
- The Plugin registration in ADR-001 shows `lib.rs` doing double duty as both library root and composition root
- `headless_app.rs` (for battle simulation) is referenced in tools/ but has no home in the architecture README

**Recommendation**: Add `app/`, `content/`, `tools/`, and `modding/` to the Architecture README §8 directory structure, at minimum as top-level entries even if their internal structure references the constitution/架构设计 for details. Move Plugin registration responsibility from `lib.rs` to `app/app_plugin.rs` to match the architecture design.

**Severity**: MEDIUM — P2

---

### [MEDIUM] 6. Mod API Gateway Count Mismatch

**Location**: `Fre项目架构设计.md` §八 (Mod API) vs Architecture README §3 (Feature Module Map)

**Constitutional Rule**: 宪法 §3.7 — Mod API 采用 Facade + Gateway 模式。

**Violation**: The architecture design doc (§八) lists 14 Gateways in `core/mod_api/`:
```
combat_gateway, character_gateway, spell_gateway, quest_gateway,
party_gateway, camp_gateway, summon_gateway, terrain_gateway,
craft_gateway, economy_gateway, inventory_gateway, faction_gateway,
progression_gateway, narrative_gateway
```

But the architecture's own domain model (ADR-000 Architecture README §3) has slightly different domain coverage:
- Architecture has 14 Business Domains (terrain + faction counted separately from tactical)
- Architecture design doc has 15 Business Domains (terrain, faction, tactical are 3 separate)
- No Gateway for `tactical` domain in the Mod API list
- No Gateway for `reaction` domain in the Mod API list

The constitution §3.7 lists Gateways for: combat, character, spell, quest, party, camp, summon, terrain, craft, economy, inventory, faction, progression, narrative. Note: **no reaction_gateway, no tactical_gateway, no terrain_gateway**.

Wait, terrain IS listed. But reaction and tactical are missing. The architecture design doc matches the constitution's Gateway list exactly. So the domain count is:
- 15 Business Domains in constitution/domain docs
- 14 Gateways in constitution/架构设计 (tactical and reaction are missing)

This means either:
- Tactical/Reaction domains don't need Mod APIs (they are entirely internal)
- Or they are omitted by oversight

**Recommendation**: Clarify why `tactical` and `reaction` domains don't have Gateways. If intentional, add a note. If oversight, create the Gateways.

**Severity**: MEDIUM — P2

---

### [MEDIUM] 7. `src/core/mod_api/` Location in Architecture README

**Location**: `docs/01-architecture/README.md` §8 vs constitution vs Fre项目架构设计.md

**Constitutional Rule**: 宪法 §3.7 — Mod API 属于 Core 层。

**Violation**: The constitution places `mod_api/` under `src/core/`. The architecture design doc places it under `src/core/mod_api/`. But the Architecture README §8 directory structure has no `core/` directory at all — features are flat. This means `mod_api/` has no home in the architecture README's directory structure.

**Recommendation**: Once the layer model issue (#1 in this report) is resolved, ensure `mod_api/` is placed consistently. If the flat model is adopted, `mod_api/` could be a top-level directory; if the nested model is adopted, it stays under `src/core/mod_api/`.

**Severity**: MEDIUM — P2

---

### [LOW] 8. Document Status Lifetime Mismatch

**Location**: All three README's Appendix/Status tables

**Violation**: The status tracking across all three trees uses different labels:

| Tree | Status Labels Used |
|------|-------------------|
| Architecture | `proposed` (file), `pending` (README) |
| Domain | `Draft` (file), `stable` (README) |
| Data | `draft` (file), `stable` (README) |

There is no shared status taxonomy. The constitution §19.3 defines semantic versioning for architecture versions but not for document maturity. Without a shared taxonomy:
- "draft" and "Draft" are treated as different values
- "proposed" and "pending" are confused
- There's no "approved" or "superseded" state in the current set

**Recommendation**: Define a unified document status taxonomy:
```
draft → proposed → approved → deprecated → superseded
```
Apply consistently across all three trees.

**Severity**: LOW — P3

---

## 📋 Summary

| Severity | Count | Issues |
|----------|-------|--------|
| **CRITICAL** | 2 | Layer model mismatch (3 competing structures), Feature-to-domain count mismatch |
| **HIGH** | 2 | Domain filename vs architecture module name inconsistency, metadata format inconsistency |
| **MEDIUM** | 3 | Missing `app/`/`content/`/`tools/`/`modding/` in architecture README, Mod API Gateway count mismatch, `mod_api/` location |
| **LOW** | 1 | Missing unified document status taxonomy |

---

## 🎯 Conclusion

### FAIL — Critical cross-layer inconsistencies found

The three document trees have critical structural inconsistencies that make them mutually incompatible as implementation guides:

1. **Three competing directory models** (Constitution's DDD三层+横切四层 vs Architecture README's flat 7-layer vs implicit domain model). A developer cannot determine where `combat/` or `tag/` should live in `src/`.

2. **Feature-to-domain count mismatch**: Architecture defines ~36 features but domain covers only 30. Missing domain docs for: grid_map, turn_phase, movement, input, registry, pipeline, replay, save, common.

### Required Actions

- **P0**: Select and canonize ONE directory structure model across all documents
- **P0**: Align architecture features exactly with existing domain documents (30 domains) or expand domain docs to cover all 36+ features
- **P1**: Resolve tactical_domain.md 1:3 mapping (one domain doc serving three features)
- **P1**: Standardize metadata format across all three trees (adopt data schema format as template)
- **P2**: Add missing `app/`, `content/`, `tools/`, `modding/` directories to architecture README
- **P2**: Clarify Mod API Gateway coverage for tactical and reaction domains
- **P3**: Define a unified document status taxonomy

### Urgent Recommendation

Before any `src/` code is written, an **Architecture Decision Record (ADR)** must be created that:
1. Explicitly reconciles the directory structure conflict
2. Is signed off by @architect, @domain-designer, and @data-architect
3. Updates the constitution if necessary (constitution is not immutable — it has revision procedures)
4. Ensures all three document trees reflect the final decision

This is a **blocking issue** for all implementation work.
