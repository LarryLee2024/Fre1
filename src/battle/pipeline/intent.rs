// 战斗意图模块：行动后路由、行动执行

use crate::character::{
    AttackRange, Faction, GridPosition, MovableRange, Selected, SelectionHighlight, Unit, UnitName,
};
use crate::gameplay::tag::{GameplayTag, GameplayTags};
use crate::skill::{SkillCooldowns, SkillRegistry};
use crate::turn::{AiTimer, TurnOrder, TurnPhase, TurnState};
use bevy::prelude::*;

/// 战斗意图：记录谁攻击谁、用什么技能
#[derive(Resource, Default)]
pub struct CombatIntent {
    /// 攻击者实体（玩家通过 Selected 查找，AI 直接设置）
    pub source_entity: Option<Entity>,
    /// 目标坐标
    pub target_coord: Option<IVec2>,
    /// 选择的技能 ID
    pub skill_id: Option<String>,
}

/// 移动前位置（用于取消时回退）
#[derive(Resource, Default)]
pub struct PrevPosition {
    pub coord: Option<IVec2>,
}

/// 行动完成后统一路由
/// 新逻辑：从 TurnOrder 队列前进到下一个存活的单位
fn route_after_action(
    turn_order: &mut TurnOrder,
    units: &Query<&Unit, Without<Selected>>,
    next_phase: &mut ResMut<NextState<TurnPhase>>,
    ai_timer: &mut AiTimer,
    turn_state: &mut TurnState,
) {
    // 跳过已死亡的单位，直到找到存活的单位或队列耗尽
    loop {
        match turn_order.advance() {
            Some(next_entity) => {
                // 检查单位是否存活
                if let Ok(unit) = units.get(next_entity) {
                    // 更新当前阵营
                    turn_state.current_faction = unit.faction;
                    // 如果下一个是 AI，重置计时器
                    if unit.faction == Faction::Enemy {
                        ai_timer.timer.reset();
                    }
                    next_phase.set(TurnPhase::SelectUnit);
                    return;
                }
                // 单位已死亡，继续前进到下一个
            }
            None => {
                // 队列耗尽，回合结束
                next_phase.set(TurnPhase::TurnEnd);
                return;
            }
        }
    }
}

/// 执行攻击（OnEnter ExecuteAction）
///
/// 统一路由：玩家和 AI 行动完后都走同一套规则
pub fn execute_action_on_enter(
    mut selected_units: Query<
        (
            Entity,
            &mut Unit,
            &GridPosition,
            &UnitName,
            &GameplayTags,
            &mut SkillCooldowns,
        ),
        With<Selected>,
    >,
    all_units: Query<&Unit, Without<Selected>>,
    mut turn_order: ResMut<TurnOrder>,
    mut turn_state: ResMut<TurnState>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    mut commands: Commands,
    combat_intent: Res<CombatIntent>,
    range_entities: Query<
        (Entity, Option<&GridPosition>),
        Or<(With<MovableRange>, With<AttackRange>)>,
    >,
    highlights: Query<Entity, With<SelectionHighlight>>,
    skill_registry: Res<SkillRegistry>,
    mut ai_timer: ResMut<AiTimer>,
) {
    crate::character::clear_markers(&mut commands, &range_entities, &highlights);

    // 玩家单位：通过 Selected 查找
    if let Ok((entity, mut unit, _pos, _name, tags, mut cooldowns)) = selected_units.single_mut() {
        if tags.has(GameplayTag::STUN) {
            unit.acted = true;
            commands.entity(entity).remove::<Selected>();
            // 晕眩也走统一路由
            route_after_action(
                &mut turn_order,
                &all_units,
                &mut next_phase,
                &mut ai_timer,
                &mut turn_state,
            );
            return;
        }

        if let Some(skill_id) = combat_intent.skill_id.as_deref() {
            if let Some(skill_data) = skill_registry.get(skill_id) {
                if skill_data.cooldown > 0 {
                    cooldowns.set(skill_id, skill_data.cooldown);
                }
            }
        }

        unit.acted = true;
        commands.entity(entity).remove::<Selected>();

        // 统一路由：从队列前进到下一个单位
        route_after_action(
            &mut turn_order,
            &all_units,
            &mut next_phase,
            &mut ai_timer,
            &mut turn_state,
        );
        return;
    }

    // AI 单位：同样走统一路由
    route_after_action(
        &mut turn_order,
        &all_units,
        &mut next_phase,
        &mut ai_timer,
        &mut turn_state,
    );
}

/// 待机（OnEnter WaitAction）
pub fn wait_action_on_enter(
    mut selected_units: Query<(Entity, &mut Unit), With<Selected>>,
    all_units: Query<&Unit, Without<Selected>>,
    mut turn_order: ResMut<TurnOrder>,
    mut turn_state: ResMut<TurnState>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    mut commands: Commands,
    range_entities: Query<
        (Entity, Option<&GridPosition>),
        Or<(With<MovableRange>, With<AttackRange>)>,
    >,
    highlights: Query<Entity, With<SelectionHighlight>>,
    mut ai_timer: ResMut<AiTimer>,
) {
    crate::character::clear_markers(&mut commands, &range_entities, &highlights);

    if let Ok((entity, mut unit)) = selected_units.single_mut() {
        unit.acted = true;
        commands.entity(entity).remove::<Selected>();
    }

    // 统一路由：从队列前进到下一个单位
    route_after_action(
        &mut turn_order,
        &all_units,
        &mut next_phase,
        &mut ai_timer,
        &mut turn_state,
    );
}
