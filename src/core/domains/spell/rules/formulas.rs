//! 法术计算公式 — 纯函数
//!
//! 包括豁免 DC 计算、专注打断 DC、升环增益等。
//! 详见 docs/02-domain/domains/spell_domain.md §5.3, §5.4
//!
//! 来源：
//! - D&D 5e SRD §10：豁免 DC = 8 + 熟练 + 属性调整值
//! - D&D 5e SRD §10：专注 DC = max(10, 伤害/2)
//! - ADR-023 §4：升环机制（每升一环固定增益值）

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
/// `DC = max(base_dc, floor(伤害 / 2))`
///
/// D&D 5e 规则：每次受到伤害均需进行体质豁免，DC = max(base_dc, 所受伤害的一半)。
/// `base_dc` 默认值为 10，可通过 `SpellConfig::concentration_base_dc` 配置覆盖。
///
/// # 参数
/// - `damage`: 受到的伤害值
/// - `base_dc`: 专注打断的基础 DC（通常从 SpellConfig 读取）
///
/// # 返回值
/// 专注打断 DC。
pub fn calc_concentration_dc(damage: u32, base_dc: u32) -> u32 {
    base_dc.max(damage / 2)
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

/// 根据角色等级获取熟练加值。
///
/// 委托给 `LevelProgressionTable::proficiency_bonus` 查表（1-20 级），
/// 越界时返回默认值 +2，消除与 progression 域的硬编码重复。
pub fn proficiency_bonus_for_level(level: u8) -> i32 {
    match level {
        1..=20 => crate::core::domains::progression::LevelProgressionTable::default()
            .proficiency_bonus(level as u32),
        _ => 2,
    }
}
