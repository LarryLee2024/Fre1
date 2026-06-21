//! UiPlugin — UI 表现层主 Plugin
//!
//! 注册顺序（Phase 11，在 Infra Phase 8 和 ScenePlugin Phase 9 之后）：
//! 1. ThemePlugin       — 主题与设计令牌
//! 2. FocusPlugin       — 键盘/手柄焦点导航系统
//! 3. PrimitivesPlugin  — UI 原语（Button/Panel/Text/List/etc.）
//! 4. WidgetsPlugin     — 游戏业务控件（当前为骨架）
//! 5. ScreenPlugin      — 全屏页面（主菜单等）
//! 6. OverlayPlugin     — UI 浮层系统（通知/工具提示/伤害数字）
//! 7. Projections       — Domain Event 到 ViewModel 的 Observer 转换
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §8

use bevy::prelude::*;

use super::application::bridge::process_ui_commands;
use super::binding::Dirty;
use super::focus::FocusPlugin;
use super::overlay::OverlayPlugin;
use super::primitives::PrimitivesPlugin;
use super::projections::battle::{on_effect_applied_projection, on_turn_started_projection};
use super::screens::ScreenPlugin;
use super::settings::{UiSettings, load_settings};
use super::theme::ThemePlugin;
use super::view_models::{
    UiStore, battle_hud::BattleHudVm, character_panel::CharacterPanelVm, skill_panel::SkillPanelVm,
};
use super::widgets::WidgetsPlugin;

/// UiPlugin — L3 UI 表现层入口
///
/// 注册顺序不可变：Theme → Focus → Primitives → Widgets → Screens → Overlay → Projections。
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // 0. Binding + ViewModel + Settings infrastructure
        app.init_resource::<UiStore>();
        app.register_type::<UiStore>();
        app.register_type::<BattleHudVm>();
        app.register_type::<CharacterPanelVm>();
        app.register_type::<SkillPanelVm>();
        app.register_type::<Dirty<BattleHudVm>>();
        app.register_type::<Dirty<CharacterPanelVm>>();
        app.register_type::<Dirty<SkillPanelVm>>();
        app.insert_resource(load_settings());
        app.register_type::<UiSettings>();

        // 1. Theme — 必须在所有 UI 组件之前注册
        app.add_plugins(ThemePlugin);
        // 2. Focus — 键盘/手柄焦点导航系统
        app.add_plugins(FocusPlugin);
        // 3. Primitives — 基础 UI 原语
        app.add_plugins(PrimitivesPlugin);
        // 4. Widgets — 游戏业务控件（骨架阶段）
        app.add_plugins(WidgetsPlugin);
        // 5. Screens — 全屏页面（主菜单等）
        app.add_plugins(ScreenPlugin);
        // 6. Overlay — UI 浮层系统
        app.add_plugins(OverlayPlugin);

        // 7. Projections — domain event to ViewModel observers
        app.add_observer(on_turn_started_projection);
        app.add_observer(on_effect_applied_projection);

        // 8. Bridge — UiCommand to GameCommand wiring (observer pattern)
        app.add_observer(process_ui_commands);
    }
}
