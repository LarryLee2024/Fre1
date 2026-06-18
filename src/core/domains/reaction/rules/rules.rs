//! 反应业务规则 — 纯函数
//!
//! 包括反应可用性校验、优先级计算、机会攻击规则、法术反制判定等。
//! 详见 docs/02-domain/domains/reaction_domain.md §3, §5

use super::super::components::{CounterspellVerdict, ReactionState, ReactionTrigger, ReactionType};

// ─── 反应可用性规则 ──────────────────────────────────────────────────

/// 检查单位是否可以使用反应。
///
/// 不变量 3.1：每个单位每回合最多只能使用 1 次反应（除非有额外反应次数）。
///
/// # 参数
/// - `state`: 单位的反应槽位状态
///
/// # 返回值
/// `true` 表示可以使用反应。
pub fn can_react(state: &ReactionState) -> bool {
    state.can_react()
}

/// 检查反应是否可以在当前时机被触发。
///
/// 不变量 3.2：反应默认在回合外触发。
///
/// # 参数
/// - `is_own_turn`: 当前是否是该单位的己方回合
/// - `reaction_type`: 反应类型
///
/// # 返回值
/// `true` 表示可以在当前时机触发。
pub fn can_trigger_on_turn(is_own_turn: bool, reaction_type: &ReactionType) -> bool {
    if !is_own_turn {
        // 回合外，所有反应都可触发
        return true;
    }
    // 己方回合内，只有特定反应可触发（如被攻击时护盾术）
    matches!(reaction_type, ReactionType::Shield)
}

// ─── 反应优先级计算 ────────────────────────────────────────────────

/// 计算反应的触发优先级。
///
/// 优先级规则（reaction_domain.md §1）：
/// 1. 防御型反应（护盾术/援护格挡）> 进攻型反应（机会攻击/法术反制）
/// 2. 同类型反应：按先攻值从高到低
///
/// # 参数
/// - `reaction_type`: 反应类型
/// - `initiative`: 单位的先攻值
/// - `defense_bonus`: 防御型反应的额外优先级加成
///
/// # 返回值
/// 优先级数值（越高越优先）。
pub fn calc_priority(reaction_type: &ReactionType, initiative: u32, defense_bonus: u32) -> u32 {
    let base = match reaction_type {
        // 防御型：护盾术 > 援护格挡
        ReactionType::Shield => 3000,
        ReactionType::Guardian => 2000,
        // 进攻型：法术反制 > 机会攻击
        ReactionType::Counterspell => 1500,
        ReactionType::OpportunityAttack => 1000,
        // 特殊反应：默认中等优先级
        ReactionType::Special { .. } => 500,
    };
    let defensive_bonus = match reaction_type {
        ReactionType::Shield | ReactionType::Guardian => defense_bonus,
        _ => 0,
    };
    base + defensive_bonus + initiative
}

// ─── 机会攻击规则 ──────────────────────────────────────────────────

/// 检查是否满足机会攻击触发条件。
///
/// 不变量 3.3：机会攻击只在目标单位主动离开己方威胁区时触发。
///
/// # 参数
/// - `trigger`: 触发上下文
/// - `is_forced_movement`: 是否为强制移动（推开、传送等）
///
/// # 返回值
/// `true` 表示可以触发机会攻击。
pub fn can_opportunity_attack(trigger: &ReactionTrigger, is_forced_movement: bool) -> bool {
    if is_forced_movement {
        // 强制移动不触发机会攻击
        return false;
    }
    matches!(trigger, ReactionTrigger::LeaveThreatRange { .. })
}

/// 计算机会攻击的命中判定简化结果。
///
/// # 参数
/// - `attack_bonus`: 攻击者的攻击加值
/// - `target_ac`: 目标的护甲等级
/// - `roll`: d20 骰子结果
///
/// # 返回值
/// `(bool, bool)` — (是否命中, 是否重击)
pub fn resolve_opportunity_attack_hit(
    attack_bonus: i32,
    target_ac: i32,
    roll: i32,
) -> (bool, bool) {
    if roll == 20 {
        (true, true) // 重击
    } else if roll == 1 {
        (false, false) // 必定未命中
    } else {
        let total = roll + attack_bonus;
        (total >= target_ac, false)
    }
}

// ─── 法术反制规则 ──────────────────────────────────────────────────

/// 判定法术反制结果。
///
/// 不变量 3.4：反制法术后，目标法术的环级 > 反制法术的环级时，
/// 需要施法属性检定（DC = 10 + 目标环级 - 反制环级）。
///
/// # 参数
/// - `target_level`: 目标法术的环级（1-9）
/// - `counter_level`: 反制使用的环级
///
/// # 返回值
/// `CounterspellVerdict` 判定结果。
pub fn resolve_counterspell(target_level: u8, counter_level: u8) -> CounterspellVerdict {
    if counter_level >= target_level {
        CounterspellVerdict::AutoSuccess
    } else {
        // DC = 10 + 目标环级 - 反制环级（D&D 5e 规则）
        let dc = 10 + (target_level as u32).saturating_sub(counter_level as u32);
        CounterspellVerdict::CheckRequired { dc, roll: None }
    }
}

/// 执行法术反制的施法属性检定。
///
/// # 参数
/// - `dc`: 检定 DC
/// - `roll`: d20 骰子结果
/// - `modifier`: 施法属性调整值 + 熟练加值
///
/// # 返回值
/// `true` 表示检定通过，反制成功。
pub fn resolve_counterspell_check(dc: u32, roll: i32, modifier: i32) -> bool {
    if roll == 20 {
        return true;
    }
    if roll == 1 {
        return false;
    }
    (roll + modifier) >= dc as i32
}

// ─── 援护规则 ──────────────────────────────────────────────────────

/// 检查援护者是否在目标的相邻格。
///
/// 不变量 3.5：援护者必须在被援护目标的相邻格内。
///
/// # 参数
/// - `guardian_x`: 援护者的 x 坐标
/// - `guardian_y`: 援护者的 y 坐标
/// - `target_x`: 目标的 x 坐标
/// - `target_y`: 目标的 y 坐标
///
/// # 返回值
/// `true` 表示在相邻格内。
pub fn is_adjacent(guardian_x: i32, guardian_y: i32, target_x: i32, target_y: i32) -> bool {
    let dx = (guardian_x - target_x).abs();
    let dy = (guardian_y - target_y).abs();
    // 四连通网格：相邻格要求 max(|dx|, |dy|) == 1
    dx.max(dy) == 1
}

// ─── 护盾术规则 ──────────────────────────────────────────────────

/// 计算护盾术后的 AC，并判定攻击是否仍然命中。
///
/// # 参数
/// - `original_ac`: 原始 AC
/// - `attack_roll`: 攻击检定结果
/// - `shield_bonus`: 护盾术提供的 AC 加值（通常为 5）
///
/// # 返回值
/// `(i32, bool)` — (护盾术后的 AC, 是否仍然命中)
pub fn apply_shield_ac(original_ac: i32, attack_roll: i32, shield_bonus: i32) -> (i32, bool) {
    let boosted_ac = original_ac + shield_bonus;
    let still_hit = attack_roll >= boosted_ac;
    (boosted_ac, still_hit)
}
