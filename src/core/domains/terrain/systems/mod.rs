//! systems — Terrain 域 ECS 系统

pub(crate) mod hazard_system;
pub(crate) mod surface_system;
pub(crate) mod terrain_effect_system;

pub(crate) use surface_system::on_turn_end_surface_recovery;
pub(crate) use terrain_effect_system::notify_tile_entered;
