---
name: unit-selection-domain-added
description: Unit Selection domain rules added as section 11 of tactical_domain.md
metadata:
  type: project
---

Unit Selection domain rules (section 11) were added to `docs/02-domain/domains/tactical_domain.md` covering:
- Five-level state hierarchy (Hovered/Focused/Selected/Targeted/Activated) with strict semantic separation
- Selection state machine (Browsing -> UnitSelected -> ActionSelect -> TargetSelect -> TargetLocked -> AwaitingExecution)
- PickTarget enum (Unit/Tile/Skill/Item) with PickContext-based validity rules
- PickContext state machine (6 modes: Normal/UnitSelected/ActionSelect/TargetSelect/TargetLocked/AwaitingExecution)
- Selectability conditions (5 requirements) and unselectable cases
- 10 invariants covering mutual exclusion, boundary validation, confirmation integrity, execution irreversibility
- 9 forbidden actions
- 4 processes (select, target, cancel, complete)
- 7 domain events with read/write classification
- Boundary definitions with Targeting capability, Combat (TurnQueue), ActionPoints, Ability

**Why:** ADR-PICK-000 prerequisite — needed clear selection semantics before implementing the pick system.

**How to apply:** Section 11 defines the full selection domain. When implementing selection logic, all state transitions must follow the defined rules. The Targeting capability within the selection state is only responsible for target validity verification, not for selection flow management.
