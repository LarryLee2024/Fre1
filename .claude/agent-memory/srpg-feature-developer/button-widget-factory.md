---
name: button-widget-factory-implementation
description: Button widget implemented with Factory pattern under src/ui/widgets/button/
metadata:
  type: reference
---

# Button Widget Factory Implementation

The Button widget was implemented following the Factory pattern specified in `docs/06-ui/01-architecture/architecture.md` (section 9). The theme infrastructure (UiColors, UiSpacing, UiTypography, Theme resource) was already in place.

## Key files

- `/src/ui/widgets/button/components.rs` — `ButtonVariant` (Primary/Secondary/Danger/Ghost), `ButtonState` (variant, disabled, label), `ButtonInteraction` (hovered, pressed, just_clicked)
- `/src/ui/widgets/button/events.rs` — `ButtonClicked` event (fired via `commands.trigger()` — Observer pattern per ADR-054)
- `/src/ui/widgets/button/factory.rs` — `spawn_button()` factory function, the only way to create buttons. Takes `&Theme` for colors/spacing/typography tokens. No raw Color or Val::Px values.
- `/src/ui/widgets/button/systems.rs` — `button_interaction_system()` reads Bevy's built-in `Interaction` component, tracks hovered/pressed/just_clicked, fires `ButtonClicked` via trigger(), updates BackgroundColor per state+variant
- `/src/ui/widgets/button/mod.rs` — ButtonPlugin registers types, events, and the interaction system

## Architecture compliance

- NO BSN (as required by architecture.md section 2.5)
- Factory pattern only — screens cannot `commands.spawn(Button, ...)` directly
- All colors reference `Theme.colors.*` (semantic tokens like `accent_primary`, `feedback_negative`, `surface_secondary`)
- All spacing references `Theme.spacing.*` (tokens like `md`, `sm`, `button_height`)
- All font sizes reference `Theme.typography.*` (tokens like `size_body`)
- Bevy 0.19 APIs: `Button`, `Interaction`, `BackgroundColor`, `BorderColor::all()`, `FontSize::Px()`, `ButtonInput<T>`
- No hardcoded `Color::srgb()` or `Val::Px()` values in widget code

## Existing theme tokens used

- `accent_primary / accent_hover / accent_pressed` — Primary button states
- `surface_secondary` — Secondary/Ghost button backgrounds  
- `feedback_negative / surface_danger` — Danger button states
- `surface_disabled` — Disabled state background
- `text_primary / text_disabled` — Text colors
- `border_default` — Secondary button border
- `spacing.md / spacing.sm` — Button padding
- `spacing.button_height` — Minimum button height
- `typography.size_body` — Button label font size

## Wiring

UiPlugin is registered in Phase 11 of AppPlugin (after Infra Phase 8 and ScenePlugin Phase 9). WidgetsPlugin registers ButtonPlugin. ThemePlugin is registered first within UiPlugin.
