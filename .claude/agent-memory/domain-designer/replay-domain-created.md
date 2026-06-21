---
name: replay-domain-created
description: Replay domain rules document created at docs/02-domain/domains/replay_domain.md
metadata:
  type: project
---

A new Replay domain rules document (`docs/02-domain/domains/replay_domain.md`) was created in draft status. It covers the full replay lifecycle (Idle / Recording / Playing state machine), 9 invariants (seed determinism, frame integrity, RNG isolation, etc.), bridge contracts with Combat and other domains, and the Event History complementarity relationship. Created referencing ADR-041 and ADR-048.

**Why:** ADR-048 explicitly noted this document was missing. The replay system's domain rules need formal documentation to guide the bridge layer implementation.

**How to apply:** Reference this doc when working on replay-related features, Combat-Replay bridge, or determinism verification.
