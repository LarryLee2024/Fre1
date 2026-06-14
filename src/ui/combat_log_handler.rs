// 战斗日志表现层：监听 Message，格式化写入 CombatLog
// 遵循「Logic 发消息，Presentation 响应」原则

use crate::core::battle::{
    CharacterDied, CombatLog, DamageApplied, DotApplied, HealApplied, HotApplied, LogSegment,
    StunApplied, log_color,
};
use crate::core::character::Faction;
use crate::core::equipment::{ItemEquipped, ItemUnequipped};
use crate::infrastructure::localization::{CurrentLocale, LocalizationService};
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
    localization: Res<LocalizationService>,
    locale: Res<CurrentLocale>,
) {
    let skill_text = localization.resolve("ui.combat_log.skill", &locale.0, None);
    let attack_text = localization.resolve("ui.combat_log.attack", &locale.0, None);
    let attacked_text = localization.resolve("ui.combat_log.attacked", &locale.0, None);
    let dealt_text = localization.resolve("ui.combat_log.dealt", &locale.0, None);
    let damage_text = localization.resolve("ui.combat_log.damage", &locale.0, None);

    for msg in damage_reader.read() {
        bevy::log::debug!(
            target: "combat_log",
            entity = ?msg.target,
            attacker = %msg.attacker_name,
            target_name = %msg.target_name,
            amount = msg.amount,
            "伤害已应用"
        );
        let skill_label = if msg.is_skill {
            &skill_text
        } else {
            &attack_text
        };
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
                text: attacked_text.clone(),
                color: log_color::NORMAL,
            },
            LogSegment {
                text: format!("[{}]", msg.target_name),
                color: faction_log_color(msg.target_faction),
            },
            LogSegment {
                text: dealt_text.clone(),
                color: log_color::NORMAL,
            },
            LogSegment {
                text: format!("[{}]", msg.amount),
                color: log_color::DAMAGE,
            },
            LogSegment {
                text: damage_text.clone(),
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
        bevy::log::debug!(
            target: "combat_log",
            entity = ?msg.target,
            target_name = %msg.target_name,
            amount = msg.amount,
            "治疗已应用"
        );
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
    localization: Res<LocalizationService>,
    locale: Res<CurrentLocale>,
) {
    let defeated_text = localization.resolve("ui.combat_log.defeated", &locale.0, None);
    for msg in died_reader.read() {
        bevy::log::debug!(
            target: "combat_log",
            entity = ?msg.entity,
            name = %msg.name,
            faction = ?msg.faction,
            "角色已死亡"
        );
        combat_log.push(vec![
            LogSegment {
                text: format!("[{}]", msg.name),
                color: faction_log_color(msg.faction),
            },
            LogSegment {
                text: defeated_text.clone(),
                color: log_color::KILL,
            },
        ]);
    }
}

/// 响应晕眩消息：写入战斗日志
pub fn on_stun_applied(
    mut stun_reader: MessageReader<StunApplied>,
    mut combat_log: ResMut<CombatLog>,
    localization: Res<LocalizationService>,
    locale: Res<CurrentLocale>,
) {
    let stunned_text = localization.resolve("ui.combat_log.stunned", &locale.0, None);
    for msg in stun_reader.read() {
        bevy::log::debug!(target: "combat_log", entity = ?msg.target, target_name = %msg.target_name, "晕眩已应用");
        combat_log.push(vec![
            LogSegment {
                text: format!("[{}]", msg.target_name),
                color: log_color::NORMAL,
            },
            LogSegment {
                text: stunned_text.clone(),
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
        bevy::log::debug!(
            target: "combat_log",
            entity = ?msg.target,
            target_name = %msg.target_name,
            amount = msg.amount,
            "DoT已应用"
        );
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
        bevy::log::debug!(
            target: "combat_log",
            entity = ?msg.target,
            target_name = %msg.target_name,
            amount = msg.amount,
            "HoT已应用"
        );
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

/// 响应装备穿戴消息：写入战斗日志
pub fn on_item_equipped(
    mut reader: MessageReader<ItemEquipped>,
    mut combat_log: ResMut<CombatLog>,
) {
    for msg in reader.read() {
        combat_log.push(vec![
            LogSegment {
                text: format!("装备 [{}]", msg.def_id),
                color: log_color::HEAL,
            },
            LogSegment {
                text: format!(" 已装备到 {}", msg.slot.label()),
                color: log_color::NORMAL,
            },
        ]);
    }
}

/// 响应装备脱卸消息：写入战斗日志
pub fn on_item_unequipped(
    mut reader: MessageReader<ItemUnequipped>,
    mut combat_log: ResMut<CombatLog>,
) {
    for msg in reader.read() {
        combat_log.push(vec![
            LogSegment {
                text: format!("卸下 [{}]", msg.def_id),
                color: log_color::NORMAL,
            },
            LogSegment {
                text: format!(" 从 {}", msg.slot.label()),
                color: log_color::NORMAL,
            },
        ]);
    }
}
