---
name: bevy-0.19-observer-api
description: Bevy 0.19 observer pattern uses `On<T>` type and `add_observer()`, not `observe()` or `Trigger<T>`
metadata:
  type: reference
---

In Bevy 0.19, the observer/event API differs from later Bevy versions:

- **Trigger events**: `commands.trigger(ButtonClicked { entity })` (same as later)
- **Observe registration**: `app.add_observer(fn_name)` -- NOT `app.observe(fn_name)`
- **Observer parameter type**: `On<ButtonClicked>` -- NOT `Trigger<ButtonClicked>`. Import from `bevy::ecs::observer::On`.
- **Access the event**: `on.event()` returns a `&ButtonClicked`
- **Hierarchy**: `EntityCommands::set_parent_in_place(parent)` -- NOT `set_parent(parent)`. The `set_parent_in_place` method is the available one in 0.19.
- **TextFont**: Has many extra fields (`font_features`, `font_smoothing`, `font_variations`). Always use `..default()` when constructing.
