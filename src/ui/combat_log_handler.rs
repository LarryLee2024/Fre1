// 战斗日志表现层：监听 Message，格式化写入 CombatLog
// 遵循「Logic 发消息，Presentation 响应」原则

use crate::battle::{
    CharacterDied, CombatLog, DamageApplied, DotApplied, HealApplied, HotApplied, LogSegment,
    StunApplied, log_color,
};
use crate::character::Faction;
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;

/// 阵营对应的日志颜色
fn faction_log_color(faction: Faction) -> Color {
    match faction {
        Faction::Player => log_color::PLAYER,
        Faction::Enemy => log_color::ENEMY,
    }
}

/// 响应伤害消息：写入战斗日志
pub fn on_damage_applied(
    mut damage_reader: MessageReader<DamageApplied>,
    mut combat_log: ResMut<CombatLog>,
) {
    for msg in damage_reader.read() {
        bevy::log::debug!("[Message] DamageApplied: {} → {} ({}dmg)", msg.attacker_name, msg.target_name, msg.amount);
        let skill_label = if msg.is_skill { "技能" } else { "攻击" };
        combat_log.push(vec![
            LogSegment {
                text: format!("[{}]", msg.attacker_name),
                color: faction_log_color(msg.attacker_faction),
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
                text: format!("[{}]", msg.target_name),
                color: faction_log_color(msg.target_faction),
            },
            LogSegment {
                text: " 造成 ".to_string(),
                color: log_color::NORMAL,
            },
            LogSegment {
                text: format!("[{}]", msg.amount),
                color: log_color::DAMAGE,
            },
            LogSegment {
                text: " 伤害".to_string(),
                color: log_color::NORMAL,
            },
            LogSegment {
                text: format!(" ({})", msg.terrain_label),
                color: log_color::TERRAIN,
            },
        ]);
    }
}

/// 响应治疗消息：写入战斗日志
pub fn on_heal_applied(
    mut heal_reader: MessageReader<HealApplied>,
    mut combat_log: ResMut<CombatLog>,
) {
    for msg in heal_reader.read() {
        bevy::log::debug!("[Message] HealApplied: {} +{}HP", msg.target_name, msg.amount);
        combat_log.push(vec![
            LogSegment {
                text: format!("[{}]", msg.target_name),
                color: log_color::NORMAL,
            },
            LogSegment {
                text: format!(" 恢复 {} HP", msg.amount),
                color: log_color::HEAL,
            },
        ]);
    }
}

/// 响应角色死亡消息：写入战斗日志
pub fn on_character_died_log(
    mut died_reader: MessageReader<CharacterDied>,
    mut combat_log: ResMut<CombatLog>,
) {
    for msg in died_reader.read() {
        bevy::log::debug!("[Message] CharacterDied(log): {} ({:?})", msg.name, msg.faction);
        combat_log.push(vec![
            LogSegment {
                text: format!("[{}]", msg.name),
                color: faction_log_color(msg.faction),
            },
            LogSegment {
                text: " 被击败！".to_string(),
                color: log_color::KILL,
            },
        ]);
    }
}

/// 响应晕眩消息：写入战斗日志
pub fn on_stun_applied(
    mut stun_reader: MessageReader<StunApplied>,
    mut combat_log: ResMut<CombatLog>,
) {
    for msg in stun_reader.read() {
        bevy::log::debug!("[Message] StunApplied: {}", msg.target_name);
        combat_log.push(vec![
            LogSegment {
                text: format!("[{}]", msg.target_name),
                color: log_color::NORMAL,
            },
            LogSegment {
                text: " 处于晕眩，无法行动".to_string(),
                color: log_color::DAMAGE,
            },
        ]);
    }
}

/// 响应 DoT 消息：写入战斗日志
pub fn on_dot_applied(
    mut dot_reader: MessageReader<DotApplied>,
    mut combat_log: ResMut<CombatLog>,
) {
    for msg in dot_reader.read() {
        bevy::log::debug!("[Message] DotApplied: {} -{}dmg", msg.target_name, msg.amount);
        combat_log.push(vec![
            LogSegment {
                text: format!("[{}]", msg.target_name),
                color: log_color::NORMAL,
            },
            LogSegment {
                text: format!(" 受到 {} 持续伤害", msg.amount),
                color: log_color::DAMAGE,
            },
        ]);
    }
}

/// 响应 HoT 消息：写入战斗日志
pub fn on_hot_applied(
    mut hot_reader: MessageReader<HotApplied>,
    mut combat_log: ResMut<CombatLog>,
) {
    for msg in hot_reader.read() {
        bevy::log::debug!("[Message] HotApplied: {} +{}HP", msg.target_name, msg.amount);
        combat_log.push(vec![
            LogSegment {
                text: format!("[{}]", msg.target_name),
                color: log_color::NORMAL,
            },
            LogSegment {
                text: format!(" 恢复 {} HP", msg.amount),
                color: log_color::HEAL,
            },
        ]);
    }
}
