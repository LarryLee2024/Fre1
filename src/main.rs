mod ai;
mod assets;
mod battle;
mod buff;
mod camera;
mod character;
mod core;
mod input;
mod map;
mod skill;
mod status;
mod turn;
mod ui;

use ai::AiPlugin;
use ai::AiBehaviorPlugin;
use assets::AssetsPlugin;
use battle::BattlePlugin;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use buff::BuffPlugin;
use camera::CameraPlugin;
use character::CharacterPlugin;
use core::attribute_def::AttributeDefPlugin;
use core::effect::EffectPlugin;
use core::modifier_rule::ModifierRulePlugin;
use core::tag_def::TagDefPlugin;
use input::InputPlugin;
use skill::SkillPlugin;
use status::StatusPlugin;
use turn::{AppState, TurnPlugin};
use ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "回合制战棋".to_string(),
                resolution: (1024u32, 768u32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()))
        // 数据层插件
        .add_plugins((
            SkillPlugin,
            BuffPlugin,
            AiBehaviorPlugin,
        ))
        // 核心层插件
        .add_plugins((
            EffectPlugin,
            ModifierRulePlugin,
            AttributeDefPlugin,
            TagDefPlugin,
        ))
        // 游戏逻辑插件
        .add_plugins((
            AssetsPlugin,
            TurnPlugin,
            CameraPlugin,
            CharacterPlugin,
            BattlePlugin,
            AiPlugin,
            StatusPlugin,
        ))
        // 表现层插件
        .add_plugins((
            UiPlugin,
            InputPlugin,
        ))
        .add_systems(Startup, |mut next: ResMut<NextState<AppState>>| {
            next.set(AppState::InGame);
        })
        .run();
}
