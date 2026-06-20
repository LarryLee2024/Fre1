//! Progression Systems — 经验结算、等级晋升、天赋解锁等系统
//!
//! 使用 Bevy Observer 模式处理经验获取、升级检查、属性提升等逻辑。
//! 详见 docs/02-domain/domains/progression_domain.md §5

use bevy::prelude::*;

use crate::core::domains::progression::components::{
    ClassLevels, Experience, LevelProgressionTable, TalentTree,
};
use crate::core::domains::progression::events::{
    ASICompleted, ClassGained, ExperienceGained, LevelUp, TalentUnlocked,
};

/// 经验值不可回退 — 所有经验变更的校验。
///
/// 不变量 3.2：经验值只增不减。
pub(crate) fn enforce_xp_invariant(
    trigger: On<ExperienceGained>,
    mut query: Query<&mut Experience>,
    mut commands: Commands,
    balance: Res<LevelProgressionTable>,
) {
    let ev = trigger.event();
    let Ok(mut xp) = query.get_mut(ev.entity) else {
        tracing::warn!(target: "progression",
            event = "progression.xp_gained.missing_component",
            entity = ?ev.entity,
            "实体 {:?} 没有 Experience 组件，经验获取被忽略",
            ev.entity
        );
        return;
    };

    // 满级时不增加经验（不变量 3.1）
    if xp.is_max_level {
        tracing::warn!(target: "progression",
            event = "progression.xp_gained.max_level",
            entity = ?ev.entity,
            amount = ev.amount,
            "实体 {:?} 已达满级，经验 +{} 被忽略",
            ev.entity, ev.amount
        );
        return;
    }

    let old_level = xp.level;
    xp.add_xp(ev.amount);

    tracing::trace!(target: "progression",
        event = "progression.xp_gained.added",
        entity = ?ev.entity,
        amount = ev.amount,
        total = xp.total_xp_earned,
        level = xp.level,
        "经验增加：实体={:?}, +{}（累计: {}, 等级: {}）",
        ev.entity, ev.amount, xp.total_xp_earned, xp.level
    );

    commands.trigger(ExperienceGained {
        entity: ev.entity,
        amount: ev.amount,
        source: ev.source.clone(),
        total_xp: xp.total_xp_earned,
        current_level: xp.level,
    });

    // 检查是否触发升级
    let next_threshold = balance.xp_for_level(xp.level + 1);
    if xp.can_level_up(next_threshold) {
        // 触发升级流程 — 发布 LevelUp 事件
        commands.trigger(LevelUp {
            entity: ev.entity,
            old_level,
            new_level: xp.level + 1,
            class_id: crate::core::domains::progression::components::ClassId::new("default"),
            is_asi_level: crate::core::domains::progression::rules::formulas::is_asi_level(
                xp.level + 1,
            ),
        });
    }
}

/// 升级处理系统。
///
/// 处理 LevelUp 事件：扣除经验、增加等级、更新 ClassLevels、检查 ASI。
pub(crate) fn handle_level_up(
    trigger: On<LevelUp>,
    mut query: Query<(&mut Experience, Option<&mut ClassLevels>)>,
    mut commands: Commands,
    balance: Res<LevelProgressionTable>,
) {
    let ev = trigger.event();
    let Ok((mut xp, class_levels)) = query.get_mut(ev.entity) else {
        tracing::warn!(target: "progression",
            event = "progression.level_up.missing_component",
            entity = ?ev.entity,
            "LevelUp: 实体 {:?} 没有 Experience 组件，升级失败",
            ev.entity
        );
        return;
    };

    let xp_cost = balance.xp_for_level(ev.new_level);
    xp.apply_level_up(xp_cost);

    // 更新 ClassLevels
    if let Some(mut cls) = class_levels {
        cls.advance_class(ev.class_id.clone());
        commands.trigger(ClassGained {
            entity: ev.entity,
            class_id: ev.class_id.clone(),
            new_level: cls.level_in_class(&ev.class_id),
        });
    }

    // 检查 ASI
    if ev.is_asi_level {
        commands.trigger(ASICompleted {
            entity: ev.entity,
            level: ev.new_level,
            choices: Vec::new(),
        });
    }
}

/// 天赋解锁系统。
///
/// 处理 TalentUnlocked 事件：更新天赋树状态。
/// 注意：调用方必须确保前置条件已检查（不变量 3.3）。
pub(crate) fn on_talent_unlocked(
    trigger: On<TalentUnlocked>,
    mut query: Query<&mut TalentTree>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    let Ok(mut tree) = query.get_mut(ev.entity) else {
        tracing::warn!(target: "progression",
            event = "progression.talent_unlocked.missing_component",
            entity = ?ev.entity,
            "TalentUnlocked: 实体 {:?} 没有 TalentTree 组件，天赋解锁失败",
            ev.entity
        );
        return;
    };

    let talent_id = crate::core::domains::progression::components::TalentId::new(&ev.talent_id);
    tree.unlock(talent_id);

    commands.trigger(TalentUnlocked {
        entity: ev.entity,
        talent_id: ev.talent_id.clone(),
    });
}

/// 满级检查系统。
///
/// 定期检查所有 Experience 组件，确保满级实体的 is_max_level 标志正确。
pub(crate) fn check_max_level_system(mut query: Query<&mut Experience>) {
    for mut xp in query.iter_mut() {
        if xp.level >= 20 && !xp.is_max_level {
            xp.is_max_level = true;
            tracing::debug!(target: "progression",
                event = "progression.max_level_reached",
                "实体达到满级"
            );
        }
    }
}
