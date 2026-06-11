// Trait 触发器：在战斗管线中触发 OnAttack/OnHit/OnKill 的 Trait 效果
// 遵循 Logic/Presentation 分离：只生成额外效果推入 EffectQueue，不直接播放动画

use crate::character::{
    TraitCollection, TraitEffect, TraitEffectHandlerRegistry, TraitRegistry, TraitTrigger,
};
use crate::core::effect::{EffectQueue, PendingEffect, PendingEffectData};
use bevy::prelude::*;

/// 在攻击生成阶段触发攻击者的 OnAttack Trait 效果
/// 将 OnAttack Trait 中的 ApplyBuff 效果推入 EffectQueue
pub fn trigger_on_attack_traits(
    attacker: Entity,
    target: Entity,
    trait_collection: &TraitCollection,
    trait_registry: &TraitRegistry,
    effect_handlers: &TraitEffectHandlerRegistry,
    queue: &mut EffectQueue,
) {
    trigger_traits(
        TraitTrigger::OnAttack,
        attacker,
        Some(target),
        trait_collection,
        trait_registry,
        effect_handlers,
        queue,
    );
}

/// 在被攻击时触发目标的 OnHit Trait 效果
pub fn trigger_on_hit_traits(
    target: Entity,
    attacker: Entity,
    trait_collection: &TraitCollection,
    trait_registry: &TraitRegistry,
    effect_handlers: &TraitEffectHandlerRegistry,
    queue: &mut EffectQueue,
) {
    trigger_traits(
        TraitTrigger::OnHit,
        target,
        Some(attacker),
        trait_collection,
        trait_registry,
        effect_handlers,
        queue,
    );
}

/// 在击杀时触发攻击者的 OnKill Trait 效果
pub fn trigger_on_kill_traits(
    killer: Entity,
    victim: Entity,
    trait_collection: &TraitCollection,
    trait_registry: &TraitRegistry,
    effect_handlers: &TraitEffectHandlerRegistry,
    queue: &mut EffectQueue,
) {
    trigger_traits(
        TraitTrigger::OnKill,
        killer,
        Some(victim),
        trait_collection,
        trait_registry,
        effect_handlers,
        queue,
    );
}

/// 通用 Trait 触发：遍历 TraitCollection，匹配 trigger 类型，将 ApplyBuff 效果推入队列
fn trigger_traits(
    trigger: TraitTrigger,
    source_entity: Entity,
    target_entity: Option<Entity>,
    trait_collection: &TraitCollection,
    trait_registry: &TraitRegistry,
    _effect_handlers: &TraitEffectHandlerRegistry,
    queue: &mut EffectQueue,
) {
    for entry in &trait_collection.entries {
        if let Some(trait_data) = trait_registry.get(&entry.trait_id) {
            if trait_data.trigger != trigger {
                continue;
            }
            // 处理触发型 Trait 的效果
            for effect in &trait_data.effects {
                if let TraitEffect::ApplyBuff { buff_id, duration } = effect {
                    let target = target_entity.unwrap_or(source_entity);
                    queue.push(PendingEffect {
                        source: source_entity,
                        target,
                        data: PendingEffectData::ApplyBuff {
                            buff_id: buff_id.clone(),
                            duration: *duration,
                        },
                        source_tags: vec![],
                        terrain_id: String::new(),
                    });
                    bevy::log::trace!(
                        target: "battle",
                        trigger = ?trigger,
                        trait_id = %entry.trait_id,
                        source_entity = ?source_entity,
                        target_entity = ?target,
                        buff_id = %buff_id,
                        "Trait 触发效果入队"
                    );
                }
                // GrantTag 和 ModifyAttribute 是 Passive 效果，不在触发器中处理
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::{TraitData, TraitSource};
    use crate::core::attribute::{AttributeKind, AttributeModifierDef, ModifierOp};

    fn make_test_registry() -> TraitRegistry {
        let mut registry = TraitRegistry::default();
        registry.traits.insert(
            "on_attack_buff".into(),
            TraitData {
                id: "on_attack_buff".into(),
                name: "攻击时加Buff".into(),
                description: String::new(),
                trigger: TraitTrigger::OnAttack,
                effects: vec![TraitEffect::ApplyBuff {
                    buff_id: "attack_up".into(),
                    duration: 2,
                }],
            },
        );
        registry.traits.insert(
            "on_hit_counter".into(),
            TraitData {
                id: "on_hit_counter".into(),
                name: "被击时反击".into(),
                description: String::new(),
                trigger: TraitTrigger::OnHit,
                effects: vec![TraitEffect::ApplyBuff {
                    buff_id: "defense_up".into(),
                    duration: 1,
                }],
            },
        );
        registry.traits.insert(
            "passive_trait".into(),
            TraitData {
                id: "passive_trait".into(),
                name: "被动".into(),
                description: String::new(),
                trigger: TraitTrigger::Passive,
                effects: vec![TraitEffect::ModifyAttribute(AttributeModifierDef {
                    kind: AttributeKind::Attack,
                    op: ModifierOp::Add,
                    value: 5.0,
                })],
            },
        );
        registry
    }

    #[test]
    fn on_attack_触发apply_buff() {
        let registry = make_test_registry();
        let handlers = TraitEffectHandlerRegistry::with_defaults();
        let mut collection = TraitCollection::default();
        collection.add_entry("on_attack_buff".into(), TraitSource::Intrinsic);

        let mut queue = EffectQueue::default();
        let attacker = Entity::from_bits(1);
        let target = Entity::from_bits(2);

        trigger_on_attack_traits(
            attacker,
            target,
            &collection,
            &registry,
            &handlers,
            &mut queue,
        );

        assert_eq!(queue.pending.len(), 1);
        assert_eq!(queue.pending[0].source, attacker);
        assert_eq!(queue.pending[0].target, target);
        if let PendingEffectData::ApplyBuff { buff_id, duration } = &queue.pending[0].data {
            assert_eq!(buff_id, "attack_up");
            assert_eq!(*duration, 2);
        } else {
            panic!("期望 ApplyBuff 效果");
        }
    }

    #[test]
    fn on_hit_触发apply_buff() {
        let registry = make_test_registry();
        let handlers = TraitEffectHandlerRegistry::with_defaults();
        let mut collection = TraitCollection::default();
        collection.add_entry("on_hit_counter".into(), TraitSource::Intrinsic);

        let mut queue = EffectQueue::default();
        let target = Entity::from_bits(1);
        let attacker = Entity::from_bits(2);

        trigger_on_hit_traits(
            target,
            attacker,
            &collection,
            &registry,
            &handlers,
            &mut queue,
        );

        assert_eq!(queue.pending.len(), 1);
    }

    #[test]
    fn passive_trait_不触发() {
        let registry = make_test_registry();
        let handlers = TraitEffectHandlerRegistry::with_defaults();
        let mut collection = TraitCollection::default();
        collection.add_entry("passive_trait".into(), TraitSource::Intrinsic);

        let mut queue = EffectQueue::default();
        let attacker = Entity::from_bits(1);
        let target = Entity::from_bits(2);

        trigger_on_attack_traits(
            attacker,
            target,
            &collection,
            &registry,
            &handlers,
            &mut queue,
        );

        assert!(queue.pending.is_empty());
    }

    #[test]
    fn 多个on_attack_trait_全部触发() {
        let mut registry = TraitRegistry::default();
        registry.traits.insert(
            "attack_buff_a".into(),
            TraitData {
                id: "attack_buff_a".into(),
                name: "A".into(),
                description: String::new(),
                trigger: TraitTrigger::OnAttack,
                effects: vec![TraitEffect::ApplyBuff {
                    buff_id: "attack_up".into(),
                    duration: 1,
                }],
            },
        );
        registry.traits.insert(
            "attack_buff_b".into(),
            TraitData {
                id: "attack_buff_b".into(),
                name: "B".into(),
                description: String::new(),
                trigger: TraitTrigger::OnAttack,
                effects: vec![TraitEffect::ApplyBuff {
                    buff_id: "crit_up".into(),
                    duration: 2,
                }],
            },
        );

        let handlers = TraitEffectHandlerRegistry::with_defaults();
        let mut collection = TraitCollection::default();
        collection.add_entry("attack_buff_a".into(), TraitSource::Intrinsic);
        collection.add_entry(
            "attack_buff_b".into(),
            TraitSource::Equipment {
                slot: crate::equipment::EquipmentSlot::MainHand,
            },
        );

        let mut queue = EffectQueue::default();
        let attacker = Entity::from_bits(1);
        let target = Entity::from_bits(2);

        trigger_on_attack_traits(
            attacker,
            target,
            &collection,
            &registry,
            &handlers,
            &mut queue,
        );

        assert_eq!(queue.pending.len(), 2);
    }
}
