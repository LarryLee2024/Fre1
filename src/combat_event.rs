// 战斗事件模块：使用 Effect Pipeline 执行攻击
// 替代原来的 execute_attack 大函数，实现 生成→修饰→执行 三步管道

use crate::assets::CnFont;
use crate::combat_log::{CombatLog, LogSegment, log_color};
use crate::core::attribute::{AttributeKind, Attributes};
use crate::core::effect::{
    calculate_damage_from_effect, EffectDef, EffectQueue, PendingEffect, PendingEffectData,
};
use crate::core::modifier_rule::ModifierRuleRegistry;
use crate::core::tag::GameplayTags;
use crate::data::buff_data::{apply_buff, ActiveBuffs, BuffRegistry};
use crate::data::skill_data::{SkillCooldowns, SkillRegistry};
use crate::map::{GameMap, Terrain, Tile};
use crate::unit::{
    AttackRange, Faction, GridPosition, MovableRange, Selected, SelectionHighlight, Unit, UnitName,
};
use crate::vfx;
use bevy::prelude::*;

/// 攻击目标坐标 + 选择的技能（合并为单一资源以减少系统参数数量）
#[derive(Resource, Default)]
pub struct CombatIntent {
    pub target_coord: Option<IVec2>,
    pub skill_id: Option<String>,
}

/// 移动前位置（用于取消时回退）
#[derive(Resource, Default)]
pub struct PrevPosition {
    pub coord: Option<IVec2>,
}

// ── Effect Pipeline 系统 ──

/// 步骤 1：生成战斗效果（从技能定义 + 属性计算）
pub fn generate_combat_effects(
    mut queue: ResMut<EffectQueue>,
    selected_units: Query<
        (Entity, &Unit, &GridPosition, &UnitName, &Attributes, &GameplayTags, &SkillCooldowns),
        With<Selected>,
    >,
    targets: Query<
        (Entity, &Unit, &GridPosition, &UnitName, &Attributes, &GameplayTags, &Transform),
        Without<Selected>,
    >,
    tiles: Query<&Tile>,
    combat_intent: Res<CombatIntent>,
    skill_registry: Res<SkillRegistry>,
    _map: Res<GameMap>,
) {
    let Ok((source_entity, source_unit, _source_gp, _source_name, source_attrs, source_tags, _source_cooldowns)) =
        selected_units.single()
    else {
        return;
    };

    // 晕眩检查
    if source_tags.has(crate::core::tag::GameplayTag::STUN) {
        return;
    }

    let Some(target_coord) = combat_intent.target_coord else {
        return;
    };

    // 确定使用的技能
    let skill_id = combat_intent
        .skill_id
        .as_deref()
        .unwrap_or("basic_attack");
    let Some(skill_data) = skill_registry.get(skill_id) else {
        return;
    };

    // 查找目标
    for (target_entity, target_unit, target_gp, _target_name, target_attrs, _target_tags, _target_transform) in
        &targets
    {
        if target_gp.coord != target_coord || target_unit.faction == source_unit.faction {
            continue;
        }

        let terrain = tiles
            .iter()
            .find_map(|t| {
                if t.coord == target_gp.coord {
                    Some(t.terrain)
                } else {
                    None
                }
            })
            .unwrap_or(Terrain::Plain);

        // 从技能效果定义生成 PendingEffect
        for effect_def in &skill_data.effects {
            match effect_def {
                EffectDef::Damage {
                    multiplier,
                    ignore_def_percent,
                } => {
                    let effective_atk = source_attrs.get(AttributeKind::Atk);
                    let effective_def = target_attrs.get(AttributeKind::Def);
                    let base_def = target_attrs.base.get(&AttributeKind::Def).copied().unwrap_or(0.0);

                    let amount = calculate_damage_from_effect(
                        effective_atk,
                        effective_def,
                        base_def,
                        *multiplier,
                        *ignore_def_percent,
                        terrain,
                    );

                    queue.push(PendingEffect {
                        source: source_entity,
                        target: target_entity,
                        data: PendingEffectData::Damage {
                            amount,
                            is_skill: skill_id != "basic_attack",
                        },
                        source_tags: skill_data.tags.clone(),
                        terrain,
                    });
                }
                EffectDef::Heal { amount } => {
                    queue.push(PendingEffect {
                        source: source_entity,
                        target: target_entity,
                        data: PendingEffectData::Heal { amount: *amount },
                        source_tags: skill_data.tags.clone(),
                        terrain,
                    });
                }
                EffectDef::ApplyBuff { buff_id, duration } => {
                    queue.push(PendingEffect {
                        source: source_entity,
                        target: target_entity,
                        data: PendingEffectData::ApplyBuff {
                            buff_id: buff_id.clone(),
                            duration: *duration,
                        },
                        source_tags: skill_data.tags.clone(),
                        terrain,
                    });
                }
                EffectDef::Cleanse => {
                    queue.push(PendingEffect {
                        source: source_entity,
                        target: target_entity,
                        data: PendingEffectData::Cleanse,
                        source_tags: skill_data.tags.clone(),
                        terrain,
                    });
                }
            }
        }
        break; // 只处理第一个匹配的目标
    }
}

/// 步骤 2：修饰效果（从 ModifierRuleRegistry 加载规则）
pub fn modify_effects(mut queue: ResMut<EffectQueue>, tags_query: Query<&GameplayTags>, rules: Res<ModifierRuleRegistry>) {
    for effect in &mut queue.pending {
        if let PendingEffectData::Damage { ref mut amount, .. } = effect.data {
            if let Ok(target_tags) = tags_query.get(effect.target) {
                *amount = rules.apply_damage_modifiers(*amount, &effect.source_tags, target_tags);
            }
        }
    }
}

/// 步骤 3：执行效果（扣血/加 Buff/特效/日志/击杀）
pub fn execute_effects(
    commands: Commands,
    queue: ResMut<EffectQueue>,
    attrs_query: Query<&mut Attributes>,
    buffs_query: Query<&mut ActiveBuffs>,
    tags_query: Query<&mut GameplayTags>,
    gp_query: Query<&GridPosition>,
    name_query: Query<&UnitName>,
    unit_query: Query<&Unit>,
    combat_log: ResMut<CombatLog>,
    buff_registry: Res<BuffRegistry>,
    map: Res<GameMap>,
    cn_font: Res<CnFont>,
) {
    execute_effects_inline(
        commands,
        queue,
        attrs_query,
        buffs_query,
        tags_query,
        gp_query,
        name_query,
        unit_query,
        combat_log,
        buff_registry,
        map,
        cn_font,
    );
}

/// 执行效果的内联实现（供 AI 直接调用）
pub fn execute_effects_inline(
    mut commands: Commands,
    mut queue: ResMut<EffectQueue>,
    mut attrs_query: Query<&mut Attributes>,
    mut buffs_query: Query<&mut ActiveBuffs>,
    mut tags_query: Query<&mut GameplayTags>,
    gp_query: Query<&GridPosition>,
    name_query: Query<&UnitName>,
    unit_query: Query<&Unit>,
    mut combat_log: ResMut<CombatLog>,
    buff_registry: Res<BuffRegistry>,
    map: Res<GameMap>,
    cn_font: Res<CnFont>,
) {
    for effect in queue.pending.drain(..) {
        match effect.data {
            PendingEffectData::Damage { amount, is_skill } => {
                // 扣血
                if let Ok(mut target_attrs) = attrs_query.get_mut(effect.target) {
                    let hp = target_attrs.get(AttributeKind::Hp);
                    let new_hp = (hp - amount as f32).max(0.0);
                    target_attrs.set_base(AttributeKind::Hp, new_hp);
                }

                // 伤害数字弹出
                if let Ok(target_gp) = gp_query.get(effect.target) {
                    let world_pos = map.coord_to_world(target_gp.coord);
                    vfx::spawn_damage_popup(&mut commands, world_pos, amount, &cn_font.handle, is_skill);
                }

                // 战斗日志
                let attacker_color = unit_query
                    .get(effect.source)
                    .map(|u| {
                        if u.faction == Faction::Player {
                            log_color::PLAYER
                        } else {
                            log_color::ENEMY
                        }
                    })
                    .unwrap_or(log_color::NORMAL);
                let defender_color = unit_query
                    .get(effect.target)
                    .map(|u| {
                        if u.faction == Faction::Player {
                            log_color::PLAYER
                        } else {
                            log_color::ENEMY
                        }
                    })
                    .unwrap_or(log_color::NORMAL);

                let attacker_name = name_query
                    .get(effect.source)
                    .map(|n| n.0.as_str())
                    .unwrap_or("???");
                let target_name = name_query
                    .get(effect.target)
                    .map(|n| n.0.as_str())
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
                if let Ok(target_attrs) = attrs_query.get(effect.target) {
                    if target_attrs.get(AttributeKind::Hp) <= 0.0 {
                        combat_log.push(vec![
                            LogSegment { text: format!("[{}]", target_name), color: defender_color },
                            LogSegment { text: " 被击败！".to_string(), color: log_color::KILL },
                        ]);
                        commands.entity(effect.target).try_despawn();
                    }
                }
            }
            PendingEffectData::Heal { amount } => {
                if let Ok(mut target_attrs) = attrs_query.get_mut(effect.target) {
                    let hp = target_attrs.get(AttributeKind::Hp);
                    let max_hp = target_attrs.get(AttributeKind::MaxHp);
                    let new_hp = (hp + amount as f32).min(max_hp);
                    target_attrs.set_base(AttributeKind::Hp, new_hp);
                }

                let target_name = name_query
                    .get(effect.target)
                    .map(|n| n.0.as_str())
                    .unwrap_or("???");
                combat_log.push(vec![
                    LogSegment { text: format!("[{}]", target_name), color: log_color::NORMAL },
                    LogSegment { text: format!(" 恢复 {} HP", amount), color: log_color::HEAL },
                ]);
            }
            PendingEffectData::ApplyBuff { buff_id, duration } => {
                if let (Ok(mut target_buffs), Ok(mut target_attrs), Ok(mut target_tags)) = (
                    buffs_query.get_mut(effect.target),
                    attrs_query.get_mut(effect.target),
                    tags_query.get_mut(effect.target),
                ) {
                    if let Some(buff_data) = buff_registry.get(&buff_id) {
                        apply_buff(
                            &mut target_buffs,
                            &mut target_attrs,
                            &mut target_tags,
                            buff_data,
                            Some(effect.source),
                            duration,
                        );
                    }
                }
            }
            PendingEffectData::Cleanse => {
                if let (Ok(mut target_buffs), Ok(mut target_attrs), Ok(mut target_tags)) = (
                    buffs_query.get_mut(effect.target),
                    attrs_query.get_mut(effect.target),
                    tags_query.get_mut(effect.target),
                ) {
                    crate::data::buff_data::remove_all_debuffs(
                        &mut target_buffs,
                        &mut target_attrs,
                        &mut target_tags,
                    );
                }
            }
        }
    }
}

// ── OnEnter 系统 ──

/// 执行攻击（OnEnter ExecuteAction）
pub fn execute_action_on_enter(
    mut selected_units: Query<(Entity, &mut Unit, &GridPosition, &UnitName, &GameplayTags, &mut SkillCooldowns), With<Selected>>,
    mut next_phase: ResMut<NextState<crate::turn::TurnPhase>>,
    mut commands: Commands,
    combat_intent: Res<CombatIntent>,
    range_entities: Query<(Entity, Option<&GridPosition>), Or<(With<MovableRange>, With<AttackRange>)>>,
    highlights: Query<Entity, With<SelectionHighlight>>,
    skill_registry: Res<SkillRegistry>,
) {
    // 清除范围标记和高亮
    crate::input::clear_markers(&mut commands, &range_entities, &highlights);

    if let Ok((entity, mut unit, _pos, _name, tags, mut cooldowns)) = selected_units.single_mut() {
        // 晕眩检查
        if tags.has(crate::core::tag::GameplayTag::STUN) {
            unit.acted = true;
            commands.entity(entity).remove::<Selected>();
            next_phase.set(crate::turn::TurnPhase::TurnEnd);
            return;
        }

        // 设置技能冷却
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

    next_phase.set(crate::turn::TurnPhase::TurnEnd);
}

/// 待机（OnEnter WaitAction）
pub fn wait_action_on_enter(
    mut selected_units: Query<(Entity, &mut Unit), With<Selected>>,
    mut next_phase: ResMut<NextState<crate::turn::TurnPhase>>,
    mut commands: Commands,
    range_entities: Query<(Entity, Option<&GridPosition>), Or<(With<MovableRange>, With<AttackRange>)>>,
    highlights: Query<Entity, With<SelectionHighlight>>,
) {
    crate::input::clear_markers(&mut commands, &range_entities, &highlights);

    if let Ok((entity, mut unit)) = selected_units.single_mut() {
        unit.acted = true;
        commands.entity(entity).remove::<Selected>();
    }

    next_phase.set(crate::turn::TurnPhase::TurnEnd);
}

/// 战斗事件插件
pub struct CombatEventPlugin;

impl Plugin for CombatEventPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::TurnPhase;
        app.init_resource::<CombatIntent>()
            .init_resource::<PrevPosition>()
            .add_systems(
                OnEnter(TurnPhase::ExecuteAction),
                (
                    generate_combat_effects,
                    modify_effects,
                    execute_effects,
                    execute_action_on_enter,
                )
                    .chain(),
            )
            .add_systems(OnEnter(TurnPhase::WaitAction), wait_action_on_enter);
    }
}
