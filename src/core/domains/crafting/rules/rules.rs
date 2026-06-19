//! 制作/锻造业务规则 — 纯函数
//!
//! 包括配方检查、附魔规则、升级规则。
//! 详见 docs/02-domain/domains/crafting_domain.md §5

use super::super::components::{
    CraftingStation, EnchantmentDef, EnchantmentSlot, MaterialCost, RecipeDef, UpgradeLevel,
};

// ─── 配方检查规则 ──────────────────────────────────────────────

/// 检查材料是否充足。
///
/// 不变量 3.1：材料充足性。
pub fn check_materials_available(
    recipe: &RecipeDef,
    inventory_fn: &impl Fn(&str) -> u32,
) -> Result<(), String> {
    for material in &recipe.materials {
        let available = inventory_fn(&material.item_id);
        if available < material.quantity {
            return Err(format!(
                "材料不足：需要 {}x {}，仅有 {}",
                material.quantity, material.item_id, available
            ));
        }
    }
    Ok(())
}

/// 检查制作台是否匹配。
///
/// 不变量 3.2：制作台匹配。
pub fn check_station_match(recipe: &RecipeDef, actual_station: CraftingStation) -> bool {
    recipe.station == actual_station
}

/// 检查技能是否满足。
///
/// 技能检定：d20 + skill_bonus >= DC
pub fn check_skill_requirement(recipe: &RecipeDef, skill_bonus: i32, die_result: i32) -> bool {
    match &recipe.skill_requirement {
        Some(req) => {
            let total = die_result + skill_bonus;
            total >= req.dc as i32
        }
        None => true,
    }
}

// ─── 附魔规则 ──────────────────────────────────────────────────

/// 检查是否有空闲的附魔槽位。
///
/// 不变量 3.3：附魔槽位上限制。
pub fn has_free_enchantment_slot(slot: &EnchantmentSlot) -> bool {
    (slot.active_enchants.len() as u32) < slot.max_slots
}

/// 检查附魔词条是否互斥。
///
/// 不变量 3.5：互斥词条防冲突。
pub fn check_enchant_exclusivity(
    enchant: &EnchantmentDef,
    existing_enchants: &[String],
    enchantment_defs: &[EnchantmentDef],
) -> Option<usize> {
    let Some(ref new_group) = enchant.exclusive_group else {
        return None; // 无互斥组
    };
    for (i, existing_id) in existing_enchants.iter().enumerate() {
        if let Some(existing_ench) = enchantment_defs.iter().find(|e| e.id == *existing_id)
            && let Some(ref existing_group) = existing_ench.exclusive_group
            && existing_group == new_group
        {
            return Some(i); // 返回互斥词条的位置
        }
    }
    None
}

/// 检查升级等级上限。
///
/// 不变量 3.4：升级等级上限。
pub fn check_upgrade_limit(level: &UpgradeLevel) -> bool {
    level.can_upgrade()
}

// ─── 制作结果规则 ──────────────────────────────────────────────

/// 技能检定。
pub fn perform_skill_check(skill_bonus: i32, _dc: u32, _die_max: u32) -> bool {
    // 简化实现：使用 skill_bonus 判断（完整实现需要 RNG）
    skill_bonus > 0
}

/// 计算制作失败时保留的材料。
pub fn calc_fail_materials_lost(materials: &[MaterialCost], retention: f32) -> Vec<(String, u32)> {
    materials
        .iter()
        .map(|m| {
            let lost = (m.quantity as f32 * (1.0 - retention)).ceil() as u32;
            (m.item_id.clone(), lost)
        })
        .filter(|(_, qty)| *qty > 0)
        .collect()
}
