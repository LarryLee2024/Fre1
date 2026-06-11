// Battle Debugger：战斗状态快照面板（F1）
// 遵循铁律：关键系统必须拥有可视化观察窗口
// 显示：回合号、当前行动单位、已发生事件数

use crate::battle::{BattleEntry, BattleRecord};
use crate::character::{Faction, Unit, UnitName};
use crate::turn::TurnOrder;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContext;

/// 战斗状态调试面板（由 mod.rs 的条件渲染系统调用）
pub fn battle_debugger_system_inner(
    mut egui_ctx: Query<&mut EguiContext, With<bevy::window::PrimaryWindow>>,
    battle_record: Res<BattleRecord>,
    turn_order: Res<TurnOrder>,
    units: Query<(&UnitName, &Unit)>,
    _visible: bool,
) {
    let Ok(mut ctx) = egui_ctx.single_mut() else {
        return;
    };
    let ctx = ctx.get_mut();

    bevy_inspector_egui::egui::Window::new("Battle Debugger")
        .default_pos([10.0, 10.0])
        .default_size([260.0, 200.0])
        .show(ctx, |ui| {
            // 回合信息
            ui.label(format!("回合: {}", turn_order.turn_number));

            // 当前行动单位
            if let Some(current) = turn_order.current_unit() {
                if let Ok((name, unit)) = units.get(current) {
                    let faction = match unit.faction {
                        Faction::Player => "[友]",
                        Faction::Enemy => "[敌]",
                    };
                    ui.label(format!(
                        "当前行动: {}{} (e:{})",
                        faction,
                        name.0,
                        current.index()
                    ));
                } else {
                    ui.label(format!("当前行动: Entity({})", current.index()));
                }
            } else {
                ui.label("当前行动: (无)");
            }

            // 队列进度
            if !turn_order.queue.is_empty() {
                ui.label(format!(
                    "队列进度: {}/{}",
                    turn_order.current_index + 1,
                    turn_order.queue.len()
                ));
            }

            ui.separator();

            // 事件统计
            let mut damage_count = 0u32;
            let mut heal_count = 0u32;
            let mut death_count = 0u32;
            let mut dot_count = 0u32;

            for entry in &battle_record.entries {
                match entry {
                    BattleEntry::DamageApplied { .. } => damage_count += 1,
                    BattleEntry::HealApplied { .. } => heal_count += 1,
                    BattleEntry::CharacterDied { .. } => death_count += 1,
                    BattleEntry::DotApplied { .. } => dot_count += 1,
                    _ => {}
                }
            }

            ui.label(format!("总事件数: {}", battle_record.entries.len()));
            ui.label(format!("  伤害: {}  治疗: {}", damage_count, heal_count));
            ui.label(format!("  DoT: {}  死亡: {}", dot_count, death_count));
        });
}
