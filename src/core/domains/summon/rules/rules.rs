//! 召唤业务规则 — 纯函数
//!
//! 包括位置检查、专注规则、消失规则。
//! 详见 docs/02-domain/domains/summon_domain.md §5

use super::super::components::{SummonBond, SummonSlotManager, SummonTemplateDef};

// ─── 位置规则 ──────────────────────────────────────────────────

/// 检查召唤位置是否可通行。
///
/// 不变量 3.5：占位不冲突。
pub fn is_position_valid(_x: i32, _y: i32, is_passable: bool, is_occupied: bool) -> bool {
    is_passable && !is_occupied
}

// ─── 专注规则 ──────────────────────────────────────────────────

/// 检查是否可以创建专注召唤。
///
/// 不变量 3.2：专注召唤唯一性。
pub fn can_create_concentration_summon(is_already_concentrating: bool) -> bool {
    !is_already_concentrating
}

// ─── 召唤槽位规则 ──────────────────────────────────────────────

/// 检查是否有空闲的召唤槽位。
pub fn has_free_summon_slot(manager: &SummonSlotManager) -> bool {
    manager.has_free_slot()
}

/// 检查召唤者是否存活。
///
/// 不变量 3.1：召唤者生死约束。
pub fn is_caster_alive(caster_alive: bool) -> bool {
    caster_alive
}

// ─── 嵌套召唤规则 ──────────────────────────────────────────────

/// 检查是否允许嵌套召唤。
///
/// 🟥 禁止：嵌套召唤（不变量 3.5 及 §4 禁止事项）。
pub fn can_summon_from_summon(caster_bond: Option<&SummonBond>, allow_nested: bool) -> bool {
    if caster_bond.is_some() {
        return false; // 召唤物不能再召唤
    }
    allow_nested // 非召唤物的 Entity 根据配置决定
}

// ─── 创建规则 ──────────────────────────────────────────────────

/// 检查召唤物创建的所有前置条件。
pub fn can_create_summon(
    template: &SummonTemplateDef,
    manager: &SummonSlotManager,
    is_caster_alive: bool,
    is_concentrating: bool,
    position_passable: bool,
    position_occupied: bool,
    caster_bond: Option<&SummonBond>,
    allow_nested: bool,
) -> Result<(), String> {
    if !is_caster_alive {
        return Err("召唤者已死亡".into());
    }
    if !has_free_summon_slot(manager) {
        return Err("召唤槽位已满".into());
    }
    if template.summon_cost.requires_concentration
        && !can_create_concentration_summon(is_concentrating)
    {
        return Err("已有一个专注召唤".into());
    }
    if !is_position_valid(0, 0, position_passable, position_occupied) {
        return Err("召唤位置不可用".into());
    }
    if !can_summon_from_summon(caster_bond, allow_nested) {
        return Err("禁止嵌套召唤".into());
    }
    Ok(())
}

// ─── 消失规则 ──────────────────────────────────────────────────

/// 召唤物是否应该因召唤者死亡而消失。
pub fn should_expire_on_caster_death() -> bool {
    true // 不变量 3.1
}

/// 召唤物是否应该因专注打断而消失。
pub fn should_expire_on_concentration_broken(template: &SummonTemplateDef) -> bool {
    template.summon_cost.requires_concentration
}
