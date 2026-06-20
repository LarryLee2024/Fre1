//! 经验与成长计算公式 — 纯函数
//!
//! 包括升级经验曲线、熟练加值计算、属性成长率等。
//! 详见 docs/02-domain/domains/progression_domain.md §1
//!
//! 来源：
//! - D&D 5e SRD §4-5：升级经验表与熟练加值表
//! - ADR-030 §3：经验曲线设计决策
//! - 等级上限 20、ASI 等级 [4,8,12,16,19] 为 D&D 5e 标准

use super::super::components::LevelProgressionTable;

/// D&D 5e 默认经验表（累计 XP）。
///
/// 索引 0 = 到 2 级需要的累计经验值。
pub(crate) const DEFAULT_XP_THRESHOLDS: [u64; 20] = [
    0, 300, 900, 2700, 6500, 14000, 23000, 34000, 48000, 64000, 85000, 100000, 120000, 140000,
    165000, 195000, 225000, 265000, 305000, 355000,
];

/// D&D 5e 默认熟练加值表。
///
/// 索引 0 = 1 级。
pub(crate) const DEFAULT_PROFICIENCY_BONUS: [i32; 20] =
    [2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6];

/// ASI 触发等级（D&D 5e 标准）。
pub(crate) const ASI_LEVELS: [u32; 5] = [4, 8, 12, 16, 19];

/// 等级上限。
pub(crate) const MAX_LEVEL: u32 = 20;

/// 计算从当前等级升到下一级所需的经验值。
///
/// # 参数
/// - `current_level`: 当前等级（1-19）
///
/// # 返回值
/// 升到下一级所需的额外经验值。
/// 如果已是满级，返回 0。
pub fn xp_to_next_level(current_level: u32) -> u64 {
    if current_level >= MAX_LEVEL || current_level == 0 {
        return 0;
    }
    let idx = (current_level - 1) as usize;
    if idx < DEFAULT_XP_THRESHOLDS.len() - 1 {
        DEFAULT_XP_THRESHOLDS[idx + 1] - DEFAULT_XP_THRESHOLDS[idx]
    } else {
        0
    }
}

/// 计算到指定等级所需的累计经验值。
///
/// # 参数
/// - `target_level`: 目标等级（1-20）
///
/// # 返回值
/// 达到该等级所需的累计经验值。
pub fn cumulative_xp_for_level(target_level: u32) -> u64 {
    if target_level <= 1 {
        return 0;
    }
    let idx = (target_level - 1) as usize;
    if idx < DEFAULT_XP_THRESHOLDS.len() {
        DEFAULT_XP_THRESHOLDS[idx]
    } else {
        u64::MAX
    }
}

/// 计算给定总经验值对应的等级。
///
/// # 参数
/// - `total_xp`: 累计获得的总经验值
///
/// # 返回值
/// 对应的等级（1-20）。
pub fn level_from_xp(total_xp: u64) -> u32 {
    let mut level = 1u32;
    for (i, &threshold) in DEFAULT_XP_THRESHOLDS.iter().enumerate() {
        if total_xp >= threshold {
            level = (i + 1) as u32;
        } else {
            break;
        }
    }
    level.min(MAX_LEVEL)
}

/// 计算给定等级的熟练加值。
///
/// # 参数
/// - `level`: 角色总等级（1-20）
///
/// # 返回值
/// 熟练加值（+2 到 +6）。
pub fn proficiency_bonus(level: u32) -> i32 {
    if level == 0 {
        return 2;
    }
    let idx = (level as usize).saturating_sub(1);
    if idx < DEFAULT_PROFICIENCY_BONUS.len() {
        DEFAULT_PROFICIENCY_BONUS[idx]
    } else {
        DEFAULT_PROFICIENCY_BONUS[DEFAULT_PROFICIENCY_BONUS.len() - 1]
    }
}

/// 检查指定等级是否为 ASI 等级。
///
/// # 参数
/// - `level`: 检查的等级
///
/// # 返回值
/// `true` 如果该等级需要选择 ASI 或专长。
pub fn is_asi_level(level: u32) -> bool {
    ASI_LEVELS.contains(&level)
}

/// 计算从 LevelProgressionTable 获取下一个 ASI 等级。
///
/// 如果当前等级已达到或超过最后 ASI 等级，返回当前等级。
pub fn next_asi_level(current_level: u32) -> Option<u32> {
    ASI_LEVELS.iter().copied().find(|&l| l > current_level)
}

/// 获取 ASI 等级列表的引用。
pub fn asi_levels() -> &'static [u32; 5] {
    &ASI_LEVELS
}

/// 使用 LevelProgressionTable 计算升级经验需求。
///
/// 如果当前等级 >= max_level，返回 None。
pub fn xp_requirement_from_table(table: &LevelProgressionTable, current_level: u32) -> Option<u64> {
    if current_level >= table.max_level {
        return None;
    }
    Some(table.xp_range_for_level(current_level + 1).0)
}
