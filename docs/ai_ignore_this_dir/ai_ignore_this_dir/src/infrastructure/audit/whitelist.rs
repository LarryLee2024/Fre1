/// 事件白名单管理（ADR-006）
///
/// 所有正式领域事件必须收录在白名单中。
/// 新增事件必须先更新白名单，防止为临时副作用随意新增领域事件。
use std::collections::HashSet;

use bevy::prelude::Resource;

/// 白名单条目
#[derive(Debug, Clone)]
pub struct WhitelistEntry {
    /// 事件类型名称
    pub event_type: &'static str,
    /// 事件来源领域
    pub domain: &'static str,
    /// 事件描述
    pub description: &'static str,
}

/// 白名单校验结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhitelistStatus {
    /// 事件在白名单中
    Approved,
    /// 事件不在白名单中（拒绝记录）
    Rejected { reason: String },
}

/// 事件白名单
///
/// ADR-006 §Definition/Instance: 运行时白名单校验器
#[derive(Debug, Resource)]
pub struct EventWhitelist {
    /// 已批准的事件类型集合
    approved: HashSet<&'static str>,
}

impl Default for EventWhitelist {
    fn default() -> Self {
        let mut approved = HashSet::new();

        // ── 战斗事件 ──
        approved.insert("battle_started");
        approved.insert("battle_ended");
        approved.insert("damage_applied");
        approved.insert("heal_applied");
        approved.insert("unit_died");

        // ── 回合事件 ──
        approved.insert("turn_started");
        approved.insert("turn_ended");

        // ── 单位事件 ──
        approved.insert("unit_moved");
        approved.insert("unit_attacked");

        // ── Buff 事件 ──
        approved.insert("buff_applied");
        approved.insert("buff_removed");
        approved.insert("buff_expired");
        approved.insert("stun_applied");
        approved.insert("dot_applied");
        approved.insert("hot_applied");

        // ── 技能事件 ──
        approved.insert("skill_activated");
        approved.insert("skill_cast_started");
        approved.insert("skill_cast_finished");

        // ── 装备事件 ──
        approved.insert("equipment_equipped");
        approved.insert("equipment_unequipped");

        // ── 物品事件 ──
        approved.insert("item_used");
        approved.insert("item_transferred");

        // ── 配置事件 ──
        approved.insert("config_loaded");

        // ── 快照事件 ──
        approved.insert("snapshot_created");

        // ── 关卡事件 ──
        approved.insert("level_completed");

        // ── 任务事件 ──
        approved.insert("quest_accepted");
        approved.insert("quest_completed");

        Self { approved }
    }
}

impl EventWhitelist {
    /// 获取所有白名单事件的完整清单
    pub fn entries() -> Vec<WhitelistEntry> {
        vec![
            // 战斗
            WhitelistEntry {
                event_type: "battle_started",
                domain: "battle",
                description: "战斗开始",
            },
            WhitelistEntry {
                event_type: "battle_ended",
                domain: "battle",
                description: "战斗结束",
            },
            WhitelistEntry {
                event_type: "damage_applied",
                domain: "battle",
                description: "伤害应用",
            },
            WhitelistEntry {
                event_type: "heal_applied",
                domain: "battle",
                description: "治疗应用",
            },
            WhitelistEntry {
                event_type: "unit_died",
                domain: "battle",
                description: "单位死亡",
            },
            // 回合
            WhitelistEntry {
                event_type: "turn_started",
                domain: "turn",
                description: "回合开始",
            },
            WhitelistEntry {
                event_type: "turn_ended",
                domain: "turn",
                description: "回合结束",
            },
            // 单位
            WhitelistEntry {
                event_type: "unit_moved",
                domain: "character",
                description: "单位移动",
            },
            WhitelistEntry {
                event_type: "unit_attacked",
                domain: "battle",
                description: "单位攻击",
            },
            // Buff
            WhitelistEntry {
                event_type: "buff_applied",
                domain: "buff",
                description: "Buff 施加",
            },
            WhitelistEntry {
                event_type: "buff_removed",
                domain: "buff",
                description: "Buff 移除",
            },
            WhitelistEntry {
                event_type: "buff_expired",
                domain: "buff",
                description: "Buff 过期",
            },
            WhitelistEntry {
                event_type: "stun_applied",
                domain: "buff",
                description: "晕眩施加",
            },
            WhitelistEntry {
                event_type: "dot_applied",
                domain: "buff",
                description: "DoT 结算",
            },
            WhitelistEntry {
                event_type: "hot_applied",
                domain: "buff",
                description: "HoT 结算",
            },
            // 技能
            WhitelistEntry {
                event_type: "skill_activated",
                domain: "skill",
                description: "技能激活",
            },
            WhitelistEntry {
                event_type: "skill_cast_started",
                domain: "skill",
                description: "技能施放开始",
            },
            WhitelistEntry {
                event_type: "skill_cast_finished",
                domain: "skill",
                description: "技能施放结束",
            },
            // 装备
            WhitelistEntry {
                event_type: "equipment_equipped",
                domain: "equipment",
                description: "装备穿戴",
            },
            WhitelistEntry {
                event_type: "equipment_unequipped",
                domain: "equipment",
                description: "装备脱卸",
            },
            // 物品
            WhitelistEntry {
                event_type: "item_used",
                domain: "inventory",
                description: "物品使用",
            },
            WhitelistEntry {
                event_type: "item_transferred",
                domain: "inventory",
                description: "物品转移",
            },
            // 配置
            WhitelistEntry {
                event_type: "config_loaded",
                domain: "config",
                description: "配置加载",
            },
            // 快照
            WhitelistEntry {
                event_type: "snapshot_created",
                domain: "core",
                description: "场景快照创建",
            },
            // 关卡
            WhitelistEntry {
                event_type: "level_completed",
                domain: "campaign",
                description: "关卡完成",
            },
            // 任务
            WhitelistEntry {
                event_type: "quest_accepted",
                domain: "campaign",
                description: "任务接受",
            },
            WhitelistEntry {
                event_type: "quest_completed",
                domain: "campaign",
                description: "任务完成",
            },
        ]
    }

    /// 检查事件类型是否在白名单中
    pub fn check(&self, event_type: &str) -> WhitelistStatus {
        if self.approved.contains(event_type) {
            WhitelistStatus::Approved
        } else {
            WhitelistStatus::Rejected {
                reason: format!(
                    "事件类型 '{}' 不在白名单中。新增事件必须先更新 EventWhitelist。",
                    event_type
                ),
            }
        }
    }

    /// 添加事件类型到白名单（用于扩展）
    pub fn register(&mut self, event_type: &'static str) {
        self.approved.insert(event_type);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 白名单_战斗事件通过校验() {
        let whitelist = EventWhitelist::default();
        assert_eq!(whitelist.check("damage_applied"), WhitelistStatus::Approved);
        assert_eq!(whitelist.check("turn_started"), WhitelistStatus::Approved);
    }

    #[test]
    fn 白名单_未注册事件被拒绝() {
        let whitelist = EventWhitelist::default();
        let result = whitelist.check("unknown_event");
        assert!(matches!(result, WhitelistStatus::Rejected { .. }));
    }

    #[test]
    fn 白名单_注册新事件() {
        let mut whitelist = EventWhitelist::default();
        assert!(matches!(
            whitelist.check("custom_event"),
            WhitelistStatus::Rejected { .. }
        ));
        whitelist.register("custom_event");
        assert_eq!(whitelist.check("custom_event"), WhitelistStatus::Approved);
    }

    #[test]
    fn 白名单_entries_返回完整列表() {
        let entries = EventWhitelist::entries();
        assert!(!entries.is_empty());
        // 验证关键事件存在
        assert!(entries.iter().any(|e| e.event_type == "damage_applied"));
        assert!(entries.iter().any(|e| e.event_type == "turn_started"));
        assert!(entries.iter().any(|e| e.event_type == "buff_applied"));
    }
}
