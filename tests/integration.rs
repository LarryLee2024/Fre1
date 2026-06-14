//! Feature Test — 业务功能完整流程验证

#[path = "integration/buff.rs"]
mod buff;
#[path = "integration/buff_damage.rs"]
mod buff_damage;
#[path = "integration/buff_lifecycle.rs"]
mod buff_lifecycle;
#[path = "integration/campaign.rs"]
mod campaign;
#[path = "common/mod.rs"]
mod common;
#[path = "integration/consumable.rs"]
mod consumable;
#[path = "integration/death.rs"]
mod death;
#[path = "integration/equipment.rs"]
mod equipment;
#[path = "integration/inventory.rs"]
mod inventory;
#[path = "integration/skill.rs"]
mod skill;
#[path = "integration/traits.rs"]
mod traits;
#[path = "integration/turn.rs"]
mod turn;
#[path = "integration/combat_pipeline.rs"]
mod combat_pipeline;
#[path = "integration/skill_system.rs"]
mod skill_system;
#[path = "integration/turn_flow.rs"]
mod turn_flow;
#[path = "integration/terrain_combat.rs"]
mod terrain_combat;
#[path = "integration/edge_cases.rs"]
mod edge_cases;
#[path = "integration/ui_screens.rs"]
mod ui_screens;
#[path = "integration/unified_movement.rs"]
mod unified_movement;
