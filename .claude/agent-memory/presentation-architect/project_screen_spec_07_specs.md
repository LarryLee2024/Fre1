---
name: screen-spec-07-specs-created
description: Created 07-specs/ directory with README.md and screen-spec-template.md (17-field SSPEC format) as the foundation for the UI Screen Specification refactor
metadata:
  type: project
---

Created `docs/06-ui/07-specs/` directory structure on 2026-06-22 as Step 1 of the UI Screen Specification refactor execution plan (docs/11-refactor/ui-screen-spec-execution-plan.md).

**Files created**:
- `docs/06-ui/07-specs/README.md` (id: 07-specs.README, status: draft) -- 总纲 containing: SSPEC goal, Screen=Layout/Widget=Implementation two-body constraint, 14 AI generation rules (R01-R14), 18 DoD checklist items (D01-D18), zero-Figma strategy, scope boundaries (must/must-not describe), file lifecycle (draft->review->active->deprecated), and maintenance rules.
- `docs/06-ui/07-specs/screen-spec-template.md` (status: draft) -- Complete 17-field SSPEC template with YAML frontmatter, Chinese descriptions + English identifiers, per the plan's BattleScreen example. Fields: Screen Header / ASCII Wireframe / Widget Tree / Flexbox Layout / Responsive Rules / Region Responsibility / Widget Contract / State Mapping (per-region) / Focus Navigation / Interaction Zones / Overlay Definition / Lifecycle / Data Ownership / Layout Intent / Scroll & Overflow Policy / Event Contract / Screen Metrics. Includes DoD Checklist appendix and reference documents index.
- Empty `screens/` and `references/` directories created for subsequent phases.

**Why**: The SSPEC format bridges the gap between 06-ui/ runtime architecture (Projection/ViewModel/Widget Contract) and AI-generated UI code -- providing a complete layout+interaction spec so AI produces correct UI on the first try (~40% -> ~80%).

**How to apply**: When reviewing or generating UI Screen specs, use screen-spec-template.md as the canonical format. P0 fields (1-14) must be complete before status->active; P1 fields (15-17) are wip-allowed. Reference README.md for the 14 AI rules (R01-R14) and 18 DoD items (D01-D18).

**Related**: [[project_composite_widget_layer]], [[project_implementation_patterns]], [[primitives_isolation_layer]], [[project_code_alignment_0621]]
