//! 营地/休息业务规则 — 纯函数
//!
//! 包括短休/长休规则、安全要求、中断规则、生命骰规则。
//! 详见 docs/02-domain/domains/camp_rest_domain.md §3, §5

use bevy::prelude::*;

use super::super::components::{HitDicePool, RestPhase, RestState, RestType};

/// 检查是否可以开始短休（不变量 3.2：安全区域/非战斗状态）。
///
/// # 参数
/// - `in_combat`: 是否在战斗中
/// - `in_safe_area`: 是否在安全区域
///
/// # 返回值
/// `Ok(())` 或错误描述。
pub fn can_short_rest(in_combat: bool, in_safe_area: bool) -> Result<(), String> {
    if in_combat {
        return Err("战斗状态中无法短休".to_string());
    }
    if !in_safe_area {
        return Err("不在安全区域，无法短休".to_string());
    }
    Ok(())
}

/// 检查是否可以开始长休（不变量 3.1 + 3.3）。
///
/// # 参数
/// - `in_combat`: 是否在战斗中
/// - `in_safe_area`: 是否在安全区域（有营地/避难所）
/// - `last_long_rest_frame`: 上次长休完成的帧计数
/// - `current_frame`: 当前帧计数
/// - `frames_per_day`: 游戏中一天的帧数
///
/// # 返回值
/// `Ok(())` 或错误描述。
pub fn can_long_rest(
    in_combat: bool,
    in_safe_area: bool,
    last_long_rest_frame: Option<u64>,
    current_frame: u64,
    frames_per_day: u64,
) -> Result<(), String> {
    if in_combat {
        return Err("战斗状态中无法长休".to_string());
    }
    if !in_safe_area {
        return Err("不在安全区域，无法长休（需要营地或避难所）".to_string());
    }
    // 不变量 3.1：24 小时内最多 1 次长休
    if let Some(last) = last_long_rest_frame {
        let elapsed = current_frame.saturating_sub(last);
        if elapsed < frames_per_day {
            return Err(format!(
                "24 小时内已进行过长休，还需 {} 帧",
                frames_per_day - elapsed
            ));
        }
    }
    Ok(())
}

/// 应用短休效果（纯函数）。
///
/// 不变量 3.2：短休只能在安全区域/非战斗状态。
///
/// # 参数
/// - `hp_per_die`: 每个生命骰的恢复量
/// - `dice_to_spend`: 要消耗的生命骰数量
/// - `current_hp`: 当前 HP
/// - `max_hp`: 最大 HP
///
/// # 返回值
/// `(new_hp, actual_dice_used, actual_healing)`。
pub fn apply_short_rest_healing(
    hp_per_die: u32,
    dice_to_spend: u32,
    current_hp: u32,
    max_hp: u32,
) -> (u32, u32, u32) {
    let max_heal = max_hp.saturating_sub(current_hp);
    let total_healing_raw = hp_per_die * dice_to_spend;
    let actual_healing = total_healing_raw.min(max_heal);
    let dice_needed = if hp_per_die > 0 {
        actual_healing.div_ceil(hp_per_die)
    } else {
        0
    };
    let actual_dice = dice_needed.min(dice_to_spend);
    let new_hp = current_hp + actual_healing;
    (new_hp, actual_dice, actual_healing)
}

/// 检查长休中断是否导致失败（不变量 3.5：中断累计超 1 小时 = 失败）。
///
/// # 参数
/// - `interrupt_duration_minutes`: 累计中断分钟数
///
/// # 返回值
/// `true` 如果中断超过 1 小时（60 分钟）。
pub fn is_long_rest_interrupted(interrupt_duration_minutes: u32) -> bool {
    interrupt_duration_minutes >= 60
}

/// 应用长休效果（恢复全部 HP）。
///
/// 不变量 3.4：生命骰恢复上限为 ceil(等级/2)。
///
/// # 参数
/// - `max_hp`: 最大 HP
///
/// # 返回值
/// 恢复后的 HP。
pub fn apply_long_rest_hp_recovery(max_hp: u32) -> u32 {
    max_hp
}

/// 验证生命骰消耗请求。
///
/// # 参数
/// - `pool`: 生命骰池
/// - `requested`: 请求消耗的数量
///
/// # 返回值
/// `Ok(())` 或错误描述。
pub fn validate_hit_dice_spend(pool: &HitDicePool, requested: u32) -> Result<(), String> {
    if requested == 0 {
        return Err("必须消耗至少 1 个生命骰".to_string());
    }
    if requested > pool.current {
        return Err(format!(
            "生命骰不足（需要 {}, 可用 {}）",
            requested, pool.current
        ));
    }
    Ok(())
}

/// 检查角色是否处于可进行休息的状态。
///
/// # 参数
/// - `rest_state`: 当前休息状态
///
/// # 返回值
/// `Ok(())` 或错误描述。
pub fn can_start_rest(rest_state: &RestState) -> Result<(), String> {
    if rest_state.phase.is_resting() {
        return Err("已在休息中".to_string());
    }
    if rest_state.phase == RestPhase::Complete {
        return Err("本次休息已完成，请重置状态".to_string());
    }
    Ok(())
}

/// 检查是否可以触发营地事件。
///
/// # 参数
/// - `rest_state`: 当前休息状态
///
/// # 返回值
/// `true` 如果处于轻活动阶段（可触发营地事件）。
pub fn can_trigger_camp_event(rest_state: &RestState) -> bool {
    rest_state.rest_type == Some(RestType::LongRest) && rest_state.phase == RestPhase::LightActivity
}
