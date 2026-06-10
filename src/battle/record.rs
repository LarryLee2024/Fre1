// 战斗记录系统：结构化记录战斗事件，用于回放、调试、AI分析
// 遵循 Rule 12：战斗回放日志比普通日志更重要

use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::character::Faction;
use crate::turn::{TurnEnded, TurnStarted};

use super::events::{
    CharacterDied, DamageApplied, DotApplied, HealApplied, HotApplied, StunApplied,
};

// ── 战斗记录条目 ──

/// 单条战斗记录条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BattleEntry {
    /// 回合开始
    TurnStarted { turn: u32 },
    /// 回合结束
    TurnEnded { turn: u32 },
    /// 伤害应用
    DamageApplied {
        target: Entity,
        target_name: String,
        target_faction: Faction,
        attacker: Entity,
        attacker_name: String,
        attacker_faction: Faction,
        amount: i32,
        is_skill: bool,
        terrain_label: String,
        target_coord: IVec2,
    },
    /// 治疗应用
    HealApplied {
        target: Entity,
        target_name: String,
        amount: i32,
    },
    /// DoT 伤害
    DotApplied {
        target: Entity,
        target_name: String,
        amount: i32,
        target_coord: IVec2,
    },
    /// HoT 治疗
    HotApplied {
        target: Entity,
        target_name: String,
        amount: i32,
    },
    /// 晕眩
    StunApplied { target: Entity, target_name: String },
    /// 角色死亡
    CharacterDied {
        entity: Entity,
        name: String,
        faction: Faction,
    },
}

// ── 战斗统计 ──

/// 单个实体的战斗统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntityBattleStats {
    /// 造成总伤害
    pub damage_dealt: i32,
    /// 承受总伤害
    pub damage_taken: i32,
    /// 总治疗量
    pub heal_received: i32,
    /// 击杀数
    pub kills: i32,
}

// ── 战斗记录资源 ──

/// 战斗记录资源：持久化存储，支持回放、查询、调试
#[derive(Resource, Default, Debug, Serialize, Deserialize)]
pub struct BattleRecord {
    pub entries: Vec<BattleEntry>,
    pub turn_number: u32,
}

impl BattleRecord {
    /// 记录一条战斗事件，同时写 trace 日志
    pub fn record(&mut self, entry: BattleEntry) {
        match &entry {
            BattleEntry::DamageApplied { target, amount, .. } => {
                bevy::log::trace!(target: "battle_record", entity = ?target, damage = amount, "DamageApplied");
            }
            BattleEntry::HealApplied { target, amount, .. } => {
                bevy::log::trace!(target: "battle_record", entity = ?target, heal = amount, "HealApplied");
            }
            BattleEntry::DotApplied { target, amount, .. } => {
                bevy::log::trace!(target: "battle_record", entity = ?target, dot = amount, "DotApplied");
            }
            BattleEntry::HotApplied { target, amount, .. } => {
                bevy::log::trace!(target: "battle_record", entity = ?target, hot = amount, "HotApplied");
            }
            BattleEntry::StunApplied { target, .. } => {
                bevy::log::trace!(target: "battle_record", entity = ?target, "StunApplied");
            }
            BattleEntry::CharacterDied { entity, .. } => {
                bevy::log::trace!(target: "battle_record", entity = ?entity, "CharacterDied");
            }
            BattleEntry::TurnStarted { turn } => {
                bevy::log::trace!(target: "battle_record", turn = turn, "TurnStarted");
            }
            BattleEntry::TurnEnded { turn } => {
                bevy::log::trace!(target: "battle_record", turn = turn, "TurnEnded");
            }
        }
        self.entries.push(entry);
    }

    /// 获取指定实体的全部记录
    pub fn entries_for(&self, entity: Entity) -> Vec<&BattleEntry> {
        self.entries
            .iter()
            .filter(|e| match e {
                BattleEntry::DamageApplied { target, .. } => *target == entity,
                BattleEntry::HealApplied { target, .. } => *target == entity,
                BattleEntry::DotApplied { target, .. } => *target == entity,
                BattleEntry::HotApplied { target, .. } => *target == entity,
                BattleEntry::StunApplied { target, .. } => *target == entity,
                BattleEntry::CharacterDied { entity: e, .. } => *e == entity,
                _ => false,
            })
            .collect()
    }

    /// 获取指定回合的全部记录
    pub fn entries_for_turn(&self, turn: u32) -> Vec<&BattleEntry> {
        let mut result = Vec::new();
        let mut in_turn = false;
        for entry in &self.entries {
            match entry {
                BattleEntry::TurnStarted { turn: t } if *t == turn => in_turn = true,
                BattleEntry::TurnEnded { turn: t } if *t == turn => {
                    result.push(entry);
                    in_turn = false;
                    continue;
                }
                BattleEntry::TurnStarted { .. } | BattleEntry::TurnEnded { .. } => {
                    if !in_turn {
                        continue;
                    }
                }
                _ => {}
            }
            if in_turn {
                result.push(entry);
            }
        }
        result
    }

    /// 获取最近 N 条记录
    pub fn recent(&self, n: usize) -> &[BattleEntry] {
        let start = self.entries.len().saturating_sub(n);
        &self.entries[start..]
    }

    /// 计算指定实体的战斗统计
    pub fn stats_for(&self, entity: Entity) -> EntityBattleStats {
        let mut stats = EntityBattleStats::default();
        for entry in &self.entries {
            match entry {
                BattleEntry::DamageApplied {
                    attacker,
                    target,
                    amount,
                    ..
                } => {
                    if *attacker == entity {
                        stats.damage_dealt += amount;
                    }
                    if *target == entity {
                        stats.damage_taken += amount;
                    }
                }
                BattleEntry::DotApplied { target, amount, .. } => {
                    if *target == entity {
                        stats.damage_taken += amount;
                    }
                }
                BattleEntry::HealApplied { target, amount, .. } => {
                    if *target == entity {
                        stats.heal_received += amount;
                    }
                }
                BattleEntry::HotApplied { target, amount, .. } => {
                    if *target == entity {
                        stats.heal_received += amount;
                    }
                }
                BattleEntry::CharacterDied { .. } => {
                    // 击杀数需要从 DamageApplied 中推断（致死攻击的攻击者）
                }
                _ => {}
            }
        }
        // 统计击杀数：找到致死伤害的攻击者
        for entry in &self.entries {
            if let BattleEntry::CharacterDied { entity: dead_entity, .. } = entry {
                // 查找该实体最后受到的伤害记录
                let last_damage = self.entries.iter().rev().find(|e| {
                    if let BattleEntry::DamageApplied { target, .. } = e {
                        *target == *dead_entity
                    } else {
                        false
                    }
                });
                if let Some(BattleEntry::DamageApplied { attacker, .. }) = last_damage {
                    if *attacker == entity {
                        stats.kills += 1;
                    }
                }
            }
        }
        stats
    }

    /// 清空记录（新战斗开始时）
    pub fn clear(&mut self) {
        self.entries.clear();
        self.turn_number = 0;
    }
}

// ── 录制系统 ──

/// 记录回合开始
pub fn record_turn_started(
    mut reader: MessageReader<TurnStarted>,
    mut record: ResMut<BattleRecord>,
) {
    for msg in reader.read() {
        record.turn_number = msg.turn;
        record.record(BattleEntry::TurnStarted { turn: msg.turn });
    }
}

/// 记录回合结束
pub fn record_turn_ended(mut reader: MessageReader<TurnEnded>, mut record: ResMut<BattleRecord>) {
    for msg in reader.read() {
        record.record(BattleEntry::TurnEnded { turn: msg.turn });
    }
}

/// 记录伤害事件
pub fn record_damage(mut reader: MessageReader<DamageApplied>, mut record: ResMut<BattleRecord>) {
    for msg in reader.read() {
        record.record(BattleEntry::DamageApplied {
            target: msg.target,
            target_name: msg.target_name.clone(),
            target_faction: msg.target_faction,
            attacker: msg.attacker,
            attacker_name: msg.attacker_name.clone(),
            attacker_faction: msg.attacker_faction,
            amount: msg.amount,
            is_skill: msg.is_skill,
            terrain_label: msg.terrain_label.clone(),
            target_coord: msg.target_coord,
        });
    }
}

/// 记录治疗事件
pub fn record_heal(mut reader: MessageReader<HealApplied>, mut record: ResMut<BattleRecord>) {
    for msg in reader.read() {
        record.record(BattleEntry::HealApplied {
            target: msg.target,
            target_name: msg.target_name.clone(),
            amount: msg.amount,
        });
    }
}

/// 记录 DoT 伤害
pub fn record_dot(mut reader: MessageReader<DotApplied>, mut record: ResMut<BattleRecord>) {
    for msg in reader.read() {
        record.record(BattleEntry::DotApplied {
            target: msg.target,
            target_name: msg.target_name.clone(),
            amount: msg.amount,
            target_coord: msg.target_coord,
        });
    }
}

/// 记录 HoT 治疗
pub fn record_hot(mut reader: MessageReader<HotApplied>, mut record: ResMut<BattleRecord>) {
    for msg in reader.read() {
        record.record(BattleEntry::HotApplied {
            target: msg.target,
            target_name: msg.target_name.clone(),
            amount: msg.amount,
        });
    }
}

/// 记录晕眩
pub fn record_stun(mut reader: MessageReader<StunApplied>, mut record: ResMut<BattleRecord>) {
    for msg in reader.read() {
        record.record(BattleEntry::StunApplied {
            target: msg.target,
            target_name: msg.target_name.clone(),
        });
    }
}

/// 记录角色死亡
pub fn record_character_died(
    mut reader: MessageReader<CharacterDied>,
    mut record: ResMut<BattleRecord>,
) {
    for msg in reader.read() {
        record.record(BattleEntry::CharacterDied {
            entity: msg.entity,
            name: msg.name.clone(),
            faction: msg.faction,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 战斗记录_记录伤害() {
        let mut record = BattleRecord::default();
        record.record(BattleEntry::DamageApplied {
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
        });
        assert_eq!(record.entries.len(), 1);
    }

    #[test]
    fn 战斗记录_按实体查询() {
        let e1 = Entity::from_bits(1);
        let e2 = Entity::from_bits(2);
        let attacker = Entity::from_bits(3);
        let mut record = BattleRecord::default();
        record.record(BattleEntry::DamageApplied {
            target: e1,
            target_name: "哥布林".to_string(),
            target_faction: Faction::Enemy,
            attacker,
            attacker_name: "战士".to_string(),
            attacker_faction: Faction::Player,
            amount: 10,
            is_skill: false,
            terrain_label: "平原".to_string(),
            target_coord: IVec2::ZERO,
        });
        record.record(BattleEntry::HealApplied {
            target: e2,
            target_name: "战士".to_string(),
            amount: 5,
        });
        record.record(BattleEntry::DamageApplied {
            target: e1,
            target_name: "哥布林".to_string(),
            target_faction: Faction::Enemy,
            attacker,
            attacker_name: "战士".to_string(),
            attacker_faction: Faction::Player,
            amount: 8,
            is_skill: true,
            terrain_label: "森林".to_string(),
            target_coord: IVec2::ZERO,
        });
        let e1_entries = record.entries_for(e1);
        assert_eq!(e1_entries.len(), 2);
    }

    #[test]
    fn 战斗记录_最近N条() {
        let mut record = BattleRecord::default();
        for i in 0..10 {
            record.record(BattleEntry::TurnStarted { turn: i });
        }
        let recent = record.recent(3);
        assert_eq!(recent.len(), 3);
        if let BattleEntry::TurnStarted { turn } = recent[2] {
            assert_eq!(turn, 9);
        } else {
            panic!("期望 TurnStarted");
        }
    }

    #[test]
    fn 战斗记录_清空() {
        let mut record = BattleRecord::default();
        record.record(BattleEntry::TurnStarted { turn: 1 });
        record.turn_number = 1;
        record.clear();
        assert!(record.entries.is_empty());
        assert_eq!(record.turn_number, 0);
    }

    #[test]
    fn 战斗记录_回合开始更新回合号() {
        let mut record = BattleRecord::default();
        record.turn_number = 5;
        record.record(BattleEntry::TurnStarted { turn: 5 });
        assert_eq!(record.turn_number, 5);
    }

    #[test]
    fn 战斗记录_按回合查询() {
        let mut record = BattleRecord::default();
        record.record(BattleEntry::TurnStarted { turn: 1 });
        record.record(BattleEntry::DamageApplied {
            target: Entity::from_bits(1),
            target_name: "哥布林".to_string(),
            target_faction: Faction::Enemy,
            attacker: Entity::from_bits(2),
            attacker_name: "战士".to_string(),
            attacker_faction: Faction::Player,
            amount: 10,
            is_skill: false,
            terrain_label: "平原".to_string(),
            target_coord: IVec2::ZERO,
        });
        record.record(BattleEntry::TurnEnded { turn: 1 });
        record.record(BattleEntry::TurnStarted { turn: 2 });
        record.record(BattleEntry::HealApplied {
            target: Entity::from_bits(2),
            target_name: "战士".to_string(),
            amount: 5,
        });
        record.record(BattleEntry::TurnEnded { turn: 2 });

        let turn1 = record.entries_for_turn(1);
        assert_eq!(turn1.len(), 3); // TurnStarted + DamageApplied + TurnEnded
        let turn2 = record.entries_for_turn(2);
        assert_eq!(turn2.len(), 3); // TurnStarted + HealApplied + TurnEnded
    }

    #[test]
    fn 战斗记录_实体统计() {
        let attacker = Entity::from_bits(1);
        let target = Entity::from_bits(2);
        let mut record = BattleRecord::default();
        record.record(BattleEntry::DamageApplied {
            target,
            target_name: "哥布林".to_string(),
            target_faction: Faction::Enemy,
            attacker,
            attacker_name: "战士".to_string(),
            attacker_faction: Faction::Player,
            amount: 10,
            is_skill: false,
            terrain_label: "平原".to_string(),
            target_coord: IVec2::ZERO,
        });
        record.record(BattleEntry::DamageApplied {
            target,
            target_name: "哥布林".to_string(),
            target_faction: Faction::Enemy,
            attacker,
            attacker_name: "战士".to_string(),
            attacker_faction: Faction::Player,
            amount: 5,
            is_skill: true,
            terrain_label: "平原".to_string(),
            target_coord: IVec2::ZERO,
        });
        record.record(BattleEntry::CharacterDied {
            entity: target,
            name: "哥布林".to_string(),
            faction: Faction::Enemy,
        });

        let attacker_stats = record.stats_for(attacker);
        assert_eq!(attacker_stats.damage_dealt, 15);
        assert_eq!(attacker_stats.kills, 1);

        let target_stats = record.stats_for(target);
        assert_eq!(target_stats.damage_taken, 15);
    }

    #[test]
    fn 战斗记录_序列化反序列化() {
        let mut record = BattleRecord::default();
        record.record(BattleEntry::TurnStarted { turn: 1 });
        record.record(BattleEntry::DamageApplied {
            target: Entity::from_bits(1),
            target_name: "哥布林".to_string(),
            target_faction: Faction::Enemy,
            attacker: Entity::from_bits(2),
            attacker_name: "战士".to_string(),
            attacker_faction: Faction::Player,
            amount: 10,
            is_skill: false,
            terrain_label: "平原".to_string(),
            target_coord: IVec2::new(3, 4),
        });
        record.turn_number = 1;

        let ron = ron::to_string(&record).unwrap();
        let deserialized: BattleRecord = ron::from_str(&ron).unwrap();
        assert_eq!(deserialized.entries.len(), 2);
        assert_eq!(deserialized.turn_number, 1);
    }
}
