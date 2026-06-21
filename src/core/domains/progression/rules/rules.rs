//! 成长业务规则 — 纯函数
//!
//! 包括经验规则、升级判定、天赋前置检查、ASI 规则等。
//! 详见 docs/02-domain/domains/progression_domain.md §3, §5

use super::formulas::{MAX_LEVEL, cumulative_xp_for_level, xp_to_next_level};

/// 检查角色是否可以升级。
///
/// # 规则
/// - 不变量 3.1：等级不得超过 20
/// - 不变量 3.2：经验足够升至下一级
///
/// # 参数
/// - `current_level`: 当前等级
/// - `current_xp`: 当前累积的经验值
///
/// # 返回值
/// `true` 如果条件满足。
pub fn can_level_up(current_level: u32, current_xp: u64) -> bool {
    if current_level >= MAX_LEVEL {
        return false;
    }
    let required = cumulative_xp_for_level(current_level + 1);
    current_xp >= required
}

/// 计算升级后剩余的经验值。
///
/// # 参数
/// - `current_xp`: 当前累积的经验值
/// - `current_level`: 当前等级
///
/// # 返回值
/// 升级后的剩余经验值（扣除升级所需）。
pub fn xp_after_level_up(current_xp: u64, current_level: u32) -> u64 {
    let required = cumulative_xp_for_level(current_level + 1);
    current_xp.saturating_sub(required)
}

/// 检查是否可以连续升级（经验足够升多级）。
///
/// # 返回值
/// 可以达到的最高等级（不超过 MAX_LEVEL）。
pub fn max_achievable_level(current_xp: u64, current_level: u32) -> u32 {
    let mut level = current_level;
    let mut xp = current_xp;
    while level < MAX_LEVEL {
        let needed = xp_to_next_level(level);
        if xp >= needed {
            xp -= needed;
            level += 1;
        } else {
            break;
        }
    }
    level
}

/// 计算总等级对应的熟练加值（简化函数）。
///
/// 重复 formulas::proficiency_bonus，但作为业务规则层的便捷入口。
pub fn proficiency_for_total_level(total_level: u32) -> i32 {
    super::formulas::proficiency_bonus(total_level)
}

/// 检查天赋解锁的前置条件是否满足。
///
/// # 参数
/// - `required_level`: 前置等级要求
/// - `current_level`: 当前角色等级
/// - `required_talents`: 前置天赋 ID 列表
/// - `unlocked_talents`: 已解锁的天赋 ID 列表
///
/// # 返回值
/// `Ok(())` 或带描述的 `Err`。
pub fn check_talent_prerequisites(
    required_level: u32,
    current_level: u32,
    required_talents: &[String],
    unlocked_talents: &[String],
) -> Result<(), String> {
    // 天赋解锁前置条件：等级和前置天赋必须同时满足
    if current_level < required_level {
        return Err(format!(
            "需要等级 {}，当前等级 {}",
            required_level, current_level
        ));
    }

    // 前置天赋检查：依赖链中的天赋必须已解锁
    for prereq in required_talents {
        if !unlocked_talents.contains(prereq) {
            return Err(format!("前置天赋 {} 尚未解锁", prereq));
        }
    }

    Ok(())
}

/// 检查是否可以开始新职业（多职业前置条件）。
///
/// # 参数
/// - `current_total_level`: 当前总等级
/// - `attribute_checks`: 属性检查列表 `(属性名, 需求值, 实际值)`
///
/// # 返回值
/// `Ok(())` 或带描述的 `Err`。
pub fn check_multiclass_prerequisites(
    current_total_level: u32,
    attribute_checks: &[(&str, i32, i32)],
) -> Result<(), String> {
    if current_total_level >= MAX_LEVEL {
        return Err("已满级，无法开始新职业".to_string());
    }

    for (attr_name, required, actual) in attribute_checks {
        if *actual < *required {
            return Err(format!(
                "开始新职业需要 {} >= {}，当前 {}",
                attr_name, required, actual
            ));
        }
    }

    Ok(())
}

/// 检查 ASI 属性提升是否有效（不变量 3.5：不可跳过，属性上限 20）。
///
/// # 参数
/// - `current_value`: 当前属性值
/// - `increase`: 提升量
///
/// # 返回值
/// `Ok(new_value)` 或 `Err`。
pub fn check_asi_attribute_increase(current_value: i32, increase: i32) -> Result<i32, String> {
    let new_value = current_value + increase;
    if new_value > 20 {
        return Err(format!(
            "属性值不能超过 20（当前 {}，尝试 +{}）",
            current_value, increase
        ));
    }
    if new_value < current_value {
        return Err("属性值不能降低".to_string());
    }
    Ok(new_value)
}

/// 验证经验获取（不变量 3.2：经验只增不减）。
///
/// # 参数
/// - `amount`: 新增经验量
///
/// # 返回值
/// `Ok(())` 或 `Err`（只有 amount 为 0 时可能警告，但允许）。
pub fn validate_xp_gain(amount: u64) -> Result<(), String> {
    if amount == 0 {
        return Err("经验增加量不能为 0".to_string());
    }
    Ok(())
}
