---
name: l2-entity-definitions
description: L2 Entity layer with 6 Def types (CharacterDef, MonsterDef, ItemDef, EquipmentDef, ConsumableDef, SummonDef) in entities/ subdirectory
metadata:
  type: reference
---

L2 Entity Def layer designed and documented at `docs/03-content/definitions/entities/`. Contains 6 Def types:

- **CharacterDef** — Playable character template, embeds CreatureBase fields + equipment_slots, class_id (L3 forward ref), starting_equipment, camp_rest_bonuses
- **MonsterDef** — Enemy template, shares CreatureBase with CharacterDef but simplified (no slots, uses equipment_override), has difficulty_rating, loot_table (L3 forward ref), ai_behavior_hints
- **ItemDef** — Base item (materials, quest items, currency), embeds ItemBase fields, has item_type Generic/Material/Quest/Currency/Key
- **EquipmentDef** — Wearable item, embeds ItemBase + slot_type, stat_modifiers (via ModifierDef only, P0 rule), passive_abilities, set_id, durability
- **ConsumableDef** — Consumable item, embeds ItemBase + use_effect (EffectDef ref), target_rule, use_conditions, consume_behavior
- **SummonDef** — Temporary entity, independent structure (no CreatureBase/ItemBase), has its own duration/ai_behavior/dismissal_conditions

Key design patterns established:
- **CreatureBase** and **ItemBase** as schema-level inline patterns (not shared structs, each Def self-contained)
- **Forward References** to L3 marked with LazyRef pattern (class_id → ProgressionDef, loot_table → LootTableDef)
- All Defs follow the 5-section doc format (Overview, Def Structure, Registry Pattern, Validation Rules, RON Example)
- EquipmentDef stat_modifiers must go through ModifierDef (P0: no raw attribute modification outside Modifier pipeline)
- Layer dependency enforcement: L2 may NOT reference L3/L4 (forward references are the only exception with deferred validation)

Updated `definitions/README.md` and `03-content/README.md` to include L2 references, directory trees, and file status tables.
