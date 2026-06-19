//! AppPlugin — Composition Root
//!
//! 唯一知道所有层的 Plugin，按 Phase 0–9 顺序注册。
//! 详见 `docs/01-architecture/README.md` §6.1

use crate::core::CorePlugin;
use crate::core::domains::combat::integration::replay::CombatReplayBridgePlugin;
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
        // Phase 1–7: Core (L1) — 委托给 CorePlugin
        // ════════════════════════════════════════════
        app.add_plugins(CorePlugin);

        // ════════════════════════════════════════════
        // Phase 8: Infrastructure (L2)
        // ════════════════════════════════════════════
        app.add_plugins(RegistryPlugin)
            .add_plugins(PipelinePlugin)
            .add_plugins(ReplayPlugin)
            .add_plugins(SavePlugin)
            .add_plugins(InputPlugin);

        // ── Replay→Combat 桥接层（必须在 CombatPlugin + ReplayPlugin 之后注册）──
        app.add_plugins(CombatReplayBridgePlugin);

        // ════════════════════════════════════════════
        // Phase 9: Cross-cutting
        // ════════════════════════════════════════════
        app.add_plugins(ContentPlugin).add_plugins(ModdingPlugin);

        #[cfg(feature = "dev")]
        app.add_plugins(DevToolsPlugin);
    }
}
