---
name: rng-system-refactoring
description: Removed homemade MurmurHash3 DeterministicRng from core/replay, replaced with ChaCha12-backed 4-stream DeterministicRng in shared/random/
metadata:
  type: reference
---

## RNG System Refactoring (2026-06-21)

Moved `DeterministicRng` (4-stream, one per RngStream) from `core/capabilities/runtime/replay/foundation/` to `shared/random/`. Replaced the homemade MurmurHash3 mixing with ChaCha12 CSPRNG (via SeededRng wrapper).

### Key changes:
- **New location**: `src/shared/random/mod.rs` now contains `SeededRng`, `RngStream`, `RngSeeds`, `DeterministicRng`
- **Removed**: `GameRng` (deprecated single-stream), homemade MurmurHash3 mixing in old `DeterministicRng`
- **Removed from core**: `DeterministicRng` struct and all methods in `foundation/values.rs`
- **Removed from core**: `RngStream` enum and `RngSeeds` struct from `foundation/types.rs`
- **`RngSeeds::uniform()`** now uses `wrapping_add(1/2/3)` offsets (was all same seed)
- **Infra layer**: `infra::replay::resources::DeterministicRng` wrapper removed; `DeterministicRng` re-exported from shared with `Resource` derive
- **Combat recording**: Fixed Core->Infra violation by importing `DeterministicRng` from `shared::random`

### Old API preserved:
- `DeterministicRng::new(RngSeeds)`, `::with_seed(u64)`
- `get_seed()`, `set_seed()`, `get_all_seeds()`, `set_all_seeds()`
- `next_u64()`, `next_f32()`, `gen_bool()`, `gen_range()`

### Internal now uses:
- Each stream has its own `SeededRng` (ChaCha12) instance
- `set_seed`/`set_all_seeds` reinitialize the ChaCha12 instance with the new seed
- No shared mutable state between streams (unlike old counter-based approach)

### Why no Reflect:
`SeededRng` wraps `ChaCha12Rng` which doesn't implement `Reflect`. The old `DeterministicRng` used `HashMap<RngStream, u64>` counters which were Reflect-compatible. Since we couldn't add Reflect to the new struct, we removed the infra wrapper and made the shared type directly a `Resource`.
