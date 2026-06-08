use crate::battle::CombatIntent;
use crate::battle::manhattan_distance;
use crate::buff::ActiveBuffs;
use crate::character::{AiBehaviorId, Faction, GridPosition, Unit, UnitName};
use crate::gameplay::attribute::{AttributeKind, Attributes};
use crate::gameplay::tag::GameplayTags;
use crate::map::{GameMap, TerrainCostRegistry, TerrainMapCache, find_reachable_tiles};
use crate::skill::{SkillCooldowns, SkillRegistry, SkillSlots, effective_skill_range};
use crate::turn::{AiTimer, TurnPhase, TurnState};
use bevy::prelude::*;

use super::behavior::AiBehaviorRegistry;
use super::movement::select_move_coord;
use super::skill_select::select_skill;
use super::strategy::AiStrategyRegistry;
use super::targeting::{UnitSnapshot, select_target_coord};

/// 敌方 AI 系统：决策 → 移动 → 设置 CombatIntent → 切换到 ExecuteAction
///
/// 攻击效果不再在此系统内执行，而是通过统一的 Effect Pipeline 处理：
/// ExecuteAction → generate → modify → execute
pub fn enemy_ai_system(
    time: Res<Time>,
    mut ai_timer: ResMut<AiTimer>,
    turn_state: Res<TurnState>,
    turn_phase: Res<State<TurnPhase>>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    map: Res<GameMap>,
    skill_registry: Res<SkillRegistry>,
    ai_behavior_registry: Res<AiBehaviorRegistry>,
    ai_strategy_registry: Res<AiStrategyRegistry>,
    cost_registry: Res<TerrainCostRegistry>,
    terrain_cache: Res<TerrainMapCache>,
    mut combat_intent: ResMut<CombatIntent>,
    mut units: Query<(
        Entity,
        &mut Unit,
        &mut GridPosition,
        &mut Transform,
        &UnitName,
        &Attributes,
        &SkillSlots,
        &mut SkillCooldowns,
        &mut ActiveBuffs,
        &mut GameplayTags,
        &AiBehaviorId,
    )>,
) {
    if turn_state.current_faction != Faction::Enemy {
        return;
    }
    if *turn_phase.get() != TurnPhase::SelectUnit {
        return;
    }

    ai_timer.timer.tick(time.delta());
    if !ai_timer.timer.just_finished() {
        return;
    }

    // 收集所有单位快照
    let snapshots: Vec<UnitSnapshot> = units
        .iter()
        .map(
            |(e, u, gp, _, _name, attrs, skills, cooldowns, _, tags, ai_id)| UnitSnapshot {
                entity: e,
                faction: u.faction,
                coord: gp.coord,
                atk: attrs.get(AttributeKind::Atk),
                hp: attrs.get(AttributeKind::Hp),
                max_hp: attrs.get(AttributeKind::MaxHp),
                mov: attrs.get(AttributeKind::Mov) as u32,
                attack_range: attrs.get(AttributeKind::AttackRange) as u32,
                acted: u.acted,
                skill_ids: skills.skill_ids.clone(),
                cooldowns: cooldowns.clone(),
                ai_behavior_id: ai_id.0.clone(),
                tags: tags.clone(),
            },
        )
        .collect();

    let terrain_map = &terrain_cache.map;

    let player_positions: Vec<IVec2> = snapshots
        .iter()
        .filter(|s| s.faction == Faction::Player)
        .map(|s| s.coord)
        .collect();

    if player_positions.is_empty() {
        return;
    }

    // 找到第一个未行动的敌方单位
    let Some(snapshot) = snapshots
        .iter()
        .find(|s| s.faction == Faction::Enemy && !s.acted)
    else {
        // 所有敌方单位已行动，切换回合
        next_phase.set(TurnPhase::TurnEnd);
        return;
    };

    // 获取 AI 行为配置
    let behavior = ai_behavior_registry
        .get(&snapshot.ai_behavior_id)
        .unwrap_or_else(|| ai_behavior_registry.default_behavior());

    // 根据目标策略选择目标
    let target_coord = select_target_coord(
        &snapshots,
        snapshot.coord,
        ai_strategy_registry.target_selector(&behavior.target_strategy),
    );

    let occupation_map: std::collections::HashMap<IVec2, bool> = snapshots
        .iter()
        .filter(|s| s.faction == Faction::Player)
        .map(|s| (s.coord, true))
        .collect();

    // 根据单位标签解析地形成本计算器
    let calculator = cost_registry.resolve_from_tags(&snapshot.tags);
    let reachable = find_reachable_tiles(
        snapshot.coord,
        snapshot.mov,
        &map,
        &terrain_map,
        &occupation_map,
        calculator,
    );

    // 根据移动策略选择移动位置
    let best_coord = select_move_coord(
        &reachable,
        snapshot.coord,
        target_coord,
        snapshot.attack_range,
        ai_strategy_registry.move_selector(&behavior.move_strategy),
    );

    // 根据技能策略选择技能
    let skill_id = select_skill(
        &snapshot.skill_ids,
        &snapshot.cooldowns,
        ai_strategy_registry.skill_selector(&behavior.skill_strategy),
        &behavior.skill_priority,
    );

    let effective_range = skill_registry
        .get(skill_id)
        .map(|sd| effective_skill_range(sd, snapshot.attack_range))
        .unwrap_or(snapshot.attack_range);

    let attack_target = snapshots
        .iter()
        .filter(|s| s.faction == Faction::Player)
        .find(|s| manhattan_distance(best_coord, s.coord) <= effective_range)
        .map(|s| s.entity);

    // 移动单位
    let world_pos = map.coord_to_world(best_coord);
    if let Ok((_, _, mut gp, mut transform, _, _, _, _, _, _, _)) = units.get_mut(snapshot.entity) {
        gp.coord = best_coord;
        transform.translation.x = world_pos.x;
        transform.translation.y = world_pos.y;
    }

    // 设置冷却
    if let Some(skill_data) = skill_registry.get(skill_id) {
        if skill_data.cooldown > 0 {
            if let Ok((_, _, _, _, _, _, _, mut cooldowns, _, _, _)) =
                units.get_mut(snapshot.entity)
            {
                cooldowns.set(skill_id, skill_data.cooldown);
            }
        }
    }

    // 标记已行动
    if let Ok((_, mut unit, _, _, _, _, _, _, _, _, _)) = units.get_mut(snapshot.entity) {
        unit.acted = true;
    }

    // 设置 CombatIntent，让 Effect Pipeline 处理攻击
    if let Some(target_entity) = attack_target {
        let target_coord = units
            .get(target_entity)
            .map(|(_, _, gp, _, _, _, _, _, _, _, _)| gp.coord)
            .unwrap_or_default();

        combat_intent.source_entity = Some(snapshot.entity);
        combat_intent.target_coord = Some(target_coord);
        combat_intent.skill_id = Some(skill_id.to_string());

        // 切换到 ExecuteAction，走统一的 Effect Pipeline
        next_phase.set(TurnPhase::ExecuteAction);
    } else {
        // 没有攻击目标，检查是否所有敌方已行动
        let all_enemy_acted = units
            .iter()
            .filter(|(_, u, _, _, _, _, _, _, _, _, _)| u.faction == Faction::Enemy)
            .all(|(_, u, _, _, _, _, _, _, _, _, _)| u.acted);

        if all_enemy_acted {
            next_phase.set(TurnPhase::TurnEnd);
        } else {
            next_phase.set(TurnPhase::SelectUnit);
        }
    }
}
