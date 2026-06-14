use crate::core::attribute::BuffInstanceId;
use crate::core::buff::domain::DurationPolicy;
use crate::core::tag::GameplayTag;
use bevy::prelude::*;

/// 运行时 Buff 实例
#[derive(Clone, Debug, Reflect)]
pub struct BuffInstance {
    pub instance_id: BuffInstanceId,
    pub buff_id: String,
    pub name: String,
    pub remaining_turns: u32,
    /// 新字段（ADR-021）：持续策略
    pub duration_policy: DurationPolicy,
    pub source_entity: Option<Entity>,
    pub tags: Vec<GameplayTag>,
    pub is_buff: bool,
    pub dot_damage: i32,
    pub hot_heal: i32,
}

impl BuffInstance {
    pub fn is_debuff(&self) -> bool {
        !self.is_buff
    }

    pub fn label(&self) -> String {
        self.name.clone()
    }
}

/// 活跃 Buff 列表组件（替代原 StatusEffects）
#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct ActiveBuffs {
    pub instances: Vec<BuffInstance>,
    next_id: u64,
}

impl ActiveBuffs {
    /// 添加 Buff 实例，返回实例 ID
    pub fn add(&mut self, buff: BuffInstance) -> BuffInstanceId {
        let id = buff.instance_id;
        // 同源同 buff_id 刷新持续时间
        if let Some(source) = buff.source_entity {
            if let Some(existing) = self
                .instances
                .iter_mut()
                .find(|b| b.source_entity == Some(source) && b.buff_id == buff.buff_id)
            {
                existing.remaining_turns = buff.remaining_turns;
                return existing.instance_id;
            }
        }
        self.instances.push(buff);
        id
    }

    /// 生成下一个唯一 ID
    pub fn next_instance_id(&mut self) -> BuffInstanceId {
        self.next_id += 1;
        BuffInstanceId(self.next_id)
    }

    /// 移除指定实例 ID 的 Buff
    pub fn remove(&mut self, instance_id: BuffInstanceId) -> Option<BuffInstance> {
        if let Some(idx) = self
            .instances
            .iter()
            .position(|b| b.instance_id == instance_id)
        {
            Some(self.instances.remove(idx))
        } else {
            None
        }
    }

    /// 每回合结算：先移除已过期的，再递减剩余持续时间
    pub fn tick(&mut self) {
        self.instances.retain(|inst| inst.remaining_turns > 0);
        for inst in &mut self.instances {
            // 只有 DurationPolicy::Turns 才递减
            if matches!(inst.duration_policy, DurationPolicy::Turns(_)) {
                inst.remaining_turns -= 1;
            }
        }
    }

    pub fn is_stunned(&self) -> bool {
        self.instances
            .iter()
            .any(|b| b.remaining_turns > 0 && b.tags.contains(&GameplayTag::STUN))
    }

    /// 消耗晕眩：移除所有带 STUN 标签的 Buff，返回是否原本处于晕眩
    pub fn consume_stun(&mut self) -> bool {
        let was = self.is_stunned();
        self.instances
            .retain(|b| !b.tags.contains(&GameplayTag::STUN));
        was
    }

    pub fn dot_damage(&self) -> i32 {
        self.instances
            .iter()
            .filter(|b| b.remaining_turns > 0)
            .map(|b| b.dot_damage)
            .sum()
    }

    pub fn hot_heal(&self) -> i32 {
        self.instances
            .iter()
            .filter(|b| b.remaining_turns > 0)
            .map(|b| b.hot_heal)
            .sum()
    }

    /// 移除所有 Debuff
    pub fn remove_debuffs(&mut self) {
        self.instances.retain(|b| b.is_buff);
    }

    pub fn len(&self) -> usize {
        self.instances.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, BuffInstance> {
        self.instances.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // ── ActiveBuffs 基础操作 ──

    #[test]
    fn 活跃buff_添加和查询() {
        let mut buffs = ActiveBuffs::default();
        let id = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id,
            buff_id: "attack_up".into(),
            name: "攻+5".into(),
            remaining_turns: 2,
            duration_policy: DurationPolicy::Turns(2),
            source_entity: None,
            tags: vec![GameplayTag::BUFF],
            is_buff: true,
            dot_damage: 0,
            hot_heal: 0,
        });
        assert_eq!(buffs.len(), 1);
        assert!(!buffs.is_empty());
    }

    #[test]
    fn 活跃buff_移除() {
        let mut buffs = ActiveBuffs::default();
        let id = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id,
            buff_id: "attack_up".into(),
            name: "攻+5".into(),
            remaining_turns: 2,
            duration_policy: DurationPolicy::Turns(2),
            source_entity: None,
            tags: vec![GameplayTag::BUFF],
            is_buff: true,
            dot_damage: 0,
            hot_heal: 0,
        });
        let removed = buffs.remove(id);
        assert!(removed.is_some());
        assert_eq!(buffs.len(), 0);
        assert!(buffs.is_empty());
    }

    #[test]
    fn 活跃buff_移除不存在的返回none() {
        let mut buffs = ActiveBuffs::default();
        let result = buffs.remove(BuffInstanceId(999));
        assert!(result.is_none());
    }

    #[test]
    fn 活跃buff_同源同id刷新持续时间() {
        let mut buffs = ActiveBuffs::default();
        let source = Entity::from_bits(42);

        let id1 = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id1,
            buff_id: "poison".into(),
            name: "毒".into(),
            remaining_turns: 1,
            duration_policy: DurationPolicy::Turns(1),
            source_entity: Some(source),
            tags: vec![GameplayTag::DEBUFF],
            is_buff: false,
            dot_damage: 3,
            hot_heal: 0,
        });

        let id2 = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id2,
            buff_id: "poison".into(),
            name: "毒".into(),
            remaining_turns: 3,
            duration_policy: DurationPolicy::Turns(3),
            source_entity: Some(source),
            tags: vec![GameplayTag::DEBUFF],
            is_buff: false,
            dot_damage: 3,
            hot_heal: 0,
        });

        assert_eq!(buffs.len(), 1);
        assert_eq!(buffs.instances[0].remaining_turns, 3);
        assert_eq!(buffs.instances[0].instance_id, id1);
    }

    #[test]
    fn 活跃buff_不同源同id不刷新() {
        let mut buffs = ActiveBuffs::default();
        let source_a = Entity::from_bits(1);
        let source_b = Entity::from_bits(2);

        let id1 = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id1,
            buff_id: "poison".into(),
            name: "毒".into(),
            remaining_turns: 1,
            duration_policy: DurationPolicy::Turns(1),
            source_entity: Some(source_a),
            tags: vec![GameplayTag::DEBUFF],
            is_buff: false,
            dot_damage: 3,
            hot_heal: 0,
        });

        let id2 = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id2,
            buff_id: "poison".into(),
            name: "毒".into(),
            remaining_turns: 3,
            duration_policy: DurationPolicy::Turns(3),
            source_entity: Some(source_b),
            tags: vec![GameplayTag::DEBUFF],
            is_buff: false,
            dot_damage: 3,
            hot_heal: 0,
        });

        assert_eq!(buffs.len(), 2);
    }

    // ── Tick ──

    #[test]
    fn 活跃buff_tick_递减持续时间() {
        let mut buffs = ActiveBuffs::default();
        let id = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id,
            buff_id: "attack_up".into(),
            name: "攻+5".into(),
            remaining_turns: 2,
            duration_policy: DurationPolicy::Turns(2),
            source_entity: None,
            tags: vec![GameplayTag::BUFF],
            is_buff: true,
            dot_damage: 0,
            hot_heal: 0,
        });
        buffs.tick();
        assert_eq!(buffs.instances[0].remaining_turns, 1);
    }

    #[test]
    fn 活跃buff_tick_递减后过期() {
        let mut buffs = ActiveBuffs::default();
        let id = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id,
            buff_id: "attack_up".into(),
            name: "攻+5".into(),
            remaining_turns: 1,
            duration_policy: DurationPolicy::Turns(1),
            source_entity: None,
            tags: vec![GameplayTag::BUFF],
            is_buff: true,
            dot_damage: 0,
            hot_heal: 0,
        });
        buffs.tick();
        assert_eq!(buffs.len(), 1);
        assert_eq!(buffs.instances[0].remaining_turns, 0);
        buffs.tick();
        assert!(buffs.is_empty());
    }

    #[test]
    fn 活跃buff_tick_非turns策略不递减() {
        let mut buffs = ActiveBuffs::default();
        let id = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id,
            buff_id: "permanent_buff".into(),
            name: "永久".into(),
            remaining_turns: 5,
            duration_policy: DurationPolicy::Permanent,
            source_entity: None,
            tags: vec![GameplayTag::BUFF],
            is_buff: true,
            dot_damage: 0,
            hot_heal: 0,
        });
        buffs.tick();
        // Permanent buff 不应递减
        assert_eq!(buffs.instances[0].remaining_turns, 5);
    }

    // ── 晕眩 ──

    #[test]
    fn 活跃buff_晕眩检测() {
        let mut buffs = ActiveBuffs::default();
        assert!(!buffs.is_stunned());

        let id = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id,
            buff_id: "stun".into(),
            name: "晕".into(),
            remaining_turns: 1,
            duration_policy: DurationPolicy::Turns(1),
            source_entity: None,
            tags: vec![GameplayTag::DEBUFF, GameplayTag::STUN],
            is_buff: false,
            dot_damage: 0,
            hot_heal: 0,
        });
        assert!(buffs.is_stunned());
    }

    #[test]
    fn 活跃buff_消耗晕眩() {
        let mut buffs = ActiveBuffs::default();
        let id = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id,
            buff_id: "stun".into(),
            name: "晕".into(),
            remaining_turns: 1,
            duration_policy: DurationPolicy::Turns(1),
            source_entity: None,
            tags: vec![GameplayTag::DEBUFF, GameplayTag::STUN],
            is_buff: false,
            dot_damage: 0,
            hot_heal: 0,
        });
        let was_stunned = buffs.consume_stun();
        assert!(was_stunned);
        assert!(!buffs.is_stunned());
        assert!(buffs.is_empty());
    }

    // ── DoT / HoT ──

    #[test]
    fn 活跃buff_dot_hot汇总() {
        let mut buffs = ActiveBuffs::default();
        let id1 = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id1,
            buff_id: "poison".into(),
            name: "毒".into(),
            remaining_turns: 2,
            duration_policy: DurationPolicy::Turns(2),
            source_entity: None,
            tags: vec![GameplayTag::DEBUFF],
            is_buff: false,
            dot_damage: 3,
            hot_heal: 0,
        });
        let id2 = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id2,
            buff_id: "regen".into(),
            name: "愈".into(),
            remaining_turns: 2,
            duration_policy: DurationPolicy::Turns(2),
            source_entity: None,
            tags: vec![GameplayTag::BUFF],
            is_buff: true,
            dot_damage: 0,
            hot_heal: 4,
        });
        assert_eq!(buffs.dot_damage(), 3);
        assert_eq!(buffs.hot_heal(), 4);
    }

    // ── 移除所有 Debuff ──

    #[test]
    fn 活跃buff_移除所有debuff() {
        let mut buffs = ActiveBuffs::default();
        let id1 = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id1,
            buff_id: "attack_up".into(),
            name: "攻+5".into(),
            remaining_turns: 2,
            duration_policy: DurationPolicy::Turns(2),
            source_entity: None,
            tags: vec![GameplayTag::BUFF],
            is_buff: true,
            dot_damage: 0,
            hot_heal: 0,
        });
        let id2 = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id2,
            buff_id: "poison".into(),
            name: "毒".into(),
            remaining_turns: 2,
            duration_policy: DurationPolicy::Turns(2),
            source_entity: None,
            tags: vec![GameplayTag::DEBUFF],
            is_buff: false,
            dot_damage: 3,
            hot_heal: 0,
        });
        buffs.remove_debuffs();
        assert_eq!(buffs.len(), 1);
        assert_eq!(buffs.instances[0].buff_id, "attack_up");
    }
}
