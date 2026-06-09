// 步骤 3：执行效果（纯逻辑：扣血/加 Buff/击杀判定）
// 表现层（VFX/日志）通过 Message 响应，不在此处调用

use crate::battle::{CharacterDied, DamageApplied, HealApplied};
use crate::buff::{ActiveBuffs, BuffRegistry, apply_buff, remove_all_debuffs};
use crate::character::{Dead, Faction, GridPosition, Unit, UnitName};
use crate::core::attribute::{AttributeKind, Attributes};
use crate::core::effect::{EffectQueue, PendingEffectData};
use crate::core::tag::GameplayTags;
use crate::map::TerrainRegistry;
use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;

/// 执行效果（系统入口，委托给 execute_effects_inline）
pub fn execute_effects(
    mut commands: Commands,
    mut queue: ResMut<EffectQueue>,
    mut attrs_query: Query<&mut Attributes>,
    mut buffs_query: Query<&mut ActiveBuffs>,
    mut tags_query: Query<&mut GameplayTags>,
    gp_query: Query<&GridPosition>,
    name_query: Query<&UnitName>,
    unit_query: Query<&Unit>,
    buff_registry: Res<BuffRegistry>,
    terrain_registry: Res<TerrainRegistry>,
    mut died_writer: MessageWriter<CharacterDied>,
    mut damage_writer: MessageWriter<DamageApplied>,
    mut heal_writer: MessageWriter<HealApplied>,
) {
    execute_effects_inline(
        &mut commands,
        &mut queue,
        &mut attrs_query,
        &mut buffs_query,
        &mut tags_query,
        &gp_query,
        &name_query,
        &unit_query,
        &buff_registry,
        &terrain_registry,
        &mut died_writer,
        &mut damage_writer,
        &mut heal_writer,
    );
}

/// 执行效果的内联实现
#[allow(clippy::too_many_arguments)]
pub fn execute_effects_inline(
    commands: &mut Commands,
    queue: &mut ResMut<EffectQueue>,
    attrs_query: &mut Query<&mut Attributes>,
    buffs_query: &mut Query<&mut ActiveBuffs>,
    tags_query: &mut Query<&mut GameplayTags>,
    gp_query: &Query<&GridPosition>,
    name_query: &Query<&UnitName>,
    unit_query: &Query<&Unit>,
    buff_registry: &BuffRegistry,
    terrain_registry: &TerrainRegistry,
    died_writer: &mut MessageWriter<CharacterDied>,
    damage_writer: &mut MessageWriter<DamageApplied>,
    heal_writer: &mut MessageWriter<HealApplied>,
) {
    for effect in queue.pending.drain(..) {
        let attacker_name = name_query
            .get(effect.source)
            .map(|n| n.0.as_str())
            .unwrap_or("???")
            .to_string();
        let attacker_faction = unit_query
            .get(effect.source)
            .map(|u| u.faction)
            .unwrap_or(Faction::Enemy);
        let target_name = name_query
            .get(effect.target)
            .map(|n| n.0.as_str())
            .unwrap_or("???")
            .to_string();
        let target_faction = unit_query
            .get(effect.target)
            .map(|u| u.faction)
            .unwrap_or(Faction::Enemy);
        let target_coord = gp_query
            .get(effect.target)
            .map(|gp| gp.coord)
            .unwrap_or(IVec2::ZERO);
        let terrain_label = terrain_registry
            .get(&effect.terrain_id)
            .map(|def| def.name.as_str())
            .unwrap_or("???")
            .to_string();

        match effect.data {
            PendingEffectData::Damage { amount, is_skill } => {
                if let Ok(mut target_attrs) = attrs_query.get_mut(effect.target) {
                    apply_damage_effect(
                        &mut target_attrs,
                        effect.target,
                        &target_name,
                        target_faction,
                        &attacker_name,
                        attacker_faction,
                        amount,
                        is_skill,
                        &terrain_label,
                        target_coord,
                        commands,
                        died_writer,
                        damage_writer,
                    );
                }
            }
            PendingEffectData::Heal { amount } => {
                if let Ok(mut target_attrs) = attrs_query.get_mut(effect.target) {
                    apply_heal_effect(
                        &mut target_attrs,
                        effect.target,
                        &target_name,
                        amount,
                        heal_writer,
                    );
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
                        buff_registry,
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

// ── 纯逻辑效果执行函数 ──

/// 应用伤害效果：扣血 + 死亡判定，通过 Message 通知表现层
#[allow(clippy::too_many_arguments)]
pub fn apply_damage_effect(
    target_attrs: &mut Attributes,
    target_entity: Entity,
    target_name: &str,
    target_faction: Faction,
    attacker_name: &str,
    attacker_faction: Faction,
    amount: i32,
    is_skill: bool,
    terrain_label: &str,
    target_coord: IVec2,
    commands: &mut Commands,
    died_writer: &mut MessageWriter<CharacterDied>,
    damage_writer: &mut MessageWriter<DamageApplied>,
) {
    // 扣血
    let hp = target_attrs.get(AttributeKind::Hp);
    let new_hp = (hp - amount as f32).max(0.0);
    target_attrs.set_base(AttributeKind::Hp, new_hp);

    // 发送伤害消息（VFX/日志/表现层响应）
    damage_writer.write(DamageApplied {
        target: target_entity,
        target_name: target_name.to_string(),
        target_faction,
        attacker_name: attacker_name.to_string(),
        attacker_faction,
        amount,
        is_skill,
        terrain_label: terrain_label.to_string(),
        target_coord,
    });

    // 死亡判定
    if new_hp <= 0.0 {
        commands.entity(target_entity).insert(Dead);
        died_writer.write(CharacterDied {
            entity: target_entity,
            name: target_name.to_string(),
            faction: target_faction,
        });
    }
}

/// 应用治疗效果：回血，通过 Message 通知表现层
pub fn apply_heal_effect(
    target_attrs: &mut Attributes,
    target_entity: Entity,
    target_name: &str,
    amount: i32,
    heal_writer: &mut MessageWriter<HealApplied>,
) {
    let hp = target_attrs.get(AttributeKind::Hp);
    let max_hp = target_attrs.get(AttributeKind::MaxHp);
    let new_hp = (hp + amount as f32).min(max_hp);
    target_attrs.set_base(AttributeKind::Hp, new_hp);

    heal_writer.write(HealApplied {
        target: target_entity,
        target_name: target_name.to_string(),
        amount,
    });
}

/// 应用 Buff 效果（纯逻辑，无表现层调用）
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
        apply_buff(
            target_buffs,
            target_attrs,
            target_tags,
            buff_data,
            Some(source),
            duration,
        );
    }
}

/// 应用净化效果（纯逻辑，无表现层调用）
pub fn apply_cleanse_effect(
    target_buffs: &mut ActiveBuffs,
    target_attrs: &mut Attributes,
    target_tags: &mut GameplayTags,
) {
    remove_all_debuffs(target_buffs, target_attrs, target_tags);
}
