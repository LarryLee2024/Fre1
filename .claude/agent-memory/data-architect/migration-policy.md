---
name: migration-policy
description: Data migration policy covering Content Def, Save, and Replay migration with chain-based incremental strategy
metadata:
  type: reference
---

Schema: `docs/04-data/foundation/migration_policy.md` — Complete migration policy for three data types (Content Def / Save / Replay) using chain-based incremental migration. N-2 compatibility policy (current + 2 prior versions). ContentMigration trait from Content Platform reused for Def migration. Save and Replay use separate version lines (u32 monotonic). All changes must follow the versioning flow chart in section 4.4. Migration tests require unit/round-trip/integration/fuzz/performance coverage with golden files. [[event-history-architecture]] — Replay migration ensures old replays remain deterministic under format changes.
