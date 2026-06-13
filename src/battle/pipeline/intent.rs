// 战斗意图模块：行动后路由、行动执行

use crate::character::{
    AttackRange, Faction, GridPosition, MovableRange, Selected, SelectionHighlight, Unit, UnitName,
};
use crate::core::attribute::AttributeKind;
use crate::core::attribute::Attributes;
use crate::core::tag::{GameplayTag, GameplayTags};
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

/// 判断单位是否存活（HP > 0）
fn is_alive(attrs: &Attributes) -> bool {
    attrs.get(AttributeKind::Hp) > 0.0
}

/// 清除 CombatIntent（不变量6：Execute 后必须清除）
fn clear_combat_intent(intent: &mut CombatIntent) {
    intent.source_entity = None;
    intent.target_coord = None;
    intent.skill_id = None;
}

/// 行动完成后统一路由
/// 新逻辑：从 TurnOrder 队列前进到下一个存活的单位
/// 注意：通过检查 HP 判断存活，不依赖 Dead 组件（Dead 是 deferred command，
/// 在 OnEnter 阶段尚未应用）
/// B0001 fix: 接受 &non_selected_units 完整 query 类型以避免查询冲突
fn route_after_action(
    turn_order: &mut TurnOrder,
    non_selected_units: &Query<
        (&mut Unit, &Attributes, &mut SkillCooldowns, &GameplayTags),
        Without<Selected>,
    >,
    next_phase: &mut ResMut<NextState<TurnPhase>>,
    ai_timer: &mut AiTimer,
    turn_state: &mut TurnState,
) {
    // 跳过已死亡的单位，直到找到存活的单位或队列耗尽
    loop {
        match turn_order.advance() {
            Some(next_entity) => {
                // 检查单位是否存活（通过 HP 判断，不依赖 Dead 组件）
                if let Ok((unit, attrs, _, _)) = non_selected_units.get(next_entity) {
                    if !is_alive(attrs) {
                        // 单位已死亡（HP=0），继续前进到下一个
                        continue;
                    }
                    // 更新当前阵营
                    turn_state.current_faction = unit.faction;
                    // 如果下一个是 AI，重置计时器
                    if unit.faction == Faction::Enemy {
                        ai_timer.timer.reset();
                    }
                    next_phase.set(TurnPhase::SelectUnit);
                    return;
                }
                // 单位不存在，继续前进到下一个
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
/// AI 通过 CombatIntent.source_entity 标识，与玩家共享 Effect Pipeline（不变量1合规）
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
    mut non_selected_units: Query<
        (&mut Unit, &Attributes, &mut SkillCooldowns, &GameplayTags),
        Without<Selected>,
    >,
    mut turn_order: ResMut<TurnOrder>,
    mut turn_state: ResMut<TurnState>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    mut commands: Commands,
    mut combat_intent: ResMut<CombatIntent>,
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
            // 不变量6：Execute 后清除 CombatIntent
            clear_combat_intent(&mut combat_intent);
            // 晕眩也走统一路由
            route_after_action(
                &mut turn_order,
                &non_selected_units,
                &mut next_phase,
                &mut ai_timer,
                &mut turn_state,
            );
            return;
        }

        if let Some(skill_id) = combat_intent.skill_id.as_deref() {
            if let Some(skill_data) = skill_registry.get(skill_id) {
                bevy::log::info!(
                    target: "battle",
                    event = "skill_activated",
                    unit = %_name.0,
                    skill_id = %skill_id,
                    "技能已使用"
                );
                if skill_data.cooldown > 0 {
                    cooldowns.set(skill_id, skill_data.cooldown);
                }
            }
        }

        unit.acted = true;
        commands.entity(entity).remove::<Selected>();

        // 不变量6：Execute 后清除 CombatIntent
        clear_combat_intent(&mut combat_intent);

        // 统一路由：从队列前进到下一个单位
        route_after_action(
            &mut turn_order,
            &non_selected_units,
            &mut next_phase,
            &mut ai_timer,
            &mut turn_state,
        );
        return;
    }

    // AI 单位：通过 CombatIntent.source_entity 查找，与玩家共享 Effect Pipeline
    if let Some(source_entity) = combat_intent.source_entity {
        if let Ok((mut unit, _attrs, mut cooldowns, tags)) =
            non_selected_units.get_mut(source_entity)
        {
            // 晕眩检查
            if tags.has(GameplayTag::STUN) {
                unit.acted = true;
                // 不变量6：Execute 后清除 CombatIntent
                clear_combat_intent(&mut combat_intent);
                route_after_action(
                    &mut turn_order,
                    &non_selected_units,
                    &mut next_phase,
                    &mut ai_timer,
                    &mut turn_state,
                );
                return;
            }

            // 设置冷却（与玩家走同一套逻辑）
            if let Some(skill_id) = combat_intent.skill_id.as_deref() {
                if let Some(skill_data) = skill_registry.get(skill_id) {
                    bevy::log::info!(
                        target: "battle",
                        event = "skill_activated",
                        unit = ?source_entity,
                        skill_id = %skill_id,
                        "AI技能已使用"
                    );
                    if skill_data.cooldown > 0 {
                        cooldowns.set(skill_id, skill_data.cooldown);
                    }
                }
            }

            unit.acted = true;
        }
    }

    // 不变量6：Execute 后清除 CombatIntent
    clear_combat_intent(&mut combat_intent);

    // 统一路由
    route_after_action(
        &mut turn_order,
        &non_selected_units,
        &mut next_phase,
        &mut ai_timer,
        &mut turn_state,
    );
}

/// 待机（OnEnter WaitAction）
/// AI 和玩家共用，AI 通过 CombatIntent.source_entity 标识
pub fn wait_action_on_enter(
    mut selected_units: Query<(Entity, &mut Unit), With<Selected>>,
    mut non_selected_units: Query<
        (&mut Unit, &Attributes, &mut SkillCooldowns, &GameplayTags),
        Without<Selected>,
    >,
    mut turn_order: ResMut<TurnOrder>,
    mut turn_state: ResMut<TurnState>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    mut commands: Commands,
    mut combat_intent: ResMut<CombatIntent>,
    range_entities: Query<
        (Entity, Option<&GridPosition>),
        Or<(With<MovableRange>, With<AttackRange>)>,
    >,
    highlights: Query<Entity, With<SelectionHighlight>>,
    mut ai_timer: ResMut<AiTimer>,
) {
    crate::character::clear_markers(&mut commands, &range_entities, &highlights);

    // 玩家单位
    if let Ok((entity, mut unit)) = selected_units.single_mut() {
        unit.acted = true;
        commands.entity(entity).remove::<Selected>();
    }

    // AI 单位：通过 CombatIntent.source_entity 标识
    if let Some(source_entity) = combat_intent.source_entity {
        if let Ok((mut unit, _attrs, _cooldowns, _tags)) = non_selected_units.get_mut(source_entity)
        {
            unit.acted = true;
        }
    }

    // 不变量6：Execute 后清除 CombatIntent
    clear_combat_intent(&mut combat_intent);

    // 统一路由：从队列前进到下一个单位
    route_after_action(
        &mut turn_order,
        &non_selected_units,
        &mut next_phase,
        &mut ai_timer,
        &mut turn_state,
    );
}
