// 战斗事件 Message：逻辑层通知，表现层响应
// 遵循「Logic 发消息，Presentation 响应」原则

use crate::character::{Faction, GridPosition, Unit, UnitName};
use crate::turn::TurnOrder;
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;

use super::record::DamageBreakdown;

/// 角色死亡消息
#[derive(Message, Debug, Clone)]
pub struct CharacterDied {
    pub entity: Entity,
    /// 死亡单位名称
    pub name: String,
    /// 死亡单位阵营
    pub faction: Faction,
}

/// 伤害应用消息：逻辑层发送，VFX/日志/表现层响应
#[derive(Message, Debug, Clone)]
pub struct DamageApplied {
    pub target: Entity,
    /// 伤害目标名称
    pub target_name: String,
    /// 目标阵营
    pub target_faction: Faction,
    /// 攻击者实体
    pub attacker: Entity,
    /// 攻击者名称
    pub attacker_name: String,
    /// 攻击者阵营
    pub attacker_faction: Faction,
    /// 伤害量
    pub amount: i32,
    /// 是否技能攻击
    pub is_skill: bool,
    /// 地形标签
    pub terrain_label: String,
    /// 目标格子坐标
    pub target_coord: IVec2,
    /// 伤害分解（可选）
    pub breakdown: Option<DamageBreakdown>,
}

/// 治疗应用消息
#[derive(Message, Debug, Clone)]
pub struct HealApplied {
    pub target: Entity,
    /// 目标名称
    pub target_name: String,
    /// 治疗量
    pub amount: i32,
}

/// 晕眩消息
#[derive(Message, Debug, Clone)]
pub struct StunApplied {
    pub target: Entity,
    pub target_name: String,
}

/// DoT 伤害消息
#[derive(Message, Debug, Clone)]
pub struct DotApplied {
    pub target: Entity,
    pub target_name: String,
    pub amount: i32,
    pub target_coord: IVec2,
}

/// HoT 治疗消息
#[derive(Message, Debug, Clone)]
pub struct HotApplied {
    pub target: Entity,
    pub target_name: String,
    pub amount: i32,
}

/// 响应 CharacterDied 消息：从行动队列移除死亡单位，despawn 实体
pub fn on_character_died(
    mut commands: Commands,
    mut died_reader: MessageReader<CharacterDied>,
    mut turn_order: ResMut<TurnOrder>,
    unit_ids: Query<&crate::character::UnitId>,
) {
    for msg in died_reader.read() {
        let unit_id = unit_ids.get(msg.entity).map(|id| id.0.as_str()).unwrap_or("?");
        bevy::log::info!(target: "battle", entity = ?msg.entity, unit_id = %unit_id, name = %msg.name, faction = ?msg.faction, "角色已死亡，从行动队列移除");
        // 找到被移除实体的位置，修正 current_index
        if let Some(pos) = turn_order.queue.iter().position(|&e| e == msg.entity) {
            turn_order.queue.remove(pos);
            // 如果被移除的实体在 current_index 之前，current_index 需要减 1
            // 因为 remove 会导致后面的元素前移
            if pos < turn_order.current_index {
                turn_order.current_index -= 1;
            }
            // 防御性：确保 current_index 不越界
            turn_order.current_index = turn_order.current_index.min(turn_order.queue.len());
        }
        commands.entity(msg.entity).try_despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::Faction;
    use crate::turn::TurnOrder;
    use bevy::prelude::*;

    #[test]
    fn character_died_消息字段() {
        let msg = CharacterDied {
            entity: Entity::from_bits(42),
            name: "哥布林".to_string(),
            faction: Faction::Enemy,
        };
        assert_eq!(msg.entity, Entity::from_bits(42));
        assert_eq!(msg.name, "哥布林");
    }

    #[test]
    fn damage_applied_消息字段() {
        let msg = DamageApplied {
            target: Entity::from_bits(1),
            target_name: "哥布林".to_string(),
            target_faction: Faction::Enemy,
            attacker: Entity::from_bits(2),
            attacker_name: "战士".to_string(),
            attacker_faction: Faction::Player,
            amount: 15,
            is_skill: false,
            terrain_label: "平原".to_string(),
            target_coord: IVec2::new(3, 4),
            breakdown: None,
        };
        assert_eq!(msg.amount, 15);
        assert_eq!(msg.attacker, Entity::from_bits(2));
    }

    #[test]
    fn heal_applied_消息字段() {
        let msg = HealApplied {
            target: Entity::from_bits(1),
            target_name: "战士".to_string(),
            amount: 10,
        };
        assert_eq!(msg.amount, 10);
    }

    #[test]
    fn dot_applied_消息字段() {
        let msg = DotApplied {
            target: Entity::from_bits(1),
            target_name: "战士".to_string(),
            amount: 5,
            target_coord: IVec2::new(2, 3),
        };
        assert_eq!(msg.amount, 5);
    }

    #[test]
    fn hot_applied_消息字段() {
        let msg = HotApplied {
            target: Entity::from_bits(1),
            target_name: "战士".to_string(),
            amount: 8,
        };
        assert_eq!(msg.amount, 8);
    }

    #[test]
    fn stun_applied_消息字段() {
        let msg = StunApplied {
            target: Entity::from_bits(1),
            target_name: "战士".to_string(),
        };
        assert_eq!(msg.target, Entity::from_bits(1));
    }

    #[test]
    fn on_character_died_从行动队列移除() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TurnOrder {
                queue: vec![],
                current_index: 0,
                turn_number: 1,
            })
            .add_message::<CharacterDied>()
            .add_systems(Update, on_character_died);
        let e1 = app.world_mut().spawn_empty().id();
        let e2 = app.world_mut().spawn_empty().id();
        {
            let mut turn_order = app.world_mut().resource_mut::<TurnOrder>();
            turn_order.queue = vec![e1, e2];
        }
        app.world_mut().write_message(CharacterDied {
            entity: e1,
            name: "战士".to_string(),
            faction: Faction::Player,
        });
        app.update();
        let turn_order = app.world().resource::<TurnOrder>();
        assert_eq!(turn_order.queue, vec![e2]);
    }

    #[test]
    fn on_character_died_队列中无死亡单位则不变() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TurnOrder {
                queue: vec![],
                current_index: 0,
                turn_number: 1,
            })
            .add_message::<CharacterDied>()
            .add_systems(Update, on_character_died);
        let e1 = app.world_mut().spawn_empty().id();
        {
            let mut turn_order = app.world_mut().resource_mut::<TurnOrder>();
            turn_order.queue = vec![e1];
        }
        app.update();
        let turn_order = app.world().resource::<TurnOrder>();
        assert_eq!(turn_order.queue, vec![e1]);
    }

    #[test]
    fn on_character_died_多次死亡消息按序移除() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TurnOrder {
                queue: vec![],
                current_index: 0,
                turn_number: 1,
            })
            .add_message::<CharacterDied>()
            .add_systems(Update, on_character_died);
        let e1 = app.world_mut().spawn_empty().id();
        let e2 = app.world_mut().spawn_empty().id();
        let e3 = app.world_mut().spawn_empty().id();
        {
            let mut turn_order = app.world_mut().resource_mut::<TurnOrder>();
            turn_order.queue = vec![e1, e2, e3];
        }
        app.world_mut().write_message(CharacterDied {
            entity: e1,
            name: "战士".to_string(),
            faction: Faction::Player,
        });
        app.world_mut().write_message(CharacterDied {
            entity: e3,
            name: "法师".to_string(),
            faction: Faction::Player,
        });
        app.update();
        let turn_order = app.world().resource::<TurnOrder>();
        assert_eq!(turn_order.queue, vec![e2]);
    }
}
