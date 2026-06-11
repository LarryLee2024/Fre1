// Damage Breakdown Viewer：伤害来源分解面板
// 遵循铁律：关键系统必须拥有可视化观察窗口
// 数据源：BattleRecord（结构化记录），不读取 CombatLog

use crate::battle::{BattleEntry, BattleRecord, DamageBreakdown};
use crate::character::UnitName;
use bevy::prelude::*;

/// 伤害条目（纯数据，Logic/Presentation 分离）
pub struct DamageEntry {
    pub target: Entity,
    pub target_name: String,
    pub attacker_name: String,
    pub amount: i32,
    pub is_skill: bool,
    pub breakdown: Option<DamageBreakdown>,
}

/// 提取最近 N 条伤害记录（纯函数，可单元测试）
pub fn filter_damage_entries(record: &BattleRecord, limit: usize) -> Vec<DamageEntry> {
    record
        .entries
        .iter()
        .rev()
        .filter_map(|e| {
            if let BattleEntry::DamageApplied {
                target,
                target_name,
                attacker_name,
                amount,
                is_skill,
                breakdown,
                ..
            } = e
            {
                Some(DamageEntry {
                    target: *target,
                    target_name: target_name.clone(),
                    attacker_name: attacker_name.clone(),
                    amount: *amount,
                    is_skill: *is_skill,
                    breakdown: breakdown.clone(),
                })
            } else {
                None
            }
        })
        .take(limit)
        .collect()
}

/// 渲染伤害分解面板内容（由 mod.rs 的条件渲染系统调用）
pub fn render_damage_panel(
    ui: &mut bevy_inspector_egui::egui::Ui,
    battle_record: &BattleRecord,
    _units: &Query<&UnitName>,
) {
    let damage_entries = filter_damage_entries(battle_record, 20);

    if damage_entries.is_empty() {
        ui.label("暂无伤害记录");
        return;
    }

    for (idx, entry) in damage_entries.iter().enumerate() {
        let skill_label = if entry.is_skill { " [技能]" } else { "" };
        let header = format!(
            "#{} {} → {}  伤害:{}{}",
            idx + 1,
            entry.attacker_name,
            entry.target_name,
            entry.amount,
            skill_label
        );

        bevy_inspector_egui::egui::CollapsingHeader::new(&header)
            .default_open(false)
            .show(ui, |ui| {
                if let Some(bd) = &entry.breakdown {
                    render_breakdown(ui, bd);
                } else {
                    ui.label("  (无分解数据)");
                    ui.label(format!("  最终伤害: {}", entry.amount));
                }
            });
    }
}

fn render_breakdown(ui: &mut bevy_inspector_egui::egui::Ui, bd: &DamageBreakdown) {
    ui.label(format!("  原始效果值: {}", bd.base_amount));

    if !bd.modifiers.is_empty() {
        ui.label("  修饰符:");
        for m in &bd.modifiers {
            let sign = if m.after >= m.before { "+" } else { "" };
            let diff = m.after - m.before;
            ui.label(format!("    {}{} ({})", sign, diff, m.rule_name));
        }
    }

    ui.label(format!("  修饰后伤害: {}", bd.modified_amount));

    if bd.actual_damage != bd.modified_amount {
        let diff = bd.actual_damage - bd.modified_amount;
        let sign = if diff >= 0 { "+" } else { "" };
        ui.label(format!(
            "  实际扣血: {} ({}{})",
            bd.actual_damage, sign, diff
        ));
    } else {
        ui.label(format!("  实际扣血: {}", bd.actual_damage));
    }
}
