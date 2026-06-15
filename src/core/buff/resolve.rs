// 持续效果结算：DoT/HoT/晕眩/tick，由 BuffPlugin 注册
// 纯逻辑：只做数值计算和状态变更，通过 Message 通知表现层
// 原 status.rs，移入 buff 模块统一管理

use crate::core::ability::SkillCooldowns;
use crate::core::attribute::{Attributes, BuffInstanceId, ModifierSource};
use crate::core::battle::{DotApplied, HotApplied, StunApplied};
use crate::core::buff::domain::DurationPolicy;
use crate::core::character::{Dead, GridPosition, PersistentTags, Unit, UnitName};
use crate::core::tag::{GameplayTag, GameplayTags};
use crate::core::turn::NeedsResolve;
use crate::shared::event::battle as shared_battle;
use crate::shared::event::buff::{BuffRemoveReason, BuffRemoved};
use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;

use super::{ActiveBuffs, remove_buff};

/// 持续效果结算系统：每回合开始时，对所有单位结算 DoT/HoT/晕眩，并 tick
/// 纯逻辑：只修改数值和状态，通过 Message 通知表现层
/// 通过 NeedsResolve 标记确保每回合只结算一次
/// 新逻辑：队列驱动模式下，每回合所有单位都行动，因此对所有单位结算
pub fn resolve_status_effects(
    mut commands: Commands,
    mut needs_resolve: ResMut<NeedsResolve>,
    mut units: Query<(
        Entity,
        &mut Unit,
        &UnitName,
        &GridPosition,
        &mut Attributes,
        &mut ActiveBuffs,
        &mut GameplayTags,
        &mut SkillCooldowns,
        &PersistentTags,
        Option<&crate::core::character::UnitId>,
    )>,
    mut dot_writer: MessageWriter<DotApplied>,
    mut hot_writer: MessageWriter<HotApplied>,
    mut stun_writer: MessageWriter<StunApplied>,
    mut buff_expired_writer: MessageWriter<BuffRemoved>,
) {
    // 只有回合切换后的首次 SelectUnit 才结算
    if !needs_resolve.0 {
        return;
    }
    needs_resolve.0 = false;

    for (
        entity,
        mut unit,
        name,
        gp,
        mut attrs,
        mut buffs,
        mut tags,
        mut cooldowns,
        persistent_tags,
        unit_id,
    ) in &mut units
    {
        // 队列驱动模式：所有单位都结算（不再按阵营过滤）
        // TODO(future): Once all entities carry &UnitId component, remove fallback to to_bits()
        let shared_uid = |e: Entity| {
            unit_id
                .map(|uid| crate::shared::ids::UnitId::new(&uid.0))
                .unwrap_or_else(|| crate::shared::ids::UnitId::new(e.to_bits().to_string()))
        };

        // 1. 晕眩结算：被晕眩的单位本回合无法行动，消耗 Stun
        if buffs.is_stunned() {
            unit.acted = true;
            let stun_ids: Vec<BuffInstanceId> = buffs
                .instances
                .iter()
                .filter(|b| b.tags.contains(&GameplayTag::CONTROL_HARD))
                .map(|b| b.instance_id)
                .collect();
            for id in stun_ids {
                remove_buff(&mut buffs, &mut attrs, &mut tags, id);
            }
            // 发送晕眩消息（表现层响应）
            stun_writer.write(StunApplied {
                target: entity,
                target_name: name.0.clone(),
            });
            // 共享事件（供 LogObserver 使用）
            commands.write_message(shared_battle::StunApplied {
                target: shared_uid(entity),
                target_name: name.0.clone(),
                duration: 0,
            });
        }

        // 2. 结算本回合 DoT 伤害
        let dot = buffs.dot_damage();
        if dot > 0 {
            let actual_damage = attrs.take_damage(dot);

            // 发送 DoT 消息（表现层响应）
            dot_writer.write(DotApplied {
                target: entity,
                target_name: name.0.clone(),
                amount: actual_damage,
                target_coord: gp.coord,
            });
            // 共享事件（供 LogObserver 使用）
            commands.write_message(shared_battle::DotApplied {
                target: shared_uid(entity),
                target_name: name.0.clone(),
                amount: actual_damage,
            });

            // DoT 死亡判定：只添加 Dead Tag，CharacterDied 由 Dead Observer 统一发送
            // 规则3：禁止在 HP 变化时内联死亡处理（宪法 5.0 分层：Hook+Observer+Message）
            if !attrs.is_alive() {
                commands.entity(entity).insert(Dead);
            }
        }

        // 3. 结算本回合 HoT 治疗
        let hot = buffs.hot_heal();
        if hot > 0 {
            let actual_heal = attrs.heal(hot);

            // 发送 HoT 消息（表现层响应）
            hot_writer.write(HotApplied {
                target: entity,
                target_name: name.0.clone(),
                amount: hot,
            });
            // 共享事件（供 LogObserver 使用）
            commands.write_message(shared_battle::HotApplied {
                target: shared_uid(entity),
                target_name: name.0.clone(),
                amount: hot,
            });
        }

        // 4. tick 递减持续时间，移除过期的 Buff
        // 仅 DurationPolicy::Turns 的 buff 会在 tick 中过期
        let expired_buffs: Vec<String> = buffs
            .instances
            .iter()
            .filter(|inst| {
                matches!(inst.duration_policy, DurationPolicy::Turns(_))
                    && inst.remaining_turns <= 1
            })
            .map(|inst| inst.buff_id.clone())
            .collect();
        tick_buffs(&mut buffs, &mut attrs, &mut tags, &persistent_tags);
        for buff_id in &expired_buffs {
            // TODO(future): Once all entities carry &UnitId component, remove fallback
            let target_id = unit_id
                .map(|uid| crate::shared::ids::UnitId::new(&uid.0))
                .unwrap_or_else(|| crate::shared::ids::UnitId::new(entity.to_bits().to_string()));
            buff_expired_writer.write(BuffRemoved {
                target: target_id,
                target_name: name.0.clone(),
                buff_id: crate::shared::ids::EffectId::new(buff_id),
                reason: BuffRemoveReason::Expired,
            });
        }

        // 5. tick 技能冷却
        cooldowns.tick();
    }
}

/// tick 所有 Buff：递减持续时间，移除过期的并清理其修饰符和标签
///
/// 仅 DurationPolicy::Turns 的 buff 会被递减和过期清理。
/// 其他策略（UntilDeath, Permanent 等）由外部事件触发移除。
pub(crate) fn tick_buffs(
    buffs: &mut ActiveBuffs,
    attrs: &mut Attributes,
    tags: &mut GameplayTags,
    persistent: &PersistentTags,
) {
    // 仅识别 DurationPolicy::Turns 且即将过期的 buff
    let expired_ids: Vec<ModifierSource> = buffs
        .instances
        .iter()
        .filter(|inst| {
            matches!(inst.duration_policy, DurationPolicy::Turns(_)) && inst.remaining_turns <= 1
        })
        .map(|inst| inst.instance_id.to_modifier_source())
        .collect();

    buffs.tick();

    for id in expired_ids {
        attrs.remove_modifiers_from(id);
    }

    rebuild_tags(buffs, tags, persistent);
}

/// 从所有活跃 Buff 重新构建 GameplayTags（保留 Trait + Equipment 授予的标签）
pub(crate) fn rebuild_tags(
    buffs: &ActiveBuffs,
    tags: &mut GameplayTags,
    persistent: &PersistentTags,
) {
    let mut new_tags = GameplayTags::default();
    // 第一层：Trait 授予（最持久）
    new_tags.0 |= persistent.from_traits.0;
    // 第二层：装备授予（穿脱变化）
    new_tags.0 |= persistent.from_equipment.0;
    // 第三层：Buff 授予（临时）
    for buff in &buffs.instances {
        // 跳过已过期的 buff（remaining_turns == 0，修饰符已清理但实例仍在列表中）
        if buff.remaining_turns == 0 {
            continue;
        }
        for tag in &buff.tags {
            new_tags.add(*tag);
        }
    }

    tags.0 = new_tags.0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::attribute::{BuffInstanceId, ModifierOp, ModifierSource};
    use crate::core::buff::BuffInstance;

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
            duration_policy: DurationPolicy::Turns(remaining),
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
        attrs.add_modifier(
            "phys_atk".into(),
            ModifierOp::Add,
            5,
            ModifierSource::buff_source(1),
        );
        let attack_before = attrs.get("phys_atk");

        let mut tags = GameplayTags::default();
        let trait_tags = PersistentTags::default();

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
        let trait_tags = PersistentTags::default();

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
        attrs.add_modifier(
            "phys_atk".into(),
            ModifierOp::Add,
            5,
            ModifierSource::buff_source(1),
        );
        let attack_before = attrs.get("phys_atk");

        let mut tags = GameplayTags::default();
        let trait_tags = PersistentTags::default();

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
        let trait_tags = PersistentTags::default();

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
    fn tick_buffs_过期buff标签被清理() {
        let mut buffs = ActiveBuffs::default();
        buffs.add(make_test_buff(
            1,
            "fire_shield",
            1,
            vec![GameplayTag::BUFF, GameplayTag::DMG_FIRE],
            true,
        ));

        let mut attrs = Attributes::default();
        attrs.fill_vital_resources();
        let mut tags = GameplayTags::default();
        let trait_tags = PersistentTags::default();

        tick_buffs(&mut buffs, &mut attrs, &mut tags, &trait_tags);

        // 过期 buff（remaining_turns=0）的标签不应出现
        assert!(!tags.has(GameplayTag::DMG_FIRE));
        assert!(!tags.has(GameplayTag::BUFF));
    }

    #[test]
    fn tick_buffs_空buff列表() {
        let mut buffs = ActiveBuffs::default();
        let mut attrs = Attributes::default();
        attrs.fill_vital_resources();
        let mut tags = GameplayTags::default();
        let trait_tags = PersistentTags::default();

        tick_buffs(&mut buffs, &mut attrs, &mut tags, &trait_tags);

        assert!(buffs.is_empty());
    }

    #[test]
    fn tick_buffs_permanent_buff不递减() {
        let mut buffs = ActiveBuffs::default();
        buffs.add(BuffInstance {
            instance_id: BuffInstanceId(1),
            buff_id: "permanent".into(),
            name: "永久".into(),
            remaining_turns: 5,
            duration_policy: DurationPolicy::Permanent,
            source_entity: None,
            tags: vec![GameplayTag::BUFF],
            is_buff: true,
            dot_damage: 0,
            hot_heal: 0,
        });

        let mut attrs = Attributes::default();
        attrs.fill_vital_resources();
        let mut tags = GameplayTags::default();
        let trait_tags = PersistentTags::default();

        tick_buffs(&mut buffs, &mut attrs, &mut tags, &trait_tags);

        // Permanent buff 不应递减
        assert_eq!(buffs.len(), 1);
        assert_eq!(buffs.instances[0].remaining_turns, 5);
    }

    // ── rebuild_tags 测试 ──

    #[test]
    fn rebuild_tags_从活跃buff重建标签() {
        let mut buffs = ActiveBuffs::default();
        buffs.add(make_test_buff(
            1,
            "fire_shield",
            3,
            vec![GameplayTag::BUFF, GameplayTag::DMG_FIRE],
            true,
        ));

        let mut tags = GameplayTags::default();
        let trait_tags = PersistentTags::default();

        rebuild_tags(&buffs, &mut tags, &trait_tags);

        assert!(tags.has(GameplayTag::DMG_FIRE));
        assert!(tags.has(GameplayTag::BUFF));
    }

    #[test]
    fn rebuild_tags_保留trait授予的标签() {
        let buffs = ActiveBuffs::default();
        let mut tags = GameplayTags::default();
        let trait_tags = PersistentTags {
            from_traits: GameplayTags::from_tags(&[GameplayTag::ALLY]),
            from_equipment: GameplayTags::default(),
        };

        rebuild_tags(&buffs, &mut tags, &trait_tags);

        assert!(tags.has(GameplayTag::ALLY));
    }

    #[test]
    fn rebuild_tags_清除非trait非buff标签() {
        let buffs = ActiveBuffs::default();
        let mut tags = GameplayTags::from_tags(&[GameplayTag::DMG_FIRE, GameplayTag::ALLY]);
        let trait_tags = PersistentTags::default();

        rebuild_tags(&buffs, &mut tags, &trait_tags);

        // FIRE 来自旧 buff，trait_tags 为空 → 应该被清除
        assert!(!tags.has(GameplayTag::DMG_FIRE));
        assert!(!tags.has(GameplayTag::ALLY));
    }

    #[test]
    fn rebuild_tags_多buff多标签合并() {
        let mut buffs = ActiveBuffs::default();
        buffs.add(make_test_buff(
            1,
            "fire",
            3,
            vec![GameplayTag::DMG_FIRE],
            true,
        ));
        buffs.add(make_test_buff(
            2,
            "stun",
            3,
            vec![GameplayTag::CONTROL_HARD, GameplayTag::DEBUFF],
            false,
        ));

        let mut tags = GameplayTags::default();
        let trait_tags = PersistentTags::default();

        rebuild_tags(&buffs, &mut tags, &trait_tags);

        assert!(tags.has(GameplayTag::DMG_FIRE));
        assert!(tags.has(GameplayTag::CONTROL_HARD));
        assert!(tags.has(GameplayTag::DEBUFF));
    }

    #[test]
    fn rebuild_tags_空buff空trait() {
        let buffs = ActiveBuffs::default();
        let mut tags = GameplayTags::from_tags(&[GameplayTag::DMG_FIRE]);
        let trait_tags = PersistentTags::default();

        rebuild_tags(&buffs, &mut tags, &trait_tags);

        assert!(!tags.has(GameplayTag::DMG_FIRE));
    }
}
