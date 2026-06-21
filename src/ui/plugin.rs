//! UiPlugin — UI 表现层主 Plugin
//!
//! 注册顺序（Phase 11，在 Infra Phase 8 和 ScenePlugin Phase 9 之后）：
//! 1. ThemePlugin       — 主题与设计令牌
//! 2. FocusPlugin       — 键盘/手柄焦点导航系统
//! 3. PrimitivesPlugin  — UI 原语（Button/Panel/Text/List/etc.）
//! 4. WidgetsPlugin     — 游戏业务控件（当前为骨架）
//! 5. ScreenPlugin      — 全屏页面（主菜单等）
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §8

use bevy::prelude::*;

use super::binding::Dirty;
use super::focus::FocusPlugin;
use super::primitives::PrimitivesPlugin;
use super::screens::ScreenPlugin;
use super::theme::ThemePlugin;
use super::view_models::{battle_hud::BattleHudVm, character_panel::CharacterPanelVm, skill_panel::SkillPanelVm, UiStore};
use super::widgets::WidgetsPlugin;

/// UiPlugin — L3 UI 表现层入口
///
/// 注册顺序不可变：Theme → Focus → Primitives → Widgets → Screens。
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // 0. Binding + ViewModel infrastructure — registered before UI rendering
        app.init_resource::<UiStore>();
        app.register_type::<UiStore>();
        app.register_type::<BattleHudVm>();
        app.register_type::<CharacterPanelVm>();
        app.register_type::<SkillPanelVm>();
        app.register_type::<Dirty<BattleHudVm>>();
        app.register_type::<Dirty<CharacterPanelVm>>();
        app.register_type::<Dirty<SkillPanelVm>>();

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
    }
}
