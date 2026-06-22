---
name: screen-spec-schema-compatibility
description: 2026-06-22 Screen Spec 数据兼容性审查结果 — 四维度分析
metadata:
  type: reference
---

Key findings from Screen Spec data compatibility review:
- **Three divergent BattleHudVm definitions**: code (f32), schema doc (u32 + enum), projection-viewmodel.md (f32). Schema doc must be reconciled with code.
- **UiBinding::BuffSlot(u8) missing**: widget-id-map references it but variant does not exist in code.
- **UiBinding::None does not exist**: widget-id-map should use "(no binding)" notation.
- **ViewModels missing for composite widgets**: CharacterStatusPanelVm, TurnOrderBarVm, TurnIndicatorVm, CharacterPortraitVm, InventoryGridVm and 7+ others referenced by widget-composites.md but not implemented.
- **Per-region Loading/Empty/Error state unsupported**: Dirty<T> binary flag insufficient for Screen Spec's per-region state mapping.
- **UiStore only has 3 fields**: Screen Specs need inventory, shop, quest_log fields not yet in code.
- **Schema doc missing UiBinding variants**: CharacterLevel, Text, Icon exist in code but not in ui-presentation-schema.md.
- **Phase is &'static str, not enum**: phase_key non-serializable — future save/replay risk.
- **Overall**: Screen Specs can be written now; data layer gaps must be filled before implementation.
