mod ai;
mod assets;
mod battle;
mod buff;
mod camera;
mod character;
mod gameplay;
mod input;
mod map;
mod skill;
mod status;
mod turn;
mod ui;

use ai::AiBehaviorPlugin;
use ai::AiPlugin;
use assets::AssetsPlugin;
use battle::BattlePlugin;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use buff::BuffPlugin;
use camera::CameraPlugin;
use character::CharacterPlugin;
use gameplay::attribute_def::AttributeDefPlugin;
use gameplay::effect::EffectPlugin;
use gameplay::modifier_rule::ModifierRulePlugin;
use gameplay::tag_def::TagDefPlugin;
use input::InputPlugin;
use map::MapPlugin;
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
        .add_plugins((SkillPlugin, BuffPlugin, AiBehaviorPlugin))
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
            MapPlugin,
            CharacterPlugin,
            BattlePlugin,
            AiPlugin,
            StatusPlugin,
        ))
        // 表现层插件
        .add_plugins((UiPlugin, InputPlugin))
        .add_systems(Startup, |mut next: ResMut<NextState<AppState>>| {
            next.set(AppState::InGame);
        })
        .run();
}
