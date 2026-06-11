// Damage Breakdown Viewer：伤害来源分解面板
// 遵循铁律：关键系统必须拥有可视化观察窗口
// 数据源：BattleRecord（结构化记录），不读取 CombatLog

use crate::battle::{BattleEntry, BattleRecord, DamageBreakdown};
use crate::character::UnitName;
use bevy::prelude::*;

/// 渲染伤害分解面板内容（由 mod.rs 的条件渲染系统调用）
pub fn render_damage_panel(
    ui: &mut bevy_inspector_egui::egui::Ui,
    battle_record: &BattleRecord,
    _units: &Query<&UnitName>,
) {
    // 提取最近 20 条伤害记录
    let damage_entries: Vec<_> = battle_record
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
                Some((
                    *target,
                    target_name.clone(),
                    attacker_name.clone(),
                    *amount,
                    *is_skill,
                    breakdown.clone(),
                ))
            } else {
                None
            }
        })
        .take(20)
        .collect();

    if damage_entries.is_empty() {
        ui.label("暂无伤害记录");
        return;
    }

    for (idx, (target, target_name, attacker_name, amount, is_skill, breakdown)) in
        damage_entries.iter().enumerate()
    {
        let skill_label = if *is_skill { " [技能]" } else { "" };
        let header = format!(
            "#{} {} → {}  伤害:{}{}",
            idx + 1,
            attacker_name,
            target_name,
            amount,
            skill_label
        );

        bevy_inspector_egui::egui::CollapsingHeader::new(&header)
            .default_open(false)
            .show(ui, |ui| {
                if let Some(bd) = breakdown {
                    render_breakdown(ui, bd);
                } else {
                    ui.label("  (无分解数据)");
                    ui.label(format!("  最终伤害: {}", amount));
                }
                let _ = target;
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
