---
name: adr-063-review-completed
description: ADR-063 (Macro Governance) was approved after review. Key fixes: fixed duplicate/out-of-order section numbering, added missing constitution principles 8 and 9, updated stale src/macros.rs references to reflect completed migration, added Communication Design/边界定义/Definition-Instance Design/替代方案 sections, added to README index.
metadata:
  type: project
---

ADR-063 (Macro Governance) reviewed and accepted on 2026-06-21 by @architect.

**Issues found and fixed:**
- Three duplicate "Declarative vs Procedural 分离" sections with conflicting numbers -- consolidated into one
- Section numbering was broken (went 1,2,3,4,2,4,5,6,7,4,5) -- renumbered to 1-11 matching constitution section 16.6
- Missing constitution principles 8 (macro must be replaceable by function) and 9 (no macro nesting) -- added full sections
- Migration plan had placeholder names (宏1-宏6) -- replaced with actual macro names from codebase audit
- Health assessment table referenced non-existent `src/macros.rs` -- corrected to current 8-file layout (macros already migrated per task #88)
- Missing from README ADR index -- added entry
- Missing standard ADR sections -- added Communication Design, 边界定义, Definition/Instance Design, 替代方案, 引用的领域规则

**File changed:** `docs/01-architecture/40-cross-cutting/ADR-063-macro-governance.md`
**README updated:** `docs/01-architecture/README.md` (index table + appendix B + version to v5.4)
