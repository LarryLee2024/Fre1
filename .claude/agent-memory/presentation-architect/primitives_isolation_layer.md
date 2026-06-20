---
name: primitives-isolation-layer
description: Three-layer UI architecture with primitives/ as the only bridge to Bevy UI low-level types
metadata:
  type: project
---

The UI codebase was restructured from a flat `widgets/` model into a three-layer architecture enforced by directory structure:

```
src/ui/
├── primitives/    L3-P: UI 原语层 — button, progress_bar, panel, text, list, modal
├── widgets/       L3-W: 游戏业务控件 — composites (molecules + organisms), tooltip, notification
└── screens/       L3-S: 页面 — compose widgets
```

**Core principle: Primitives isolation.** Only `primitives/` can depend on low-level Bevy UI implementations (Node, Button, Interaction, BackgroundColor). Business widgets in `widgets/` and screens in `screens/` must go through primitives' factory functions and components.

**Dependency rules:**
- `primitives/` → `theme/` (allowed)
- `widgets/` → `primitives/` + `theme/` (allowed; forbidden to import Bevy UI types directly)
- `screens/` → `widgets/` + `primitives/` + `theme/` (allowed via Factory)

**Why:** At 500k+ LOC, swapping the UI framework (e.g., bevy_ui version upgrade or migration) is prohibitively expensive. The primitives layer ensures the blast radius of any low-level UI change stays contained to `primitives/`.

**How to apply:** When reviewing UI code, check that `widgets/` and `screens/` never import `bevy::ui::Node`, `Button`, `Interaction`, `BackgroundColor`, etc. Those imports are only permitted in `primitives/`. Violations should be rejected in code review.

**Docs updated:** (7 files across the architecture doc tree)
- `docs/06-ui/01-architecture/architecture.md` — directory tree, §3.6 three-layer rules, §4.4 rendering layers, §6.5 fifth iron rule, §8.2 PrimitivesPlugin
- `docs/06-ui/02-design-system/widget-atoms.md` — renamed to "Primitives — UI 原语层", added isolation note
- `docs/06-ui/02-design-system/widget-composites.md` — added architecture isolation note, updated §7 directory tree
- `docs/06-ui/README.md` — updated directory listing and layer overview
- `docs/01-architecture/40-cross-cutting/ADR-055-ui-presentation-architecture.md` — added DR-XXX Primitives isolation
- `docs/00-governance/ai-constitution-complete.md` — added §第九编 article on Primitives isolation
- `docs/02-domain/capabilities/ui-presentation.md` — INV-UI-009 → INV-UI-010, added 5th iron rule
