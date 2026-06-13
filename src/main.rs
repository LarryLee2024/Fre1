mod ai;
mod assets;
mod battle;
mod buff;
mod campaign;
mod character;
mod core;
mod debug;
mod equipment;
pub mod infrastructure;
mod input;
mod inventory;
mod map;
mod skill;
mod turn;
mod ui;

use ai::AiBehaviorPlugin;
use ai::AiPlugin;
use assets::AssetsPlugin;
use battle::BattlePlugin;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use buff::BuffPlugin;
use campaign::CampaignPlugin;
use character::CharacterPlugin;
use core::attribute_def::AttributeDefPlugin;
use core::effect::EffectPlugin;
use core::modifier_rule::ModifierRulePlugin;
use core::tag_def::TagDefPlugin;
use debug::DebugPlugin;
use equipment::EquipmentPlugin;
use infrastructure::audit::AuditPlugin;
use infrastructure::logging::LogPlugin;
use input::InputPlugin;
use inventory::InventoryPlugin;
use map::MapPlugin;
use skill::SkillPlugin;
use turn::TurnPlugin;
use ui::UiPlugin;

fn main() {
    // 资产路径：使用编译时项目根目录的绝对路径
    // Bevy AssetPlugin 以可执行文件目录为基准解析 file_path，需要用绝对路径
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let asset_path = format!("{manifest_dir}/assets");

    App::new()
        .add_plugins(
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
        .add_plugins(EguiPlugin::default())
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
        .add_plugins((UiPlugin, InputPlugin, DebugPlugin))
        .run();
}
