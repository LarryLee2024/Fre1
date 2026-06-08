// AI 模块：敌方自动行动，通过 EffectQueue 执行效果

use crate::assets::CnFont;
use crate::combat::manhattan_distance;
use crate::combat_log::{CombatLog, LogSegment, log_color};
use crate::core::attribute::{AttributeKind, Attributes};
use crate::core::effect::{calculate_damage_from_effect, EffectDef, EffectQueue, PendingEffect, PendingEffectData};
use crate::core::modifier_rule::ModifierRuleRegistry;
use crate::core::tag::GameplayTags;
use crate::data::ai_behavior::{AiBehaviorRegistry, MoveStrategy, SkillStrategy, TargetStrategy};
use crate::data::buff_data::{apply_buff, ActiveBuffs, BuffRegistry};
use crate::data::skill_data::{effective_skill_range, SkillCooldowns, SkillRegistry, SkillSlots};
use crate::map::{GameMap, Terrain, Tile};
use crate::pathfinding::{build_tile_terrain_map, find_reachable_tiles};
use crate::turn::{AiTimer, TurnPhase, TurnState};
use crate::unit::{AiBehaviorId, Faction, GridPosition, Unit, UnitName};
use crate::vfx;
use bevy::prelude::*;

/// 单位快照（避免借用冲突）
struct UnitSnapshot {
    entity: Entity,
    faction: Faction,
    coord: IVec2,
    atk: f32,
    hp: f32,
    max_hp: f32,
    mov: u32,
    attack_range: u32,
    acted: bool,
    skill_ids: Vec<String>,
    cooldowns: SkillCooldowns,
    ai_behavior_id: String,
}

/// 敌方 AI 系统：决策 → 移动 → 推入 EffectQueue → 修饰 → 执行
pub fn enemy_ai_system(
    mut commands: Commands,
    time: Res<Time>,
    mut ai_timer: ResMut<AiTimer>,
    turn_state: Res<TurnState>,
    turn_phase: Res<State<TurnPhase>>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    cn_font: Res<CnFont>,
    map: Res<GameMap>,
    skill_registry: Res<SkillRegistry>,
    buff_registry: Res<BuffRegistry>,
    modifier_rules: Res<ModifierRuleRegistry>,
    ai_behavior_registry: Res<AiBehaviorRegistry>,
    mut effect_queue: ResMut<EffectQueue>,
    mut combat_log: ResMut<CombatLog>,
    mut units: Query<(
        Entity,
        &mut Unit,
        &mut GridPosition,
        &mut Transform,
        &UnitName,
        &mut Attributes,
        &SkillSlots,
        &mut SkillCooldowns,
        &mut ActiveBuffs,
        &mut GameplayTags,
        &AiBehaviorId,
    )>,
    tiles: Query<&Tile>,
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
        .map(|(e, u, gp, _, _name, attrs, skills, cooldowns, _, _, ai_id)| UnitSnapshot {
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
        })
        .collect();

    let terrain_map = build_tile_terrain_map(&tiles);

    let player_positions: Vec<IVec2> = snapshots
        .iter()
        .filter(|s| s.faction == Faction::Player)
        .map(|s| s.coord)
        .collect();

    if player_positions.is_empty() {
        return;
    }

    // 记录 AI 行动
    struct AiAction {
        entity: Entity,
        move_to: IVec2,
        attack_target: Option<Entity>,
        skill_id: String,
        atk: f32,
    }

    let mut actions: Vec<AiAction> = Vec::new();

    for snapshot in snapshots
        .iter()
        .filter(|s| s.faction == Faction::Enemy && !s.acted)
    {
        // 获取 AI 行为配置
        let behavior = ai_behavior_registry
            .get(&snapshot.ai_behavior_id)
            .unwrap_or_else(|| ai_behavior_registry.default_behavior());

        // 根据目标策略选择目标
        let target_coord = select_target_coord(&snapshots, snapshot.coord, behavior.target_strategy);

        let occupation_map: std::collections::HashMap<IVec2, bool> = snapshots
            .iter()
            .filter(|s| s.faction == Faction::Player)
            .map(|s| (s.coord, true))
            .collect();

        let reachable =
            find_reachable_tiles(snapshot.coord, snapshot.mov, &map, &terrain_map, &occupation_map);

        // 根据移动策略选择移动位置
        let best_coord = select_move_coord(
            &reachable,
            snapshot.coord,
            target_coord,
            snapshot.attack_range,
            behavior.move_strategy,
        );

        // 根据技能策略选择技能
        let skill_id = select_skill(
            &snapshot.skill_ids,
            &snapshot.cooldowns,
            behavior.skill_strategy,
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

        actions.push(AiAction {
            entity: snapshot.entity,
            move_to: best_coord,
            attack_target,
            skill_id: skill_id.to_string(),
            atk: snapshot.atk,
        });
    }

    // 应用行动
    for action in actions {
        // 移动
        let world_pos = map.coord_to_world(action.move_to);
        if let Ok((_, _, mut gp, mut transform, _, _, _, _, _, _, _)) = units.get_mut(action.entity) {
            gp.coord = action.move_to;
            transform.translation.x = world_pos.x;
            transform.translation.y = world_pos.y;
        }

        // 通过 EffectQueue 执行攻击效果
        if let Some(target_entity) = action.attack_target {
            if let Some(skill_data) = skill_registry.get(&action.skill_id) {
                // 设置冷却
                if skill_data.cooldown > 0 {
                    if let Ok((_, _, _, _, _, _, _, mut cooldowns, _, _, _)) = units.get_mut(action.entity) {
                        cooldowns.set(&action.skill_id, skill_data.cooldown);
                    }
                }

                // 获取目标信息并推入 EffectQueue
                let target_info = units.get(target_entity).ok().map(|(_, _, gp, _, _, attrs, _, _, _, _, _)| {
                    let terrain = tiles
                        .iter()
                        .find(|t| t.coord == gp.coord)
                        .map(|t| t.terrain)
                        .unwrap_or(Terrain::Plain);
                    (attrs.clone(), gp.coord, terrain)
                });

                if let Some((target_attrs, _target_gp, terrain)) = target_info {
                    for effect_def in &skill_data.effects {
                        match effect_def {
                            EffectDef::Damage { multiplier, ignore_def_percent } => {
                                let effective_def = target_attrs.get(AttributeKind::Def);
                                let base_def = target_attrs.base.get(&AttributeKind::Def).copied().unwrap_or(0.0);

                                let amount = calculate_damage_from_effect(
                                    action.atk,
                                    effective_def,
                                    base_def,
                                    *multiplier,
                                    *ignore_def_percent,
                                    terrain,
                                );

                                effect_queue.push(PendingEffect {
                                    source: action.entity,
                                    target: target_entity,
                                    data: PendingEffectData::Damage {
                                        amount,
                                        is_skill: action.skill_id != "basic_attack",
                                    },
                                    source_tags: skill_data.tags.clone(),
                                    terrain,
                                });
                            }
                            EffectDef::ApplyBuff { buff_id, duration } => {
                                effect_queue.push(PendingEffect {
                                    source: action.entity,
                                    target: target_entity,
                                    data: PendingEffectData::ApplyBuff {
                                        buff_id: buff_id.clone(),
                                        duration: *duration,
                                    },
                                    source_tags: skill_data.tags.clone(),
                                    terrain,
                                });
                            }
                            EffectDef::Heal { .. } | EffectDef::Cleanse => {
                                // AI 不会治疗/净化玩家
                            }
                        }
                    }
                }
            }
        }

        // 标记已行动
        if let Ok((_, mut unit, _, _, _, _, _, _, _, _, _)) = units.get_mut(action.entity) {
            unit.acted = true;
        }
    }

    // 修饰 + 执行 EffectQueue 中的所有效果
    // AI 不经过 TurnPhase::ExecuteAction，所以在这里直接执行
    if !effect_queue.pending.is_empty() {
        // 步骤 2：修饰效果（使用 ModifierRuleRegistry）
        for effect in &mut effect_queue.pending {
            if let PendingEffectData::Damage { ref mut amount, .. } = effect.data {
                if let Ok((_, _, _, _, _, _, _, _, _, tags, _)) = units.get(effect.target) {
                    *amount = modifier_rules.apply_damage_modifiers(*amount, &effect.source_tags, tags);
                }
            }
        }

        // 步骤 3：执行效果（扣血/加 Buff/特效/日志/击杀）
        execute_ai_effects(
            &mut commands,
            &mut effect_queue,
            &mut units,
            &mut combat_log,
            &buff_registry,
            &map,
            &cn_font,
        );
    }

    // 检查是否所有敌方单位都已行动
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

/// AI 执行效果队列（内联版本，使用 AI 的合并 Query）
fn execute_ai_effects(
    commands: &mut Commands,
    queue: &mut EffectQueue,
    units: &mut Query<(
        Entity,
        &mut Unit,
        &mut GridPosition,
        &mut Transform,
        &UnitName,
        &mut Attributes,
        &SkillSlots,
        &mut SkillCooldowns,
        &mut ActiveBuffs,
        &mut GameplayTags,
        &AiBehaviorId,
    )>,
    combat_log: &mut CombatLog,
    buff_registry: &BuffRegistry,
    map: &GameMap,
    cn_font: &CnFont,
) {
    for effect in queue.pending.drain(..) {
        match effect.data {
            PendingEffectData::Damage { amount, is_skill } => {
                // 扣血
                if let Ok((_, _, _, _, _, mut attrs, _, _, _, _, _)) = units.get_mut(effect.target) {
                    let hp = attrs.get(AttributeKind::Hp);
                    let new_hp = (hp - amount as f32).max(0.0);
                    attrs.set_base(AttributeKind::Hp, new_hp);
                }

                // 伤害数字弹出
                if let Ok((_, _, gp, _, _, _, _, _, _, _, _)) = units.get(effect.target) {
                    let world_pos = map.coord_to_world(gp.coord);
                    vfx::spawn_damage_popup(commands, world_pos, amount, &cn_font.handle, is_skill);
                }

                // 战斗日志
                let attacker_color = units
                    .get(effect.source)
                    .map(|(_, u, _, _, _, _, _, _, _, _, _)| {
                        if u.faction == Faction::Player { log_color::PLAYER } else { log_color::ENEMY }
                    })
                    .unwrap_or(log_color::NORMAL);
                let defender_color = units
                    .get(effect.target)
                    .map(|(_, u, _, _, _, _, _, _, _, _, _)| {
                        if u.faction == Faction::Player { log_color::PLAYER } else { log_color::ENEMY }
                    })
                    .unwrap_or(log_color::NORMAL);

                let attacker_name = units
                    .get(effect.source)
                    .map(|(_, _, _, _, name, _, _, _, _, _, _)| name.0.as_str())
                    .unwrap_or("???");
                let target_name = units
                    .get(effect.target)
                    .map(|(_, _, _, _, name, _, _, _, _, _, _)| name.0.as_str())
                    .unwrap_or("???");

                let skill_label = if is_skill { "技能" } else { "攻击" };

                combat_log.push(vec![
                    LogSegment { text: format!("[{}]", attacker_name), color: attacker_color },
                    LogSegment { text: format!(" 使用[{}]", skill_label), color: log_color::TURN },
                    LogSegment { text: " 攻击 ".to_string(), color: log_color::NORMAL },
                    LogSegment { text: format!("[{}]", target_name), color: defender_color },
                    LogSegment { text: " 造成 ".to_string(), color: log_color::NORMAL },
                    LogSegment { text: format!("[{}]", amount), color: log_color::DAMAGE },
                    LogSegment { text: " 伤害".to_string(), color: log_color::NORMAL },
                    LogSegment { text: format!(" ({})", effect.terrain.label()), color: log_color::TERRAIN },
                ]);

                // 击杀处理
                if let Ok((_, _, _, _, target_name, attrs, _, _, _, _, _)) = units.get(effect.target) {
                    if attrs.get(AttributeKind::Hp) <= 0.0 {
                        combat_log.push(vec![
                            LogSegment { text: format!("[{}]", target_name.0), color: defender_color },
                            LogSegment { text: " 被击败！".to_string(), color: log_color::KILL },
                        ]);
                        commands.entity(effect.target).try_despawn();
                    }
                }
            }
            PendingEffectData::Heal { amount } => {
                if let Ok((_, _, _, _, _, mut attrs, _, _, _, _, _)) = units.get_mut(effect.target) {
                    let hp = attrs.get(AttributeKind::Hp);
                    let max_hp = attrs.get(AttributeKind::MaxHp);
                    let new_hp = (hp + amount as f32).min(max_hp);
                    attrs.set_base(AttributeKind::Hp, new_hp);
                }

                let target_name = units
                    .get(effect.target)
                    .map(|(_, _, _, _, name, _, _, _, _, _, _)| name.0.as_str())
                    .unwrap_or("???");
                combat_log.push(vec![
                    LogSegment { text: format!("[{}]", target_name), color: log_color::NORMAL },
                    LogSegment { text: format!(" 恢复 {} HP", amount), color: log_color::HEAL },
                ]);
            }
            PendingEffectData::ApplyBuff { buff_id, duration } => {
                if let (Ok((_, _, _, _, _, mut attrs, _, _, mut buffs, mut tags, _)), Some(buff_data)) = (
                    units.get_mut(effect.target),
                    buff_registry.get(&buff_id),
                ) {
                    apply_buff(&mut buffs, &mut attrs, &mut tags, buff_data, Some(effect.source), duration);
                }
            }
            PendingEffectData::Cleanse => {
                if let Ok((_, _, _, _, _, mut attrs, _, _, mut buffs, mut tags, _)) = units.get_mut(effect.target) {
                    crate::data::buff_data::remove_all_debuffs(&mut buffs, &mut attrs, &mut tags);
                }
            }
        }
    }
}

/// 根据目标策略选择目标坐标
fn select_target_coord(
    snapshots: &[UnitSnapshot],
    my_coord: IVec2,
    strategy: TargetStrategy,
) -> IVec2 {
    let players: Vec<&UnitSnapshot> = snapshots
        .iter()
        .filter(|s| s.faction == Faction::Player)
        .collect();

    match strategy {
        TargetStrategy::Nearest => {
            players
                .iter()
                .min_by_key(|s| manhattan_distance(my_coord, s.coord))
                .map(|s| s.coord)
        }
        TargetStrategy::Weakest => {
            players
                .iter()
                .min_by_key(|s| s.hp as i32)
                .map(|s| s.coord)
        }
        TargetStrategy::MostDangerous => {
            players
                .iter()
                .max_by_key(|s| s.atk as i32)
                .map(|s| s.coord)
        }
        TargetStrategy::LowestHpPercent => {
            players
                .iter()
                .min_by_key(|s| {
                    if s.max_hp > 0.0 { (s.hp / s.max_hp * 100.0) as i32 } else { 0 }
                })
                .map(|s| s.coord)
        }
    }
    .unwrap_or(my_coord)
}

/// 根据移动策略选择移动坐标
fn select_move_coord(
    reachable: &std::collections::HashMap<IVec2, u32>,
    my_coord: IVec2,
    target_coord: IVec2,
    attack_range: u32,
    strategy: MoveStrategy,
) -> IVec2 {
    if reachable.is_empty() {
        return my_coord;
    }

    match strategy {
        MoveStrategy::Aggressive => {
            // 贪心靠近目标
            reachable
                .keys()
                .min_by_key(|coord| manhattan_distance(**coord, target_coord))
                .copied()
                .unwrap_or(my_coord)
        }
        MoveStrategy::Cautious => {
            // 保持攻击距离，不靠近超过攻击范围
            let at_range: Vec<_> = reachable
                .keys()
                .filter(|coord| manhattan_distance(**coord, target_coord) <= attack_range)
                .collect();

            if at_range.is_empty() {
                // 没有在攻击范围内的位置，靠近
                reachable
                    .keys()
                    .min_by_key(|coord| manhattan_distance(**coord, target_coord))
                    .copied()
                    .unwrap_or(my_coord)
            } else {
                // 选择最远的（保持距离）
                at_range
                    .iter()
                    .max_by_key(|coord| manhattan_distance(***coord, target_coord))
                    .map(|c| **c)
                    .unwrap_or(my_coord)
            }
        }
        MoveStrategy::Support => {
            // 优先靠近友军（暂用最近目标逻辑）
            reachable
                .keys()
                .min_by_key(|coord| manhattan_distance(**coord, target_coord))
                .copied()
                .unwrap_or(my_coord)
        }
    }
}

/// 根据技能策略选择技能
fn select_skill<'a>(
    skill_ids: &'a [String],
    cooldowns: &SkillCooldowns,
    strategy: SkillStrategy,
    priority: &'a [String],
) -> &'a str {
    match strategy {
        SkillStrategy::PreferSpecial => {
            // 优先特殊技能（跳过冷却中的），否则基础攻击
            skill_ids
                .iter()
                .find(|id| *id != "basic_attack" && cooldowns.get(id) == 0)
                .map(|s| s.as_str())
                .unwrap_or("basic_attack")
        }
        SkillStrategy::PreferBasic => {
            // 优先基础攻击
            if cooldowns.get("basic_attack") == 0 {
                "basic_attack"
            } else {
                skill_ids
                    .iter()
                    .find(|id| cooldowns.get(id) == 0)
                    .map(|s| s.as_str())
                    .unwrap_or("basic_attack")
            }
        }
        SkillStrategy::ByPriority => {
            // 按优先级列表选择
            if !priority.is_empty() {
                for preferred in priority {
                    if skill_ids.iter().any(|id| id == preferred) && cooldowns.get(preferred) == 0 {
                        return preferred.as_str();
                    }
                }
            }
            // 回退：优先特殊技能
            skill_ids
                .iter()
                .find(|id| *id != "basic_attack" && cooldowns.get(id) == 0)
                .map(|s| s.as_str())
                .unwrap_or("basic_attack")
        }
    }
}

/// AI 插件
pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::AppState;
        app.add_systems(Update, enemy_ai_system.run_if(in_state(AppState::InGame)));
    }
}
