---
name: shared-collections-iterator-extensions
description: Three zero-dependency Iterator extension traits in shared/collections: GroupByMap, TakeWhileInclusive, PartitionMap
metadata:
  type: reference
---

## Implementation Details

Three iterator extension traits live in `src/shared/collections/`:

1. **GroupByMap** (`group_by_map.rs`) — extension trait providing `group_by_map()` which folds items into `HashMap<K, Vec<V>>` via a `(K, V)` mapping closure. Blanket impl across all Iterators.

2. **TakeWhileInclusive** (`take_while_inclusive.rs`) — struct + extension trait (`TakeWhileInclusiveExt`). Unlike std's `take_while`, the first element that **fails** the predicate is still yielded before iteration stops. Stores raw `I` (not `Fuse<I>`) since the `done: bool` flag manages termination internally. Manually impls `Debug` (skips the predicate field). Provides `get_ref()`, `get_mut()`, `into_inner()`.

3. **PartitionMap** (`partition_map.rs`) — extension trait providing `partition_map()` which partitions items into `(Vec<A>, Vec<B>)` via a `Result<A, B>` mapping. Key gotcha: the blanket impl requires `mut f` on the parameter (FnMut needs &mut self).

All files follow shared/ conventions: Chinese doc comments, zero Bevy/ECS dependency, tests in `tests/{unit}/` subdirectory. Module declared `pub(crate)` in `src/shared/mod.rs`.

Test structure: `tests/mod.rs` -> `tests/unit/mod.rs` -> individual `*_test.rs` files. Tests use `use crate::shared::collections::{...}` import path.
