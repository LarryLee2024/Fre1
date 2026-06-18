//! 背包/物品业务规则 — 纯函数
//!
//! 包括堆叠规则、槽位规则、负重计算、装备条件检查。
//! 详见 docs/02-domain/domains/inventory_domain.md §3, §5

/// 默认最大堆叠数。
pub const DEFAULT_MAX_STACK: u32 = 99;

/// 默认负重系数（力量 × 该值 = 最大负重磅数）。
pub const WEIGHT_MULTIPLIER: f32 = 15.0;

/// 默认背包最大格数。
pub const DEFAULT_MAX_SLOTS: u32 = 20;

/// 检查物品是否可以堆叠到已有堆叠中。
///
/// # 参数
/// - `current_quantity`: 当前堆叠数量
/// - `max_stack`: 最大堆叠数
///
/// # 返回值
/// `true` 如果当前数量未达到最大堆叠。
pub fn can_stack(current_quantity: u32, max_stack: u32) -> bool {
    current_quantity < max_stack
}

/// 计算可以添加到现有堆叠的数量。
///
/// # 参数
/// - `current_quantity`: 当前堆叠数量
/// - `max_stack`: 最大堆叠数
/// - `incoming_quantity`: 要添加的数量
///
/// # 返回值
/// 实际可以添加的数量。
pub fn stackable_amount(current_quantity: u32, max_stack: u32, incoming_quantity: u32) -> u32 {
    let space = max_stack.saturating_sub(current_quantity);
    space.min(incoming_quantity)
}

/// 计算负重上限。
///
/// # 参数
/// - `strength`: 力量属性值
///
/// # 返回值
/// 最大负重（磅）。
pub fn max_weight_from_strength(strength: i32) -> f32 {
    (strength as f32) * WEIGHT_MULTIPLIER
}

/// 检查是否超过负重上限。
///
/// # 参数
/// - `total_weight`: 当前总重量
/// - `max_weight`: 最大负重
/// - `additional_weight`: 附加重量
///
/// # 返回值
/// `true` 如果会超过负重。
pub fn would_exceed_weight(total_weight: f32, max_weight: f32, additional_weight: f32) -> bool {
    total_weight + additional_weight > max_weight
}

/// 检查装备槽位是否兼容（双手武器检查）。
///
/// 不变量 3.2：双手武器同时占用 MainHand + OffHand。
///
/// # 参数
/// - `is_two_handed`: 物品是否为双手武器
/// - `off_hand_free`: 副手槽位是否空闲
///
/// # 返回值
/// `Ok(())` 或 `Err` 描述原因。
pub fn check_slot_compatibility(is_two_handed: bool, off_hand_free: bool) -> Result<(), String> {
    if is_two_handed && !off_hand_free {
        return Err("双手武器需要主手和副手槽位，但副手已被占用".to_string());
    }
    Ok(())
}

/// 计算稀有度对物品基础价格的倍率。
///
/// # 参数
/// - `rarity_index`: 稀有度索引（0=Common, 1=Uncommon, ..., 4=Legendary）
///
/// # 返回值
/// 价格倍率。
pub fn rarity_price_multiplier(rarity_index: u32) -> f32 {
    match rarity_index {
        0 => 1.0,  // Common
        1 => 2.0,  // Uncommon
        2 => 5.0,  // Rare
        3 => 15.0, // VeryRare
        4 => 50.0, // Legendary
        _ => 1.0,
    }
}

/// 检查背包是否有空间容纳新物品。
///
/// # 参数
/// - `current_slots`: 当前已用格数
/// - `max_slots`: 最大格数
/// - `can_stack`: 是否可以堆叠到现有物品（不需要新格子）
///
/// # 返回值
/// `true` 如果有空间。
pub fn has_inventory_space(current_slots: usize, max_slots: u32, can_stack: bool) -> bool {
    if can_stack {
        true // 堆叠不需要新格子
    } else {
        (current_slots as u32) < max_slots
    }
}
