// 战斗测试辅助：简化攻击、施法、回合推进等操作

use bevy::prelude::*;
use tactical_rpg::core::attribute::{AttributeKind, Attributes};
use tactical_rpg::core::effect::{EffectQueue, PendingEffect, PendingEffectData};

/// 入队伤害效果（普通攻击）
pub fn deal_damage(app: &mut App, source: Entity, target: Entity, amount: i32) {
    let mut queue = app.world_mut().resource_mut::<EffectQueue>();
    queue.pending.push(PendingEffect {
        source,
        target,
        data: PendingEffectData::Damage {
            amount,
            is_skill: false,
            base_amount: None,
            modifiers: Vec::new(),
        },
        source_tags: vec![],
        terrain_id: String::new(),
    });
}

/// 入队技能伤害效果
pub fn deal_skill_damage(app: &mut App, source: Entity, target: Entity, amount: i32) {
    let mut queue = app.world_mut().resource_mut::<EffectQueue>();
    queue.pending.push(PendingEffect {
        source,
        target,
        data: PendingEffectData::Damage {
            amount,
            is_skill: true,
            base_amount: None,
            modifiers: Vec::new(),
        },
        source_tags: vec![],
        terrain_id: String::new(),
    });
}

/// 入队治疗效果
pub fn deal_heal(app: &mut App, target: Entity, amount: i32) {
    let mut queue = app.world_mut().resource_mut::<EffectQueue>();
    queue.pending.push(PendingEffect {
        source: Entity::PLACEHOLDER,
        target,
        data: PendingEffectData::Heal {
            amount,
            base_amount: None,
        },
        source_tags: vec![],
        terrain_id: String::new(),
    });
}

/// 入队 Buff 效果
pub fn apply_buff(app: &mut App, target: Entity, buff_id: &str, duration: u32) {
    let mut queue = app.world_mut().resource_mut::<EffectQueue>();
    queue.pending.push(PendingEffect {
        source: Entity::PLACEHOLDER,
        target,
        data: PendingEffectData::ApplyBuff {
            buff_id: buff_id.into(),
            duration,
        },
        source_tags: vec![],
        terrain_id: String::new(),
    });
}

/// 推进一个 Update tick
pub fn tick(app: &mut App) {
    app.update();
}

/// 推进 N 个 Update tick
pub fn tick_n(app: &mut App, n: u32) {
    for _ in 0..n {
        app.update();
    }
}

/// 获取角色当前 HP
pub fn get_hp(app: &App, entity: Entity) -> f32 {
    app.world()
        .get::<Attributes>(entity)
        .unwrap()
        .get(AttributeKind::Hp)
}

/// 获取角色当前 MP
pub fn get_mp(app: &App, entity: Entity) -> f32 {
    app.world()
        .get::<Attributes>(entity)
        .unwrap()
        .get(AttributeKind::Mp)
}
