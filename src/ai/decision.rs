use crate::battle::CombatIntent;
use crate::battle::manhattan_distance;
use crate::buff::ActiveBuffs;
use crate::character::{
    AiBehaviorId, Faction, GridPosition, MovingUnit, Unit, UnitName, spawn_path_arrows,
};
use crate::core::attribute::{AttributeKind, Attributes};
use crate::core::tag::GameplayTags;
use crate::map::TerrainRegistry;
use crate::map::runtime::{OccupancyGrid, TerrainGrid};
use crate::map::{GameMap, TerrainCostRegistry, find_reachable_tiles, reconstruct_path};
use crate::skill::{SkillCooldowns, SkillRegistry, SkillSlots, effective_skill_range};
use crate::turn::{AiTimer, TurnOrder, TurnPhase};
use bevy::prelude::*;

use super::behavior::AiBehaviorRegistry;
use super::movement::select_move_coord;
use super::skill_select::select_skill;
use super::strategy::AiStrategyRegistry;
use super::targeting::{UnitSnapshot, select_target_coord};

/// 敌方 AI 系统：基于 TurnOrder 队列驱动
///
/// 当 TurnOrder 当前单位是敌方时，AI 计时器到期后自动决策
/// 攻击效果通过统一的 Effect Pipeline 处理
pub fn enemy_ai_system(
    mut commands: Commands,
    time: Res<Time>,
    mut ai_timer: ResMut<AiTimer>,
    turn_order: Res<TurnOrder>,
    turn_phase: Res<State<TurnPhase>>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    map: Res<GameMap>,
    skill_registry: Res<SkillRegistry>,
    ai_behavior_registry: Res<AiBehaviorRegistry>,
    ai_strategy_registry: Res<AiStrategyRegistry>,
    cost_registry: Res<TerrainCostRegistry>,
    terrain_grid: Res<TerrainGrid>,
    terrain_registry: Res<TerrainRegistry>,
    occupancy: Res<OccupancyGrid>,
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
    // 只在 SelectUnit 阶段执行
    if *turn_phase.get() != TurnPhase::SelectUnit {
        return;
    }

    // 从 TurnOrder 获取当前应该行动的单位
    let current_entity = match turn_order.current_unit() {
        Some(e) => e,
        None => return,
    };

    // 检查当前单位是否是敌方
    let current_faction = units
        .get(current_entity)
        .map(|(_, u, _, _, _, _, _, _, _, _, _)| u.faction)
        .ok();
    if current_faction != Some(Faction::Enemy) {
        return;
    }

    // AI 计时器
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
                atk: attrs.get(AttributeKind::Attack),
                hp: attrs.get(AttributeKind::Hp),
                max_hp: attrs.get(AttributeKind::MaxHp),
                mov: attrs.get(AttributeKind::MoveRange) as u32,
                attack_range: attrs.get(AttributeKind::AttackRange) as u32,
                acted: u.acted,
                skill_ids: skills.skill_ids.clone(),
                cooldowns: cooldowns.clone(),
                ai_behavior_id: ai_id.0.clone(),
                tags: tags.clone(),
            },
        )
        .collect();

    let player_positions: Vec<IVec2> = snapshots
        .iter()
        .filter(|s| s.faction == Faction::Player)
        .map(|s| s.coord)
        .collect();

    if player_positions.is_empty() {
        return;
    }

    // 获取当前行动的敌方单位快照
    let Some(snapshot) = snapshots.iter().find(|s| s.entity == current_entity) else {
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

    // 根据单位标签解析地形成本计算器
    let calculator = cost_registry.resolve_from_tags(&snapshot.tags);
    let reachable = find_reachable_tiles(
        snapshot.coord,
        snapshot.mov,
        &map,
        &terrain_grid,
        &terrain_registry,
        &occupancy,
        Some(snapshot.entity),
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

    // 移动单位（动画移动）
    let path = if best_coord != snapshot.coord {
        let path = reconstruct_path(
            snapshot.coord,
            best_coord,
            &reachable,
            snapshot.mov,
            &map,
            &terrain_grid,
            &terrain_registry,
            calculator,
        );
        spawn_path_arrows(&mut commands, &map, &path);
        path
    } else {
        vec![]
    };

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

        if path.is_empty() {
            next_phase.set(TurnPhase::ExecuteAction);
        } else {
            commands.entity(snapshot.entity).insert(MovingUnit {
                path,
                current_index: 0,
                speed: 0.15,
                elapsed: 0.0,
                next_phase: TurnPhase::ExecuteAction,
            });
        }
    } else {
        // 没有攻击目标，待机
        if path.is_empty() {
            // 不移动，路由到下一个单位
            // 由 route_after_action 处理
            next_phase.set(TurnPhase::WaitAction);
        } else {
            commands.entity(snapshot.entity).insert(MovingUnit {
                path,
                current_index: 0,
                speed: 0.15,
                elapsed: 0.0,
                next_phase: TurnPhase::WaitAction,
            });
        }
    }
}
