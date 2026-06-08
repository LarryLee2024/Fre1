// 战斗意图资源 + OnEnter 系统

use crate::character::{AttackRange, GridPosition, MovableRange, Selected, SelectionHighlight, Unit, UnitName};
use crate::gameplay::tag::{GameplayTag, GameplayTags};
use crate::skill::{SkillCooldowns, SkillRegistry};
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

/// 执行攻击（OnEnter ExecuteAction）
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
    mut next_phase: ResMut<NextState<crate::turn::TurnPhase>>,
    mut commands: Commands,
    combat_intent: Res<CombatIntent>,
    range_entities: Query<
        (Entity, Option<&GridPosition>),
        Or<(With<MovableRange>, With<AttackRange>)>,
    >,
    highlights: Query<Entity, With<SelectionHighlight>>,
    skill_registry: Res<SkillRegistry>,
) {
    crate::character::clear_markers(&mut commands, &range_entities, &highlights);

    // 玩家单位：通过 Selected 查找
    if let Ok((entity, mut unit, _pos, _name, tags, mut cooldowns)) = selected_units.single_mut() {
        if tags.has(GameplayTag::STUN) {
            unit.acted = true;
            commands.entity(entity).remove::<Selected>();
            next_phase.set(crate::turn::TurnPhase::TurnEnd);
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
    }
    // AI 单位：acted 和冷却已在 decision.rs 中处理，无需额外操作

    next_phase.set(crate::turn::TurnPhase::TurnEnd);
}

/// 待机（OnEnter WaitAction）
pub fn wait_action_on_enter(
    mut selected_units: Query<(Entity, &mut Unit), With<Selected>>,
    mut next_phase: ResMut<NextState<crate::turn::TurnPhase>>,
    mut commands: Commands,
    range_entities: Query<
        (Entity, Option<&GridPosition>),
        Or<(With<MovableRange>, With<AttackRange>)>,
    >,
    highlights: Query<Entity, With<SelectionHighlight>>,
) {
    crate::character::clear_markers(&mut commands, &range_entities, &highlights);

    if let Ok((entity, mut unit)) = selected_units.single_mut() {
        unit.acted = true;
        commands.entity(entity).remove::<Selected>();
    }

    next_phase.set(crate::turn::TurnPhase::TurnEnd);
}
