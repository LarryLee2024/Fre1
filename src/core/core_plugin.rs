//! CorePlugin — L1 领域规则层总 Plugin
//!
//! 聚合 Core 层所有能力与业务领域 Plugin 的子 Plugin。
//! 当前为骨架，按需暴露 Core 层公共 Resource。

use crate::core::capabilities::{
    ability::AbilityPlugin, aggregator::AggregatorPlugin, attribute::AttributePlugin,
    condition::ConditionPlugin, cue::CuePlugin, effect::EffectPlugin, event::EventPlugin,
    execution::ExecutionPlugin, gameplay_context::GameplayContextPlugin, modifier::ModifierPlugin,
    runtime::RuntimePlugin, spec::SpecPlugin, stacking::StackingPlugin, tag::TagPlugin,
    targeting::TargetingPlugin, trigger::TriggerPlugin,
};
use crate::core::domains::{
    camp_rest::CampRestPlugin, combat::CombatPlugin, crafting::CraftingPlugin,
    economy::EconomyPlugin, faction::FactionPlugin, inventory::InventoryPlugin,
    narrative::NarrativePlugin, party::PartyPlugin, progression::ProgressionPlugin,
    quest::QuestPlugin, reaction::ReactionPlugin, spell::SpellPlugin, summon::SummonPlugin,
    tactical::TacticalPlugin, terrain::TerrainPlugin,
};
use bevy::prelude::*;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        // Capabilities — Foundation
        app.add_plugins(TagPlugin)
            .add_plugins(AttributePlugin)
            .add_plugins(ModifierPlugin)
            .add_plugins(AggregatorPlugin)
            .add_plugins(GameplayContextPlugin);

        // Capabilities — Logic Skeleton
        app.add_plugins(SpecPlugin)
            .add_plugins(ConditionPlugin)
            .add_plugins(TriggerPlugin)
            .add_plugins(EventPlugin);

        // Capabilities — Behavior
        app.add_plugins(AbilityPlugin)
            .add_plugins(TargetingPlugin)
            .add_plugins(ExecutionPlugin)
            .add_plugins(EffectPlugin)
            .add_plugins(StackingPlugin)
            .add_plugins(CuePlugin);

        // Capabilities — Runtime
        app.add_plugins(RuntimePlugin);

        // Business Domains — Foundation
        app.add_plugins(TacticalPlugin)
            .add_plugins(TerrainPlugin)
            .add_plugins(FactionPlugin);

        // Business Domains — Core
        app.add_plugins(CombatPlugin)
            .add_plugins(SpellPlugin)
            .add_plugins(ReactionPlugin)
            .add_plugins(ProgressionPlugin)
            .add_plugins(InventoryPlugin)
            .add_plugins(PartyPlugin)
            .add_plugins(CampRestPlugin);

        // Business Domains — Narrative & Economy
        app.add_plugins(NarrativePlugin)
            .add_plugins(QuestPlugin)
            .add_plugins(EconomyPlugin)
            .add_plugins(CraftingPlugin)
            .add_plugins(SummonPlugin);
    }
}
