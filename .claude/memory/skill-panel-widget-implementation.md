---
name: skill-panel-widget-implementation
description: SkillPanel widget as Organism combining SkillSlots, registered via SkillPanelPlugin
metadata:
  type: reference
---

Created `src/ui/widgets/skill_panel/` as a composite Organism widget that combines multiple `SkillSlot` widgets into a panel container. Pattern:
- **components.rs**: Tag component (`SkillPanel`) + index component (`SkillSlotIndex(usize)`)
- **factory.rs**: `spawn_skill_panel()` uses `spawn_panel(Group)` for container, calls `spawn_skill_slot()` for each slot, then overwrites `SkillSlotState.skill_id` to match `SkillPanelVm` keys (important: `spawn_skill_slot` defaults skill_id=0, must be overridden for VM matching)
- **mod.rs**: `SkillPanelPlugin` only registers types (no systems -- child SkillSlots use existing `SkillSlotPlugin` systems)
- Integrated into BattleScreen Z7 via `src/ui/screens/battle/mod.rs`
- Registered in `src/ui/widgets/mod.rs` via `WidgetsPlugin`
- Connected to existing `SkillPanelVm` (3 default skills: attack/id=1/cd=0, fireball/id=2/cd=3, heal/id=3/cd=2)

Why the skill_id override: `refresh_skill_slot_from_vm` in `skill_slot/systems.rs` matches by `SkillSlotState.skill_id`, defaulting to 0 in `spawn_skill_slot` factory. Without override all 3 slots would match the first VM entry.
