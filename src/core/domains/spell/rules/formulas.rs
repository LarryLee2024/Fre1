//! 法术计算公式 — 纯函数
//!
//! 包括豁免 DC 计算、专注打断 DC、升环增益等。
//! 详见 docs/02-domain/domains/spell_domain.md §5.3, §5.4

// ─── 豁免 DC 计算 ──────────────────────────────────────────────────

/// 计算法术豁免 DC。
///
/// # 公式
/// `DC = 8 + 熟练加值 + 施法属性调整值 + 其他加值`
///
/// # 参数
/// - `proficiency_bonus`: 熟练加值（通常为 2-6）
/// - `casting_modifier`: 施法属性调整值（-1 到 +5）
/// - `other_bonuses`: 额外的 DC 加值（如魔法装备）
///
/// # 返回值
/// 豁免 DC 值。
pub fn calc_save_dc(proficiency_bonus: i32, casting_modifier: i32, other_bonuses: i32) -> u32 {
    (8 + proficiency_bonus + casting_modifier + other_bonuses).max(1) as u32
}

/// 计算专注打断检定 DC。
///
/// # 公式
/// `DC = max(10, floor(伤害 / 2))`
///
/// D&D 5e 规则：每次受到伤害均需进行体质豁免，DC = max(10, 所受伤害的一半)。
///
/// # 参数
/// - `damage`: 受到的伤害值
///
/// # 返回值
/// 专注打断 DC。
pub fn calc_concentration_dc(damage: u32) -> u32 {
    (10u32).max(damage / 2)
}

/// 计算升环施法时的额外效果数值。
///
/// # 公式（简化版）
/// `scale = upcast_level - base_level`
///
/// # 参数
/// - `base_level`: 法术基础环级
/// - `upcast_level`: 施放时使用的环级
/// - `per_level_bonus`: 每升一环的增益值
///
/// # 返回值
/// 总增益值。
pub fn calc_upcast_bonus(base_level: u8, upcast_level: u8, per_level_bonus: i32) -> i32 {
    if upcast_level > base_level {
        let levels = (upcast_level - base_level) as i32;
        levels * per_level_bonus
    } else {
        0
    }
}

// ─── 熟练加值表 ──────────────────────────────────────────────────

/// 根据角色等级获取熟练加值（D&D 5e 标准表）。
///
/// | 等级范围 | 熟练加值 |
/// |----------|---------|
/// | 1-4      | +2      |
/// | 5-8      | +3      |
/// | 9-12     | +4      |
/// | 13-16    | +5      |
/// | 17-20    | +6      |
pub fn proficiency_bonus_for_level(level: u8) -> i32 {
    match level {
        1..=4 => 2,
        5..=8 => 3,
        9..=12 => 4,
        13..=16 => 5,
        17..=20 => 6,
        _ => 2, // 默认 1 级
    }
}
