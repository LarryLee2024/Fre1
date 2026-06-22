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
use super::projections::battle::{
    on_battle_started_projection, on_character_panel_projection, on_damage_dealt_projection,
    on_effect_applied_projection, on_turn_ended_projection, on_turn_started_projection,
    on_turn_started_skill_projection, on_unit_died_projection,
};
use super::projections::economy::on_currency_changed_projection;
use super::screens::ScreenPlugin;
use super::settings::{UiSettings, load_settings};
use super::theme::ThemePlugin;
use super::view_models::{
    UiStore, battle_hud::BattleHudVm, character_panel::CharacterPanelVm, economy::EconomyVm,
    inventory::InventoryVm, shop::ShopPanelVm, skill_panel::SkillPanelVm,
};
use super::widgets::WidgetsPlugin;

/// UiPlugin — L3 UI 表现层入口
///
/// 注册顺序不可变：Theme → Focus → Primitives → Widgets → Screens → Overlay → Projections。
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // 0. Binding + ViewModel + Settings 基础设施
        app.init_resource::<UiStore>();
        app.register_type::<UiStore>();
        app.register_type::<BattleHudVm>();
        app.register_type::<CharacterPanelVm>();
        app.register_type::<SkillPanelVm>();
        app.register_type::<InventoryVm>();
        app.register_type::<ShopPanelVm>();
        app.register_type::<EconomyVm>();
        app.register_type::<Dirty<BattleHudVm>>();
        app.register_type::<Dirty<CharacterPanelVm>>();
        app.register_type::<Dirty<SkillPanelVm>>();
        app.register_type::<Dirty<InventoryVm>>();
        app.register_type::<Dirty<ShopPanelVm>>();
        app.register_type::<Dirty<EconomyVm>>();
        app.init_resource::<InventoryVm>();
        app.init_resource::<ShopPanelVm>();
        app.init_resource::<EconomyVm>();
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

        // 7. Projections — 领域事件到 ViewModel 的 Observer 转换
        app.add_observer(on_battle_started_projection);
        app.add_observer(on_turn_started_projection);
        app.add_observer(on_turn_ended_projection);
        app.add_observer(on_character_panel_projection);
        app.add_observer(on_effect_applied_projection);
        app.add_observer(on_turn_started_skill_projection);
        app.add_observer(on_damage_dealt_projection);
        app.add_observer(on_unit_died_projection);

        // Economy — 金币变更投影
        app.add_observer(on_currency_changed_projection);

        // 8. Bridge — UiCommand 到 GameCommand 的桥接（Observer 模式）
        app.add_observer(process_ui_commands);
    }
}
