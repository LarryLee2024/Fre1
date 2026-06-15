//! GAS 链集成测试
//!
//! 验证 ADR-026 GAS 执行链的完整流程：
//! Ability → Targeting → Effect → Stacking → Execution → Modifier → Attribute → Tag → Cue

#[cfg(test)]
mod tests {
    use crate::core::ability::*;
    use crate::core::attribute::*;
    use crate::core::buff::*;
    use crate::core::cue::*;
    use crate::core::effect::*;
    use crate::core::execution::*;
    use crate::core::modifier::*;
    use crate::core::stacking::*;
    use crate::core::tag::*;
    use crate::infrastructure::pipeline::*;
    use crate::infrastructure::registry::*;
    use crate::infrastructure::replay::*;
    use bevy::prelude::*;
    use std::collections::HashMap;

    /// 测试 StackingRule 4-enum 完整场景
    #[test]
    fn stacking_rule_complete_scenarios() {
        // 场景1: Replace - 替换旧实例
        let rule = StackingRule::Replace;
        let ctx = StackingContext {
            current_stacks: 3,
            rule,
        };
        let result = resolve_stacking(Some(&ctx), rule);
        assert_eq!(result, StackingResult::Replaced);

        // 场景2: RefreshDuration - 刷新持续时间
        let rule = StackingRule::RefreshDuration;
        let ctx = StackingContext {
            current_stacks: 1,
            rule,
        };
        let result = resolve_stacking(Some(&ctx), rule);
        assert_eq!(result, StackingResult::Refreshed);

        // 场景3: StackAdd - 叠加无上限
        let rule = StackingRule::StackAdd;
        let ctx = StackingContext {
            current_stacks: 100,
            rule,
        };
        let result = resolve_stacking(Some(&ctx), rule);
        assert_eq!(result, StackingResult::Stacked { new_count: 101 });

        // 场景4: StackMax - 叠加有上限
        let rule = StackingRule::StackMax(5);
        let ctx = StackingContext {
            current_stacks: 4,
            rule,
        };
        let result = resolve_stacking(Some(&ctx), rule);
        assert_eq!(result, StackingResult::Stacked { new_count: 5 });

        // 场景5: StackMax - 达到上限
        let rule = StackingRule::StackMax(5);
        let ctx = StackingContext {
            current_stacks: 5,
            rule,
        };
        let result = resolve_stacking(Some(&ctx), rule);
        assert_eq!(result, StackingResult::Ignored { max_reached: true });
    }

    /// 测试 Execution 注册表完整流程
    #[test]
    fn execution_registry_complete() {
        let registry = ExecutionRegistry::default();

        // 验证所有内置执行器已注册
        assert!(registry.get("Damage").is_some());
        assert!(registry.get("Heal").is_some());
        assert!(registry.get("Shield").is_some());

        // 验证未知执行器返回 None
        assert!(registry.get("Unknown").is_none());
    }

    /// 测试 Execution 计算完整流程
    #[test]
    fn execution_calculation_complete() {
        // 伤害计算
        let damage_executor = DamageExecution;
        let ctx = ExecutionContext {
            source_entity: Entity::from_bits(1),
            target_entity: Entity::from_bits(2),
            source_attrs: AttributeSnapshot {
                attack: 100.0,
                defense: 0.0,
                ..default()
            },
            target_attrs: AttributeSnapshot {
                attack: 0.0,
                defense: 20.0,
                ..default()
            },
            base_value: 1.0,
            modifier_value: 0,
            stack_count: 1,
            execution_params: HashMap::new(),
            terrain_id: None,
            is_skill: false,
        };
        let result = damage_executor.calculate(&ctx);
        assert_eq!(result.value, 80);

        // 治疗计算
        let heal_executor = HealExecution;
        let ctx = ExecutionContext {
            source_entity: Entity::from_bits(1),
            target_entity: Entity::from_bits(2),
            source_attrs: AttributeSnapshot {
                attack: 50.0,
                defense: 0.0,
                ..default()
            },
            target_attrs: AttributeSnapshot::default(),
            base_value: 0.0,
            modifier_value: 0,
            stack_count: 1,
            execution_params: HashMap::new(),
            terrain_id: None,
            is_skill: false,
        };
        let result = heal_executor.calculate(&ctx);
        assert_eq!(result.value, 50);

        // 护盾计算
        let shield_executor = ShieldExecution;
        let ctx = ExecutionContext {
            source_entity: Entity::from_bits(1),
            target_entity: Entity::from_bits(2),
            source_attrs: AttributeSnapshot {
                attack: 80.0,
                defense: 0.0,
                ..default()
            },
            target_attrs: AttributeSnapshot::default(),
            base_value: 0.0,
            modifier_value: 0,
            stack_count: 1,
            execution_params: HashMap::new(),
            terrain_id: None,
            is_skill: false,
        };
        let result = shield_executor.calculate(&ctx);
        assert_eq!(result.value, 80);
    }

    /// 测试 Cue 事件发射完整流程
    #[test]
    fn cue_emission_complete() {
        let mut emitter = CueEmitter::default();

        // 发射各种 CueEvent
        emitter.emit_damage(Entity::from_bits(1), 50, true, Some(Entity::from_bits(2)));
        emitter.emit_death(Entity::from_bits(1), Some(Entity::from_bits(2)));
        emitter.emit_heal(Entity::from_bits(2), 30, Some(Entity::from_bits(1)));
        emitter.emit_buff_apply(Entity::from_bits(2), "burn".to_string(), 1);
        emitter.emit_shield(Entity::from_bits(2), 20);

        assert!(emitter.has_pending());
        assert_eq!(emitter.damage_events.len(), 1);
        assert_eq!(emitter.death_events.len(), 1);
        assert_eq!(emitter.heal_events.len(), 1);
        assert_eq!(emitter.buff_apply_events.len(), 1);
        assert_eq!(emitter.shield_events.len(), 1);

        // 清空事件
        emitter.clear();
        assert!(!emitter.has_pending());
    }

    /// 测试 EffectDef DurationDef 完整场景
    #[test]
    fn effect_def_duration_complete() {
        use crate::core::effect::DurationDef;
        use crate::core::effect::StackingDef;

        // Instant 效果
        let def = EffectDef::ApplyModifier {
            modifier_id: "damage_boost".to_string(),
            duration: DurationDef::Instant,
            stacking: StackingDef::Replace,
        };
        assert_eq!(def.duration(), Some(DurationDef::Instant));
        assert_eq!(def.stacking(), Some(StackingDef::Replace));

        // TurnLimited 效果
        let def = EffectDef::ApplyModifier {
            modifier_id: "burn".to_string(),
            duration: DurationDef::TurnLimited(3),
            stacking: StackingDef::StackMax { max_stack: 5 },
        };
        assert_eq!(def.duration(), Some(DurationDef::TurnLimited(3)));
        assert_eq!(def.stacking(), Some(StackingDef::StackMax { max_stack: 5 }));

        // Permanent 效果
        let def = EffectDef::ApplyModifier {
            modifier_id: "passive_buff".to_string(),
            duration: DurationDef::Permanent,
            stacking: StackingDef::RefreshDuration,
        };
        assert_eq!(def.duration(), Some(DurationDef::Permanent));
        assert_eq!(def.stacking(), Some(StackingDef::RefreshDuration));
    }

    /// 测试 GasPhase 完整流程
    #[test]
    fn gas_phase_complete() {
        let phases = GasPhase::all();
        assert_eq!(phases.len(), 10);

        // 验证阶段顺序
        assert_eq!(phases[0], GasPhase::Ability);
        assert_eq!(phases[1], GasPhase::Targeting);
        assert_eq!(phases[2], GasPhase::Effect);
        assert_eq!(phases[3], GasPhase::Stacking);
        assert_eq!(phases[4], GasPhase::Execution);
        assert_eq!(phases[5], GasPhase::Modifier);
        assert_eq!(phases[6], GasPhase::Attribute);
        assert_eq!(phases[7], GasPhase::Tag);
        assert_eq!(phases[8], GasPhase::Cue);
        assert_eq!(phases[9], GasPhase::Replay);

        // 验证索引
        assert_eq!(GasPhase::Ability.index(), 0);
        assert_eq!(GasPhase::Replay.index(), 9);
    }

    /// 测试 Replay 完整流程
    #[test]
    fn replay_complete() {
        let record = BattleRecord {
            seed: 42,
            turn_count: 3,
            commands: vec![
                CommandEntry {
                    turn: 1,
                    command_type: CommandType::UseSkill,
                    caster: Entity::from_bits(1),
                    target: Some(Entity::from_bits(2)),
                    data: CommandData::UseSkill {
                        skill_id: "fireball".to_string(),
                    },
                },
                CommandEntry {
                    turn: 2,
                    command_type: CommandType::Move,
                    caster: Entity::from_bits(1),
                    target: None,
                    data: CommandData::Move {
                        path: vec![IVec2::new(0, 0), IVec2::new(1, 0)],
                    },
                },
                CommandEntry {
                    turn: 3,
                    command_type: CommandType::Wait,
                    caster: Entity::from_bits(2),
                    target: None,
                    data: CommandData::Wait,
                },
            ],
        };

        let mut player = ReplayPlayer::new(record);
        assert!(!player.is_finished());

        // 回放第一条指令
        let cmd = player.next_command().unwrap();
        assert_eq!(cmd.command_type, CommandType::UseSkill);
        assert_eq!(cmd.turn, 1);

        // 回放第二条指令
        let cmd = player.next_command().unwrap();
        assert_eq!(cmd.command_type, CommandType::Move);

        // 回放第三条指令
        let cmd = player.next_command().unwrap();
        assert_eq!(cmd.command_type, CommandType::Wait);

        // 播放完毕
        assert!(player.is_finished());
        assert!(player.next_command().is_none());

        // 重置
        player.reset();
        assert!(!player.is_finished());
    }

    /// 测试 AssetRegistry 完整流程
    #[test]
    fn asset_registry_complete() {
        let mut registry = AssetRegistry::default();

        // 注册各种资产
        registry.register_ability("fireball".to_string());
        registry.register_ability("icebolt".to_string());
        registry.register_effect("damage".to_string());
        registry.register_effect("heal".to_string());
        registry.register_execution("Damage".to_string());
        registry.register_tag("FIRE".to_string());

        // 验证注册
        assert!(registry.has_ability("fireball"));
        assert!(registry.has_ability("icebolt"));
        assert!(!registry.has_ability("thunderbolt"));

        assert!(registry.has_effect("damage"));
        assert!(registry.has_effect("heal"));
        assert!(!registry.has_effect("shield"));

        assert!(registry.has_execution("Damage"));
        assert!(!registry.has_execution("Unknown"));

        // 验证无重复
        registry.register_ability("fireball".to_string());
        assert_eq!(registry.abilities.len(), 2);
    }
}
