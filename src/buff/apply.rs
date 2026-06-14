use super::domain::BuffData;
use super::instance::{ActiveBuffs, BuffInstance};
use crate::core::attribute::{Attributes, BuffInstanceId};
use crate::core::tag::GameplayTags;
use bevy::prelude::*;

/// 给目标施加 Buff（修改 ActiveBuffs + Attributes + GameplayTags）
///
/// 施加管线：查找定义 → Cleanse 检查 → 同源刷新检查 → 生成实例 → 添加修饰符 → 添加标签
pub fn apply_buff(
    active_buffs: &mut ActiveBuffs,
    attributes: &mut Attributes,
    gameplay_tags: &mut GameplayTags,
    buff_data: &BuffData,
    source_entity: Option<Entity>,
    duration: u32,
) -> BuffInstanceId {
    // Step2: Cleanse 特殊处理：立即驱散所有 debuff
    if buff_data.is_cleanse {
        remove_all_debuffs(active_buffs, attributes, gameplay_tags);
        return BuffInstanceId(0);
    }

    // Step3: 同源同 buff_id 刷新持续时间（不新增实例、不重复添加修饰符/标签）
    if let Some(source) = source_entity {
        if let Some(existing) = active_buffs
            .instances
            .iter_mut()
            .find(|b| b.source_entity == Some(source) && b.buff_id == buff_data.id)
        {
            existing.remaining_turns = duration;
            return existing.instance_id;
        }
    }

    // Step3: 生成实例
    let instance_id = active_buffs.next_instance_id();

    let instance = BuffInstance {
        instance_id,
        buff_id: buff_data.id.clone(),
        name: buff_data.name.clone(),
        remaining_turns: duration,
        source_entity,
        tags: buff_data.tags.clone(),
        is_buff: buff_data.is_buff,
        dot_damage: buff_data.dot_damage,
        hot_heal: buff_data.hot_heal,
    };

    // Step4: 添加修饰符到 Attributes（使用 ModifierSource::buff_source）
    attributes.add_modifiers_from_def(&buff_data.modifiers, instance_id.to_modifier_source());

    // Step5: 添加标签到 GameplayTags
    for tag in &buff_data.tags {
        gameplay_tags.add(*tag);
    }

    active_buffs.add(instance);

    bevy::log::info!(
        target: "buff",
        event = "buff_applied",
        buff_id = %buff_data.id,
        instance_id = %instance_id.0,
        source = ?source_entity,
        remaining_turns = duration,
        "Buff 已施加"
    );

    instance_id
}

/// 移除指定 Buff 实例（清理 Attributes + GameplayTags）
pub fn remove_buff(
    active_buffs: &mut ActiveBuffs,
    attributes: &mut Attributes,
    gameplay_tags: &mut GameplayTags,
    instance_id: BuffInstanceId,
) {
    if let Some(removed) = active_buffs.remove(instance_id) {
        bevy::log::info!(
            target: "buff",
            event = "buff_removed",
            buff_id = %removed.buff_id,
            instance_id = %instance_id.0,
            source = ?removed.source_entity,
            "Buff 已移除"
        );

        // 移除修饰符（转换为 ModifierSource）
        attributes.remove_modifiers_from(instance_id.to_modifier_source());

        // 移除标签（仅当没有其他 Buff 提供相同标签时）
        let remaining_tags: Vec<crate::core::tag::GameplayTag> = active_buffs
            .instances
            .iter()
            .flat_map(|b| b.tags.iter())
            .copied()
            .collect();
        for tag in &removed.tags {
            if !remaining_tags.contains(tag) {
                gameplay_tags.remove(*tag);
            }
        }
    }
}

/// 移除所有 Debuff
pub fn remove_all_debuffs(
    active_buffs: &mut ActiveBuffs,
    attributes: &mut Attributes,
    gameplay_tags: &mut GameplayTags,
) {
    let debuff_ids: Vec<BuffInstanceId> = active_buffs
        .instances
        .iter()
        .filter(|b| b.is_debuff())
        .map(|b| b.instance_id)
        .collect();
    for id in debuff_ids {
        remove_buff(active_buffs, attributes, gameplay_tags, id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::attribute::{AttributeKind, AttributeModifierDef, ModifierOp};
    use crate::core::tag::GameplayTag;
    use bevy::prelude::*;

    /// 辅助：创建一个简单的 BuffData
    fn make_buff(
        id: &str,
        is_buff: bool,
        modifiers: Vec<AttributeModifierDef>,
        tags: Vec<GameplayTag>,
    ) -> BuffData {
        BuffData {
            id: id.into(),
            name: id.into(),
            default_duration: 2,
            modifiers,
            tags,
            dot_damage: 0,
            hot_heal: 0,
            is_stun: false,
            is_cleanse: false,
            is_buff,
        }
    }

    /// 辅助：构建测试用 Attributes（战士模板：Might=5, Vitality=5 → Attack=10, Defense=5）
    fn make_test_attrs() -> Attributes {
        let mut attrs = Attributes::default();
        attrs.set_base(AttributeKind::Might, 5.0);
        attrs.set_base(AttributeKind::Vitality, 5.0);
        attrs.set_base(AttributeKind::Agility, 6.0);
        attrs.set_base(AttributeKind::Dexterity, 3.0);
        attrs.set_base(AttributeKind::Intelligence, 2.0);
        attrs.set_base(AttributeKind::Willpower, 3.0);
        attrs.set_base(AttributeKind::Presence, 2.0);
        attrs.set_base(AttributeKind::Luck, 2.0);
        attrs.set_base_attack_range(1);
        attrs.fill_vital_resources();
        attrs
    }

    #[test]
    fn apply_buff_添加修饰符和标签() {
        let mut buffs = ActiveBuffs::default();
        let mut attrs = make_test_attrs();
        let mut tags = GameplayTags::default();

        let buff_data = make_buff(
            "attack_up",
            true,
            vec![AttributeModifierDef {
                kind: AttributeKind::Attack,
                op: ModifierOp::Add,
                value: 5.0,
            }],
            vec![GameplayTag::BUFF],
        );

        apply_buff(&mut buffs, &mut attrs, &mut tags, &buff_data, None, 2);

        assert_eq!(buffs.len(), 1);
        // Attack = Might*2 + 5 = 10 + 5 = 15
        assert_eq!(attrs.get(AttributeKind::Attack), 15.0);
        assert!(tags.has(GameplayTag::BUFF));
    }

    #[test]
    fn remove_buff_清理修饰符和标签() {
        let mut buffs = ActiveBuffs::default();
        let mut attrs = make_test_attrs();
        let mut tags = GameplayTags::default();

        let buff_data = make_buff(
            "attack_up",
            true,
            vec![AttributeModifierDef {
                kind: AttributeKind::Attack,
                op: ModifierOp::Add,
                value: 5.0,
            }],
            vec![GameplayTag::BUFF],
        );

        let instance_id = apply_buff(&mut buffs, &mut attrs, &mut tags, &buff_data, None, 2);
        // Attack = 10 + 5 = 15
        assert_eq!(attrs.get(AttributeKind::Attack), 15.0);

        remove_buff(&mut buffs, &mut attrs, &mut tags, instance_id);
        // Attack 恢复为 10
        assert_eq!(attrs.get(AttributeKind::Attack), 10.0);
        assert!(!tags.has(GameplayTag::BUFF));
        assert!(buffs.is_empty());
    }

    #[test]
    fn remove_buff_共享标签不被误删() {
        let mut buffs = ActiveBuffs::default();
        let mut attrs = make_test_attrs();
        let mut tags = GameplayTags::default();

        let buff_a = make_buff(
            "buff_a",
            true,
            vec![AttributeModifierDef {
                kind: AttributeKind::Attack,
                op: ModifierOp::Add,
                value: 5.0,
            }],
            vec![GameplayTag::BUFF, GameplayTag::FIRE],
        );
        let buff_b = make_buff(
            "buff_b",
            true,
            vec![AttributeModifierDef {
                kind: AttributeKind::Defense,
                op: ModifierOp::Add,
                value: 3.0,
            }],
            vec![GameplayTag::BUFF, GameplayTag::FIRE],
        );

        let id_a = apply_buff(&mut buffs, &mut attrs, &mut tags, &buff_a, None, 2);
        let _id_b = apply_buff(&mut buffs, &mut attrs, &mut tags, &buff_b, None, 2);

        remove_buff(&mut buffs, &mut attrs, &mut tags, id_a);
        assert!(tags.has(GameplayTag::BUFF));
        assert!(tags.has(GameplayTag::FIRE));
    }

    #[test]
    fn apply_buff_cleanse_驱散所有debuff() {
        let mut buffs = ActiveBuffs::default();
        let mut attrs = make_test_attrs();
        let mut tags = GameplayTags::default();

        let debuff = make_buff(
            "attack_down",
            false,
            vec![AttributeModifierDef {
                kind: AttributeKind::Attack,
                op: ModifierOp::Add,
                value: -5.0,
            }],
            vec![GameplayTag::DEBUFF],
        );
        apply_buff(&mut buffs, &mut attrs, &mut tags, &debuff, None, 2);
        // Attack = 10 - 5 = 5
        assert_eq!(attrs.get(AttributeKind::Attack), 5.0);

        let cleanse = BuffData {
            id: "cleanse".into(),
            name: "驱散".into(),
            default_duration: 0,
            modifiers: vec![],
            tags: vec![GameplayTag::BUFF],
            dot_damage: 0,
            hot_heal: 0,
            is_stun: false,
            is_cleanse: true,
            is_buff: true,
        };
        apply_buff(&mut buffs, &mut attrs, &mut tags, &cleanse, None, 0);

        assert!(buffs.is_empty());
        // Attack 恢复为 10
        assert_eq!(attrs.get(AttributeKind::Attack), 10.0);
    }

    #[test]
    fn remove_all_debuffs_只移除debuff保留buff() {
        let mut buffs = ActiveBuffs::default();
        let mut attrs = make_test_attrs();
        let mut tags = GameplayTags::default();

        let buff = make_buff(
            "attack_up",
            true,
            vec![AttributeModifierDef {
                kind: AttributeKind::Attack,
                op: ModifierOp::Add,
                value: 5.0,
            }],
            vec![GameplayTag::BUFF],
        );
        let debuff = make_buff(
            "defense_down",
            false,
            vec![AttributeModifierDef {
                kind: AttributeKind::Defense,
                op: ModifierOp::Add,
                value: -3.0,
            }],
            vec![GameplayTag::DEBUFF],
        );

        apply_buff(&mut buffs, &mut attrs, &mut tags, &buff, None, 2);
        apply_buff(&mut buffs, &mut attrs, &mut tags, &debuff, None, 2);
        // Attack = 10 + 5 = 15, Defense = 5 - 3 = 2
        assert_eq!(attrs.get(AttributeKind::Attack), 15.0);
        assert_eq!(attrs.get(AttributeKind::Defense), 2.0);

        remove_all_debuffs(&mut buffs, &mut attrs, &mut tags);
        assert_eq!(buffs.len(), 1);
        assert_eq!(buffs.instances[0].buff_id, "attack_up");
        // Attack 仍为 15，Defense 恢复为 5
        assert_eq!(attrs.get(AttributeKind::Attack), 15.0);
        assert_eq!(attrs.get(AttributeKind::Defense), 5.0);
    }

    #[test]
    fn apply_buff_同源刷新不重复添加修饰符() {
        let mut buffs = ActiveBuffs::default();
        let mut attrs = make_test_attrs();
        let mut tags = GameplayTags::default();
        let source = Entity::from_bits(42);

        let buff_data = make_buff(
            "attack_up",
            true,
            vec![AttributeModifierDef {
                kind: AttributeKind::Attack,
                op: ModifierOp::Add,
                value: 5.0,
            }],
            vec![GameplayTag::BUFF],
        );

        // 首次施加：Attack = 10 + 5 = 15
        let id1 = apply_buff(
            &mut buffs,
            &mut attrs,
            &mut tags,
            &buff_data,
            Some(source),
            2,
        );
        assert_eq!(attrs.get(AttributeKind::Attack), 15.0);
        assert_eq!(buffs.len(), 1);

        // 同源刷新：持续时间刷新，修饰符不重复添加
        let id2 = apply_buff(
            &mut buffs,
            &mut attrs,
            &mut tags,
            &buff_data,
            Some(source),
            3,
        );
        assert_eq!(id2, id1); // 返回同一 instance_id
        assert_eq!(buffs.len(), 1); // 不新增实例
        assert_eq!(buffs.instances[0].remaining_turns, 3); // 持续时间刷新
        assert_eq!(attrs.get(AttributeKind::Attack), 15.0); // 修饰符不重复
    }

    #[test]
    fn apply_buff_不同源同id可共存() {
        let mut buffs = ActiveBuffs::default();
        let mut attrs = make_test_attrs();
        let mut tags = GameplayTags::default();
        let source_a = Entity::from_bits(1);
        let source_b = Entity::from_bits(2);

        let buff_data = make_buff(
            "attack_up",
            true,
            vec![AttributeModifierDef {
                kind: AttributeKind::Attack,
                op: ModifierOp::Add,
                value: 5.0,
            }],
            vec![GameplayTag::BUFF],
        );

        apply_buff(
            &mut buffs,
            &mut attrs,
            &mut tags,
            &buff_data,
            Some(source_a),
            2,
        );
        apply_buff(
            &mut buffs,
            &mut attrs,
            &mut tags,
            &buff_data,
            Some(source_b),
            2,
        );

        // 不同源：两个实例共存，修饰符叠加
        assert_eq!(buffs.len(), 2);
        // Attack = 10 + 5 + 5 = 20
        assert_eq!(attrs.get(AttributeKind::Attack), 20.0);
    }
}
