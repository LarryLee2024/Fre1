// 战斗事件模块：统一攻击处理逻辑，消除 input/ai 重复代码

use crate::assets::CnFont;
use crate::combat::{calculate_damage, skill_name};
use crate::combat_log::{CombatLog, LogSegment, log_color};
use crate::map::Terrain;
use crate::unit::{Faction, Skill, Unit, UnitName};
use crate::vfx;
use bevy::prelude::*;

/// 攻击结果
pub struct AttackResult {
    /// 造成伤害
    pub damage: i32,
    /// 是否击杀
    pub killed: bool,
}

/// 执行攻击（统一入口）
///
/// 包含：伤害计算、扣血、伤害弹出、战斗日志、击杀处理
/// input.rs 和 ai.rs 共用此函数，消除重复逻辑
pub fn execute_attack(
    commands: &mut Commands,
    attacker_unit: &Unit,
    attacker_name: &str,
    target_entity: Entity,
    target_unit: &mut Unit,
    _target_pos: IVec2,
    target_name: &UnitName,
    target_translation: Vec2,
    terrain: Terrain,
    cn_font: &CnFont,
    combat_log: &mut CombatLog,
) -> AttackResult {
    // 伤害计算
    let damage = calculate_damage(attacker_unit, target_unit, terrain);
    target_unit.hp -= damage;

    // 伤害数字弹出
    let is_crit = attacker_unit.skill != Skill::None;
    vfx::spawn_damage_popup(commands, target_translation, damage, &cn_font.handle, is_crit);

    // 战斗日志
    let attacker_color =
        if attacker_unit.faction == Faction::Player { log_color::PLAYER } else { log_color::ENEMY };
    let defender_color =
        if target_unit.faction == Faction::Player { log_color::PLAYER } else { log_color::ENEMY };
    let skill_label = skill_name(&attacker_unit.skill);

    combat_log.push(vec![
        LogSegment {
            text: format!("[{}]", attacker_name),
            color: attacker_color,
        },
        LogSegment {
            text: format!(" 使用[{}]", skill_label),
            color: log_color::TURN,
        },
        LogSegment {
            text: " 攻击 ".to_string(),
            color: log_color::NORMAL,
        },
        LogSegment {
            text: format!("[{}]", target_name.0),
            color: defender_color,
        },
        LogSegment {
            text: " 造成 ".to_string(),
            color: log_color::NORMAL,
        },
        LogSegment {
            text: format!("[{}]", damage),
            color: log_color::DAMAGE,
        },
        LogSegment {
            text: " 伤害".to_string(),
            color: log_color::NORMAL,
        },
        LogSegment {
            text: format!(" ({})", terrain.label()),
            color: log_color::TERRAIN,
        },
    ]);

    // 击杀处理
    let killed = target_unit.hp <= 0;
    if killed {
        combat_log.push(vec![
            LogSegment {
                text: format!("[{}]", target_name.0),
                color: defender_color,
            },
            LogSegment {
                text: " 被击败！".to_string(),
                color: log_color::KILL,
            },
        ]);
        commands.entity(target_entity).despawn();
    }

    AttackResult { damage, killed }
}
