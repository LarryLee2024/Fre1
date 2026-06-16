//! AppPlugin — Composition Root
//!
//! 唯一知道所有层的 Plugin，按 Phase 0–9 顺序注册。
//! 详见 `docs/01-architecture/README.md` §6.1

use crate::core::{
    capabilities::{
        ability::AbilityPlugin, aggregator::AggregatorPlugin, attribute::AttributePlugin,
        condition::ConditionPlugin, cue::CuePlugin, effect::EffectPlugin, event::EventPlugin,
        execution::ExecutionPlugin, gameplay_context::GameplayContextPlugin,
        modifier::ModifierPlugin, runtime::RuntimePlugin, spec::SpecPlugin,
        stacking::StackingPlugin, tag::TagPlugin, targeting::TargetingPlugin,
        trigger::TriggerPlugin,
    },
    domains::{
        camp_rest::CampRestPlugin, combat::CombatPlugin, crafting::CraftingPlugin,
        economy::EconomyPlugin, faction::FactionPlugin, inventory::InventoryPlugin,
        narrative::NarrativePlugin, party::PartyPlugin, progression::ProgressionPlugin,
        quest::QuestPlugin, reaction::ReactionPlugin, spell::SpellPlugin, summon::SummonPlugin,
        tactical::TacticalPlugin, terrain::TerrainPlugin,
    },
};
use crate::infra::{
    input::InputPlugin, pipeline::PipelinePlugin, registry::RegistryPlugin, replay::ReplayPlugin,
    save::SavePlugin,
};
use crate::{content::ContentPlugin, modding::ModdingPlugin, shared::SharedPlugin};
use bevy::prelude::*;

#[cfg(feature = "dev")]
use crate::tools::DevToolsPlugin;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // ════════════════════════════════════════════
        // Phase 0: Core Bevy + Shared (L0)
        // ════════════════════════════════════════════
        app.add_plugins(DefaultPlugins).add_plugins(SharedPlugin);

        // ════════════════════════════════════════════
        // Phase 1: Capabilities — Foundation (L1 Core)
        // ════════════════════════════════════════════
        app.add_plugins(TagPlugin)
            .add_plugins(AttributePlugin)
            .add_plugins(ModifierPlugin)
            .add_plugins(AggregatorPlugin)
            .add_plugins(GameplayContextPlugin);

        // ════════════════════════════════════════════
        // Phase 2: Capabilities — Logic Skeleton (L1 Core)
        // ════════════════════════════════════════════
        app.add_plugins(SpecPlugin)
            .add_plugins(ConditionPlugin)
            .add_plugins(TriggerPlugin)
            .add_plugins(EventPlugin);

        // ════════════════════════════════════════════
        // Phase 3: Capabilities — Behavior (L1 Core)
        // ════════════════════════════════════════════
        app.add_plugins(AbilityPlugin)
            .add_plugins(TargetingPlugin)
            .add_plugins(ExecutionPlugin)
            .add_plugins(EffectPlugin)
            .add_plugins(StackingPlugin)
            .add_plugins(CuePlugin);

        // ════════════════════════════════════════════
        // Phase 4: Capabilities — Runtime (L1 Core)
        // ════════════════════════════════════════════
        app.add_plugins(RuntimePlugin);

        // ════════════════════════════════════════════
        // Phase 5: Business Domains — Foundation (L1 Core)
        // ════════════════════════════════════════════
        app.add_plugins(TacticalPlugin)
            .add_plugins(TerrainPlugin)
            .add_plugins(FactionPlugin);

        // ════════════════════════════════════════════
        // Phase 6: Business Domains — Core (L1 Core)
        // ════════════════════════════════════════════
        app.add_plugins(CombatPlugin)
            .add_plugins(SpellPlugin)
            .add_plugins(ReactionPlugin)
            .add_plugins(ProgressionPlugin)
            .add_plugins(InventoryPlugin)
            .add_plugins(PartyPlugin)
            .add_plugins(CampRestPlugin);

        // ════════════════════════════════════════════
        // Phase 7: Business Domains — Narrative & Economy (L1 Core)
        // ════════════════════════════════════════════
        app.add_plugins(NarrativePlugin)
            .add_plugins(QuestPlugin)
            .add_plugins(EconomyPlugin)
            .add_plugins(CraftingPlugin)
            .add_plugins(SummonPlugin);

        // ════════════════════════════════════════════
        // Phase 8: Infrastructure (L2)
        // ════════════════════════════════════════════
        app.add_plugins(RegistryPlugin)
            .add_plugins(PipelinePlugin)
            .add_plugins(ReplayPlugin)
            .add_plugins(SavePlugin)
            .add_plugins(InputPlugin);

        // ════════════════════════════════════════════
        // Phase 9: Cross-cutting
        // ════════════════════════════════════════════
        app.add_plugins(ContentPlugin).add_plugins(ModdingPlugin);

        #[cfg(feature = "dev")]
        app.add_plugins(DevToolsPlugin);
    }
}
