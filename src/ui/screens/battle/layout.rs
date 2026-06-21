//! BattleScreen Zone Layout — 9-zone absolute positioning constants
//!
//! All spacing values derive from theme.spacing tokens (sm=8, md=16, lg=24, xl=32).

use bevy::prelude::*;

use crate::ui::theme::Theme;

/// Zone identifiers for visibility control
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum BattleZone {
    /// Z1: Top-Left — Turn Indicator
    Z1TopLeft,
    /// Z2: Top-Center — Phase Text + Turn Number
    Z2TopCenter,
    /// Z3: Top-Right — Unit Summary [P2]
    Z3TopRight,
    /// Z5: Bottom-Left — Character Card
    Z5BottomLeft,
    /// Z6: Bottom-Center — Action Menu
    Z6BottomCenter,
    /// Z7: Bottom-Right — SkillPanel [P1] + EndTurnButton
    Z7BottomRight,
    /// Z8: Bottom Full-Width — TurnOrderBar [P2]
    Z8BottomBar,
}

/// Spawn a zone container anchored to a specific screen position.
/// Zone containers are siblings with PositionType::Absolute.
pub fn spawn_zone(commands: &mut Commands, theme: &Theme, zone: BattleZone) -> Entity {
    let (node, name) = zone_layout(zone, theme);
    commands.spawn((node, Name::new(name), zone)).id()
}

fn zone_layout(zone: BattleZone, theme: &Theme) -> (Node, &'static str) {
    let s = &theme.spacing;
    match zone {
        BattleZone::Z1TopLeft => (
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(s.sm),
                left: Val::Px(s.sm),
                width: Val::Auto,
                height: Val::Px(56.0),
                ..default()
            },
            "Zone_Z1_TopLeft",
        ),
        BattleZone::Z2TopCenter => (
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(s.sm),
                // TODO[P2][UI][2026-07-21]: Center horizontally via flex parent or transform
                width: Val::Auto,
                height: Val::Px(56.0),
                ..default()
            },
            "Zone_Z2_TopCenter",
        ),
        BattleZone::Z3TopRight => (
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(s.sm),
                right: Val::Px(s.sm),
                width: Val::Px(200.0),
                height: Val::Auto,
                ..default()
            },
            "Zone_Z3_TopRight",
        ),
        BattleZone::Z5BottomLeft => (
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(s.sm),
                left: Val::Px(s.sm),
                width: Val::Px(280.0),
                height: Val::Px(200.0),
                ..default()
            },
            "Zone_Z5_BottomLeft",
        ),
        BattleZone::Z6BottomCenter => (
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(s.sm),
                // Centered via left: 50% + transform. Uses Auto width.
                width: Val::Auto,
                height: Val::Auto,
                ..default()
            },
            "Zone_Z6_BottomCenter",
        ),
        BattleZone::Z7BottomRight => (
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(s.sm),
                right: Val::Px(s.sm),
                width: Val::Px(240.0),
                height: Val::Auto,
                ..default()
            },
            "Zone_Z7_BottomRight",
        ),
        BattleZone::Z8BottomBar => (
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(48.0),
                ..default()
            },
            "Zone_Z8_BottomBar",
        ),
    }
}
