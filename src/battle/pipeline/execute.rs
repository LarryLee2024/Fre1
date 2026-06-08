// 步骤 3：执行效果（扣血/加 Buff/特效/日志/击杀）
// 包含共享的 apply_* 函数，供 AI 模块直接调用

use crate::assets::CnFont;
use crate::battle::log::{CombatLog, LogSegment, log_color};
use crate::gameplay::attribute::{AttributeKind, Attributes};
use crate::gameplay::effect::{EffectQueue, PendingEffectData};
use crate::gameplay::tag::GameplayTags;
use crate::buff::{ActiveBuffs, BuffRegistry, apply_buff, remove_all_debuffs};
use crate::map::GameMap;
use crate::character::{Faction, GridPosition, Unit, UnitName};
use crate::ui::vfx;
use bevy::prelude::*;

/// 执行效果（系统入口，委托给 execute_effects_inline）
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
        commands, queue, attrs_query, buffs_query, tags_query,
        gp_query, name_query, unit_query, combat_log, buff_registry, map, cn_font,
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

        match effect.data {
            PendingEffectData::Damage { amount, is_skill } => {
                if let (Ok(mut target_attrs), Ok(target_gp)) = (
                    attrs_query.get_mut(effect.target),
                    gp_query.get(effect.target),
                ) {
                    apply_damage_effect(
                        &mut target_attrs,
                        target_gp,
                        target_name,
                        effect.target,
                        defender_color,
                        attacker_name,
                        attacker_color,
                        amount,
                        is_skill,
                        effect.terrain.label(),
                        &mut commands,
                        &mut combat_log,
                        &map,
                        &cn_font,
                    );
                }
            }
            PendingEffectData::Heal { amount } => {
                if let Ok(mut target_attrs) = attrs_query.get_mut(effect.target) {
                    apply_heal_effect(&mut target_attrs, target_name, amount, &mut combat_log);
                }
            }
            PendingEffectData::ApplyBuff { buff_id, duration } => {
                if let (Ok(mut target_buffs), Ok(mut target_attrs), Ok(mut target_tags)) = (
                    buffs_query.get_mut(effect.target),
                    attrs_query.get_mut(effect.target),
                    tags_query.get_mut(effect.target),
                ) {
                    apply_buff_effect(
                        &mut target_buffs,
                        &mut target_attrs,
                        &mut target_tags,
                        &buff_id,
                        effect.source,
                        duration,
                        &buff_registry,
                    );
                }
            }
            PendingEffectData::Cleanse => {
                if let (Ok(mut target_buffs), Ok(mut target_attrs), Ok(mut target_tags)) = (
                    buffs_query.get_mut(effect.target),
                    attrs_query.get_mut(effect.target),
                    tags_query.get_mut(effect.target),
                ) {
                    apply_cleanse_effect(&mut target_buffs, &mut target_attrs, &mut target_tags);
                }
            }
        }
    }
}

// ── 共享效果执行函数（消除 execute_effects_inline 和 execute_ai_effects 的代码重复）──

/// 应用伤害效果：扣血、弹出伤害数字、战斗日志、击杀处理
#[allow(clippy::too_many_arguments)]
pub fn apply_damage_effect(
    target_attrs: &mut Attributes,
    target_gp: &GridPosition,
    target_name: &str,
    target_entity: Entity,
    target_color: Color,
    attacker_name: &str,
    attacker_color: Color,
    amount: i32,
    is_skill: bool,
    terrain_label: &str,
    commands: &mut Commands,
    combat_log: &mut CombatLog,
    map: &GameMap,
    cn_font: &CnFont,
) {
    let hp = target_attrs.get(AttributeKind::Hp);
    let new_hp = (hp - amount as f32).max(0.0);
    target_attrs.set_base(AttributeKind::Hp, new_hp);

    let world_pos = map.coord_to_world(target_gp.coord);
    vfx::spawn_damage_popup(commands, world_pos, amount, &cn_font.handle, is_skill);

    let skill_label = if is_skill { "技能" } else { "攻击" };
    combat_log.push(vec![
        LogSegment { text: format!("[{}]", attacker_name), color: attacker_color },
        LogSegment { text: format!(" 使用[{}]", skill_label), color: log_color::TURN },
        LogSegment { text: " 攻击 ".to_string(), color: log_color::NORMAL },
        LogSegment { text: format!("[{}]", target_name), color: target_color },
        LogSegment { text: " 造成 ".to_string(), color: log_color::NORMAL },
        LogSegment { text: format!("[{}]", amount), color: log_color::DAMAGE },
        LogSegment { text: " 伤害".to_string(), color: log_color::NORMAL },
        LogSegment { text: format!(" ({})", terrain_label), color: log_color::TERRAIN },
    ]);

    if new_hp <= 0.0 {
        combat_log.push(vec![
            LogSegment { text: format!("[{}]", target_name), color: target_color },
            LogSegment { text: " 被击败！".to_string(), color: log_color::KILL },
        ]);
        commands.entity(target_entity).try_despawn();
    }
}

/// 应用治疗效果
pub fn apply_heal_effect(
    target_attrs: &mut Attributes,
    target_name: &str,
    amount: i32,
    combat_log: &mut CombatLog,
) {
    let hp = target_attrs.get(AttributeKind::Hp);
    let max_hp = target_attrs.get(AttributeKind::MaxHp);
    let new_hp = (hp + amount as f32).min(max_hp);
    target_attrs.set_base(AttributeKind::Hp, new_hp);

    combat_log.push(vec![
        LogSegment { text: format!("[{}]", target_name), color: log_color::NORMAL },
        LogSegment { text: format!(" 恢复 {} HP", amount), color: log_color::HEAL },
    ]);
}

/// 应用 Buff 效果
pub fn apply_buff_effect(
    target_buffs: &mut ActiveBuffs,
    target_attrs: &mut Attributes,
    target_tags: &mut GameplayTags,
    buff_id: &str,
    source: Entity,
    duration: u32,
    buff_registry: &BuffRegistry,
) {
    if let Some(buff_data) = buff_registry.get(buff_id) {
        apply_buff(target_buffs, target_attrs, target_tags, buff_data, Some(source), duration);
    }
}

/// 应用净化效果
pub fn apply_cleanse_effect(
    target_buffs: &mut ActiveBuffs,
    target_attrs: &mut Attributes,
    target_tags: &mut GameplayTags,
) {
    remove_all_debuffs(target_buffs, target_attrs, target_tags);
}
