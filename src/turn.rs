// 回合管理模块：状态机、回合切换、SystemSet 编排

use crate::character::{Faction, Unit};
use bevy::prelude::*;

/// 游戏主状态
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
    GameOver,
}

/// 回合阶段（SubState，仅在 InGame 时激活）
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, SubStates)]
#[source(AppState = AppState::InGame)]
pub enum TurnPhase {
    #[default]
    /// 选择单位
    SelectUnit,
    /// 移动阶段
    MoveUnit,
    /// 行动菜单（右键弹出）
    ActionMenu,
    /// 选择攻击目标
    SelectTarget,
    /// 执行攻击
    ExecuteAction,
    /// 待机
    WaitAction,
    /// 回合结束
    TurnEnd,
}

/// 当前回合阵营与回合数
#[derive(Resource)]
pub struct TurnState {
    pub current_faction: Faction,
    pub turn_number: u32,
}

impl Default for TurnState {
    fn default() -> Self {
        Self {
            current_faction: Faction::Player,
            turn_number: 1,
        }
    }
}

/// AI 行动延迟计时器
#[derive(Resource)]
pub struct AiTimer {
    pub timer: Timer,
}

/// 标记是否需要结算持续效果（防止 SelectUnit 多次进入时重复结算）
#[derive(Resource, Default)]
pub struct NeedsResolve(pub bool);

impl Default for AiTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.8, TimerMode::Once),
        }
    }
}

/// 跨插件系统集合：显式控制 OnEnter(InGame) 生成顺序
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameSet {
    Camera,
    Map,
    Unit,
    Ui,
}

/// 强制结束当前阵营回合（玩家按 E 结束回合时设置）
#[derive(Resource, Default)]
pub struct ForceEndFaction(pub bool);

/// 回合管理插件
pub struct TurnPlugin;

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_sub_state::<TurnPhase>()
            .init_resource::<TurnState>()
            .init_resource::<AiTimer>()
            .init_resource::<NeedsResolve>()
            .init_resource::<ForceEndFaction>()
            .configure_sets(
                OnEnter(AppState::InGame),
                (GameSet::Camera, GameSet::Map, GameSet::Unit, GameSet::Ui).chain(),
            )
            .add_systems(OnEnter(TurnPhase::TurnEnd), turn_end_on_enter);
    }
}

/// 回合结束（OnEnter）
pub fn turn_end_on_enter(
    mut turn_state: ResMut<TurnState>,
    mut units: Query<&mut Unit>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    mut ai_timer: ResMut<AiTimer>,
    mut needs_resolve: ResMut<NeedsResolve>,
    mut force_end: ResMut<ForceEndFaction>,
) {
    let current_faction = turn_state.current_faction;

    // 玩家按 E 强制结束回合时，标记所有当前阵营单位已行动
    if force_end.0 {
        for mut unit in units.iter_mut() {
            if unit.faction == current_faction {
                unit.acted = true;
            }
        }
        force_end.0 = false;
    }

    // 检查当前阵营是否所有单位都已行动
    let all_acted = units
        .iter_mut()
        .filter(|u| u.faction == current_faction)
        .all(|u| u.acted);

    if all_acted {
        let next_faction = match current_faction {
            Faction::Player => Faction::Enemy,
            Faction::Enemy => {
                turn_state.turn_number += 1;
                Faction::Player
            }
        };
        turn_state.current_faction = next_faction;

        // 阵营切换时标记需要结算持续效果
        needs_resolve.0 = true;

        // 重置新阵营单位的行动状态
        for mut unit in units.iter_mut() {
            if unit.faction == next_faction {
                unit.acted = false;
            }
        }

        // 切换到敌方时重置 AI 计时器
        if next_faction == Faction::Enemy {
            ai_timer.timer.reset();
        }
    } else if current_faction == Faction::Enemy {
        // 同阵营未全部行动，重置 AI 计时器让下一个敌方单位行动
        ai_timer.timer.reset();
    }

    next_phase.set(TurnPhase::SelectUnit);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_turn_test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
            .init_state::<TurnPhase>()
            .init_resource::<TurnState>()
            .init_resource::<AiTimer>()
            .init_resource::<NeedsResolve>()
            .init_resource::<ForceEndFaction>()
            .add_systems(OnEnter(TurnPhase::TurnEnd), turn_end_on_enter);
        app
    }

    fn spawn_unit(app: &mut App, faction: Faction, acted: bool) -> Entity {
        app.world_mut()
            .spawn(Unit { faction, acted })
            .id()
    }

    #[test]
    fn 回合结束_玩家全部行动后切换到敌方() {
        let mut app = setup_turn_test_app();

        spawn_unit(&mut app, Faction::Player, true);
        spawn_unit(&mut app, Faction::Player, true);

        app.world_mut().resource_mut::<TurnState>().current_faction = Faction::Player;

        app.world_mut()
            .resource_mut::<NextState<TurnPhase>>()
            .set(TurnPhase::TurnEnd);
        app.update();

        let turn_state = app.world().resource::<TurnState>();
        assert_eq!(turn_state.current_faction, Faction::Enemy);
        assert_eq!(turn_state.turn_number, 1);
    }

    #[test]
    fn 回合结束_敌方全部行动后切换到玩家并增加回合数() {
        let mut app = setup_turn_test_app();

        spawn_unit(&mut app, Faction::Enemy, true);
        spawn_unit(&mut app, Faction::Enemy, true);

        app.world_mut().resource_mut::<TurnState>().current_faction = Faction::Enemy;

        app.world_mut()
            .resource_mut::<NextState<TurnPhase>>()
            .set(TurnPhase::TurnEnd);
        app.update();

        let turn_state = app.world().resource::<TurnState>();
        assert_eq!(turn_state.current_faction, Faction::Player);
        assert_eq!(turn_state.turn_number, 2);
    }

    #[test]
    fn 回合结束_未全部行动时不切换阵营() {
        let mut app = setup_turn_test_app();

        spawn_unit(&mut app, Faction::Player, true);
        spawn_unit(&mut app, Faction::Player, false);

        app.world_mut().resource_mut::<TurnState>().current_faction = Faction::Player;

        app.world_mut()
            .resource_mut::<NextState<TurnPhase>>()
            .set(TurnPhase::TurnEnd);
        app.update();

        let turn_state = app.world().resource::<TurnState>();
        assert_eq!(turn_state.current_faction, Faction::Player);
    }

    #[test]
    fn 回合结束_阵营切换时重置新阵营单位行动状态() {
        let mut app = setup_turn_test_app();

        spawn_unit(&mut app, Faction::Player, true);
        let enemy = spawn_unit(&mut app, Faction::Enemy, true);

        app.world_mut().resource_mut::<TurnState>().current_faction = Faction::Player;

        app.world_mut()
            .resource_mut::<NextState<TurnPhase>>()
            .set(TurnPhase::TurnEnd);
        app.update();

        let unit = app.world().get::<Unit>(enemy).unwrap();
        assert!(!unit.acted);
    }

    #[test]
    fn 回合结束_强制结束回合时标记所有单位已行动() {
        let mut app = setup_turn_test_app();

        spawn_unit(&mut app, Faction::Player, false);
        spawn_unit(&mut app, Faction::Player, false);

        app.world_mut().resource_mut::<ForceEndFaction>().0 = true;

        app.world_mut().resource_mut::<TurnState>().current_faction = Faction::Player;

        app.world_mut()
            .resource_mut::<NextState<TurnPhase>>()
            .set(TurnPhase::TurnEnd);
        app.update();

        let turn_state = app.world().resource::<TurnState>();
        assert_eq!(turn_state.current_faction, Faction::Enemy);
    }

    #[test]
    fn 回合结束_进入后总是切换到_select_unit() {
        let mut app = setup_turn_test_app();

        spawn_unit(&mut app, Faction::Player, true);

        app.world_mut().resource_mut::<TurnState>().current_faction = Faction::Player;

        app.world_mut()
            .resource_mut::<NextState<TurnPhase>>()
            .set(TurnPhase::TurnEnd);
        app.update();

        let phase = app.world().resource::<State<TurnPhase>>();
        assert_eq!(*phase.get(), TurnPhase::SelectUnit);
    }

    #[test]
    fn 回合结束_needs_resolve标记在阵营切换时设置() {
        let mut app = setup_turn_test_app();

        spawn_unit(&mut app, Faction::Player, true);
        spawn_unit(&mut app, Faction::Player, true);

        app.world_mut().resource_mut::<TurnState>().current_faction = Faction::Player;

        app.world_mut()
            .resource_mut::<NextState<TurnPhase>>()
            .set(TurnPhase::TurnEnd);
        app.update();

        let needs_resolve = app.world().resource::<NeedsResolve>();
        assert!(needs_resolve.0);
    }
}
