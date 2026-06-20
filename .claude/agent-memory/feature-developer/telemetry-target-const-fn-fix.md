---
name: telemetry-target-const-fn-fix
description: How LogCode::target() was made const fn for use in tracing's target: parameter
metadata:
  type: reference
---

The `emit_info!/emit_warn!/emit_debug!` macros in `src/infra/logging/telemetry.rs` use `target: $code.target()` to set the tracing event's target from the LogCode. This requires `target()` to be a `const fn` because tracing's `target:` parameter needs a compile-time constant.

The original implementation used `self.code().get(..3)` (string slicing via `str::get()`) which is not const-stable. The fix was to rewrite `target()` to match on `self` directly, enumerating all ~86 LogCode variants grouped by prefix:

```rust
pub const fn target(&self) -> &'static str {
    match self {
        Self::BAT001 | Self::BAT002 | ... => "domain.combat",
        Self::TAC001 | Self::TAC002 | ... => "domain.tactical",
        ...
    }
}
```

Enum discriminant matching IS const-stable, unlike `str::get()` and `&str` pattern matching.

**Key locations:**
- `src/shared/diagnostics/log_code.rs` — `LogCode::target()` method (const fn)
- `src/infra/logging/telemetry.rs` — `emit_info!/emit_warn!/emit_debug!` macros
- `src/infra/logging/observers/*.rs` — 19 observer files using the macros
