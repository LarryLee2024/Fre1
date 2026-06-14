//! AppPlugin：游戏装配入口
//!
//! Layer 1 职责：组装整个游戏，只注册，不含逻辑。
//! 负责注册 DefaultPlugins、核心/数据层插件、UI/Input、调试工具。

use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{EguiGlobalSettings, EguiPlugin};

use crate::content::ContentPlugin;
use crate::core::ai::{AiBehaviorPlugin, AiPlugin};
use crate::core::attribute_def::AttributeDefPlugin;
use crate::core::battle::BattlePlugin;
use crate::core::buff::BuffPlugin;
use crate::core::campaign::CampaignPlugin;
use crate::core::character::CharacterPlugin;
use crate::core::effect::EffectPlugin;
use crate::core::equipment::EquipmentPlugin;
use crate::core::inventory::InventoryPlugin;
use crate::core::map::MapPlugin;
use crate::core::modifier_rule::ModifierRulePlugin;
use crate::core::skill::SkillPlugin;
use crate::core::tag_def::TagDefPlugin;
use crate::core::turn::TurnPlugin;
use crate::infrastructure::assets::AssetsPlugin;
use crate::infrastructure::audit::AuditPlugin;
use crate::infrastructure::logging::LogPlugin;
use crate::input::InputPlugin;
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
        // Content 层 — 合约声明与加载协调
        .add_plugins(ContentPlugin)
        // 数据层插件
        .add_plugins((
            SkillPlugin,
            BuffPlugin,
            AiBehaviorPlugin,
            EquipmentPlugin,
            InventoryPlugin,
        ))
        // 核心层插件
        .add_plugins((
            EffectPlugin,
            ModifierRulePlugin,
            AttributeDefPlugin,
            TagDefPlugin,
        ))
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
