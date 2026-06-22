---
name: widget-id-map-staleness-check
description: widget-id-map.md §4 sections for Screens must be cross-checked vs SSPEC before approving; even "fixed" maps may have wrong IDs
metadata:
  type: feedback
---

During settings_screen.md and inventory_screen.md reviews (2026-06-22), discovered two distinct failure modes in widget-id-map.md:

**Failure mode A (SettingsScreen)**: widget-id-map.md §4.5 has 13 stale legacy IDs that do not match the SSPEC's 27 widget IDs. The map was never updated.

**Failure mode B (InventoryScreen)**: widget-id-map.md §4.3 was "updated" from 18 to 33 IDs (quantitative fix), but the new IDs do not match the SSPEC §2.1 region index (~50% naming misalignment). Fix was applied but with wrong names — different from the SSPEC's authoritative IDs.

**Why:** The SSPEC is the detailed design; widget-id-map.md is the reference index that must be updated TO MATCH the SSPEC. Both failure modes produce the same consequence: implementers cannot determine which set of IDs is authoritative.

**How to apply:**
1. When reviewing any Screen SSPEC, always cross-check `widget-id-map.md` §4.{screen} to verify consistency.
2. Default to the SSPEC as authoritative if there are conflicts — the SSPEC is the detailed design, widget-id-map must match it exactly.
3. Verify not just the COUNT of IDs but also their NAMES — the InventoryScreen case showed that a "33 ID update" can still be wrong.
4. Block `active` status if widget-id-map is stale or misaligned — per widget-id-map.md §6, "widget_id 变更必须通过 Presentation Architect 审查" and the map must be updated to match the SSPEC before or alongside its activation.

Related memories: [[project_screen_spec_07_specs]]
