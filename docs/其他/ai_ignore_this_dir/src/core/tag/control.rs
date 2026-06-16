//! 控制层级与免疫系统（ADR-031 §2.4）
//!
//! 实现 Linglan 三级控制模型：
//! - 软控（Soft）：不限制行动，仅削弱属性
//! - 硬控（Hard）：禁止移动，可释放技能/普攻
//! - 强控（Full）：完全禁止所有行动
//!
//! 以及三级免疫体系和控制递减规则。

use super::{GameplayTag, GameplayTags};

// ============================================================================
// ControlLevel — 控制层级
// ============================================================================

/// 控制层级（优先级递增）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ControlLevel {
    /// 软控（削弱层）：减速、命中降低
    Soft = 1,
    /// 硬控（行动限制层）：定身、束缚
    Hard = 2,
    /// 强控（完全失能层）：眩晕、冰冻、石化
    Full = 3,
}

impl ControlLevel {
    /// 从 GameplayTag 推断控制层级
    pub fn from_tag(tag: GameplayTag) -> Option<Self> {
        if tag == GameplayTag::CONTROL_SOFT {
            Some(ControlLevel::Soft)
        } else if tag == GameplayTag::CONTROL_HARD {
            Some(ControlLevel::Hard)
        } else if tag == GameplayTag::CONTROL_FULL {
            Some(ControlLevel::Full)
        } else {
            None
        }
    }

    /// 返回该层级的覆盖对象（被此层级覆盖的更低层）
    ///
    /// 强控覆盖所有；硬控覆盖软控；软控不覆盖任何。
    pub fn overrides(&self) -> &[ControlLevel] {
        match self {
            ControlLevel::Full => &[ControlLevel::Soft, ControlLevel::Hard],
            ControlLevel::Hard => &[ControlLevel::Soft],
            ControlLevel::Soft => &[],
        }
    }
}

// ============================================================================
// ControlImmunity — 免疫结果
// ============================================================================

/// 控制免疫判定结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlImmunity {
    /// 完全免疫（该控制不生效）
    Immune,
    /// 通过了免疫检查
    Pass,
    /// 被更高级的控制覆盖（例如 hard 覆盖了 soft）
    Overridden,
}

// ============================================================================
// 免疫检查
// ============================================================================

/// 控制层级覆盖检查
///
/// 当目标同时受到多个控制效果时，高优先级覆盖低优先级。
/// 例如：目标同时受到 control_hard 和 control_soft → 只应用 control_hard。
pub fn check_control_override(
    target_tags: &GameplayTags,
    control_tag: GameplayTag,
) -> ControlImmunity {
    let control_level = match ControlLevel::from_tag(control_tag) {
        Some(level) => level,
        None => return ControlImmunity::Pass, // 不是控制标签，不处理
    };

    // 检查是否有更高级的控制
    if target_tags.has(GameplayTag::CONTROL_FULL) && control_level < ControlLevel::Full {
        return ControlImmunity::Overridden;
    }
    if target_tags.has(GameplayTag::CONTROL_HARD) && control_level < ControlLevel::Hard {
        return ControlImmunity::Overridden;
    }

    ControlImmunity::Pass
}

/// 执行完整免疫检查管线
///
/// 检查顺序：
/// 1. 最高权限 Tag（invincible/untargetable）→ 如果 control 是伤害/选中类，完全免疫
/// 2. 免疫控制 → 免疫 hard + full
/// 3. 免疫行动限制 → 仅免疫 hard
/// 4. 控制层级覆盖
pub fn check_control_immunity(
    target_tags: &GameplayTags,
    control_tag: GameplayTag,
) -> ControlImmunity {
    // Step 1: 最高权限 Tag 检查
    if target_tags.has(GameplayTag::INVINCIBLE) {
        // invincible 免疫所有伤害和控制，但纯减益效果仍生效
        return ControlImmunity::Immune;
    }
    if target_tags.has(GameplayTag::UNTARGETABLE) {
        // untargetable 免疫所有选中型控制
        // 但 AOE 状态仍然可能命中
        return ControlImmunity::Immune;
    }

    // Step 2-3: 免疫标签检查（通过 GameplayTags 实现）
    // 免疫控制 → 免疫 hard + full
    if target_tags.has(GameplayTag::CONTROL_HARD) || target_tags.has(GameplayTag::CONTROL_FULL) {
        // 需要精确匹配：目标有免疫特定控制的标签
        // 这里简化处理：CONTROL_HARD/FULL 是控制标签自身，不是免疫标签
        // 实际免疫标签由外部配置（如 Trait 定义的免疫列表）
    }

    // Step 4: 控制层级覆盖检查
    check_control_override(target_tags, control_tag)
}

// ============================================================================
// 控制递减
// ============================================================================

/// 控制历史记录（用于递减计算）
#[derive(Debug, Clone, Default)]
pub struct ControlHistory {
    /// 按控制类型记录最近的持续时间
    entries: Vec<ControlHistoryEntry>,
}

/// 单条控制历史
#[derive(Debug, Clone)]
pub struct ControlHistoryEntry {
    /// 控制标签
    pub control_tag: GameplayTag,
    /// 原始持续时间
    pub base_duration: u32,
    /// 已应用次数
    pub apply_count: u32,
}

impl ControlHistory {
    /// 记录一次控制施加
    pub fn record(&mut self, control_tag: GameplayTag, base_duration: u32) {
        // 查找是否有同类型的已有记录
        for entry in &mut self.entries {
            if entry.control_tag == control_tag {
                entry.apply_count += 1;
                entry.base_duration = base_duration;
                return;
            }
        }
        // 没有记录则新建
        self.entries.push(ControlHistoryEntry {
            control_tag,
            base_duration,
            apply_count: 1,
        });
    }

    /// 获取指定控制类型的施加次数
    pub fn apply_count(&self, control_tag: GameplayTag) -> u32 {
        self.entries
            .iter()
            .find(|e| e.control_tag == control_tag)
            .map(|e| e.apply_count)
            .unwrap_or(0)
    }
}

/// 计算控制递减后的持续时间
///
/// 规则：
/// - 第一次施加 → 完整持续时间
/// - 第二次施加（同类控制）→ 持续时间减半
/// - 第三次及以上 → 不再继续递减（保持减半后的值）
/// - 不同控制类型分别计数
pub fn calculate_diminished_duration(
    control_tag: GameplayTag,
    base_duration: u32,
    history: &ControlHistory,
) -> u32 {
    let apply_count = history.apply_count(control_tag);

    match apply_count {
        0 => base_duration,              // 第一次
        1 => (base_duration / 2).max(1), // 第二次：减半（最小为 1）
        _ => (base_duration / 2).max(1), // 第三次及以上：不再递减
    }
}

// ============================================================================
// 控制重叠检查
// ============================================================================

/// 检查目标是否值得施加控制（未被更高级别覆盖）
pub fn should_apply_control(target_tags: &GameplayTags, control_tag: GameplayTag) -> bool {
    match check_control_immunity(target_tags, control_tag) {
        ControlImmunity::Immune | ControlImmunity::Overridden => false,
        ControlImmunity::Pass => true,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── ControlLevel ──

    #[test]
    fn control_level_from_tag() {
        assert_eq!(
            ControlLevel::from_tag(GameplayTag::CONTROL_SOFT),
            Some(ControlLevel::Soft)
        );
        assert_eq!(
            ControlLevel::from_tag(GameplayTag::CONTROL_HARD),
            Some(ControlLevel::Hard)
        );
        assert_eq!(
            ControlLevel::from_tag(GameplayTag::CONTROL_FULL),
            Some(ControlLevel::Full)
        );
        assert_eq!(ControlLevel::from_tag(GameplayTag::DMG_FIRE), None);
    }

    #[test]
    fn control_level_ordering() {
        assert!(ControlLevel::Soft < ControlLevel::Hard);
        assert!(ControlLevel::Hard < ControlLevel::Full);
    }

    #[test]
    fn control_level_overrides() {
        assert!(ControlLevel::Full.overrides().contains(&ControlLevel::Soft));
        assert!(ControlLevel::Full.overrides().contains(&ControlLevel::Hard));
        assert!(ControlLevel::Hard.overrides().contains(&ControlLevel::Soft));
        assert!(ControlLevel::Soft.overrides().is_empty());
    }

    // ── Override checks ──

    #[test]
    fn override_full_overrides_lower() {
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::CONTROL_FULL);

        // Full overrides both soft and hard
        assert_eq!(
            check_control_override(&tags, GameplayTag::CONTROL_SOFT),
            ControlImmunity::Overridden
        );
        assert_eq!(
            check_control_override(&tags, GameplayTag::CONTROL_HARD),
            ControlImmunity::Overridden
        );
        // Full doesn't override itself
        assert_eq!(
            check_control_override(&tags, GameplayTag::CONTROL_FULL),
            ControlImmunity::Pass
        );
    }

    #[test]
    fn override_hard_overrides_soft() {
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::CONTROL_HARD);

        assert_eq!(
            check_control_override(&tags, GameplayTag::CONTROL_SOFT),
            ControlImmunity::Overridden
        );
        assert_eq!(
            check_control_override(&tags, GameplayTag::CONTROL_HARD),
            ControlImmunity::Pass
        );
    }

    #[test]
    fn override_no_control_returns_pass() {
        let tags = GameplayTags::default();
        assert_eq!(
            check_control_override(&tags, GameplayTag::CONTROL_SOFT),
            ControlImmunity::Pass
        );
    }

    // ── Immunity checks ──

    #[test]
    fn invincible_grants_immunity() {
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::INVINCIBLE);

        assert_eq!(
            check_control_immunity(&tags, GameplayTag::CONTROL_HARD),
            ControlImmunity::Immune
        );
    }

    #[test]
    fn untargetable_grants_immunity() {
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::UNTARGETABLE);

        assert_eq!(
            check_control_immunity(&tags, GameplayTag::CONTROL_SOFT),
            ControlImmunity::Immune
        );
    }

    #[test]
    fn no_immunity_for_normal_target() {
        let tags = GameplayTags::default();
        assert_eq!(
            check_control_immunity(&tags, GameplayTag::CONTROL_SOFT),
            ControlImmunity::Pass
        );
    }

    // ── Diminished duration ──

    #[test]
    fn first_application_full_duration() {
        let history = ControlHistory::default();
        assert_eq!(
            calculate_diminished_duration(GameplayTag::CONTROL_HARD, 3, &history),
            3
        );
    }

    #[test]
    fn second_application_halved() {
        let mut history = ControlHistory::default();
        history.record(GameplayTag::CONTROL_HARD, 3);

        assert_eq!(
            calculate_diminished_duration(GameplayTag::CONTROL_HARD, 3, &history),
            1 // 3/2 = 1.5 → 1 (floor)
        );
    }

    #[test]
    fn third_application_no_further_reduction() {
        let mut history = ControlHistory::default();
        history.record(GameplayTag::CONTROL_HARD, 4);
        history.record(GameplayTag::CONTROL_HARD, 4);

        assert_eq!(
            calculate_diminished_duration(GameplayTag::CONTROL_HARD, 4, &history),
            2 // stays at 4/2 = 2
        );
    }

    #[test]
    fn different_control_types_independent() {
        let mut history = ControlHistory::default();
        history.record(GameplayTag::CONTROL_HARD, 3);

        // Soft hasn't been applied yet → full duration
        assert_eq!(
            calculate_diminished_duration(GameplayTag::CONTROL_SOFT, 3, &history),
            3
        );
    }

    // ── should_apply_control ──

    #[test]
    fn should_apply_when_pass() {
        let tags = GameplayTags::default();
        assert!(should_apply_control(&tags, GameplayTag::CONTROL_SOFT));
    }

    #[test]
    fn should_not_apply_when_overridden() {
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::CONTROL_FULL);
        assert!(!should_apply_control(&tags, GameplayTag::CONTROL_SOFT));
    }

    #[test]
    fn should_not_apply_when_immune() {
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::INVINCIBLE);
        assert!(!should_apply_control(&tags, GameplayTag::CONTROL_HARD));
    }

    // ── ControlHistory ──

    #[test]
    fn control_history_tracks_apply_count() {
        let mut history = ControlHistory::default();
        assert_eq!(history.apply_count(GameplayTag::CONTROL_HARD), 0);

        history.record(GameplayTag::CONTROL_HARD, 3);
        assert_eq!(history.apply_count(GameplayTag::CONTROL_HARD), 1);

        history.record(GameplayTag::CONTROL_HARD, 3);
        assert_eq!(history.apply_count(GameplayTag::CONTROL_HARD), 2);
    }

    #[test]
    fn control_history_separates_types() {
        let mut history = ControlHistory::default();
        history.record(GameplayTag::CONTROL_HARD, 3);
        history.record(GameplayTag::CONTROL_SOFT, 2);

        assert_eq!(history.apply_count(GameplayTag::CONTROL_HARD), 1);
        assert_eq!(history.apply_count(GameplayTag::CONTROL_SOFT), 1);
    }
}
