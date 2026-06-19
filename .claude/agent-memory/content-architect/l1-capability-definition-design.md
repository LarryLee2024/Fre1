---
name: l1-capability-definitions
description: L1 Capability 层 10 个 Content Def 的类型定义文档（Asset 结构、Registry、校验规则）
metadata:
  type: reference
---

Created 10 Content Def definition files in `docs/03-content/definitions/` that bridge Data Schema (`docs/04-data/`) to runtime Asset loading.

- Each Def is a Bevy `#[derive(Asset, TypePath)]` struct with unified fields (`id`, `name_key`, `description_key`, `schema_version`)
- All text uses `LocalizationKey`, never hardcoded strings
- Each Def file documents: overview + cross-references, Rust struct definition, Registry pattern (DefRegistry<T>), validation rules (field-level, cross-Def references, cycle detection), and RON example
- Key architectural decisions documented across the files:
  - EffectDef vs BuffDef separation: EffectDef = pure effect logic, BuffDef = persistent state container wrapping an EffectDef
  - ModifierDef as reusable template: standalone Asset vs inline ModifierConfig for one-off cases
  - ConditionDef as named Condition: inline vs referenced by ID
  - StackingDef as pure rule config with no L1 dependencies
  - All Defs maintain the L1 constraint: may only reference L0 + same-layer Defs, never L2+

**Files created:**
- `definitions/README.md` — full index with upstream/downstream dependency maps, internal reference graph
- `definitions/effect-def.md` — central Def, references ModifierDef, ConditionDef, ExecutionDef, CueDef, StackingDef
- `definitions/buff-def.md` — persistent state container wrapping EffectDef + immunity/exclusive rules
- `definitions/modifier-def.md` — reusable modifier template (Add/Multiply/Override with ScalableValue)
- `docs/03-content/definitions/condition-def.md` — named condition expression tree
- `definitions/trigger-def.md` — event-to-ability bridge with frequency control
- `definitions/targeting-def.md` — target selection (type + shape + filters + LOS)
- `definitions/execution-def.md` — damage/heal/custom calculation formula references
- `definitions/stacking-def.md` — stacking strategy (None/Aggregate/Refresh/Replace)
- `definitions/cue-def.md` — VFX/SFX/Animation/Shake/Popup signal definitions

**Intentionally left as TODO:** `ability-def.md` (AbilityDef) — the combination terminal that depends on all other L1 Defs, deferred to a separate pass.

**Updated:** `docs/03-content/README.md` — added `definitions/` to document tree, added Def definition column to section 5 (L1 table) and section 6 (cross-doc mapping), added file status entries.
