// 持续效果模块：使用 ActiveBuffs + Attributes + GameplayTags
// 替代原 StatusEffect 枚举，实现数据驱动的 Buff/Debuff 系统

use crate::assets::CnFont;
use crate::battle::{CombatLog, LogSegment, log_color};
use crate::gameplay::attribute::{AttributeKind, Attributes, BuffInstanceId};
use crate::gameplay::tag::GameplayTag;
use crate::gameplay::tag::GameplayTags;
use crate::buff::{remove_buff, ActiveBuffs};
use crate::skill::SkillCooldowns;
use crate::map::GameMap;
use crate::turn::TurnState;
use crate::character::{GridPosition, TraitGrantedTags, Unit, UnitName};
use crate::ui::vfx;
use bevy::prelude::*;

/// 持续效果结算系统：在新阵营回合开始时，对该阵营所有单位结算 DoT/HoT/晕眩，并 tick
pub fn resolve_status_effects(
    mut commands: Commands,
    map: Res<GameMap>,
    turn_state: Res<TurnState>,
    cn_font: Res<CnFont>,
    mut combat_log: ResMut<CombatLog>,
    mut units: Query<(
        Entity,
        &mut Unit,
        &UnitName,
        &GridPosition,
        &mut Attributes,
        &mut ActiveBuffs,
        &mut GameplayTags,
        &mut SkillCooldowns,
        &TraitGrantedTags,
    )>,
) {
    for (entity, mut unit, name, gp, mut attrs, mut buffs, mut tags, mut cooldowns, trait_tags) in &mut units {
        if unit.faction != turn_state.current_faction {
            continue;
        }

        let world_pos = map.coord_to_world(gp.coord);

        // 1. 晕眩结算：被晕眩的单位本回合无法行动，消耗 Stun
        if buffs.is_stunned() {
            unit.acted = true;
            // 移除所有带 STUN 标签的 Buff 实例
            let stun_ids: Vec<BuffInstanceId> = buffs
                .instances
                .iter()
                .filter(|b| b.tags.contains(&GameplayTag::STUN))
                .map(|b| b.instance_id)
                .collect();
            for id in stun_ids {
                remove_buff(&mut buffs, &mut attrs, &mut tags, id);
            }
            combat_log.push(vec![
                LogSegment { text: format!("[{}]", name.0), color: log_color::NORMAL },
                LogSegment { text: " 处于晕眩，无法行动".to_string(), color: log_color::DAMAGE },
            ]);
        }

        // 2. 结算本回合 DoT 伤害
        let dot = buffs.dot_damage();
        if dot > 0 {
            let hp = attrs.get(AttributeKind::Hp);
            let new_hp = (hp - dot as f32).max(0.0);
            attrs.set_base(AttributeKind::Hp, new_hp);
            vfx::spawn_damage_popup(&mut commands, world_pos, dot, &cn_font.handle, false);
            combat_log.push(vec![
                LogSegment { text: format!("[{}]", name.0), color: log_color::NORMAL },
                LogSegment { text: format!(" 受到 {} 持续伤害", dot), color: log_color::DAMAGE },
            ]);
            if new_hp <= 0.0 {
                commands.entity(entity).try_despawn();
            }
        }

        // 3. 结算本回合 HoT 治疗
        let hot = buffs.hot_heal();
        if hot > 0 {
            let hp = attrs.get(AttributeKind::Hp);
            let max_hp = attrs.get(AttributeKind::MaxHp);
            let new_hp = (hp + hot as f32).min(max_hp);
            attrs.set_base(AttributeKind::Hp, new_hp);
            combat_log.push(vec![
                LogSegment { text: format!("[{}]", name.0), color: log_color::NORMAL },
                LogSegment { text: format!(" 恢复 {} HP", hot), color: log_color::HEAL },
            ]);
        }

        // 4. tick 递减持续时间，移除过期的 Buff
        tick_buffs(&mut buffs, &mut attrs, &mut tags, &trait_tags);

        // 5. tick 技能冷却
        cooldowns.tick();
    }
}

/// tick 所有 Buff：递减持续时间，移除过期的并清理其修饰符和标签
fn tick_buffs(buffs: &mut ActiveBuffs, attrs: &mut Attributes, tags: &mut GameplayTags, trait_tags: &TraitGrantedTags) {
    // 收集需要移除的 Buff ID（remaining_turns 将变为 0 的）
    let expired_ids: Vec<BuffInstanceId> = buffs
        .instances
        .iter()
        .filter(|inst| inst.remaining_turns <= 1)
        .map(|inst| inst.instance_id)
        .collect();

    // 先递减
    buffs.tick();

    // 移除过期的 Buff（tick 后 remaining_turns == 0 的已被 retain 移除，但修饰符还在）
    // 实际上 tick() 已经移除了 remaining_turns == 0 的实例
    // 但我们还需要清理 Attributes 中的修饰符和 GameplayTags 中的标签
    for id in expired_ids {
        attrs.remove_modifiers_from(id);
        // 标签清理需要检查是否还有其他 Buff 提供相同标签
        // 简化处理：重新计算所有活跃 Buff 的标签
    }

    // 重新同步 GameplayTags（基于当前活跃 Buff + trait 授予的标签）
    rebuild_tags_from_buffs(buffs, tags, trait_tags);
}

/// 从所有活跃 Buff 重新构建 GameplayTags（保留 trait 授予的标签）
fn rebuild_tags_from_buffs(buffs: &ActiveBuffs, tags: &mut GameplayTags, trait_tags: &TraitGrantedTags) {
    // 保留 trait 授予的标签（如 MELEE、RANGED、FIRE 等）
    let preserved_mask = trait_tags.0.0;

    // 从活跃 Buff 重建
    let mut new_tags = GameplayTags(preserved_mask);
    for buff in &buffs.instances {
        for tag in &buff.tags {
            new_tags.add(*tag);
        }
    }

    tags.0 = new_tags.0;
}

/// 持续效果插件
pub struct StatusPlugin;

impl Plugin for StatusPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::TurnPhase;
        app.add_systems(OnEnter(TurnPhase::SelectUnit), resolve_status_effects);
    }
}
