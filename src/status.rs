// 持续效果模块：Buff/Debuff 数据模型、叠加规则、tick 结算

use crate::assets::CnFont;
use crate::combat_log::{CombatLog, LogSegment, log_color};
use crate::map::GameMap;
use crate::turn::TurnState;
use crate::unit::{GridPosition, Unit, UnitName};
use crate::vfx;
use bevy::prelude::*;

/// 状态效果种类
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StatusEffect {
    // 属性修正（i32 为修正值，可正可负）
    AttackUp(i32),
    AttackDown(i32),
    DefenseUp(i32),
    DefenseDown(i32),
    MovementUp(i32),
    MovementDown(i32),
    RangeUp(i32),
    RangeDown(i32),
    // 状态条件
    Stun,
    // 持续伤害
    Poison(i32),
    Burn(i32),
    // 持续治疗
    Regen(i32),
    // 一次性（立即驱散所有 debuff）
    Cleanse,
}

impl StatusEffect {
    pub fn is_buff(&self) -> bool {
        matches!(
            self,
            Self::AttackUp(_)
                | Self::DefenseUp(_)
                | Self::MovementUp(_)
                | Self::RangeUp(_)
                | Self::Regen(_)
        )
    }

    pub fn is_debuff(&self) -> bool {
        matches!(
            self,
            Self::AttackDown(_)
                | Self::DefenseDown(_)
                | Self::MovementDown(_)
                | Self::RangeDown(_)
                | Self::Stun
                | Self::Poison(_)
                | Self::Burn(_)
        )
    }

    pub fn label(&self) -> String {
        match self {
            Self::AttackUp(v) => format!("攻+{}", v),
            Self::AttackDown(v) => format!("攻-{}", v),
            Self::DefenseUp(v) => format!("防+{}", v),
            Self::DefenseDown(v) => format!("防-{}", v),
            Self::MovementUp(v) => format!("移+{}", v),
            Self::MovementDown(v) => format!("移-{}", v),
            Self::RangeUp(v) => format!("距+{}", v),
            Self::RangeDown(v) => format!("距-{}", v),
            Self::Stun => "晕".to_string(),
            Self::Poison(v) => format!("毒-{}", v),
            Self::Burn(v) => format!("灼-{}", v),
            Self::Regen(v) => format!("愈+{}", v),
            Self::Cleanse => "驱散".to_string(),
        }
    }
}

/// 单个状态效果实例：包含效果、剩余回合、施加者
#[derive(Clone, Debug)]
pub struct StatusEffectInstance {
    pub effect: StatusEffect,
    pub remaining_turns: u32,
    pub source: Option<Entity>,
}

/// 单位身上的状态效果集合
#[derive(Component, Default, Debug, Clone)]
pub struct StatusEffects(pub Vec<StatusEffectInstance>);

impl StatusEffects {
    /// 添加效果。同源同 kind 刷新剩余回合，异源或异 kind 叠加为新实例。
    pub fn add(&mut self, inst: StatusEffectInstance) {
        if let Some(source) = inst.source {
            for existing in &mut self.0 {
                if existing.source == Some(source)
                    && std::mem::discriminant(&existing.effect)
                        == std::mem::discriminant(&inst.effect)
                {
                    existing.remaining_turns = inst.remaining_turns;
                    return;
                }
            }
        }
        self.0.push(inst);
    }

    /// 每回合结算：减少所有效果的剩余回合，<=0 的移除
    pub fn tick(&mut self) {
        for inst in &mut self.0 {
            if inst.remaining_turns > 0 {
                inst.remaining_turns -= 1;
            }
        }
        self.0.retain(|inst| inst.remaining_turns > 0);
    }

    pub fn attack_mod(&self) -> i32 {
        self.0
            .iter()
            .map(|i| match i.effect {
                StatusEffect::AttackUp(v) => v,
                StatusEffect::AttackDown(v) => -v,
                _ => 0,
            })
            .sum()
    }

    pub fn defense_mod(&self) -> i32 {
        self.0
            .iter()
            .map(|i| match i.effect {
                StatusEffect::DefenseUp(v) => v,
                StatusEffect::DefenseDown(v) => -v,
                _ => 0,
            })
            .sum()
    }

    pub fn movement_mod(&self) -> i32 {
        self.0
            .iter()
            .map(|i| match i.effect {
                StatusEffect::MovementUp(v) => v,
                StatusEffect::MovementDown(v) => -v,
                _ => 0,
            })
            .sum()
    }

    pub fn range_mod(&self) -> i32 {
        self.0
            .iter()
            .map(|i| match i.effect {
                StatusEffect::RangeUp(v) => v,
                StatusEffect::RangeDown(v) => -v,
                _ => 0,
            })
            .sum()
    }

    pub fn is_stunned(&self) -> bool {
        self.0
            .iter()
            .any(|i| matches!(i.effect, StatusEffect::Stun))
    }

    /// 消耗晕眩：移除所有 Stun 实例，返回是否原本处于晕眩
    pub fn consume_stun(&mut self) -> bool {
        let was = self.is_stunned();
        self.0.retain(|i| !matches!(i.effect, StatusEffect::Stun));
        was
    }

    pub fn dot_damage(&self) -> i32 {
        self.0
            .iter()
            .map(|i| match i.effect {
                StatusEffect::Poison(v) => v,
                StatusEffect::Burn(v) => v,
                _ => 0,
            })
            .sum()
    }

    pub fn hot_heal(&self) -> i32 {
        self.0
            .iter()
            .map(|i| match i.effect {
                StatusEffect::Regen(v) => v,
                _ => 0,
            })
            .sum()
    }

    pub fn remove_debuffs(&mut self) {
        self.0.retain(|i| !i.effect.is_debuff());
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, StatusEffectInstance> {
        self.0.iter()
    }
}

/// 给目标单位挂一个状态效果。Cleanse 立即执行，duration 忽略。
pub fn apply_status_effect(
    commands: &mut Commands,
    target: Entity,
    effect: StatusEffect,
    source: Option<Entity>,
    duration: u32,
) {
    if matches!(effect, StatusEffect::Cleanse) {
        commands
            .entity(target)
            .entry::<StatusEffects>()
            .and_modify(|mut e| {
                e.remove_debuffs();
            })
            .or_insert(StatusEffects::default());
    } else {
        let inst = StatusEffectInstance {
            effect,
            remaining_turns: duration,
            source,
        };
        let inst_for_or = inst.clone();
        commands
            .entity(target)
            .entry::<StatusEffects>()
            .and_modify(move |mut e| {
                e.add(inst);
            })
            .or_insert(StatusEffects(vec![inst_for_or]));
    }
}

/// 持续效果结算系统：在新阵营回合开始时，对该阵营所有单位结算 DoT/HoT，并 tick。
pub fn resolve_status_effects(
    mut commands: Commands,
    map: Res<GameMap>,
    turn_state: Res<TurnState>,
    cn_font: Res<CnFont>,
    mut combat_log: ResMut<CombatLog>,
    mut units: Query<(Entity, &mut Unit, &UnitName, &GridPosition, &mut StatusEffects)>,
) {
    for (entity, mut unit, name, gp, mut effects) in &mut units {
        if unit.faction != turn_state.current_faction {
            continue;
        }

        let world_pos = map.coord_to_world(gp.coord);

        // 1. 结算本回合 DoT 伤害
        let dot = effects.dot_damage();
        if dot > 0 {
            unit.hp = (unit.hp - dot).max(0);
            vfx::spawn_damage_popup(&mut commands, world_pos, dot, &cn_font.handle, false);
            combat_log.push(vec![
                LogSegment {
                    text: format!("[{}]", name.0),
                    color: log_color::NORMAL,
                },
                LogSegment {
                    text: format!(" 受到 {} 持续伤害", dot),
                    color: log_color::DAMAGE,
                },
            ]);
            if unit.hp <= 0 {
                commands.entity(entity).try_despawn();
            }
        }

        // 2. 结算本回合 HoT 治疗
        let hot = effects.hot_heal();
        if hot > 0 {
            unit.hp = (unit.hp + hot).min(unit.max_hp);
            combat_log.push(vec![
                LogSegment {
                    text: format!("[{}]", name.0),
                    color: log_color::NORMAL,
                },
                LogSegment {
                    text: format!(" 恢复 {} HP", hot),
                    color: log_color::HEAL,
                },
            ]);
        }

        // 3. 减少持续时间，移除过期
        effects.tick();
    }
}

/// 持续效果插件
pub struct StatusPlugin;

impl Plugin for StatusPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::TurnPhase;
        app.add_systems(OnEnter(TurnPhase::SelectUnit), resolve_status_effects);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inst(effect: StatusEffect, turns: u32, src: Option<Entity>) -> StatusEffectInstance {
        StatusEffectInstance {
            effect,
            remaining_turns: turns,
            source: src,
        }
    }

    fn caster(n: u32) -> Entity {
        Entity::from_raw_u32(n).expect("valid index")
    }

    // ---- add / 叠加 ----

    #[test]
    fn 添加_新源叠加为新实例() {
        let mut eff = StatusEffects::default();
        eff.add(inst(StatusEffect::AttackUp(5), 2, Some(caster(1))));
        eff.add(inst(StatusEffect::AttackUp(3), 2, Some(caster(2))));
        assert_eq!(eff.len(), 2);
        assert_eq!(eff.attack_mod(), 8);
    }

    #[test]
    fn 添加_同源同kind刷新持续时间() {
        let mut eff = StatusEffects::default();
        eff.add(inst(StatusEffect::AttackUp(5), 2, Some(caster(1))));
        eff.add(inst(StatusEffect::AttackUp(7), 1, Some(caster(1))));
        assert_eq!(eff.len(), 1);
        assert_eq!(eff.0[0].remaining_turns, 1);
    }

    #[test]
    fn 添加_同源异kind分别叠加() {
        let mut eff = StatusEffects::default();
        eff.add(inst(StatusEffect::AttackUp(5), 2, Some(caster(1))));
        eff.add(inst(StatusEffect::DefenseUp(3), 2, Some(caster(1))));
        assert_eq!(eff.len(), 2);
        assert_eq!(eff.attack_mod(), 5);
        assert_eq!(eff.defense_mod(), 3);
    }

    #[test]
    fn 添加_无源全部叠加() {
        let mut eff = StatusEffects::default();
        eff.add(inst(StatusEffect::AttackUp(5), 2, None));
        eff.add(inst(StatusEffect::AttackUp(3), 1, None));
        assert_eq!(eff.len(), 2);
    }

    // ---- tick ----

    #[test]
    fn tick_递减并移除过期() {
        let mut eff = StatusEffects::default();
        eff.add(inst(StatusEffect::AttackUp(5), 2, None));
        eff.add(inst(StatusEffect::Poison(3), 1, None));
        eff.tick();
        assert_eq!(eff.len(), 1);
        assert_eq!(eff.0[0].effect, StatusEffect::AttackUp(5));
        assert_eq!(eff.0[0].remaining_turns, 1);
        eff.tick();
        assert!(eff.is_empty());
    }

    // ---- 属性修正累加 ----

    #[test]
    fn 修正累加_攻防独立() {
        let mut eff = StatusEffects::default();
        eff.add(inst(StatusEffect::AttackUp(5), 2, None));
        eff.add(inst(StatusEffect::AttackDown(2), 2, None));
        eff.add(inst(StatusEffect::DefenseUp(3), 2, None));
        assert_eq!(eff.attack_mod(), 3);
        assert_eq!(eff.defense_mod(), 3);
    }

    // ---- Stun ----

    #[test]
    fn 晕眩_检测与消耗() {
        let mut eff = StatusEffects::default();
        assert!(!eff.is_stunned());
        eff.add(inst(StatusEffect::Stun, 1, Some(caster(1))));
        assert!(eff.is_stunned());
        assert!(eff.consume_stun());
        assert!(!eff.is_stunned());
        assert!(!eff.consume_stun());
    }

    // ---- DoT / HoT ----

    #[test]
    fn 持续伤害_累加() {
        let mut eff = StatusEffects::default();
        eff.add(inst(StatusEffect::Poison(3), 2, Some(caster(1))));
        eff.add(inst(StatusEffect::Burn(2), 1, Some(caster(2))));
        assert_eq!(eff.dot_damage(), 5);
        assert_eq!(eff.hot_heal(), 0);
    }

    #[test]
    fn 持续治疗_独立计算() {
        let mut eff = StatusEffects::default();
        eff.add(inst(StatusEffect::Regen(4), 2, None));
        eff.add(inst(StatusEffect::Poison(2), 2, None));
        assert_eq!(eff.dot_damage(), 2);
        assert_eq!(eff.hot_heal(), 4);
    }

    // ---- Cleanse ----

    #[test]
    fn 驱散_移除所有debuff() {
        let mut eff = StatusEffects::default();
        eff.add(inst(StatusEffect::AttackUp(5), 2, None));
        eff.add(inst(StatusEffect::AttackDown(3), 2, None));
        eff.add(inst(StatusEffect::Stun, 1, None));
        eff.add(inst(StatusEffect::Poison(2), 2, None));
        eff.remove_debuffs();
        assert_eq!(eff.len(), 1);
        assert_eq!(eff.0[0].effect, StatusEffect::AttackUp(5));
    }

    // ---- 标签 ----

    #[test]
    fn 标签_中文() {
        assert_eq!(StatusEffect::AttackUp(5).label(), "攻+5");
        assert_eq!(StatusEffect::AttackDown(3).label(), "攻-3");
        assert_eq!(StatusEffect::Stun.label(), "晕");
        assert_eq!(StatusEffect::Poison(2).label(), "毒-2");
    }
}
