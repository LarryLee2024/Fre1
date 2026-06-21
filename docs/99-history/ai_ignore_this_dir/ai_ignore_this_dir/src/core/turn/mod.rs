/// 回合管理模块：状态机、行动队列、胜负检查、SystemSet 编排

/// TurnOrder 行动队列（敏捷驱动排序）
mod order;
/// AppState, TurnPhase, GameOverState 状态定义
mod state;
/// 胜负条件检查系统（数据驱动，读取 LevelConfig.VictoryConditionDef）
mod victory_check;

pub use order::*;
pub use state::*;
// LevelCompleted 供 battle/record、ui 等外部模块消费
#[allow(unused_imports)]
pub use victory_check::LevelCompleted;

use bevy::prelude::*;

use crate::core::battle::{CombatIntent, CombatLogCollapsed, CombatLogPanel, PrevPosition};
use crate::core::character::Unit;
use crate::core::map::TileSprite;
use crate::ui::view_models::{CombatPreviewView, HoveredEntity, SelectedUnitView};
use crate::ui::{
    ActionHint, ActionMenuEntity, CameraController, InventoryPanel, TurnIndicator, UiFocusState,
    UnitInfoPanel,
};

/// 回合管理插件
pub struct TurnPlugin;

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_sub_state::<TurnPhase>()
            .init_resource::<TurnState>()
            .init_resource::<TurnOrder>()
            .init_resource::<AiTimer>()
            .init_resource::<NeedsResolve>()
            .init_resource::<GameOverState>()
            // 注册 Reflect 类型
            .register_type::<TurnPhase>()
            .register_type::<TurnOrder>()
            .register_type::<TurnState>()
            .register_type::<AiTimer>()
            .register_type::<NeedsResolve>()
            .register_type::<GameOverState>()
            .add_message::<TurnStarted>()
            .add_message::<TurnEnded>()
            .add_message::<ForceEndTurn>()
            .add_message::<victory_check::LevelCompleted>()
            .configure_sets(
                OnEnter(AppState::InGame),
                (GameSet::Camera, GameSet::Map, GameSet::Unit, GameSet::Ui).chain(),
            )
            // 离开战斗状态时清理所有 InGame entities 并重置资源
            .add_systems(OnExit(AppState::InGame), cleanup_ingame)
            // 胜负检查在回合结束处理之前运行（读取当前回合号，检查完再递增）
            .add_systems(
                OnEnter(TurnPhase::TurnEnd),
                (
                    victory_check::check_victory_conditions.before(turn_end_on_enter),
                    turn_end_on_enter,
                ),
            )
            // 兜底系统：全灭玩家即失败（绝对不变量防御性保障）
            // 仅在 Playing 状态时运行，终态后自动跳过（性能优化）
            .add_systems(
                Update,
                victory_check::check_all_dead_safety
                    .run_if(in_state(AppState::InGame))
                    .run_if(|game_over: Res<GameOverState>| *game_over == GameOverState::Playing),
            )
            .add_systems(
                OnEnter(AppState::InGame),
                init_turn_order.after(GameSet::Unit),
            );
    }
}

/// 清理 InGame 状态：Despawn 所有战斗实体 + 重置运行时资源
///
/// OnEnter(InGame) 的 spawn 系统负责创建，此系统负责销毁。
/// 保证进出 InGame 不会残留上一局的 entities/resources。
fn cleanup_ingame(
    mut commands: Commands,
    // 标记组件 Query — 所有在 OnEnter(InGame) 中生成的实体
    cameras: Query<Entity, With<CameraController>>,
    tiles: Query<Entity, With<TileSprite>>,
    units: Query<Entity, With<Unit>>,
    turn_indicators: Query<Entity, With<TurnIndicator>>,
    unit_info_panels: Query<Entity, With<UnitInfoPanel>>,
    combat_log_panels: Query<Entity, With<CombatLogPanel>>,
    inventory_panels: Query<Entity, With<InventoryPanel>>,
    action_hints: Query<Entity, With<ActionHint>>,
) {
    for e in &cameras {
        commands.entity(e).try_despawn();
    }
    for e in &tiles {
        commands.entity(e).try_despawn();
    }
    for e in &units {
        commands.entity(e).try_despawn();
    }
    for e in &turn_indicators {
        commands.entity(e).try_despawn();
    }
    for e in &unit_info_panels {
        commands.entity(e).try_despawn();
    }
    for e in &combat_log_panels {
        commands.entity(e).try_despawn();
    }
    for e in &inventory_panels {
        commands.entity(e).try_despawn();
    }
    for e in &action_hints {
        commands.entity(e).try_despawn();
    }
    // 重置运行时资源到默认值（init_resource 只插一次，不会自动重置）
    commands.insert_resource(TurnState::default());
    commands.insert_resource(TurnOrder::default());
    commands.insert_resource(GameOverState::default());
    commands.insert_resource(NeedsResolve::default());
    commands.insert_resource(AiTimer::default());
    commands.insert_resource(CombatIntent::default());
    commands.insert_resource(PrevPosition::default());
    commands.insert_resource(SelectedUnitView::default());
    commands.insert_resource(CombatPreviewView::default());
    commands.insert_resource(HoveredEntity::default());
    commands.insert_resource(CombatLogCollapsed::default());
    commands.insert_resource(ActionMenuEntity::default());
    commands.insert_resource(UiFocusState::default());
}
