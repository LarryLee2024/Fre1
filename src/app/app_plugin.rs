//! AppPlugin — Composition Root
//!
//! 唯一知道所有层的 Plugin，按 Phase 0–11 顺序注册。
//! 详见 `docs/01-architecture/README.md` §6.1

use crate::core::CorePlugin;
use crate::core::domains::combat::integration::replay::CombatReplayBridgePlugin;
use crate::infra::{
    camera::CameraPlugin, input::InputPlugin, localization, logging::LoggingPlugin,
    pipeline::PipelinePlugin, registry::RegistryPlugin, replay::ReplayPlugin, save::SavePlugin,
};
use crate::ui::UiPlugin;
use crate::{
    app::scenes::{ScenePlugin, test_battle::TestBattlePlugin},
    content::ContentPlugin,
    modding::ModdingPlugin,
    shared::SharedPlugin,
};
use bevy::prelude::*;

#[cfg(feature = "dev")]
use crate::tools::DevToolsPlugin;

/// 游戏主 Plugin（Composition Root）——唯一知道所有层的装配入口。
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
            .add_plugins(InputPlugin)
            .add_plugins(CameraPlugin)
            .add_plugins(LoggingPlugin)
            .add_plugins(localization::LocalizationPlugin::new());

        // ── Replay→Combat 桥接层（必须在 CombatPlugin + ReplayPlugin 之后注册）──
        app.add_plugins(CombatReplayBridgePlugin);

        // ════════════════════════════════════════════
        // Phase 9: Game State Management
        // ════════════════════════════════════════════
        // GameState 注册 + StateTransitionQueue + ScenePlugin。
        // 详见 ADR-050。
        app.add_plugins(ScenePlugin);

        // ── 测试战斗场景 Plugin ──
        app.add_plugins(TestBattlePlugin);

        // ════════════════════════════════════════════
        // Phase 10: Cross-cutting
        // ════════════════════════════════════════════
        app.add_plugins(ContentPlugin).add_plugins(ModdingPlugin);

        // ════════════════════════════════════════════
        // Phase 11: UI Presentation Layer (L3)
        // ════════════════════════════════════════════
        // UiPlugin must be registered after Localization (Phase 8),
        // ScenePlugin (Phase 9), and Content/Modding (Phase 10).
        // See docs/06-ui/01-architecture/architecture.md §8.1
        app.add_plugins(UiPlugin);

        #[cfg(feature = "dev")]
        app.add_plugins(DevToolsPlugin);
    }
}
