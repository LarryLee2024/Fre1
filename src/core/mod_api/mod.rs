//! Mod API — Mod 稳定 API 层
//!
//! 采用 Facade + Gateway 模式，为 Mod 提供稳定的只读访问入口。
//! 每个关键业务域提供一个 Gateway（如 combat_gateway, character_gateway）。
//!
//! 详见 `docs/00-governance/Fre项目架构设计.md` §八

// TODO: 实现 Gateways:
//   combat_gateway, character_gateway, spell_gateway, quest_gateway,
//   party_gateway, camp_gateway, summon_gateway, terrain_gateway,
//   craft_gateway, economy_gateway, inventory_gateway, faction_gateway,
//   progression_gateway, narrative_gateway
