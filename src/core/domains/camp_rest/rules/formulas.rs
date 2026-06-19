//! 休息与生命骰计算公式 — 纯函数
//!
//! 包括生命骰恢复量、HP 恢复量等计算。
//! 详见 docs/02-domain/domains/camp_rest_domain.md §1

use super::super::components::DiceType;

/// 短休时每个生命骰的恢复量。
///
/// # 参数
/// - `dice_type`: 生命骰类型（d6/d8/d10/d12）
/// - `constitution_modifier`: 体质调整值
///
/// # 返回值
/// 每个生命骰恢复的 HP 量（1d[type] + 体质调整值）。
pub fn hit_dice_healing_per_die(dice_type: DiceType, constitution_modifier: i32) -> u32 {
    let base = dice_type.max_value();
    let avg = base.div_ceil(2); // 平均值（取整）
    // 使用平均值而非随机值（确定性实现）
    let healing = avg.saturating_add_signed(constitution_modifier);
    healing.max(1) // 至少恢复 1 HP
}

/// 长休后生命骰最大恢复量。
///
/// 不变量 3.4：恢复上限为 ceil(等级/2)。
///
/// # 参数
/// - `character_level`: 角色等级
///
/// # 返回值
/// 长休后可拥有的最大生命骰数量。
pub fn max_hit_dice_after_long_rest(character_level: u32) -> u32 {
    character_level.div_ceil(2) // ceil(level / 2)
}

/// 长休恢复全部 HP（简化函数）。
///
/// # 参数
/// - `max_hp`: 最大 HP
/// - `current_hp`: 当前 HP
///
/// # 返回值
/// 恢复后的 HP 值（等于 max_hp）。
pub fn long_rest_hp_recovery(max_hp: u32, _current_hp: u32) -> u32 {
    max_hp
}

/// 长休 HP 恢复量（用于事件记录）。
///
/// # 参数
/// - `max_hp`: 最大 HP
/// - `current_hp`: 当前 HP
///
/// # 返回值
/// 实际恢复的 HP 量。
pub fn long_rest_hp_recovered_amount(max_hp: u32, current_hp: u32) -> u32 {
    max_hp.saturating_sub(current_hp)
}

/// 短休 HP 恢复总量。
///
/// # 参数
/// - `dice_type`: 生命骰类型
/// - `constitution_modifier`: 体质调整值
/// - `dice_count`: 消耗的生命骰数量
///
/// # 返回值
/// 恢复的 HP 总量。
pub fn short_rest_total_healing(
    dice_type: DiceType,
    constitution_modifier: i32,
    dice_count: u32,
) -> u32 {
    let per_die = hit_dice_healing_per_die(dice_type, constitution_modifier);
    per_die * dice_count
}

/// 检查 24 小时内是否已进行过长休。
///
/// # 参数
/// - `last_long_rest_frame`: 上次长休完成的帧计数
/// - `current_frame`: 当前帧计数
/// - `frames_per_game_day`: 游戏中一天的帧数（可配置，默认 86400）
///
/// # 返回值
/// `true` 如果可以进行长休。
pub fn can_long_rest_timer(
    last_long_rest_frame: Option<u64>,
    current_frame: u64,
    frames_per_game_day: u64,
) -> bool {
    match last_long_rest_frame {
        None => true,
        Some(last) => {
            let elapsed = current_frame.saturating_sub(last);
            elapsed >= frames_per_game_day
        }
    }
}
