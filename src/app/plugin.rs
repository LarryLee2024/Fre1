//! AppPlugin：游戏装配入口
//!
//! Layer 1 职责：组装整个游戏，只注册，不含逻辑。
//! 负责注册 DefaultPlugins、核心/数据层插件、UI/Input、调试工具。

use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{EguiGlobalSettings, EguiPlugin};

use crate::app::error_event::GameErrorMessage;
use crate::app::error_monitor;
use crate::content::ContentPlugin;
use crate::core::ability::AbilityPlugin;
use crate::core::ai::{AiBehaviorPlugin, AiPlugin};
use crate::core::attribute::AttributeDefPlugin;
use crate::core::battle::BattlePlugin;
use crate::core::buff::BuffPlugin;
use crate::core::campaign::CampaignPlugin;
use crate::core::character::CharacterPlugin;
use crate::core::effect::EffectPlugin;
use crate::core::equipment::EquipmentPlugin;
use crate::core::inventory::InventoryPlugin;
use crate::core::map::MapPlugin;
use crate::core::modifier::ModifierRulePlugin;
use crate::core::tag::TagDefPlugin;
use crate::core::tag::TagPlugin;
use crate::core::targeting::TargetingPlugin;
use crate::core::trigger::TriggerPlugin;
use crate::core::turn::TurnPlugin;
use crate::infrastructure::assets::AssetsPlugin;
use crate::infrastructure::audit::AuditPlugin;
use crate::infrastructure::localization::LocalizationPlugin;
use crate::infrastructure::logging::LogPlugin;
use crate::input::InputPlugin;
use crate::shared::SharedPlugin;
use crate::ui::UiPlugin;

use crate::debug::DebugPlugin;

/// App 层统一插件：组装游戏所有 Plugin
///
/// 封装了 DefaultPlugins 配置、核心 game logic plugins、Infrastructure、
/// UI 和 Debug 工具的完整注册流程。
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
        // egui 基础设施（调试面板 / World Inspector 依赖）
        // 禁止自动挂载 PrimaryEguiContext 到第一个 Camera（相机销毁会带走 egui context）
        .insert_resource(EguiGlobalSettings {
            auto_create_primary_context: false,
            ..default()
        })
        .add_plugins(EguiPlugin::default())
        // 错误消息通道 + 监控系统（在 Core 插件之前注册，确保错误可被捕获）
        .add_message::<GameErrorMessage>()
        .add_systems(Update, error_monitor::error_monitor)
        // Shared 层 — 零依赖基础设施（ids, error, events, audit 等）
        .add_plugins(SharedPlugin)
        // 本地化基础设施（语言切换、文本解析）
        .add_plugins(LocalizationPlugin)
        // Content 层 — 合约声明与加载协调
        .add_plugins(ContentPlugin)
        // ── ADR-025 七领域 DAG 层序 ──
        // Layer 1：无依赖的基础类型注册
        .add_plugins((TagPlugin, TagDefPlugin, AttributeDefPlugin))
        // Layer 2：依赖 tag + attribute
        .add_plugins(ModifierRulePlugin)
        // Layer 3：依赖 modifier + tag
        .add_plugins(EffectPlugin)
        // Layer 4：平行依赖 effect（共 3 个，无相互依赖）
        .add_plugins((BuffPlugin, TargetingPlugin, TriggerPlugin))
        // Layer 5：依赖所有下层（tag/modifier/effect/buff/targeting/trigger）
        .add_plugins(AbilityPlugin)
        // Layer 6：非七领域的数据层插件
        .add_plugins((AiBehaviorPlugin, EquipmentPlugin, InventoryPlugin))
        // 基础设施层（日志、审计）
        .add_plugins((LogPlugin, AuditPlugin))
        // 游戏逻辑插件
        .add_plugins((
            AssetsPlugin,
            TurnPlugin,
            MapPlugin,
            CharacterPlugin,
            BattlePlugin,
            AiPlugin,
        ))
        // 战役模块（在 AssetsPlugin 之后，确保 LevelRegistry 已就绪）
        .add_plugins(CampaignPlugin)
        // 表现层插件
        .add_plugins((UiPlugin, InputPlugin))
        // 调试工具
        .add_plugins(DebugPlugin);
    }
}
