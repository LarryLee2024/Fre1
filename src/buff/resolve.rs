// 持续效果结算：DoT/HoT/晕眩/tick，由 BuffPlugin 注册
// 原 status.rs，移入 buff 模块统一管理

use crate::assets::CnFont;
use crate::battle::{CombatLog, LogSegment, log_color};
use crate::character::{GridPosition, TraitGrantedTags, Unit, UnitName};
use crate::gameplay::attribute::{AttributeKind, Attributes, BuffInstanceId};
use crate::gameplay::tag::{GameplayTag, GameplayTags};
use crate::map::GameMap;
use crate::skill::SkillCooldowns;
use crate::turn::{NeedsResolve, TurnState};
use crate::ui::UiTheme;
use crate::ui::vfx;
use bevy::prelude::*;

use super::{ActiveBuffs, remove_buff};

/// 持续效果结算系统：在新阵营回合开始时，对该阵营所有单位结算 DoT/HoT/晕眩，并 tick
/// 通过 NeedsResolve 标记确保每回合只结算一次（防止 SelectUnit 多次进入时重复结算）
pub fn resolve_status_effects(
    mut commands: Commands,
    map: Res<GameMap>,
    turn_state: Res<TurnState>,
    cn_font: Res<CnFont>,
    mut combat_log: ResMut<CombatLog>,
    mut needs_resolve: ResMut<NeedsResolve>,
    theme: Res<UiTheme>,
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
    // 只有阵营切换后的首次 SelectUnit 才结算
    if !needs_resolve.0 {
        return;
    }
    needs_resolve.0 = false;

    for (entity, mut unit, name, gp, mut attrs, mut buffs, mut tags, mut cooldowns, trait_tags) in
        &mut units
    {
        if unit.faction != turn_state.current_faction {
            continue;
        }

        let world_pos = map.coord_to_world(gp.coord);

        // 1. 晕眩结算：被晕眩的单位本回合无法行动，消耗 Stun
        if buffs.is_stunned() {
            unit.acted = true;
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
                LogSegment {
                    text: format!("[{}]", name.0),
                    color: log_color::NORMAL,
                },
                LogSegment {
                    text: " 处于晕眩，无法行动".to_string(),
                    color: log_color::DAMAGE,
                },
            ]);
        }

        // 2. 结算本回合 DoT 伤害
        let dot = buffs.dot_damage();
        if dot > 0 {
            let hp = attrs.get(AttributeKind::Hp);
            let new_hp = (hp - dot as f32).max(0.0);
            attrs.set_base(AttributeKind::Hp, new_hp);
            vfx::spawn_damage_popup(
                &mut commands,
                world_pos,
                dot,
                &cn_font.handle,
                false,
                &theme,
            );
            combat_log.push(vec![
                LogSegment {
                    text: format!("[{}]", name.0),
                    color: log_color::NORMAL,
                },
                LogSegment {
                    text: format!(" 受到 {} 持续伤害", dot),
                    color: log_color::DAMAGE,
                },
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
                LogSegment {
                    text: format!("[{}]", name.0),
                    color: log_color::NORMAL,
                },
                LogSegment {
                    text: format!(" 恢复 {} HP", hot),
                    color: log_color::HEAL,
                },
            ]);
        }

        // 4. tick 递减持续时间，移除过期的 Buff
        tick_buffs(&mut buffs, &mut attrs, &mut tags, &trait_tags);

        // 5. tick 技能冷却
        cooldowns.tick();
    }
}

/// tick 所有 Buff：递减持续时间，移除过期的并清理其修饰符和标签
pub(crate) fn tick_buffs(
    buffs: &mut ActiveBuffs,
    attrs: &mut Attributes,
    tags: &mut GameplayTags,
    trait_tags: &TraitGrantedTags,
) {
    let expired_ids: Vec<BuffInstanceId> = buffs
        .instances
        .iter()
        .filter(|inst| inst.remaining_turns <= 1)
        .map(|inst| inst.instance_id)
        .collect();

    buffs.tick();

    for id in expired_ids {
        attrs.remove_modifiers_from(id);
    }

    rebuild_tags_from_buffs(buffs, tags, trait_tags);
}

/// 从所有活跃 Buff 重新构建 GameplayTags（保留 trait 授予的标签）
pub(crate) fn rebuild_tags_from_buffs(
    buffs: &ActiveBuffs,
    tags: &mut GameplayTags,
    trait_tags: &TraitGrantedTags,
) {
    let preserved_mask = trait_tags.0.0;

    let mut new_tags = GameplayTags(preserved_mask);
    for buff in &buffs.instances {
        for tag in &buff.tags {
            new_tags.add(*tag);
        }
    }

    tags.0 = new_tags.0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buff::BuffInstance;
    use crate::gameplay::attribute::{AttributeKind, AttributeModifierInstance, ModifierOp};

    fn make_test_buff(
        id: u64,
        buff_id: &str,
        remaining: u32,
        tags: Vec<GameplayTag>,
        is_buff: bool,
    ) -> BuffInstance {
        BuffInstance {
            instance_id: BuffInstanceId(id),
            buff_id: buff_id.into(),
            name: buff_id.into(),
            remaining_turns: remaining,
            source_entity: None,
            tags,
            is_buff,
            dot_damage: 0,
            hot_heal: 0,
        }
    }

    // ── tick_buffs 测试 ──

    #[test]
    fn tick_buffs_过期buff清理修饰符() {
        let mut buffs = ActiveBuffs::default();
        buffs.add(make_test_buff(
            1,
            "shield",
            1,
            vec![GameplayTag::BUFF],
            true,
        ));

        let mut attrs = Attributes::default();
        attrs.fill_vital_resources();
        attrs.add_modifier(AttributeModifierInstance {
            kind: AttributeKind::Attack,
            op: ModifierOp::Add,
            value: 5.0,
            source: BuffInstanceId(1),
        });
        let attack_before = attrs.get(AttributeKind::Attack);

        let mut tags = GameplayTags::default();
        let trait_tags = TraitGrantedTags::default();

        tick_buffs(&mut buffs, &mut attrs, &mut tags, &trait_tags);

        // 修饰符被清理，攻击力恢复
        assert!(attrs.modifiers.is_empty());
        assert!(attrs.get(AttributeKind::Attack) < attack_before);
        // buff 实例仍在列表中（remaining_turns=0），下次 tick 才移除
        assert_eq!(buffs.len(), 1);
        assert_eq!(buffs.instances[0].remaining_turns, 0);
    }

    #[test]
    fn tick_buffs_未过期buff持续时间递减() {
        let mut buffs = ActiveBuffs::default();
        buffs.add(make_test_buff(
            1,
            "shield",
            3,
            vec![GameplayTag::BUFF],
            true,
        ));

        let mut attrs = Attributes::default();
        attrs.fill_vital_resources();
        let mut tags = GameplayTags::default();
        let trait_tags = TraitGrantedTags::default();

        tick_buffs(&mut buffs, &mut attrs, &mut tags, &trait_tags);

        assert_eq!(buffs.len(), 1);
        assert_eq!(buffs.instances[0].remaining_turns, 2);
    }

    #[test]
    fn tick_buffs_清理过期buff的修饰符() {
        let mut buffs = ActiveBuffs::default();
        buffs.add(make_test_buff(
            1,
            "attack_up",
            1,
            vec![GameplayTag::BUFF],
            true,
        ));

        let mut attrs = Attributes::default();
        attrs.fill_vital_resources();
        attrs.add_modifier(AttributeModifierInstance {
            kind: AttributeKind::Attack,
            op: ModifierOp::Add,
            value: 5.0,
            source: BuffInstanceId(1),
        });
        let attack_before = attrs.get(AttributeKind::Attack);

        let mut tags = GameplayTags::default();
        let trait_tags = TraitGrantedTags::default();

        tick_buffs(&mut buffs, &mut attrs, &mut tags, &trait_tags);

        // 修饰符被清理，攻击力恢复
        assert!(attrs.modifiers.is_empty());
        assert!(attrs.get(AttributeKind::Attack) < attack_before);
    }

    #[test]
    fn tick_buffs_保留多个buff中未过期的() {
        let mut buffs = ActiveBuffs::default();
        buffs.add(make_test_buff(
            1,
            "expired",
            1,
            vec![GameplayTag::BUFF],
            true,
        ));
        buffs.add(make_test_buff(2, "alive", 3, vec![GameplayTag::BUFF], true));

        let mut attrs = Attributes::default();
        attrs.fill_vital_resources();
        let mut tags = GameplayTags::default();
        let trait_tags = TraitGrantedTags::default();

        tick_buffs(&mut buffs, &mut attrs, &mut tags, &trait_tags);

        // 两个 buff 都在（过期的 remaining_turns=0，下次 tick 才移除）
        assert_eq!(buffs.len(), 2);
        // "expired" remaining_turns=1 → tick 后变为 0
        let expired = buffs
            .instances
            .iter()
            .find(|b| b.buff_id == "expired")
            .unwrap();
        assert_eq!(expired.remaining_turns, 0);
        // "alive" remaining_turns=3 → tick 后变为 2
        let alive = buffs
            .instances
            .iter()
            .find(|b| b.buff_id == "alive")
            .unwrap();
        assert_eq!(alive.remaining_turns, 2);
    }

    #[test]
    fn tick_buffs_空buff列表() {
        let mut buffs = ActiveBuffs::default();
        let mut attrs = Attributes::default();
        attrs.fill_vital_resources();
        let mut tags = GameplayTags::default();
        let trait_tags = TraitGrantedTags::default();

        tick_buffs(&mut buffs, &mut attrs, &mut tags, &trait_tags);

        assert!(buffs.is_empty());
    }

    // ── rebuild_tags_from_buffs 测试 ──

    #[test]
    fn rebuild_tags_from_buffs_从活跃buff重建标签() {
        let mut buffs = ActiveBuffs::default();
        buffs.add(make_test_buff(
            1,
            "fire_shield",
            3,
            vec![GameplayTag::BUFF, GameplayTag::FIRE],
            true,
        ));

        let mut tags = GameplayTags::default();
        let trait_tags = TraitGrantedTags::default();

        rebuild_tags_from_buffs(&buffs, &mut tags, &trait_tags);

        assert!(tags.has(GameplayTag::FIRE));
        assert!(tags.has(GameplayTag::BUFF));
    }

    #[test]
    fn rebuild_tags_from_buffs_保留trait授予的标签() {
        let buffs = ActiveBuffs::default();
        let mut tags = GameplayTags::default();
        let trait_tags = TraitGrantedTags(GameplayTags::from_tags(&[GameplayTag::WARRIOR]));

        rebuild_tags_from_buffs(&buffs, &mut tags, &trait_tags);

        assert!(tags.has(GameplayTag::WARRIOR));
    }

    #[test]
    fn rebuild_tags_from_buffs_清除非trait非buff标签() {
        let buffs = ActiveBuffs::default();
        let mut tags = GameplayTags::from_tags(&[GameplayTag::FIRE, GameplayTag::WARRIOR]);
        let trait_tags = TraitGrantedTags::default();

        rebuild_tags_from_buffs(&buffs, &mut tags, &trait_tags);

        // FIRE 来自旧 buff，trait_tags 为空 → 应该被清除
        assert!(!tags.has(GameplayTag::FIRE));
        assert!(!tags.has(GameplayTag::WARRIOR));
    }

    #[test]
    fn rebuild_tags_from_buffs_多buff多标签合并() {
        let mut buffs = ActiveBuffs::default();
        buffs.add(make_test_buff(1, "fire", 3, vec![GameplayTag::FIRE], true));
        buffs.add(make_test_buff(
            2,
            "stun",
            3,
            vec![GameplayTag::STUN, GameplayTag::DEBUFF],
            false,
        ));

        let mut tags = GameplayTags::default();
        let trait_tags = TraitGrantedTags::default();

        rebuild_tags_from_buffs(&buffs, &mut tags, &trait_tags);

        assert!(tags.has(GameplayTag::FIRE));
        assert!(tags.has(GameplayTag::STUN));
        assert!(tags.has(GameplayTag::DEBUFF));
    }

    #[test]
    fn rebuild_tags_from_buffs_空buff空trait() {
        let buffs = ActiveBuffs::default();
        let mut tags = GameplayTags::from_tags(&[GameplayTag::FIRE]);
        let trait_tags = TraitGrantedTags::default();

        rebuild_tags_from_buffs(&buffs, &mut tags, &trait_tags);

        assert!(!tags.has(GameplayTag::FIRE));
    }
}
