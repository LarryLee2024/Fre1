# CalcBreakdown explain() Interface — Research Report

## Current State Summary

**What exists:**
- `CalcTrace` — records formula_id, inputs (HashMap<String,f32>), intermediate_values (Vec<(String,f32)>), output (f32). Used in `execute_damage()` / `execute_heal()` to trace damage, heal, and direct-mod calculations. No human-readable formula string or narrative.
- `PriceBreakdown` — economy-specific: base_price, reputation_modifier, supply_modifier, stolen_modifier, final_price. Emitted in `TransactionCompleted` event but not connected to CalcTrace.
- `ContextChain` — Vec<ChainNode> with cycle detection and length cap (default 10). Tracks origin/source/target/ability_id across reaction chains. NOT connected to CalcTrace at all.
- `RuleFailure` trait (`fn code(&self) -> &'static str`) — 13 domains implement it (combat, economy, inventory, etc.). Not tied to explainability.
- Comment rule SS9 (`.trae/rules/注释规则.md`) — mandates source doc reference and formula string for all complex formulas.

**What does NOT exist:**
- No `Explain` trait anywhere in the codebase
- No human-readable formula string field (formula_id is an opaque key, not a math expression)
- No domain-agnostic breakdown struct — only economy has PriceBreakdown
- No step-by-step narrative for intermediate calculation steps
- No Display/formatting for CalcTrace

## Key Files

| File | Role |
|------|------|
| `src/core/capabilities/execution/foundation/values.rs` | CalcTrace struct + builder methods |
| `src/core/capabilities/execution/mechanism/calculator.rs` | execute_damage/heal/... — produce CalcTrace |
| `src/core/capabilities/gameplay_context/foundation/values.rs` | ContextChain, ChainNode |
| `src/core/domains/economy/events.rs` | PriceBreakdown struct |
| `src/core/domains/economy/components.rs` | Price value object |
| `src/core/domains/economy/rules/rules.rs` | calc_buy_price / calc_sell_price |
| `src/core/domains/spell/rules/formulas.rs` | calc_save_dc, calc_concentration_dc |
| `src/core/domains/progression/rules/formulas.rs` | XP / proficiency formulas |
| `src/shared/traits/mod.rs` | RuleFailure trait (fn code()) |
| `src/core/domains/combat/integration/execution/facade.rs` | CombatExecutionFacade |

## Proposed `CalcBreakdown` Struct

```rust
/// Human-readable calculation breakdown extending CalcTrace with narrative.
pub struct CalcBreakdown {
    pub trace: CalcTrace,                    // raw trace
    pub formula_expr: String,                // e.g. "Damage = (Atk - Def) * CritMult"
    pub source_doc: String,                  // e.g. "Combat_v3.2" (satisfies rule SS9)
    pub steps: Vec<BreakdownStep>,           // step-by-step narrative
    pub chain: Option<ContextChain>,         // context chain (reactions/counters)
    pub rule_code: Option<&'static str>,     // RuleFailure::code() if rule blocked
}

pub struct BreakdownStep {
    pub label: String,          // e.g. "after_reputation_discount"
    pub operation: String,      // e.g. "base * 0.9 (Friendly)"
    pub output: f32,
}
```

This merges the raw numerical trace with the context chain (so you know *why* the calculation happened — which ability, which source/target, reaction depth) and a rule failure code (so you know *which rule* rejected the result).

## Proposed `Explain` Trait

```rust
/// Provides a human-readable breakdown of a calculation result.
pub trait Explain {
    fn explain(&self) -> CalcBreakdown;
}
```

To be implemented by domain rule outputs (Price, ExecutionResult, DamageResult, etc.). A single method keeps the interface minimal — all complexity lives in the `CalcBreakdown` struct.

## Domains That Benefit First

1. **Combat damage** — `src/core/domains/combat/integration/execution/facade.rs`. Currently records `dice_avg`, `flat_bonus`, `base_damage`, `critical_multiplier` in CalcTrace intermediates. Needs formula_expr like `"Damage = (dice_avg({dice}) + flat_bonus({level}) + attr_mod) * crit_mult"` and should carry `chain` for reaction/counter context. RuleFailure::code() answers "why did this attack fail?" (e.g. "COMBAT_NOT_YOUR_TURN").

2. **Economy pricing** — `src/core/domains/economy/rules/rules.rs`. Already has Price/PriceBreakdown with modifiers (reputation, supply, stolen). A `Price::explain()` would produce steps: `base=100 * reputation=0.9 (Friendly) * supply=1.2 (Scarce) * stolen=0.5 = 54`. Would carry `rule_code` when `can_trade_with_reputation()` fails ("ECONOMY_HATED_FACTION").

3. **AI scoring / threat assessment** — no code exists yet. When built, `explain()` will be essential for debugging why the AI chose action X over Y, showing each scoring factor as a `BreakdownStep`.

## Open Questions for Implementation

- Should `CalcBreakdown` live in `execution/foundation/` alongside CalcTrace, or in a new `explain/` module under `shared/`?
- Should `Explain` be auto-derived (proc macro) or manually implemented per domain?
- Does the `formula_expr` string live in content Def data or in the rule code?
