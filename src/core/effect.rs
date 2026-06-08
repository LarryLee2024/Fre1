// 效果管道：EffectDef → PendingEffect → 修饰 → 执行
// 替代 combat_event.rs 中的 execute_attack 大函数

use crate::core::tag::GameplayTag;
use crate::map::Terrain;
use bevy::prelude::*;
use serde::Deserialize;

/// 效果定义（用于 SkillData 中声明技能效果，支持 RON 反序列化）
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum EffectDef {
    Damage {
        multiplier: f32,
        ignore_def_percent: f32,
    },
    Heal {
        amount: i32,
    },
    ApplyBuff {
        buff_id: String,
        duration: u32,
    },
    Cleanse,
}

/// 待处理效果（运行时，进入 EffectQueue）
#[derive(Clone, Debug)]
pub struct PendingEffect {
    pub source: Entity,
    pub target: Entity,
    pub data: PendingEffectData,
    pub source_tags: Vec<GameplayTag>,
    pub terrain: Terrain,
}

/// 待处理效果数据
#[derive(Clone, Debug)]
pub enum PendingEffectData {
    Damage {
        amount: i32,
        is_skill: bool,
    },
    Heal {
        amount: i32,
    },
    ApplyBuff {
        buff_id: String,
        duration: u32,
    },
    Cleanse,
}

/// 效果执行结果
#[derive(Clone, Debug)]
pub struct EffectResult {
    pub source: Entity,
    pub target: Entity,
    pub data: EffectResultData,
}

#[derive(Clone, Debug)]
pub enum EffectResultData {
    Damage { amount: i32, killed: bool },
    Heal { amount: i32 },
    BuffApplied { buff_id: String },
    CleanseApplied,
}

/// 效果队列资源
#[derive(Resource, Default, Debug)]
pub struct EffectQueue {
    pub pending: Vec<PendingEffect>,
}

impl EffectQueue {
    pub fn push(&mut self, effect: PendingEffect) {
        self.pending.push(effect);
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn clear(&mut self) {
        self.pending.clear();
    }
}

/// 从技能效果定义生成伤害计算结果
pub fn calculate_damage_from_effect(
    effective_atk: f32,
    effective_def: f32,
    base_def: f32,
    multiplier: f32,
    ignore_def_percent: f32,
    terrain: Terrain,
) -> i32 {
    let def_ignored = base_def * (ignore_def_percent / 100.0);
    let final_def = effective_def - def_ignored;
    let base_damage = effective_atk - final_def;
    let terrain_bonus = terrain.defense_bonus() as f32;
    ((base_damage - terrain_bonus) * multiplier).max(1.0) as i32
}

/// 效果管道插件
pub struct EffectPlugin;

impl Plugin for EffectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EffectQueue>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 伤害计算_基础() {
        // ATK=10, DEF=3, multiplier=1.0, no ignore, Plain
        let dmg = calculate_damage_from_effect(10.0, 3.0, 3.0, 1.0, 0.0, Terrain::Plain);
        assert_eq!(dmg, 7);
    }

    #[test]
    fn 伤害计算_森林地形() {
        let dmg = calculate_damage_from_effect(10.0, 3.0, 3.0, 1.0, 0.0, Terrain::Forest);
        // 10 - 3 - 2 = 5
        assert_eq!(dmg, 5);
    }

    #[test]
    fn 伤害计算_最低为1() {
        let dmg = calculate_damage_from_effect(1.0, 10.0, 10.0, 1.0, 0.0, Terrain::Plain);
        assert_eq!(dmg, 1);
    }

    #[test]
    fn 伤害计算_技能倍率() {
        let dmg = calculate_damage_from_effect(10.0, 3.0, 3.0, 1.5, 0.0, Terrain::Plain);
        // (10 - 3) * 1.5 = 10.5 → 10
        assert_eq!(dmg, 10);
    }

    #[test]
    fn 伤害计算_无视防御() {
        let dmg = calculate_damage_from_effect(10.0, 10.0, 10.0, 1.3, 50.0, Terrain::Plain);
        // final_def = 10 - 10*0.5 = 5, (10 - 5) * 1.3 = 6.5 → 6
        assert_eq!(dmg, 6);
    }

    #[test]
    fn 伤害计算_100百分比无视防御() {
        let dmg = calculate_damage_from_effect(10.0, 10.0, 10.0, 1.0, 100.0, Terrain::Plain);
        // final_def = 10 - 10*1.0 = 0, (10 - 0) * 1.0 = 10
        assert_eq!(dmg, 10);
    }

    #[test]
    fn 伤害计算_山地地形无防御加成() {
        let dmg = calculate_damage_from_effect(10.0, 3.0, 3.0, 1.0, 0.0, Terrain::Mountain);
        // Mountain defense_bonus = 0, 10 - 3 = 7
        assert_eq!(dmg, 7);
    }

    #[test]
    fn 伤害计算_水域地形无防御加成() {
        let dmg = calculate_damage_from_effect(10.0, 3.0, 3.0, 1.0, 0.0, Terrain::Water);
        // Water defense_bonus = 0, 10 - 3 = 7
        assert_eq!(dmg, 7);
    }

    #[test]
    fn 伤害计算_高倍率技能() {
        let dmg = calculate_damage_from_effect(10.0, 3.0, 3.0, 3.0, 0.0, Terrain::Plain);
        // (10 - 3) * 3.0 = 21
        assert_eq!(dmg, 21);
    }

    #[test]
    fn 效果队列_push和drain() {
        let mut queue = EffectQueue::default();
        assert!(queue.is_empty());

        queue.push(PendingEffect {
            source: Entity::from_bits(1),
            target: Entity::from_bits(2),
            data: PendingEffectData::Damage { amount: 5, is_skill: false },
            source_tags: vec![],
            terrain: Terrain::Plain,
        });
        assert!(!queue.is_empty());

        let drained: Vec<_> = queue.pending.drain(..).collect();
        assert_eq!(drained.len(), 1);
        assert!(queue.is_empty());
    }

    #[test]
    fn 效果队列_clear() {
        let mut queue = EffectQueue::default();
        queue.push(PendingEffect {
            source: Entity::from_bits(1),
            target: Entity::from_bits(2),
            data: PendingEffectData::Damage { amount: 5, is_skill: false },
            source_tags: vec![],
            terrain: Terrain::Plain,
        });
        queue.clear();
        assert!(queue.is_empty());
    }
}
