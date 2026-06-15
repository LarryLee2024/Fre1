//! AppPlugin：游戏装配入口
//!
//! ADR-026：Plugin 注册顺序遵循 DAG 依赖图

use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{EguiGlobalSettings, EguiPlugin};

use crate::app::error_event::GameErrorMessage;
use crate::app::error_monitor;
use crate::content::ContentPlugin;
use crate::core::ability::AbilityPlugin;
use crate::core::ai::{AiBehaviorPlugin, AiPlugin};
use crate::core::attribute::AttributeDefPlugin;
use crate::core::battle::BattlePlugin;
#[allow(deprecated)]
use crate::core::buff::BuffPlugin;
use crate::core::campaign::CampaignPlugin;
use crate::core::character::CharacterPlugin;
use crate::core::cue::CuePlugin;
use crate::core::effect::EffectPlugin;
use crate::core::equipment::EquipmentPlugin;
use crate::core::execution::ExecutionPlugin;
use crate::core::inventory::InventoryPlugin;
use crate::core::map::MapPlugin;
use crate::core::modifier::ModifierRulePlugin;
use crate::core::stacking::StackingPlugin;
use crate::core::tag::TagDefPlugin;
use crate::core::tag::TagPlugin;
use crate::core::targeting::TargetingPlugin;
use crate::core::trigger::TriggerPlugin;
use crate::core::turn::TurnPlugin;
use crate::debug::DebugPlugin;
use crate::infrastructure::assets::AssetsPlugin;
use crate::infrastructure::audit::AuditPlugin;
use crate::infrastructure::localization::LocalizationPlugin;
use crate::infrastructure::logging::LogPlugin;
use crate::infrastructure::pipeline::BattlePipelinePlugin;
use crate::infrastructure::registry::RegistryPlugin;
use crate::infrastructure::replay::BattleReplayPlugin;
use crate::input::InputPlugin;
use crate::shared::SharedPlugin;
use crate::ui::UiPlugin;

/// App 层统一插件：组装游戏所有 Plugin
///
/// ADR-026 Plugin 注册顺序（DAG，禁止颠倒）：
/// 1. Registry → 2. Attribute + Tag → 3. Modifier → 4. Effect →
/// 5. Ability + Trigger + Targeting + Stacking + Execution →
/// 6. Cue → 7. Pipeline + Replay
pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let asset_path = format!("{manifest_dir}/assets");

        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "回合制战棋".to_string(),
                        resolution: (1024u32, 768u32).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: asset_path,
                    ..default()
                }),
        )
        .insert_resource(EguiGlobalSettings {
            auto_create_primary_context: false,
            ..default()
        })
        .add_plugins(EguiPlugin::default())
        .add_message::<GameErrorMessage>()
        .add_systems(Update, error_monitor::error_monitor)
        .add_plugins(SharedPlugin)
        // ═══ ADR-026 GAS 链 DAG 层序 ═══
        .add_plugins(RegistryPlugin)
        .add_plugins(LocalizationPlugin)
        .add_plugins(ContentPlugin)
        .add_plugins((AttributeDefPlugin, TagPlugin, TagDefPlugin))
        .add_plugins(ModifierRulePlugin)
        .add_plugins(EffectPlugin)
        .add_plugins((StackingPlugin, ExecutionPlugin))
        .add_plugins((AbilityPlugin, TargetingPlugin, TriggerPlugin))
        .add_plugins(CuePlugin)
        .add_plugins((BattlePipelinePlugin, BattleReplayPlugin))
        // ═══ 非 GAS 链模块 ═══
        .add_plugins((
            BuffPlugin,
            AiBehaviorPlugin,
            EquipmentPlugin,
            InventoryPlugin,
        ))
        .add_plugins((LogPlugin, AuditPlugin))
        .add_plugins((
            AssetsPlugin,
            TurnPlugin,
            MapPlugin,
            CharacterPlugin,
            BattlePlugin,
            AiPlugin,
        ))
        .add_plugins(CampaignPlugin)
        .add_plugins((UiPlugin, InputPlugin))
        .add_plugins(DebugPlugin);
    }
}
