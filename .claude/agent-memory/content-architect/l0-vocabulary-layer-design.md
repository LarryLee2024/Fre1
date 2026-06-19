---
name: l0-vocabulary-layer-design
description: L0 Vocabulary layer design for 6 Def types (TagDef, AttributeDef, DamageTypeDef, FactionDef, ElementDef, StatusCategoryDef) with TagCategory three-way split
metadata:
  type: reference
---

L0 Vocabulary layer completed with 6 Content Def types (2026-06-20). Key architectural decisions:

1. **TagDef is flat** -- no parent_id, no bit_index, no namespace, no is_abstract. Hierarchy belongs to Data Schema TagHierarchy, not to Content Asset. TagDef introduces `TagCategory` (Gameplay/Semantic/System) as the sole classification dimension.

2. **L0 Defs cannot have `tags` field** -- L0-to-L0 references are prohibited by content-layering.md Sec 7.4. All L0 Defs are fully self-contained with only primitive types, enums, and LocalizationKey fields.

3. **ElementDef exists because TagDef is flat** -- Since TagDef cannot express elemental interaction matrices, ElementDef provides type-safe `ElementId` for the combat pipeline. Element- Element interactions (strengths/weaknesses) are deferred to L3 Gameplay `ElementInteractionMatrix`.

4. **FactionDef relationship matrix deferred to L3** -- `relationship_overrides: Vec<(FactionId, FactionAttitude)>` would violate L0 same-layer reference rules. Relationship matrix defined in L3 Gameplay `FactionRelationshipMatrix`.

5. **L0 same-layer reference prohibition is strict** -- No cross-references among any L0 Defs (no TagId in AttributeDef, no DamageTypeId in ElementDef, etc.). Cross-layer mapping tables live at L3.

6. **SurfaceDef removed** from L0. The `SurfaceDef` type that was in content-layering.md L0 section 2.3 is not included in the vocabulary. It will reappear in L4 World if needed.

7. **Field naming convention**: All L0 Defs use `name_key` + `description_key: LocalizationKey` except TagDef which uses `desc_key: Option<LocalizationKey>` (optional for Semantic/System tags).

Files created:
- docs/03-content/definitions/vocabulary/README.md (layer index)
- docs/03-content/definitions/vocabulary/tag-def.md
- docs/03-content/definitions/vocabulary/attribute-def.md
- docs/03-content/definitions/vocabulary/damage-type-def.md
- docs/03-content/definitions/vocabulary/faction-def.md
- docs/03-content/definitions/vocabulary/element-def.md
- docs/03-content/definitions/vocabulary/status-category-def.md

Updated: definitions/README.md, content-layering.md, docs/03-content/README.md
