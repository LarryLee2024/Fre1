---
id: 10-reviews.data-vs-governance
title: Review — Data Architecture Documents vs Governance Compliance
status: completed
owner: code-reviewer
created: 2026-06-16
updated: 2026-06-16
tags:
  - review
  - data
  - governance
  - compliance
---

# Code Review Report: Data Architecture (`docs/04-data/`) vs Governance

**Reviewer**: @code-reviewer
**Scope**: `docs/04-data/` (README + 33+ schema files across 4 directories)
**Standards**: `docs/00-governance/ai-constitution-complete.md` (v5.0) + `docs/00-governance/Fre项目架构设计.md`
**Date**: 2026-06-16

---

## ✅ Checks Passed

| Check | Status |
|-------|--------|
| All schema files include standard YAML front matter (id/title/status/owner/created/updated) | ✅ Pass |
| Schema files follow the 12-section standard structure defined in README §4.2 | ✅ Pass |
| README §5 Data Laws (001-012) are well-defined with clear violation consequences | ✅ Pass |
| Def-Instance separation (Data Law 001) is embedded in all schemas via Layer Analysis §4 | ✅ Pass |
| Rule-Content separation (Data Law 002) is properly acknowledged | ✅ Pass |
| Effect as唯一业务执行入口 (Data Law 005) is explicit | ✅ Pass |
| Modifier不拥有业务逻辑 (Data Law 006) is embedded | ✅ Pass |
| Replay compatibility analysis (§7) present in all schemas | ✅ Pass |
| Save compatibility analysis (§8) present in all schemas | ✅ Pass |
| Migration strategy (§9) present in all schemas | ✅ Pass |
| ID strategy (§3) is comprehensive with prefix system for all domains | ✅ Pass |
| Four-layer data architecture (Def/Spec/Instance/Persistence) is clearly defined | ✅ Pass |
| Schema design ordering (§9.3) follows dependency chain | ✅ Pass |
| Data ownership matrix in README is comprehensive | ✅ Pass |
| Schema file naming follows `<domain>_schema.md` convention | ✅ Pass |
| Schema files are organized into `foundation/`, `capabilities/`, `infrastructure/`, `domains/` | ✅ Pass |

---

## ❌ Issues Found

### [HIGH] 1. Status Inconsistency: `draft` vs `stable` Across Schema Files

**Location**: `docs/04-data/README.md` §Appendix B (File Status) vs individual schema file headers

**Constitutional Rule**: 宪法 §19.3 — 架构版本管理要求状态一致性。

**Violation**: The README Appendix B marks ALL 33+ schema files as `✅ stable`. However, individual file headers show a mix:

| File | README says | File header says |
|------|-------------|-----------------|
| `tag_schema.md` | ✅ stable | `status: draft` |
| `combat_schema.md` | ✅ stable | `status: draft` |
| `attribute_schema.md` | ✅ stable | `status: draft` |
| `trigger_schema.md` | ✅ stable | `status: draft` |

From the files sampled, **all individual schema files say `status: draft`** in their YAML header, while the README claims they are all `stable`. This is the same issue as in the domain docs but affects 33+ files.

**Why This Is High**: Downstream features depending on these schemas cannot determine whether they represent finalized decisions or work-in-progress. If schemas are "draft", developers should expect changes; if "stable", they should be committed.

**Recommendation**: One of the following:
- Update all schema file headers from `draft` to `stable` (if they are indeed reviewed and approved)
- Update the README status table from `stable` to `draft` (if they are not yet finalized)
- Add a clear status promotion policy (draft → proposed → stable → deprecated) with review gates

**Severity**: HIGH — P1

---

### [HIGH] 2. Foundation Schema Files Missing

**Location**: `docs/04-data/foundation/` — listed in README §8 but appears to have no files

**Constitutional Rule**: 宪法 §10.1 — Data Driven 核心原则要求所有配置和 Schema 定义完整。

**Violation**: The README §8 (数据目录结构) lists the following files under `docs/04-data/foundation/`:
```
foundation/
├── id_strategy.md
├── save_architecture.md
├── replay_architecture.md
└── migration_policy.md
```

But the README Appendix B marks ALL of these as `⬜ pending` with `owner: data-architect` and no completion date. This means the foundational architectural decisions (ID strategy details, save architecture details, replay architecture details, migration policy) are **not yet documented anywhere**.

Without these foundation documents:
- `id_strategy.md` — the brief ID strategy in README §3 is insufficient for implementation (e.g., how does Registry auto-assign bit indices for Tags?)
- `save_architecture.md` — the save overview in README §6 is high-level; actual serializer/version migration/checksum details are not specified
- `replay_architecture.md` — the replay overview in README §7 lacks detailed frame format, I/O APIs, and security considerations
- `migration_policy.md` — no explicit rules for data schema evolution

**Why This Is High**: The foundation documents are the deepest dependencies in the data architecture. All 33+ schemas are designed with the expectation that these foundation docs exist, but they don't. This creates risk: when foundation docs are written later, they may force changes to the already-"stable" schemas.

**Recommendation**: Prioritize writing the 4 foundation documents. At minimum, create skeleton documents with explicit "to be defined" sections so downstream schemas know what they're waiting for. Alternatively, integrate the essential content directly into the README and remove the foundation/ placeholder references.

**Severity**: HIGH — P1

---

### [MEDIUM] 3. Data Law 012 Implementation Gap

**Location**: `docs/04-data/README.md` §5 — Data Law 012

**Constitutional Rule**: 宪法 §3.5.2 🟥 — "Domain 之间禁止直接依赖、直接调用内部实现"

**Violation**: Data Law 012 states: "Domain 之间禁止直接引用对方的数据结构，仅通过 Event 通信." However:
- There is no documented mechanism to **enforce** this at compile time or CI time
- There is no explicit mention of how Domain A obtains Domain B's data for "read" operations (the double-track rule: writes via Event, reads via Query API)
- The architecture `Fre项目架构设计.md` §7.3 explains the double-track mechanism (写操作→Event, 读操作→Query API), but the data README only mentions "仅通过 Event 通信" without the read-path exception

**Why This Is Medium**: The data README and constitution both explicitly forbid cross-domain direct data references, but neither specifies:
1. How Query APIs cross domain boundaries (they are read-only, but still cross domain boundaries)
2. What CI tool enforces this (the `dependency_checker` in tools/ is mentioned but not linked)

**Recommendation**: Update Data Law 012 description to clarify the double-track exception (reads via Query API are legitimate, only writes must go through Event). Reference the `dependency_checker` tool or ADR-040 for enforcement mechanism.

**Severity**: MEDIUM — P2

---

### [MEDIUM] 4. Schema Layer Declarations Inconsistent

**Location**: Schema file header `layer:` field

**Constitutional Rule**: 宪法 §第四编 — 四层数据架构要求每层职责清晰分离。

**Violation**: The YAML front matter requires a `layer:` field, but values observed include:
- `tag_schema.md`: `layer: definition` (single layer)
- `combat_schema.md`: `layer: instance, persistence` (two layers, comma-separated)

A schema that spans multiple layers blurs the Def/Instance separation that Data Law 001 mandates. If CombatSchema covers both Instance and Persistence layers, there should be a clear mapping showing which structs belong to which layer rather than a comma-separated list.

**Recommendation**: Define a strict policy for the `layer:` field:
- Only accept single values: `definition`, `spec`, `instance`, or `persistence`
- If a schema legitimately covers multiple layers, split it into separate files (e.g., `combat_instance_schema.md` + `combat_persistence_schema.md`)
- Or use a sub-field: `layers: [instance, persistence]` with explicit mapping

**Severity**: MEDIUM — P2

---

### [MEDIUM] 5. Schema Design Order vs Actual File Count

**Location**: `docs/04-data/README.md` §9.3 — Schema Design Order

**Constitutional Rule**: 宪法 §16.1 — 文档结构完整性。

**Violation**: The design order lists 10 phases. However, reviewing the actual file count:
- Phase 1-3 (Capabilities): 15 schemas → all present ✅
- Phase 4 (Infrastructure): 3 schemas → all present ✅  
- Phase 5-10 (Business Domains): 15 schemas → all present ✅
- Foundation (pre-Phase 1): 4 schemas → all **missing** ⚠️

This means the design was done "out of order" — 33 schemas were completed without completing the foundational 4 first. While the content is consistent (foundation concepts are embedded in the README), the actual foundation detail documents are missing.

**Recommendation**: Move essential foundation content from README into the 4 foundation documents, or formally acknowledge that the README itself serves as the foundation and demote `foundation/` to "future extension."

**Severity**: MEDIUM — P2

---

### [LOW] 6. Localization Key Format Inconsistency

**Location**: `docs/04-data/README.md` §3.2 — Localization Key format

**Constitutional Rule**: 宪法 §10.1 — 配置内容管理规范。

**Violation**: The defined localization key format is `<命名空间>.<ID>.<后缀>` with examples like `attribute.attr_000001.name`. However:
- `tag_schema.md` uses `tag.<id>.desc` format (e.g., `tag.tag_000001.desc`)
- The namespace is `tag` but the prefix is `tag_` — this creates `tag.tag_000001.desc` which duplicates "tag"
- The README example uses `ability.abl_000042.desc` — same duplication issue
- No explicit rule about whether the namespace and ID prefix should be identical or different

This is a minor consistency concern but could cause confusion for content creators who need to generate localization files.

**Recommendation**: Clarify the relationship between namespace prefix and ID prefix. Either make them the same (tag.tag_000001 → consistent) or different (ability.abl_000042 → document why).

**Severity**: LOW — P3

---

### [LOW] 7. Schema Validation Checklist References Missing

**Location**: `docs/04-data/README.md` §4.3 — Schema 评审 Checklist

**Constitutional Rule**: 宪法 §18.1 — 工程质量门禁要求。

**Violation**: The Schema 评审 Checklist lists 10 items but:
- No reference to which CI tool or process enforces these
- No mention of automated schema validation
- No "Constitution Check" item — the checklist does not reference the constitution even though Data Laws are derived from it

**Recommendation**: Add a "Constitution Compliance" item to the checklist (#11). Reference the `dependency_checker` tool or equivalent CI mechanism.

**Severity**: LOW — P3

---

## 📋 Summary

| Severity | Count | Issues |
|----------|-------|--------|
| **CRITICAL** | 0 | — |
| **HIGH** | 2 | Status inconsistency (draft vs stable), foundation schema files missing |
| **MEDIUM** | 3 | Data Law 012 enforcement gap, layer field inconsistency, design ordering issues |
| **LOW** | 2 | Localization key naming, missing CI/checklist references |

---

## 🎯 Conclusion

### PASS with High-severity issues

The data architecture documents are the most consistently formatted of the three document trees. Standard YAML metadata headers, structured sections, and comprehensive Data Laws make them well-governed. However, two HIGH issues exist:

1. **Status inconsistency**: README claims all schemas are "stable" while individual files say "draft"
2. **Missing foundation documents**: 4 critical foundation documents (id strategy, save, replay, migration) are listed but never written

### Required Actions

- **P1**: Resolve the `stable` vs `draft` status conflict across 33+ schema files
- **P1**: Write the 4 foundation documents or formally integrate their content into the README
- **P2**: Clarify Data Law 012 with the double-track exception (reads via Query API are allowed)
- **P2**: Define a strict `layer:` field policy (single value per file, or explicit mapping)
- **P3**: Clarify localization key format (namespace vs prefix relationship)

### Next Steps

After fixing HIGH issues → call **@code-reviewer** for re-review.
Foundation document creation → call **@data-architect**.
