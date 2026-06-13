/// UI 模块：面板、行动菜单、浮窗、视觉效果
/// 架构：widgets/ 基础库 + panels/ 面板模块 + 各功能模块

/// 行动菜单（攻击/技能/待机）
mod action_menu;
/// 摄像机控制（缩放/拖拽）
mod camera;
/// 战斗日志 Message 监听
mod combat_log_handler;
/// 战斗预览（伤害计算展示）
mod combat_preview;
/// 战斗 VFX（飘字/特效）
mod combat_vfx_handler;
/// UI 命令处理
mod command_handler;
/// UI 事件定义（UiCommand, MovementIntent）
pub mod events;
/// UI 焦点状态管理
mod focus;
/// 单位/格子高亮
mod highlight;
/// UI 面板子模块
mod panels;
/// GameSettings 用户偏好与 RON 持久化
pub mod settings;
/// UiTheme 视觉常量
pub mod theme;
/// 格子信息浮窗
mod tile_info;
/// 视觉效果系统
pub mod vfx;
/// ViewModel 数据结构
pub mod view_models;
/// 可复用 UI 构建块
mod widgets;

use crate::battle::CombatLogCollapsed;
use crate::character::Faction;
use crate::turn::{AppState, TurnState};
use crate::ui::view_models::*;
use bevy::prelude::*;

/// 公共 re-exports
pub use focus::{BlocksGameInput, UiFocusState};
pub use settings::GameSettings;
pub use theme::UiTheme;

/// UI 插件（组合所有 UI 子插件）
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<theme::UiTheme>()
            .insert_resource(settings::GameSettings::load())
            .add_message::<events::UiCommand>()
            .init_resource::<SelectedUnitView>()
            .init_resource::<TurnInfoView>()
            .init_resource::<CombatPreviewView>()
            .init_resource::<HoveredEntity>()
            .init_resource::<CombatLogCollapsed>()
            .init_resource::<focus::UiFocusState>()
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
            .register_type::<focus::UiFocusState>()
            // GameSettings Reflect 注册
            .register_type::<settings::GameSettings>()
            .register_type::<settings::UiSettings>()
            .register_type::<settings::ColorScheme>()
            .register_type::<settings::AccessibilitySettings>()
            .register_type::<settings::ColorBlindMode>()
            .register_type::<settings::GameplaySettings>()
            .add_plugins((
                camera::CameraPlugin,
                panels::TurnIndicatorPlugin,
                panels::UnitInfoPlugin,
                panels::CombatLogPanelPlugin,
                panels::InventoryPanelPlugin,
                panels::ActionHintPlugin,
                action_menu::ActionMenuPlugin,
                tile_info::TileInfoPlugin,
                vfx::VfxPlugin,
                combat_preview::CombatPreviewPlugin,
            ))
            .add_systems(
                Update,
                (
                    update_selected_unit_view,
                    update_turn_info_view,
                    update_combat_preview_view,
                    update_acted_unit_color,
                    // UI 焦点状态更新
                    focus::update_ui_focus_state,
                    // 设置变更自动保存
                    settings::save_settings_on_change,
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
                command_handler::handle_ui_commands
                    .run_if(in_state(AppState::InGame).and(player_turn)),
            );
    }
}

/// 只在玩家回合运行
fn player_turn(turn_state: Res<TurnState>) -> bool {
    turn_state.current_faction == Faction::Player
}

/// 已行动单位颜色变灰
fn update_acted_unit_color(
    theme: Res<UiTheme>,
    mut units: Query<(&crate::character::Unit, &mut Sprite), Without<crate::character::MovingUnit>>,
) {
    use crate::ui::theme::faction_color;
    for (unit, mut sprite) in &mut units {
        let base_color = faction_color(unit.faction, &theme);
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
