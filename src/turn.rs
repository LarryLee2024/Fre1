// 回合管理模块：状态机、敏捷驱动行动队列、SystemSet 编排

use crate::character::{Faction, Unit};
use crate::gameplay::attribute::{AttributeKind, Attributes};
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

/// 回合行动队列：所有单位按 Initiative 降序排列
#[derive(Resource, Default, Debug)]
pub struct TurnOrder {
    /// 本回合行动顺序（按 Initiative 降序）
    pub queue: Vec<Entity>,
    /// 当前行动索引
    pub current_index: usize,
    /// 当前回合号
    pub turn_number: u32,
}

impl TurnOrder {
    /// 根据所有单位的 Initiative 降序排列，生成行动队列
    pub fn build(units: &[(Entity, f32)]) -> Vec<Entity> {
        let mut sorted: Vec<_> = units.to_vec();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        sorted.into_iter().map(|(e, _)| e).collect()
    }

    /// 获取当前应该行动的单位
    pub fn current_unit(&self) -> Option<Entity> {
        self.queue.get(self.current_index).copied()
    }

    /// 前进到下一个单位，返回 None 表示回合结束
    pub fn advance(&mut self) -> Option<Entity> {
        self.current_index += 1;
        self.current_unit()
    }

    /// 当前行动单位的阵营
    pub fn current_faction(&self, units: &Query<&Unit>) -> Option<Faction> {
        self.current_unit()
            .and_then(|e| units.get(e).ok())
            .map(|u| u.faction)
    }
}

/// 回合状态（保留兼容，供 UI 和日志使用）
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
            .init_resource::<TurnOrder>()
            .init_resource::<AiTimer>()
            .init_resource::<NeedsResolve>()
            .init_resource::<ForceEndFaction>()
            .configure_sets(
                OnEnter(AppState::InGame),
                (GameSet::Camera, GameSet::Map, GameSet::Unit, GameSet::Ui).chain(),
            )
            .add_systems(OnEnter(TurnPhase::TurnEnd), turn_end_on_enter)
            .add_systems(
                OnEnter(AppState::InGame),
                init_turn_order.after(GameSet::Unit),
            );
    }
}

/// 游戏开始时初始化行动队列（第一回合）
pub fn init_turn_order(
    mut turn_state: ResMut<TurnState>,
    mut turn_order: ResMut<TurnOrder>,
    units: Query<(Entity, &Unit, &Attributes)>,
    mut ai_timer: ResMut<AiTimer>,
) {
    // 第一回合，不增加回合数
    turn_order.turn_number = turn_state.turn_number;

    // 重建行动队列
    let unit_initiatives: Vec<(Entity, f32)> = units
        .iter()
        .map(|(entity, _, attrs)| (entity, attrs.get(AttributeKind::Initiative)))
        .collect();
    turn_order.queue = TurnOrder::build(&unit_initiatives);
    turn_order.current_index = 0;

    // 更新当前阵营为队列第一个单位的阵营
    if let Some(first_entity) = turn_order.current_unit() {
        if let Ok((_, unit, _)) = units.get(first_entity) {
            turn_state.current_faction = unit.faction;
        }
    }

    // 重置 AI 计时器
    ai_timer.timer.reset();
}

/// 回合结束（OnEnter）
/// 新逻辑：队列耗尽时触发，重建行动队列，回合数+1
pub fn turn_end_on_enter(
    mut turn_state: ResMut<TurnState>,
    mut turn_order: ResMut<TurnOrder>,
    mut units: Query<(Entity, &mut Unit, &Attributes)>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    mut ai_timer: ResMut<AiTimer>,
    mut needs_resolve: ResMut<NeedsResolve>,
    mut force_end: ResMut<ForceEndFaction>,
) {
    // 强制结束：跳过当前阵营剩余单位（标记已行动）
    if force_end.0 {
        // 不需要额外操作，队列自然耗尽
        force_end.0 = false;
    }

    // 回合数+1
    turn_state.turn_number += 1;
    turn_order.turn_number = turn_state.turn_number;

    // 重置所有单位的行动状态
    for (_, mut unit, _) in &mut units {
        unit.acted = false;
    }

    // 结算持续效果
    needs_resolve.0 = true;

    // 重建行动队列：所有存活单位按 Initiative 降序
    let unit_initiatives: Vec<(Entity, f32)> = units
        .iter()
        .map(|(entity, _, attrs)| (entity, attrs.get(AttributeKind::Initiative)))
        .collect();
    turn_order.queue = TurnOrder::build(&unit_initiatives);
    turn_order.current_index = 0;

    // 更新当前阵营为队列第一个单位的阵营
    if let Some(first_entity) = turn_order.current_unit() {
        if let Ok((_, unit, _)) = units.get(first_entity) {
            turn_state.current_faction = unit.faction;
        }
    }

    // 重置 AI 计时器
    ai_timer.timer.reset();

    next_phase.set(TurnPhase::SelectUnit);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 行动队列_按initiative降序排列() {
        let e1 = Entity::from_bits(1);
        let e2 = Entity::from_bits(2);
        let e3 = Entity::from_bits(3);
        let units = vec![(e1, 10.0), (e2, 20.0), (e3, 15.0)];
        let queue = TurnOrder::build(&units);
        assert_eq!(queue, vec![e2, e3, e1]);
    }

    #[test]
    fn 行动队列_相同initiative稳定排序() {
        let e1 = Entity::from_bits(1);
        let e2 = Entity::from_bits(2);
        let units = vec![(e1, 10.0), (e2, 10.0)];
        let queue = TurnOrder::build(&units);
        assert_eq!(queue, vec![e1, e2]); // 保持原始顺序
    }

    #[test]
    fn 行动队列_空队列() {
        let queue = TurnOrder::build(&[]);
        assert!(queue.is_empty());
    }

    #[test]
    fn 行动队列_current_unit和advance() {
        let e1 = Entity::from_bits(1);
        let e2 = Entity::from_bits(2);
        let e3 = Entity::from_bits(3);
        let mut order = TurnOrder {
            queue: vec![e1, e2, e3],
            current_index: 0,
            turn_number: 1,
        };
        assert_eq!(order.current_unit(), Some(e1));
        assert_eq!(order.advance(), Some(e2));
        assert_eq!(order.current_unit(), Some(e2));
        assert_eq!(order.advance(), Some(e3));
        assert_eq!(order.advance(), None); // 队列耗尽
    }

    fn setup_turn_test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
            .init_state::<TurnPhase>()
            .init_resource::<TurnState>()
            .init_resource::<TurnOrder>()
            .init_resource::<AiTimer>()
            .init_resource::<NeedsResolve>()
            .init_resource::<ForceEndFaction>()
            .add_systems(OnEnter(TurnPhase::TurnEnd), turn_end_on_enter);
        app
    }

    #[test]
    fn 回合结束_重建队列并增加回合数() {
        let mut app = setup_turn_test_app();

        // 需要注册 Attributes 组件
        app.world_mut().spawn((
            Unit {
                faction: Faction::Player,
                acted: false,
            },
            Attributes::default(),
        ));

        app.world_mut()
            .resource_mut::<NextState<TurnPhase>>()
            .set(TurnPhase::TurnEnd);
        app.update();

        let turn_state = app.world().resource::<TurnState>();
        assert_eq!(turn_state.turn_number, 2);

        let turn_order = app.world().resource::<TurnOrder>();
        assert!(!turn_order.queue.is_empty());
        assert_eq!(turn_order.current_index, 0);
    }

    #[test]
    fn 回合结束_needs_resolve标记设置() {
        let mut app = setup_turn_test_app();

        app.world_mut().spawn((
            Unit {
                faction: Faction::Player,
                acted: true,
            },
            Attributes::default(),
        ));

        app.world_mut()
            .resource_mut::<NextState<TurnPhase>>()
            .set(TurnPhase::TurnEnd);
        app.update();

        let needs_resolve = app.world().resource::<NeedsResolve>();
        assert!(needs_resolve.0);
    }

    #[test]
    fn 回合结束_进入后总是切换到_select_unit() {
        let mut app = setup_turn_test_app();

        app.world_mut().spawn((
            Unit {
                faction: Faction::Player,
                acted: true,
            },
            Attributes::default(),
        ));

        app.world_mut()
            .resource_mut::<NextState<TurnPhase>>()
            .set(TurnPhase::TurnEnd);
        app.update();

        let phase = app.world().resource::<State<TurnPhase>>();
        assert_eq!(*phase.get(), TurnPhase::SelectUnit);
    }
}
