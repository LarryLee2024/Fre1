//! Cue 发射器 — CueEmitter Resource
//!
//! 业务层调用的发送接口，封装所有 CueEvent 的发送逻辑。
//! 业务逻辑只调用 CueEmitter 的方法，不直接使用 EventWriter。

use super::types::*;
use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;

/// Cue 发射器 — 业务层调用的发送接口
///
/// 封装所有 CueEvent 的发送逻辑，业务逻辑只调用此接口。
#[derive(Resource, Default)]
pub struct CueEmitter {
    /// 待发送的伤害事件
    pub damage_events: Vec<CueDamage>,
    /// 待发送的死亡事件
    pub death_events: Vec<CueDeath>,
    /// 待发送的治疗事件
    pub heal_events: Vec<CueHeal>,
    /// 待发送的 Buff 施加事件
    pub buff_apply_events: Vec<CueBuffApply>,
    /// 待发送的 Buff 移除事件
    pub buff_remove_events: Vec<CueBuffRemove>,
    /// 待发送的护盾事件
    pub shield_events: Vec<CueShield>,
    /// 待发送的技能释放事件
    pub skill_cast_events: Vec<CueSkillCast>,
    /// 待发送的移动事件
    pub movement_events: Vec<CueMovement>,
    /// 待发送的状态变化事件
    pub status_change_events: Vec<CueStatusChange>,
}

impl CueEmitter {
    /// 发送伤害 Cue
    pub fn emit_damage(
        &mut self,
        target: Entity,
        amount: i32,
        is_critical: bool,
        attacker: Option<Entity>,
    ) {
        self.damage_events.push(CueDamage {
            target,
            amount,
            is_critical,
            attacker,
        });
    }

    /// 发送死亡 Cue
    pub fn emit_death(&mut self, entity: Entity, killer: Option<Entity>) {
        self.death_events.push(CueDeath { entity, killer });
    }

    /// 发送治疗 Cue
    pub fn emit_heal(&mut self, target: Entity, amount: i32, source: Option<Entity>) {
        self.heal_events.push(CueHeal {
            target,
            amount,
            source,
        });
    }

    /// 发送 Buff 施加 Cue
    pub fn emit_buff_apply(&mut self, target: Entity, buff_id: String, stacks: u32) {
        self.buff_apply_events.push(CueBuffApply {
            target,
            buff_id,
            stacks,
        });
    }

    /// 发送 Buff 移除 Cue
    pub fn emit_buff_remove(&mut self, target: Entity, buff_id: String) {
        self.buff_remove_events
            .push(CueBuffRemove { target, buff_id });
    }

    /// 发送护盾 Cue
    pub fn emit_shield(&mut self, target: Entity, amount: i32) {
        self.shield_events.push(CueShield { target, amount });
    }

    /// 发送技能释放 Cue
    pub fn emit_skill_cast(&mut self, caster: Entity, skill_id: String, target_pos: Option<IVec2>) {
        self.skill_cast_events.push(CueSkillCast {
            caster,
            skill_id,
            target_pos,
        });
    }

    /// 发送移动 Cue
    pub fn emit_movement(&mut self, entity: Entity, from: IVec2, to: IVec2) {
        self.movement_events.push(CueMovement { entity, from, to });
    }

    /// 发送状态变化 Cue
    pub fn emit_status_change(&mut self, entity: Entity, status: CueStatusType, active: bool) {
        self.status_change_events.push(CueStatusChange {
            entity,
            status,
            active,
        });
    }

    /// 清空所有待发送事件
    pub fn clear(&mut self) {
        self.damage_events.clear();
        self.death_events.clear();
        self.heal_events.clear();
        self.buff_apply_events.clear();
        self.buff_remove_events.clear();
        self.shield_events.clear();
        self.skill_cast_events.clear();
        self.movement_events.clear();
        self.status_change_events.clear();
    }

    /// 是否有待发送的事件
    pub fn has_pending(&self) -> bool {
        !self.damage_events.is_empty()
            || !self.death_events.is_empty()
            || !self.heal_events.is_empty()
            || !self.buff_apply_events.is_empty()
            || !self.buff_remove_events.is_empty()
            || !self.shield_events.is_empty()
            || !self.skill_cast_events.is_empty()
            || !self.movement_events.is_empty()
            || !self.status_change_events.is_empty()
    }
}

/// 将 CueEmitter 中的事件发送到 Bevy Message 系统
pub fn flush_cue_emitter(
    mut emitter: ResMut<CueEmitter>,
    mut damage_writer: MessageWriter<CueDamage>,
    mut death_writer: MessageWriter<CueDeath>,
    mut heal_writer: MessageWriter<CueHeal>,
    mut buff_apply_writer: MessageWriter<CueBuffApply>,
    mut buff_remove_writer: MessageWriter<CueBuffRemove>,
    mut shield_writer: MessageWriter<CueShield>,
    mut skill_cast_writer: MessageWriter<CueSkillCast>,
    mut movement_writer: MessageWriter<CueMovement>,
    mut status_change_writer: MessageWriter<CueStatusChange>,
) {
    for event in emitter.damage_events.drain(..) {
        damage_writer.write(event);
    }
    for event in emitter.death_events.drain(..) {
        death_writer.write(event);
    }
    for event in emitter.heal_events.drain(..) {
        heal_writer.write(event);
    }
    for event in emitter.buff_apply_events.drain(..) {
        buff_apply_writer.write(event);
    }
    for event in emitter.buff_remove_events.drain(..) {
        buff_remove_writer.write(event);
    }
    for event in emitter.shield_events.drain(..) {
        shield_writer.write(event);
    }
    for event in emitter.skill_cast_events.drain(..) {
        skill_cast_writer.write(event);
    }
    for event in emitter.movement_events.drain(..) {
        movement_writer.write(event);
    }
    for event in emitter.status_change_events.drain(..) {
        status_change_writer.write(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn Cue发射器_发射伤害() {
        let mut emitter = CueEmitter::default();
        emitter.emit_damage(Entity::from_bits(1), 50, true, Some(Entity::from_bits(2)));
        assert_eq!(emitter.damage_events.len(), 1);
        assert_eq!(emitter.damage_events[0].amount, 50);
        assert!(emitter.damage_events[0].is_critical);
    }

    #[test]
    fn Cue发射器_有待发送事件() {
        let mut emitter = CueEmitter::default();
        assert!(!emitter.has_pending());

        emitter.emit_death(Entity::from_bits(1), None);
        assert!(emitter.has_pending());
    }

    #[test]
    fn Cue发射器_清空() {
        let mut emitter = CueEmitter::default();
        emitter.emit_damage(Entity::from_bits(1), 50, false, None);
        emitter.emit_heal(Entity::from_bits(2), 30, None);

        emitter.clear();
        assert!(!emitter.has_pending());
    }
}
