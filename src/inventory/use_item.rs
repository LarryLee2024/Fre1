// 消耗品使用系统：UseItem Message + use_item_system

use super::container::Container;
use super::definition::{ItemDef, ItemRegistry, ItemType, UseEffect};
use crate::buff::{ActiveBuffs, BuffRegistry, apply_buff};
use crate::core::attribute::{AttributeKind, AttributeModifierInstance, Attributes, ModifierOp, ModifierSource};
use crate::core::tag::GameplayTags;
use bevy::prelude::*;

/// 使用消耗品 Message
#[derive(Message, Debug, Clone)]
pub struct UseItem {
    pub user_entity: Entity,
    pub container_entity: Entity,
    pub instance_id: u64,
}

/// 物品使用完成通知
#[derive(Message, Debug, Clone)]
pub struct ItemUsed {
    pub user_entity: Entity,
    pub def_id: String,
}

/// 消耗品授予临时特性通知（跨 Feature 广播，由 Trait 系统处理）
#[derive(Message, Debug, Clone)]
pub struct GrantTempTraitEffect {
    pub target_entity: Entity,
    pub trait_id: String,
    pub duration: u32,
}

/// 消耗品释放技能通知（跨 Feature 广播，由 Skill 系统处理）
#[derive(Message, Debug, Clone)]
pub struct CastSkillEffect {
    pub caster_entity: Entity,
    pub skill_id: String,
}

/// 使用消耗品系统
pub fn use_item_system(
    mut messages: MessageReader<UseItem>,
    mut containers: Query<&mut Container>,
    mut units: Query<(&mut Attributes, &mut ActiveBuffs, &mut GameplayTags)>,
    item_registry: Res<ItemRegistry>,
    buff_registry: Res<BuffRegistry>,
    mut used_writer: MessageWriter<ItemUsed>,
    mut trait_writer: MessageWriter<GrantTempTraitEffect>,
    mut skill_writer: MessageWriter<CastSkillEffect>,
) {
    for msg in messages.read() {
        let Ok(mut container) = containers.get_mut(msg.container_entity) else {
            continue;
        };
        let Some(stack) = container.get(msg.instance_id).cloned() else {
            continue;
        };
        let Some(def) = item_registry.get(&stack.instance.def_id) else {
            continue;
        };

        if def.item_type != ItemType::Consumable {
            continue;
        }

        // 应用使用效果（通过统一查询获取 Attributes + ActiveBuffs + GameplayTags）
        if let Ok((mut attrs, mut buffs, mut tags)) = units.get_mut(msg.user_entity) {
            let pending = apply_use_effects(&def, &mut attrs, &mut buffs, &mut tags, msg.user_entity, &buff_registry);
            // 发送跨 Feature Message
            for effect in pending {
                match effect {
                    PendingEffect::GrantTempTrait { trait_id, duration } => {
                        trait_writer.write(GrantTempTraitEffect {
                            target_entity: msg.user_entity,
                            trait_id,
                            duration,
                        });
                    }
                    PendingEffect::CastSkill { skill_id } => {
                        skill_writer.write(CastSkillEffect {
                            caster_entity: msg.user_entity,
                            skill_id,
                        });
                    }
                }
            }
        }

        // 消耗一个
        container.reduce_stack(msg.instance_id, 1);

        used_writer.write(ItemUsed {
            user_entity: msg.user_entity,
            def_id: def.id.clone(),
        });

        bevy::log::info!(
            target: "inventory",
            entity = ?msg.user_entity,
            item_id = %def.id,
            "消耗品已使用"
        );
    }
}

/// 跨 Feature 待发送效果（由 apply_use_effects 返回，由系统发送 Message）
enum PendingEffect {
    GrantTempTrait { trait_id: String, duration: u32 },
    CastSkill { skill_id: String },
}

/// 应用消耗品效果
/// RestoreVital：通过 add_modifier + set_vital 组合实现（不变量6合规）
/// ApplyBuff：通过 apply_buff() 统一管线处理
/// GrantTempTrait/CastSkill：返回 PendingEffect，由系统发送跨 Feature Message
fn apply_use_effects(
    def: &ItemDef,
    attrs: &mut Attributes,
    buffs: &mut ActiveBuffs,
    tags: &mut GameplayTags,
    user_entity: Entity,
    buff_registry: &BuffRegistry,
) -> Vec<PendingEffect> {
    let mut pending = Vec::new();
    for effect in &def.use_effects {
        match effect {
            UseEffect::RestoreVital { kind, value } => {
                // 不变量6：属性修改必须通过 Modifier 管线
                // 使用 ModifierSource 追踪来源，通过 add_modifier 记录修饰，
                // 再通过 set_vital 应用实际恢复值
                let source = ModifierSource::consumable_source(user_entity);
                attrs.add_modifier(AttributeModifierInstance {
                    kind: *kind,
                    op: ModifierOp::Add,
                    value: *value,
                    source,
                });
                // 计算恢复后的值（受 MaxHp/MaxMp/MaxStamina 上限约束）
                let current = attrs.get(*kind);
                let max_kind = match kind {
                    AttributeKind::Hp => AttributeKind::MaxHp,
                    AttributeKind::Mp => AttributeKind::MaxMp,
                    AttributeKind::Stamina => AttributeKind::MaxStamina,
                    _ => *kind,
                };
                let max = attrs.get(max_kind);
                attrs.set_vital(*kind, current.min(max));
                // 立即移除修饰符（RestoreVital 是一次性效果，不是持久修饰）
                attrs.remove_modifiers_from(source);
            }
            UseEffect::ApplyBuff { buff_id, duration } => {
                // 通过 apply_buff() 统一管线处理（修饰符+标签+同源刷新）
                if let Some(buff_data) = buff_registry.get(buff_id) {
                    apply_buff(buffs, attrs, tags, buff_data, Some(user_entity), *duration);
                }
            }
            UseEffect::GrantTempTrait { trait_id, duration } => {
                // 规则3：必须应用 GrantTempTrait 效果
                // 通过跨 Feature Message 广播，由 Trait 系统处理
                pending.push(PendingEffect::GrantTempTrait {
                    trait_id: trait_id.clone(),
                    duration: *duration,
                });
            }
            UseEffect::CastSkill { skill_id } => {
                // 规则3：必须应用 CastSkill 效果
                // 通过跨 Feature Message 广播，由 Skill 系统处理
                pending.push(PendingEffect::CastSkill {
                    skill_id: skill_id.clone(),
                });
            }
        }
    }
    pending
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buff::BuffData;
    use crate::core::attribute::ModifierOp;
    use crate::core::tag::GameplayTag;
    use crate::equipment::Rarity;
    use crate::inventory::instance::ItemInstance;

    fn test_consumable_def() -> ItemDef {
        ItemDef {
            version: 1,
            id: "potion_healing".into(),
            name: "治疗药水".into(),
            description: String::new(),
            item_type: ItemType::Consumable,
            rarity: Rarity::Common,
            tags: vec![],
            stack_size: 99,
            weight: 0.5,
            modifiers: vec![],
            traits: vec![],
            requirements: vec![],
            slot: None,
            use_effects: vec![UseEffect::RestoreVital {
                kind: AttributeKind::Hp,
                value: 50.0,
            }],
            container_capacity: None,
            container_max_weight: None,
        }
    }

    fn test_buff_registry() -> BuffRegistry {
        let mut registry = BuffRegistry::default();
        registry.register(BuffData {
            id: "attack_up".into(),
            name: "攻+5".into(),
            default_duration: 3,
            modifiers: vec![crate::core::attribute::AttributeModifierDef {
                kind: AttributeKind::Attack,
                op: ModifierOp::Add,
                value: 5.0,
            }],
            tags: vec![GameplayTag::BUFF],
            dot_damage: 0,
            hot_heal: 0,
            is_stun: false,
            is_cleanse: false,
            is_buff: true,
        });
        registry
    }

    #[test]
    fn 消耗品_应用恢复效果() {
        let def = test_consumable_def();
        let mut attrs = Attributes::default();
        attrs.fill_vital_resources();
        // 先扣血，验证恢复
        attrs.set_vital(AttributeKind::Hp, 10.0);
        let mut buffs = ActiveBuffs::default();
        let mut tags = GameplayTags::default();
        let buff_registry = BuffRegistry::default();
        let user = Entity::from_bits(1);

        apply_use_effects(&def, &mut attrs, &mut buffs, &mut tags, user, &buff_registry);

        // HP 应恢复 50，但不超过 MaxHp
        let hp = attrs.get(AttributeKind::Hp);
        let max_hp = attrs.get(AttributeKind::MaxHp);
        assert_eq!(hp, (10.0 + 50.0).min(max_hp));
    }

    #[test]
    fn 消耗品_应用buff效果() {
        let mut def = test_consumable_def();
        def.use_effects = vec![UseEffect::ApplyBuff {
            buff_id: "attack_up".into(),
            duration: 2,
        }];

        let mut attrs = Attributes::default();
        attrs.fill_vital_resources();
        let mut buffs = ActiveBuffs::default();
        let mut tags = GameplayTags::default();
        let buff_registry = test_buff_registry();
        let user = Entity::from_bits(1);

        apply_use_effects(&def, &mut attrs, &mut buffs, &mut tags, user, &buff_registry);

        // Buff 实例应被添加
        assert_eq!(buffs.len(), 1);
        assert_eq!(buffs.instances[0].buff_id, "attack_up");
        // 标签应被添加
        assert!(tags.has(GameplayTag::BUFF));
    }

    #[test]
    fn 消耗品_非消耗品不处理() {
        let def = ItemDef {
            version: 1,
            id: "iron_sword".into(),
            name: "铁剑".into(),
            description: String::new(),
            item_type: ItemType::Equipment,
            rarity: Rarity::Common,
            tags: vec![],
            stack_size: 1,
            weight: 3.0,
            modifiers: vec![],
            traits: vec![],
            requirements: vec![],
            slot: None,
            use_effects: vec![],
            container_capacity: None,
            container_max_weight: None,
        };

        let mut attrs = Attributes::default();
        let mut buffs = ActiveBuffs::default();
        let mut tags = GameplayTags::default();
        let buff_registry = BuffRegistry::default();
        let user = Entity::from_bits(1);

        let pending = apply_use_effects(&def, &mut attrs, &mut buffs, &mut tags, user, &buff_registry);

        // 装备没有 use_effects，不应修改任何状态
        assert!(buffs.is_empty());
        assert!(pending.is_empty());
    }

    #[test]
    fn 消耗品_GrantTempTrait返回PendingEffect() {
        let mut def = test_consumable_def();
        def.use_effects = vec![UseEffect::GrantTempTrait {
            trait_id: "fire_resist".into(),
            duration: 3,
        }];

        let mut attrs = Attributes::default();
        attrs.fill_vital_resources();
        let mut buffs = ActiveBuffs::default();
        let mut tags = GameplayTags::default();
        let buff_registry = BuffRegistry::default();
        let user = Entity::from_bits(1);

        let pending = apply_use_effects(&def, &mut attrs, &mut buffs, &mut tags, user, &buff_registry);

        assert_eq!(pending.len(), 1);
        match &pending[0] {
            PendingEffect::GrantTempTrait { trait_id, duration } => {
                assert_eq!(trait_id, "fire_resist");
                assert_eq!(*duration, 3);
            }
            _ => panic!("期望 GrantTempTrait 效果"),
        }
    }

    #[test]
    fn 消耗品_CastSkill返回PendingEffect() {
        let mut def = test_consumable_def();
        def.use_effects = vec![UseEffect::CastSkill {
            skill_id: "fireball".into(),
        }];

        let mut attrs = Attributes::default();
        attrs.fill_vital_resources();
        let mut buffs = ActiveBuffs::default();
        let mut tags = GameplayTags::default();
        let buff_registry = BuffRegistry::default();
        let user = Entity::from_bits(1);

        let pending = apply_use_effects(&def, &mut attrs, &mut buffs, &mut tags, user, &buff_registry);

        assert_eq!(pending.len(), 1);
        match &pending[0] {
            PendingEffect::CastSkill { skill_id } => {
                assert_eq!(skill_id, "fireball");
            }
            _ => panic!("期望 CastSkill 效果"),
        }
    }
}
