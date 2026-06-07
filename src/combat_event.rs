// 战斗事件模块：统一攻击处理逻辑，消除 input/ai 重复代码

use crate::assets::CnFont;
use crate::combat::{calculate_damage, skill_name};
use crate::combat_log::{CombatLog, LogSegment, log_color};
use crate::map::{Terrain, Tile};
use crate::status::StatusEffects;
use crate::unit::{
    AttackRange, Faction, GridPosition, MovableRange, Selected, Skill, Unit, UnitName,
};
use crate::vfx;
use bevy::prelude::*;

/// 攻击结果
pub struct AttackResult {
    /// 造成伤害
    pub damage: i32,
    /// 是否击杀
    pub killed: bool,
}

/// 执行攻击（统一入口）
///
/// 包含：伤害计算、扣血、伤害弹出、战斗日志、击杀处理
/// input.rs 和 ai.rs 共用此函数，消除重复逻辑
#[allow(clippy::too_many_arguments)]
pub fn execute_attack(
    commands: &mut Commands,
    attacker_unit: &Unit,
    attacker_atk_mod: i32,
    attacker_name: &str,
    target_entity: Entity,
    target_unit: &mut Unit,
    defender_def_mod: i32,
    target_name: &UnitName,
    target_translation: Vec2,
    terrain: Terrain,
    cn_font: &CnFont,
    combat_log: &mut CombatLog,
) -> AttackResult {
    // 伤害计算
    let damage = calculate_damage(
        attacker_unit,
        attacker_atk_mod,
        target_unit,
        defender_def_mod,
        terrain,
    );
    target_unit.hp -= damage;

    // 伤害数字弹出
    let is_crit = attacker_unit.skill != Skill::None;
    vfx::spawn_damage_popup(commands, target_translation, damage, &cn_font.handle, is_crit);

    // 战斗日志
    let attacker_color =
        if attacker_unit.faction == Faction::Player { log_color::PLAYER } else { log_color::ENEMY };
    let defender_color =
        if target_unit.faction == Faction::Player { log_color::PLAYER } else { log_color::ENEMY };
    let skill_label = skill_name(&attacker_unit.skill);

    combat_log.push(vec![
        LogSegment {
            text: format!("[{}]", attacker_name),
            color: attacker_color,
        },
        LogSegment {
            text: format!(" 使用[{}]", skill_label),
            color: log_color::TURN,
        },
        LogSegment {
            text: " 攻击 ".to_string(),
            color: log_color::NORMAL,
        },
        LogSegment {
            text: format!("[{}]", target_name.0),
            color: defender_color,
        },
        LogSegment {
            text: " 造成 ".to_string(),
            color: log_color::NORMAL,
        },
        LogSegment {
            text: format!("[{}]", damage),
            color: log_color::DAMAGE,
        },
        LogSegment {
            text: " 伤害".to_string(),
            color: log_color::NORMAL,
        },
        LogSegment {
            text: format!(" ({})", terrain.label()),
            color: log_color::TERRAIN,
        },
    ]);

    // 击杀处理
    let killed = target_unit.hp <= 0;
    if killed {
        combat_log.push(vec![
            LogSegment {
                text: format!("[{}]", target_name.0),
                color: defender_color,
            },
            LogSegment {
                text: " 被击败！".to_string(),
                color: log_color::KILL,
            },
        ]);
        commands.entity(target_entity).try_despawn();
    }

    AttackResult { damage, killed }
}

/// 执行攻击（OnEnter）
pub fn execute_action_on_enter(
    mut selected_units: Query<
        (Entity, &mut Unit, &GridPosition, &UnitName, &StatusEffects),
        With<Selected>,
    >,
    mut targets: Query<
        (Entity, &mut Unit, &GridPosition, &UnitName, &Transform, &StatusEffects),
        Without<Selected>,
    >,
    tiles: Query<&Tile>,
    mut next_phase: ResMut<NextState<crate::turn::TurnPhase>>,
    mut commands: Commands,
    mut attack_target: ResMut<crate::input::AttackTarget>,
    range_markers: Query<Entity, Or<(With<MovableRange>, With<AttackRange>)>>,
    highlights: Query<Entity, With<crate::unit::SelectionHighlight>>,
    mut combat_log: ResMut<CombatLog>,
    cn_font: Res<CnFont>,
) {
    // 清除范围标记和高亮
    crate::input::clear_markers(&mut commands, &range_markers, &highlights);

    if let Ok((entity, mut unit, _pos, attacker_name, attacker_status)) =
        selected_units.single_mut()
    {
        // 晕眩检查（防御性：resolve_status_effects 已标记 acted，但此处双重保障）
        if attacker_status.is_stunned() {
            unit.acted = true;
            commands.entity(entity).remove::<Selected>();
            attack_target.coord = None;
            next_phase.set(crate::turn::TurnPhase::TurnEnd);
            return;
        }

        // 查找攻击目标
        if let Some(target_coord) = attack_target.coord {
            for (target_entity, mut target, target_pos, target_name, target_transform, target_status) in
                targets.iter_mut()
            {
                if target_pos.coord == target_coord && target.faction != unit.faction {
                    let terrain =
                        tiles
                            .iter()
                            .find_map(|t| {
                                if t.coord == target_pos.coord { Some(t.terrain) } else { None }
                            })
                            .unwrap_or(Terrain::Plain);

                    // 统一攻击处理
                    execute_attack(
                        &mut commands,
                        &unit,
                        attacker_status.attack_mod(),
                        &attacker_name.0,
                        target_entity,
                        &mut target,
                        target_status.defense_mod(),
                        target_name,
                        target_transform.translation.truncate(),
                        terrain,
                        &cn_font,
                        &mut combat_log,
                    );
                    break;
                }
            }
        }

        unit.acted = true;
        commands.entity(entity).remove::<Selected>();
    }

    attack_target.coord = None;
    next_phase.set(crate::turn::TurnPhase::TurnEnd);
}

/// 待机（OnEnter）
pub fn wait_action_on_enter(
    mut selected_units: Query<(Entity, &mut Unit), With<Selected>>,
    mut next_phase: ResMut<NextState<crate::turn::TurnPhase>>,
    mut commands: Commands,
    range_markers: Query<Entity, Or<(With<MovableRange>, With<AttackRange>)>>,
    highlights: Query<Entity, With<crate::unit::SelectionHighlight>>,
) {
    // 清除范围标记和高亮
    crate::input::clear_markers(&mut commands, &range_markers, &highlights);

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
        app.add_systems(OnEnter(TurnPhase::ExecuteAction), execute_action_on_enter)
            .add_systems(OnEnter(TurnPhase::WaitAction), wait_action_on_enter);
    }
}
