---
name: content-pipeline-single-format-only
description: Current content pipeline only supports single-format RON files (one def per file), not array format
metadata:
  type: project
---

The content pipeline in `src/content/content_plugin.rs` loads each `.ron` file as a single struct via `ron::from_str::<T>(&content)` where T is the specific Def type (e.g., `TargetingDef`, `EffectDef`). Array format `[(...), (...)]` is **not supported** -- each file must contain exactly one definition.

This was discovered when creating bulk content files: the user asked for "array format" files but the pipeline deserializes each file as a single struct with no Vec wrapper.

**Why:** The existing pattern in the codebase (all existing bck files use single format). The old `docs/ai_ignore_this_dir` codebase had `load_from_dir_vec` for array support, but the current codebase removed it.

**How to apply:** All new content files must be single-format `(field: value, ...)`. Array format content requires pipeline changes first. Use descriptive file names per def. See existing files like `single_enemy.ron`, `fireball.ron`, `fireball_explosion.ron` for the pattern.

Also: TargetingDef has no `id` field -- it is an embedded value type, not an identifiable asset. Files in `assets/config/targeting/` contain raw TargetingDef structs without IDs.
