//! 遮蔽度命中修正 — 纯函数
//!
//! 根据格子的遮蔽度为远程攻击/目标选择提供命中修正值。
//!
//! 详见 docs/02-domain/domains/terrain_domain.md §1 (Concealment 分类)

use crate::core::domains::terrain::components::Concealment;

/// 根据遮蔽度返回命中修正值。
///
/// 返回值将被 Combat/Targeting 领域用于修正远程攻击命中率。
///
/// # 返回值
/// - `Concealment::None` → 0（无修正）
/// - `Concealment::Half` → -2（半遮蔽：-2 命中）
/// - `Concealment::Full` → i32::MIN（全遮蔽：不可作为目标）
pub fn concealment_bonus(concealment: &Concealment) -> i32 {
    match concealment {
        Concealment::None => 0,
        Concealment::Half => -2,
        Concealment::Full => i32::MIN,
    }
}

/// 检查给定遮蔽度是否允许目标被选择。
///
/// Full 遮蔽的格子不允许被选为攻击/技能目标。
pub fn is_targetable(concealment: &Concealment) -> bool {
    !matches!(concealment, Concealment::Full)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_concealment_no_penalty() {
        assert_eq!(concealment_bonus(&Concealment::None), 0);
    }

    #[test]
    fn half_concealment_minus_two() {
        assert_eq!(concealment_bonus(&Concealment::Half), -2);
    }

    #[test]
    fn full_concealment_is_untargetable() {
        assert!(!is_targetable(&Concealment::Full));
    }

    #[test]
    fn none_and_half_are_targetable() {
        assert!(is_targetable(&Concealment::None));
        assert!(is_targetable(&Concealment::Half));
    }
}
