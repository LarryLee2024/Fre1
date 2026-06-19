---
name: composite-widget-layer
description: Molecule/Organism composite widget layer added between Atoms and Screens
metadata:
  type: project
---

The `docs/06-ui/` directory was restructured into 5 subdirectories (01-architecture, 02-design-system, 03-screens, 04-data-flow, 05-testing) and a new `02-design-system/widget-composites.md` file was created defining 8 Molecules and 8 Organisms.

**Key architectural decision**: Screen-to-Atom jumping was eliminated. Screens now reference Organisms first. Only MainMenuScreen, SettingsScreen, and SaveLoadScreen remain Atom-only (their complexity does not warrant composites).

**Why**: Without this layer, BattleScreen's composition tree referenced 15+ individual Atoms scattered across widget-atoms.md. With composites, BattleScreen references 3 Organisms (BattleHud, SkillPanel, TurnOrderBar), each internally composed of Molecules that reference Atoms. This creates a clean, navigable composition hierarchy.

**How to apply**: When designing a new Screen, first check if existing Organisms cover the need. If not, define a new Organism in widget-composites.md before implementing. When a Molecule candidate appears (same 3-5 atoms used together in >=2 Screens), extract it.
