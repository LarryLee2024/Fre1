use super::action_menu::ActionMenuPlugin;
use super::camera::CameraPlugin;
use super::combat_log_handler;
use super::combat_preview::CombatPreviewPlugin;
use super::combat_vfx_handler;
use super::command_handler::handle_ui_commands;
use super::events::UiCommand;
use super::focus::{UiFocusState, update_ui_focus_state};
use super::panels::{
    ActionHintPlugin, CombatLogPanelPlugin, InventoryPanelPlugin, TurnIndicatorPlugin,
    UnitInfoPlugin,
};
use super::settings::{
    AccessibilitySettings, ColorBlindMode, ColorScheme, GameSettings, GameplaySettings, UiSettings,
    save_settings_on_change,
};
use super::theme::UiTheme;
use super::tile_info::TileInfoPlugin;
use super::vfx::VfxPlugin;
use crate::battle::CombatLogCollapsed;
use crate::character::Faction;
use crate::turn::{AppState, TurnState};
use crate::ui::view_models::*;
use bevy::prelude::*;

/// UI 插件（组合所有 UI 子插件）
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiTheme>()
            .insert_resource(GameSettings::load())
            .add_message::<UiCommand>()
            .init_resource::<SelectedUnitView>()
            .init_resource::<TurnInfoView>()
            .init_resource::<GameOverState>()
            .init_resource::<CombatPreviewView>()
            .init_resource::<HoveredEntity>()
            .init_resource::<CombatLogCollapsed>()
            .init_resource::<UiFocusState>()
            // 注册 Reflect 类型
            .register_type::<CoreAttrEntry>()
            .register_type::<DerivedAttrEntry>()
            .register_type::<BuffEntry>()
            .register_type::<SkillEntry>()
            .register_type::<TraitEntry>()
            .register_type::<EquipmentSlotEntry>()
            .register_type::<InventoryEntry>()
            .register_type::<HoveredEntity>()
            .register_type::<SelectedUnitView>()
            .register_type::<CombatPreviewView>()
            .register_type::<TurnInfoView>()
            .register_type::<GameOverState>()
            .register_type::<UiFocusState>()
            // GameSettings Reflect 注册
            .register_type::<GameSettings>()
            .register_type::<UiSettings>()
            .register_type::<ColorScheme>()
            .register_type::<AccessibilitySettings>()
            .register_type::<ColorBlindMode>()
            .register_type::<GameplaySettings>()
            .add_plugins((
                CameraPlugin,
                TurnIndicatorPlugin,
                UnitInfoPlugin,
                CombatLogPanelPlugin,
                InventoryPanelPlugin,
                ActionHintPlugin,
                ActionMenuPlugin,
                TileInfoPlugin,
                VfxPlugin,
                CombatPreviewPlugin,
            ))
            .add_systems(
                Update,
                (
                    update_selected_unit_view,
                    update_turn_info_view,
                    update_game_over_state,
                    update_combat_preview_view,
                    update_acted_unit_color,
                    // UI 焦点状态更新
                    update_ui_focus_state,
                    // 设置变更自动保存
                    save_settings_on_change,
                    // 战斗日志表现层：监听 Message 写入 CombatLog
                    combat_log_handler::on_damage_applied,
                    combat_log_handler::on_heal_applied,
                    combat_log_handler::on_character_died_log,
                    combat_log_handler::on_stun_applied,
                    combat_log_handler::on_dot_applied,
                    combat_log_handler::on_hot_applied,
                    combat_log_handler::on_item_equipped,
                    combat_log_handler::on_item_unequipped,
                    // 战斗 VFX 表现层：监听 Message 生成飘字
                    combat_vfx_handler::on_damage_vfx,
                    combat_vfx_handler::on_dot_vfx,
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                handle_ui_commands.run_if(in_state(AppState::InGame).and(player_turn)),
            );
    }
}

/// 只在玩家回合运行
fn player_turn(turn_state: Res<TurnState>) -> bool {
    turn_state.current_faction == Faction::Player
}

/// 已行动单位颜色变灰
fn update_acted_unit_color(
    mut units: Query<(&crate::character::Unit, &mut Sprite), Without<crate::character::MovingUnit>>,
) {
    use crate::ui::theme::faction_color;
    for (unit, mut sprite) in &mut units {
        let base_color = faction_color(unit.faction);
        if unit.acted {
            let mut hsla = Hsla::from(base_color);
            hsla.saturation *= 0.2;
            hsla.lightness = hsla.lightness * 0.5 + 0.25;
            sprite.color = Color::from(hsla);
        } else {
            sprite.color = base_color;
        }
    }
}
