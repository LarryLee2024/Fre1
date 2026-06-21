---
name: camera-domain-created
description: Camera domain rules document created at docs/02-domain/infrastructure/camera_domain.md based on ADR-064
metadata:
  type: project
---

Camera domain rules document created at `docs/02-domain/infrastructure/camera_domain.md` based on ADR-064 (Camera System Architecture).

Key facts established:
- Camera is an Infra layer module (L2 Infrastructure), not a business Domain — placed under `docs/02-domain/infrastructure/`
- Four-state machine: Idle / FreeMove / Follow / Focus with full transition matrix
- Event-driven via CameraRequest enum — all external systems must use `commands.trigger(CameraRequest::...)`
- Pose pipeline: TargetPose → Interpolation → CurrentPose → Clamp + Shake → TransformWrite
- Camera uses CameraTarget (WorldPos/TilePos/UnitId) instead of Entity references
- CameraBounds uses pure Vec2 — decoupled from map/terrain types
- Camera must not depend on any `core::domains::*` types
- 8 invariants and 12 forbidden actions defined
- Input layer mapping (InputAction → CameraRequest), Cue layer mapping (CueType::Shake → CameraRequest::Shake), and GameState lifecycle rules documented
- Replay rules: CameraCommand subset for recording, replay consumption deferred to Phase 3
- README.md updated to include infrastructure/ category (total now 32 domain rule files)

**Why:** Camera was previously a 7-line spawn function with no interactivity. The domain rules formalize the architecture from ADR-064 into binding invariants and process definitions.

**How to apply:** When implementing Camera systems, adhere to the state machine matrix, Pose pipeline order, and event-driven constraint. Feature developer should implement foundation types first, then systems, using this document as specification. Data architect should produce `docs/04-data/infrastructure/camera_schema.md` based on these rules.
