//! ProgressionPlugin — 成长养成领域 Plugin
//!
//! 注册成长组件、事件和系统。
//! 处理经验获取、等级提升、天赋解锁、子职选择、ASI。
//!
//! 详见 ADR-030

use bevy::prelude::*;

use super::components::{
    ClassLevels, Experience, LevelProgressionTable, ProgressionMarker, SubclassChoice, TalentTree,
};
use super::systems::progression_system::{
    check_max_level_system, enforce_xp_invariant, handle_level_up, on_talent_unlocked,
};

pub struct ProgressionPlugin;

impl Plugin for ProgressionPlugin {
    fn build(&self, app: &mut App) {
        // ── 注册 Component 类型 ──
        app.register_type::<Experience>();
        app.register_type::<ClassLevels>();
        app.register_type::<TalentTree>();
        app.register_type::<SubclassChoice>();
        app.register_type::<ProgressionMarker>();

        // ── 初始化 Resource ──
        app.init_resource::<LevelProgressionTable>();

        // ── 注册 Observer System ──
        app.add_observer(enforce_xp_invariant);
        app.add_observer(handle_level_up);
        app.add_observer(on_talent_unlocked);

        // ── 注册普通 System ──
        app.add_systems(Update, check_max_level_system);
    }
}
