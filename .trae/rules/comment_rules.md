## Comments

Explain WHY.
Do not explain WHAT.

Good:

// OccupancyGrid is cached because pathfinding reads it every turn.

Bad:

// Increase hp by 10
hp += 10;

## Public API

Must have rustdoc.

## Domain Logic

Must explain:
- business reason
- invariant
- edge case

## Forbidden

- obvious comments
- commented dead code
- TODO without issue id