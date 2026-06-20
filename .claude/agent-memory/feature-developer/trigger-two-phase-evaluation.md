---
name: trigger-two-phase-evaluation
description: Evaluator functions use two-phase evaluation (pure eval + event emission) to keep purity
metadata:
  type: reference
---

Trigger capability evaluator in `src/core/capabilities/trigger/mechanism/evaluator.rs` uses a two-phase design:

1. `evaluate_trigger()` -- pure function, no side effects, returns `TriggerEvalResult`
2. `emit_trigger_events()` -- takes `&TriggerEvalResult` and emits `TriggerFired` or `TriggerSuppressed` events

This avoids the previous design smell where `can_trigger()` claimed purity but took `&mut Commands`. Callers (e.g. `src/core/domains/combat/integration/trigger/facade.rs`) call both in sequence for production use, or just `evaluate_trigger()` when only checking readiness.

**Why:** Maintains the Replay First / Logic/Presentation separation principles. Pure evaluation can be used in tests and deterministic paths without event side effects.

**How to apply:** When adding evaluator-like functions, keep the pure check separate from event emission. The `emit_trigger_events()` function is the single place where evaluation results are translated to events.
