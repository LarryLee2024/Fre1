//! 法术业务规则 — 纯函数
//!
//! 包括施法校验规则、专注规则、升环规则等。
//! 详见 docs/02-domain/domains/spell_domain.md §3, §5

use bevy::prelude::*;

use super::formulas::calc_concentration_dc;
use crate::core::domains::spell::components::{
    Concentration, SaveResult, SaveType, SpellComponents, SpellDef, SpellDefId, SpellLevel,
    SpellSlotPool,
};
use crate::core::domains::spell::error::SpellError;

// ─── 施法前置校验 ──────────────────────────────────────────────────

/// 检查法术是否在法术书中已习得。
pub fn check_spell_known(
    known_spells: &[SpellDefId],
    spell_id: &SpellDefId,
) -> Result<(), SpellError> {
    if known_spells.contains(spell_id) {
        Ok(())
    } else {
        Err(SpellError::SpellNotKnown {
            spell_id: spell_id.clone(),
        })
    }
}

/// 检查法术是否已准备。
pub fn check_spell_prepared(
    prepared_spells: &[SpellDefId],
    spell_id: &SpellDefId,
) -> Result<(), SpellError> {
    if prepared_spells.contains(spell_id) {
        Ok(())
    } else {
        Err(SpellError::SpellNotPrepared {
            spell_id: spell_id.clone(),
        })
    }
}

/// 检查施法组件可用性。
///
/// 不变量 3.3：施法前必须检查语言/姿势/材料三种施法组件的可用性。
///
/// # 参数
/// - `components`: 法术需要的施法组件
/// - `can_speak`: 是否可以说话
/// - `has_free_hand`: 是否可以自由活动一只手
/// - `has_focus`: 是否持有施法法器
///
/// # 返回值
/// `Ok(())` 或对应的错误。
pub fn check_components(
    components: &SpellComponents,
    can_speak: bool,
    has_free_hand: bool,
    has_focus: bool,
) -> Result<(), SpellError> {
    if components.verbal && !can_speak {
        return Err(SpellError::Silenced);
    }
    if components.somatic && !has_free_hand {
        return Err(SpellError::Restrained);
    }
    if let Some(material) = &components.material {
        if !has_focus && material.cost_gold.unwrap_or(0) == 0 {
            // 无消耗材料可通过法器替代
        } else if !has_focus {
            return Err(SpellError::MissingMaterial {
                description: material.description.clone(),
            });
        }
    }
    Ok(())
}

/// 检查法术位是否充足。
///
/// 不变量 3.1：法术位不可透支。
pub fn check_slot_available(
    slot_pool: &SpellSlotPool,
    level: SpellLevel,
    spell_id: &SpellDefId,
) -> Result<(), SpellError> {
    let level_num = level.as_u8();
    if level_num == 0 {
        return Ok(()); // 戏法不消耗法术位
    }
    if slot_pool.remaining(level) > 0 {
        Ok(())
    } else {
        Err(SpellError::InsufficientSlots {
            spell_id: spell_id.clone(),
            required_level: level_num,
        })
    }
}

/// 检查专注冲突。
///
/// 不变量 3.2：同一时间最多只能维持一个专注法术。
pub fn check_concentration(
    current_concentration: Option<&Concentration>,
    _new_spell_id: &SpellDefId,
) -> Result<(), SpellError> {
    if let Some(conc) = current_concentration {
        Err(SpellError::AlreadyConcentrating {
            current_spell: conc.spell_id.clone(),
        })
    } else {
        Ok(())
    }
}

/// 检查升环施法是否合法。
pub fn check_upcast(spell_def: &SpellDef, target_level: SpellLevel) -> Result<(), SpellError> {
    if !spell_def.can_upcast {
        return Err(SpellError::InvalidUpcast {
            spell_id: spell_def.id.clone(),
            target_level: target_level.as_u8(),
        });
    }
    if target_level.as_u8() <= spell_def.level.as_u8() {
        return Err(SpellError::InvalidUpcast {
            spell_id: spell_def.id.clone(),
            target_level: target_level.as_u8(),
        });
    }
    Ok(())
}

// ─── 专注管理规则 ──────────────────────────────────────────────────

/// 专注打断检定。
///
/// 不变量 3.5：专注打断检定 DC = max(base_dc, 所受伤害的一半)。
///
/// # 参数
/// - `concentration`: 当前专注状态
/// - `damage`: 受到的伤害值
/// - `con_save_roll`: 体质豁免检定的骰子结果（d20 + 体质调整值）
/// - `base_dc`: 专注打断基础 DC（通常从 SpellConfig::concentration_base_dc 读取）
///
/// # 返回值
/// `(bool, u32)` — (是否维持专注, 本次检定的 DC)
pub fn concentration_save(
    _concentration: &Concentration,
    damage: u32,
    con_save_roll: i32,
    base_dc: u32,
) -> (bool, u32) {
    let dc = calc_concentration_dc(damage, base_dc);
    let saved = con_save_roll >= dc as i32;
    (saved, dc)
}

/// 检查专注是否因回合结束而到期。
pub fn check_concentration_expiry(concentration: &Concentration) -> bool {
    concentration.elapsed_rounds >= concentration.total_duration
}

// ─── 豁免检定规则 ──────────────────────────────────────────────────

/// 计算豁免检定的总加值。
///
/// `modifier = 属性调整值 + 熟练加值（如擅长）+ 其他加值`
pub fn calc_save_modifier(
    ability_modifier: i32,
    proficient: bool,
    proficiency_bonus: i32,
    other_bonuses: i32,
) -> i32 {
    let prof = if proficient { proficiency_bonus } else { 0 };
    ability_modifier + prof + other_bonuses
}

/// 判定豁免结果。
///
/// - 自然 20 → 必定成功
/// - 自然 1 → 必定失败
/// - 否则：`roll + modifier >= DC` 为成功
pub fn resolve_save(roll: i32, modifier: i32, dc: u32) -> SaveResult {
    if roll == 20 {
        return SaveResult::Success;
    }
    if roll == 1 {
        return SaveResult::Failure;
    }
    if (roll + modifier) >= dc as i32 {
        SaveResult::Success
    } else {
        SaveResult::Failure
    }
}
