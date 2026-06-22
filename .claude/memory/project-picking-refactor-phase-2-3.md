---
name: picking-refactor-phase-2-3
description: Picking architecture refactoring Phase 2 (event flow pipeline) and Phase 3 (Selection refactoring) completed
metadata:
  type: project
created: 2026-06-23
updated: 2026-06-23
---

# Picking Refactor Phase 2+3 Complete

## Changes Made

### Step 1: test_battle/render.rs cleanup
- Deleted `on_unit_click`, `on_unit_hover`, `on_unit_unhover` entity observers
- Removed `.observe()` chain from `attach_unit_visuals`
- Removed `println!` calls and `Selection` import
- Removed `HIGHLIGHT_COLOR` and `SELECTED_COLOR` constants

### Step 2: projections/selection.rs rewritten as event-driven
- `on_unit_clicked_projection` -- consumes `UnitClicked` event, updates UiStore
- `on_selection_cleared_projection` -- consumes `SelectionCleared`, resets action menu
- `on_unit_selected_follow` -- consumes `UnitClicked`, triggers `CameraRequest::MoveTo`
- `on_unit_selected_highlight` -- consumes `UnitClicked`, sets cyan highlight on selected unit
- `on_selection_cleared_highlight` -- consumes `SelectionCleared`, restores team colors
- No more `Res<Selection>.is_changed()` polling
- No more direct `TargetPose` modification for camera

### Step 3: screens/mod.rs updated
- Old `on_selection_changed`, `on_selection_visual`, `camera_follow_selection` replaced with new observer registrations

## Architecture Compliance
- Camera follow now uses `CameraRequest::MoveTo` instead of writing `TargetPose` directly -- complies with camera architecture (ADR-050+)
- All selection projections are event-driven, no polling
- Hover visual feedback is deferred (SelectionState.hovered still tracked, no Sprite color change yet)

## Key Files
- `/Users/lf380/Code/Bevy/Fre/src/ui/projections/selection.rs`
- `/Users/lf380/Code/Bevy/Fre/src/app/scenes/test_battle/render.rs`
- `/Users/lf380/Code/Bevy/Fre/src/ui/screens/mod.rs`

## How to Apply
- New selection projections follow the pattern in `projections/battle.rs` -- observers consume events and update UiStore
- Camera interaction must go through `CameraRequest`, not `TargetPose`
