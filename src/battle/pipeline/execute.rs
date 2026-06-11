// 步骤 3：执行效果（纯逻辑：扣血/加 Buff/击杀判定）
// 表现层（VFX/日志）通过 Message 响应，不在此处调用
// 规则7：通过 EffectHandler trait 分发，禁止 match 分发

use crate::battle::{DamageApplied, HealApplied};
use crate::character::{TraitCollection, TraitEffectHandlerRegistry, TraitRegistry};
use crate::core::effect::{
    EffectHandlerRegistry, EffectQueue, ExecuteContext, ExecuteOutput, PendingEffectData,
    PendingMessage,
};
use crate::core::modifier_rule::ModifierRuleRegistry;
use crate::core::tag::GameplayTags;
use bevy::prelude::*;

use super::trait_trigger::{trigger_on_hit_traits, trigger_on_kill_traits};

/// 执行效果（系统入口，通过 EffectHandlerRegistry trait 分发）
/// 规则4：OnHit/OnKill 在 Execute 阶段触发
///
/// 使用 &mut World 模式，因为 EffectHandler trait 的 execute 方法需要 ExecuteContext
/// （持有 &mut World），而 handler 引用来自 EffectHandlerRegistry（也存储在 World 中）。
/// 通过 resource_scope 临时取出 Registry，避免同时可变/不可变借用 World 的冲突。
pub fn execute_effects(world: &mut World) {
    // 1. 收集效果数据
    let effects: Vec<_> = {
        let mut queue = world.resource_mut::<EffectQueue>();
        queue.pending.drain(..).collect()
    };

    // 收集执行结果（用于 OnHit/OnKill 触发）
    let mut results: Vec<ExecuteOutput> = Vec::new();
    // 收集所有待发送的消息
    let mut all_pending_messages: Vec<PendingMessage> = Vec::new();
    // 收集需要插入 Dead Tag 的实体
    let mut all_dead_entities: Vec<Entity> = Vec::new();

    // 2. 执行每个效果
    // 使用 resource_scope 临时取出 EffectHandlerRegistry，
    // 避免同时 &World（handler 查找）和 &mut World（ExecuteContext）的借用冲突
    world.resource_scope(|world, registry: Mut<EffectHandlerRegistry>| {
        for effect in effects {
            let type_name = effect.data.type_name();

            // 规则7：通过 EffectHandlerRegistry 查找 trait 对象分发，禁止 match 分发
            let Some(handler) = registry.find(type_name) else {
                bevy::log::warn!(
                    target: "battle",
                    effect_type = %type_name,
                    "未注册的效果处理器，跳过效果执行"
                );
                continue;
            };

            // 创建 ExecuteContext 并执行
            let mut ctx = ExecuteContext::new(world);
            if let Some(output) = handler.execute(&effect, &mut ctx) {
                bevy::log::trace!(
                    target: "battle",
                    effect_type = %type_name,
                    source = ?effect.source,
                    target = ?effect.target,
                    target_died = output.target_died,
                    "效果执行完成"
                );
                results.push(output);
            }
            all_pending_messages.extend(ctx.pending_messages);
            all_dead_entities.extend(ctx.dead_entities);
        }
    });

    // 3. 规则4：OnHit/OnKill 在 Execute 阶段触发
    // 使用 resource_scope 获取 TraitRegistry，避免与 resource_mut 借用冲突
    world.resource_scope(|world, trait_registry: Mut<TraitRegistry>| {
        world.resource_scope(
            |world, trait_effect_handlers: Mut<TraitEffectHandlerRegistry>| {
                for output in &results {
                    // OnHit：目标触发
                    let target_traits = world.get::<TraitCollection>(output.target).cloned();
                    if let Some(target_traits) = target_traits {
                        let mut queue = world.resource_mut::<EffectQueue>();
                        trigger_on_hit_traits(
                            output.target,
                            output.source,
                            &target_traits,
                            &trait_registry,
                            &trait_effect_handlers,
                            &mut queue,
                        );
                    }

                    // OnKill：攻击者击杀时触发
                    if output.target_died {
                        let killer_traits = world.get::<TraitCollection>(output.source).cloned();
                        if let Some(killer_traits) = killer_traits {
                            let mut queue = world.resource_mut::<EffectQueue>();
                            trigger_on_kill_traits(
                                output.source,
                                output.target,
                                &killer_traits,
                                &trait_registry,
                                &trait_effect_handlers,
                                &mut queue,
                            );
                        }
                    }
                }
            },
        );
    });

    // 4. 不变量1：OnHit/OnKill 产生的效果也必须经过 Modify → Execute
    // 先 drain，再 Modify，再 Execute，保证管线严格顺序
    let mut trait_effects: Vec<_> = {
        let mut queue = world.resource_mut::<EffectQueue>();
        queue.pending.drain(..).collect()
    };

    // 对 Trait 效果应用 Modify（与 modify_effects 相同逻辑）
    {
        let rules = world.resource::<ModifierRuleRegistry>();
        for effect in &mut trait_effects {
            if let Some(target_tags) = world.get::<GameplayTags>(effect.target) {
                match &mut effect.data {
                    PendingEffectData::Damage {
                        amount,
                        base_amount,
                        modifiers,
                        ..
                    } => {
                        if base_amount.is_none() {
                            *base_amount = Some(*amount);
                        }
                        let (new_amount, entries) = rules.apply_damage_modifiers_with_breakdown(
                            *amount,
                            &effect.source_tags,
                            target_tags,
                        );
                        *amount = new_amount.max(1);
                        *modifiers = entries;
                    }
                    PendingEffectData::Heal {
                        amount,
                        base_amount,
                        modifiers,
                    } => {
                        if base_amount.is_none() {
                            *base_amount = Some(*amount);
                        }
                        // 规则4：每步修饰必须记录
                        let (new_amount, entries) = rules.apply_heal_modifiers_with_breakdown(
                            *amount,
                            &effect.source_tags,
                            target_tags,
                        );
                        *amount = new_amount;
                        *modifiers = entries;
                    }
                    PendingEffectData::ApplyBuff { .. } | PendingEffectData::Cleanse => {}
                }
            }
        }
    }

    // 执行 Modify 后的 Trait 效果
    world.resource_scope(|world, registry: Mut<EffectHandlerRegistry>| {
        for effect in trait_effects {
            let type_name = effect.data.type_name();
            let Some(handler) = registry.find(type_name) else {
                continue;
            };
            let mut ctx = ExecuteContext::new(world);
            handler.execute(&effect, &mut ctx);
            all_pending_messages.extend(ctx.pending_messages);
            all_dead_entities.extend(ctx.dead_entities);
        }
    });

    // 5. 延迟插入 Dead Tag（避免在 Query 借用期间插入组件）
    for entity in all_dead_entities {
        world.entity_mut(entity).insert(crate::character::Dead);
    }

    // 6. 发送所有收集的消息
    for msg in all_pending_messages {
        match msg {
            PendingMessage::Damage(d) => {
                world
                    .resource_mut::<bevy::ecs::message::Messages<DamageApplied>>()
                    .write(d);
            }
            PendingMessage::Heal(h) => {
                world
                    .resource_mut::<bevy::ecs::message::Messages<HealApplied>>()
                    .write(h);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buff::{ActiveBuffs, BuffRegistry};
    use crate::character::{Dead, Faction, GridPosition, Unit, UnitName};
    use crate::core::attribute::{AttributeKind, Attributes};
    use crate::core::effect::{EffectQueue, PendingEffect, PendingEffectData};
    use crate::core::tag::GameplayTags;
    use crate::map::TerrainRegistry;
    use crate::skill::SkillSlots;

    fn test_buff_registry() -> BuffRegistry {
        let mut reg = BuffRegistry::default();
        reg.register_defaults();
        reg
    }

    fn test_terrain_registry() -> TerrainRegistry {
        let mut reg = TerrainRegistry::default();
        reg.register_defaults();
        reg
    }

    fn make_test_attrs(hp: f32, max_hp: f32) -> Attributes {
        let mut attrs = Attributes::default();
        attrs.set_base(AttributeKind::Vitality, ((max_hp - 5.0) / 5.0).max(0.0));
        attrs.fill_vital_resources();
        attrs.set_vital(AttributeKind::Hp, hp);
        attrs
    }

    #[test]
    fn apply_damage_effect_扣血() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<DamageApplied>()
            .add_message::<HealApplied>()
            .insert_resource(test_buff_registry())
            .insert_resource(test_terrain_registry())
            .insert_resource(EffectQueue::default())
            .insert_resource(EffectHandlerRegistry::default())
            .insert_resource(TraitRegistry::default())
            .insert_resource(TraitEffectHandlerRegistry::with_defaults())
            .add_systems(Update, execute_effects);
        let target = app
            .world_mut()
            .spawn((
                Unit {
                    faction: Faction::Enemy,
                    acted: false,
                },
                make_test_attrs(30.0, 30.0),
                SkillSlots::default(),
                ActiveBuffs::default(),
                GameplayTags::default(),
                GridPosition { coord: IVec2::ZERO },
                UnitName("哥布林".into()),
            ))
            .id();
        let source = app
            .world_mut()
            .spawn((
                Unit {
                    faction: Faction::Player,
                    acted: false,
                },
                UnitName("战士".into()),
            ))
            .id();
        let mut queue = app.world_mut().resource_mut::<EffectQueue>();
        queue.pending.push(PendingEffect {
            source,
            target,
            data: PendingEffectData::Damage {
                amount: 10,
                is_skill: false,
                base_amount: Some(10),
                modifiers: Vec::new(),
            },
            source_tags: vec![],
            terrain_id: "plain".into(),
        });
        app.update();
        let attrs = app.world().get::<Attributes>(target).unwrap();
        assert_eq!(attrs.get(AttributeKind::Hp), 20.0);
    }

    #[test]
    fn apply_damage_effect_致死添加dead标记() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<DamageApplied>()
            .add_message::<HealApplied>()
            .insert_resource(test_buff_registry())
            .insert_resource(test_terrain_registry())
            .insert_resource(EffectQueue::default())
            .insert_resource(EffectHandlerRegistry::default())
            .insert_resource(TraitRegistry::default())
            .insert_resource(TraitEffectHandlerRegistry::with_defaults())
            .add_systems(Update, execute_effects);
        let target = app
            .world_mut()
            .spawn((
                Unit {
                    faction: Faction::Enemy,
                    acted: false,
                },
                make_test_attrs(5.0, 30.0),
                SkillSlots::default(),
                ActiveBuffs::default(),
                GameplayTags::default(),
                GridPosition { coord: IVec2::ZERO },
                UnitName("哥布林".into()),
            ))
            .id();
        let source = app
            .world_mut()
            .spawn((
                Unit {
                    faction: Faction::Player,
                    acted: false,
                },
                UnitName("战士".into()),
            ))
            .id();
        let mut queue = app.world_mut().resource_mut::<EffectQueue>();
        queue.pending.push(PendingEffect {
            source,
            target,
            data: PendingEffectData::Damage {
                amount: 10,
                is_skill: false,
                base_amount: Some(10),
                modifiers: Vec::new(),
            },
            source_tags: vec![],
            terrain_id: "plain".into(),
        });
        app.update();
        assert!(app.world().get::<Dead>(target).is_some());
    }

    #[test]
    fn apply_heal_effect_回血() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<DamageApplied>()
            .add_message::<HealApplied>()
            .insert_resource(test_buff_registry())
            .insert_resource(test_terrain_registry())
            .insert_resource(EffectQueue::default())
            .insert_resource(EffectHandlerRegistry::default())
            .insert_resource(TraitRegistry::default())
            .insert_resource(TraitEffectHandlerRegistry::with_defaults())
            .add_systems(Update, execute_effects);
        let target = app
            .world_mut()
            .spawn((
                Unit {
                    faction: Faction::Player,
                    acted: false,
                },
                make_test_attrs(10.0, 30.0),
                SkillSlots::default(),
                ActiveBuffs::default(),
                GameplayTags::default(),
                GridPosition { coord: IVec2::ZERO },
                UnitName("战士".into()),
            ))
            .id();
        let source = app
            .world_mut()
            .spawn((
                Unit {
                    faction: Faction::Player,
                    acted: false,
                },
                UnitName("牧师".into()),
            ))
            .id();
        let mut queue = app.world_mut().resource_mut::<EffectQueue>();
        queue.pending.push(PendingEffect {
            source,
            target,
            data: PendingEffectData::Heal {
                amount: 15,
                base_amount: Some(15),
                modifiers: Vec::new(),
            },
            source_tags: vec![],
            terrain_id: "plain".into(),
        });
        app.update();
        let attrs = app.world().get::<Attributes>(target).unwrap();
        assert_eq!(attrs.get(AttributeKind::Hp), 25.0);
    }

    #[test]
    fn apply_heal_effect_不超过maxhp() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<DamageApplied>()
            .add_message::<HealApplied>()
            .insert_resource(test_buff_registry())
            .insert_resource(test_terrain_registry())
            .insert_resource(EffectQueue::default())
            .insert_resource(EffectHandlerRegistry::default())
            .insert_resource(TraitRegistry::default())
            .insert_resource(TraitEffectHandlerRegistry::with_defaults())
            .add_systems(Update, execute_effects);
        let target = app
            .world_mut()
            .spawn((
                Unit {
                    faction: Faction::Player,
                    acted: false,
                },
                make_test_attrs(25.0, 30.0),
                SkillSlots::default(),
                ActiveBuffs::default(),
                GameplayTags::default(),
                GridPosition { coord: IVec2::ZERO },
                UnitName("战士".into()),
            ))
            .id();
        let source = app
            .world_mut()
            .spawn((
                Unit {
                    faction: Faction::Player,
                    acted: false,
                },
                UnitName("牧师".into()),
            ))
            .id();
        let mut queue = app.world_mut().resource_mut::<EffectQueue>();
        queue.pending.push(PendingEffect {
            source,
            target,
            data: PendingEffectData::Heal {
                amount: 100,
                base_amount: Some(100),
                modifiers: Vec::new(),
            },
            source_tags: vec![],
            terrain_id: "plain".into(),
        });
        app.update();
        let attrs = app.world().get::<Attributes>(target).unwrap();
        assert_eq!(attrs.get(AttributeKind::Hp), 30.0);
    }

    #[test]
    fn apply_buff_effect_正常施加() {
        let mut buffs = ActiveBuffs::default();
        let mut attrs = Attributes::default();
        attrs.fill_vital_resources();
        let mut tags = GameplayTags::default();
        let registry = test_buff_registry();
        if let Some(buff_data) = registry.get("attack_up") {
            crate::buff::apply_buff(
                &mut buffs,
                &mut attrs,
                &mut tags,
                buff_data,
                Some(Entity::from_bits(1)),
                3,
            );
        }
        assert!(buffs.iter().any(|b| b.name == "攻+5"));
    }

    #[test]
    fn apply_buff_effect_未知buff静默跳过() {
        let mut buffs = ActiveBuffs::default();
        let mut attrs = Attributes::default();
        attrs.fill_vital_resources();
        let mut tags = GameplayTags::default();
        let registry = test_buff_registry();
        if let Some(buff_data) = registry.get("nonexistent_buff") {
            crate::buff::apply_buff(
                &mut buffs,
                &mut attrs,
                &mut tags,
                buff_data,
                Some(Entity::from_bits(1)),
                3,
            );
        }
        assert_eq!(buffs.iter().count(), 0);
    }
}
