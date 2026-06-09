// 战斗意图资源 + OnEnter 系统

use crate::character::{
    AttackRange, Faction, GridPosition, MovableRange, Selected, SelectionHighlight, Unit, UnitName,
};
use crate::gameplay::tag::{GameplayTag, GameplayTags};
use crate::skill::{SkillCooldowns, SkillRegistry};
use crate::turn::{AiTimer, TurnPhase, TurnState};
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

/// 统一的回合路由：行动完一个棋子后，检查是否切换阵营
///
/// 规则（玩家和 AI 一致）：
/// - 当前阵营所有单位已行动 → TurnEnd（阵营切换）
/// - 当前阵营还有未行动单位 → SelectUnit（继续行动）
/// - AI 额外：重置计时器
fn route_after_action(
    turn_state: &TurnState,
    all_units: &Query<&Unit, Without<Selected>>,
    ai_timer: &mut AiTimer,
    next_phase: &mut ResMut<NextState<TurnPhase>>,
) {
    let all_acted = all_units
        .iter()
        .filter(|u| u.faction == turn_state.current_faction)
        .all(|u| u.acted);

    if all_acted {
        next_phase.set(TurnPhase::TurnEnd);
    } else {
        // AI 需要重置计时器让下一个单位行动
        if turn_state.current_faction == Faction::Enemy {
            ai_timer.timer.reset();
        }
        next_phase.set(TurnPhase::SelectUnit);
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
    turn_state: Res<TurnState>,
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
            route_after_action(&turn_state, &all_units, &mut ai_timer, &mut next_phase);
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

        // 统一路由：检查是否所有玩家单位都已行动
        route_after_action(&turn_state, &all_units, &mut ai_timer, &mut next_phase);
        return;
    }

    // AI 单位：同样走统一路由
    route_after_action(&turn_state, &all_units, &mut ai_timer, &mut next_phase);
}

/// 待机（OnEnter WaitAction）
pub fn wait_action_on_enter(
    mut selected_units: Query<(Entity, &mut Unit), With<Selected>>,
    all_units: Query<&Unit, Without<Selected>>,
    turn_state: Res<TurnState>,
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

    // 统一路由
    route_after_action(&turn_state, &all_units, &mut ai_timer, &mut next_phase);
}
