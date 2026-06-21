//! Stacking 模块类型定义
//!
//! 定义 StackingRule 4-enum 冻结版（ADR-026 §六）
//! - Replace：替换旧实例
//! - RefreshDuration：仅刷新 Duration，不叠加
//! - StackAdd：叠加层数，无上限
//! - StackMax(u32)：叠加层数，上限为参数值

use bevy::prelude::*;
use serde::Deserialize;

/// 叠层策略 — 效果重复施加时的处理规则（ADR-026 冻结版）
///
/// 每个 Effect 实例携带一个 StackingRule，决定重复施加时的行为。
/// 这是 4-enum 冻结模型，禁止新增变体。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum StackingRule {
    /// 替换：用新实例完全覆盖旧实例，新实例层数始终为 1
    Replace,
    /// 刷新持续时间：仅重置 Duration tick，不叠加层数
    RefreshDuration,
    /// 叠加：层数 +1，无上限
    StackAdd,
    /// 叠加有上限：层数 +1，上限为参数值
    StackMax(u32),
}

impl Default for StackingRule {
    fn default() -> Self {
        StackingRule::Replace
    }
}

impl StackingRule {
    /// 获取最大层数
    pub fn max_stacks(&self) -> u32 {
        match self {
            StackingRule::Replace => 1,
            StackingRule::RefreshDuration => 1,
            StackingRule::StackAdd => u32::MAX,
            StackingRule::StackMax(n) => *n,
        }
    }

    /// 是否允许叠加
    pub fn allows_stacking(&self) -> bool {
        matches!(self, StackingRule::StackAdd | StackingRule::StackMax(_))
    }

    /// 是否替换
    pub fn is_replace(&self) -> bool {
        matches!(self, StackingRule::Replace)
    }

    /// 是否刷新持续时间
    pub fn is_refresh(&self) -> bool {
        matches!(self, StackingRule::RefreshDuration)
    }
}

/// 叠层结果 — resolve_stacking() 的输出
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum StackingResult {
    /// 新实例：没有已有同类型效果，直接添加
    NewlyApplied,
    /// 替换：移除旧实例，以初始状态添加新实例
    Replaced,
    /// 刷新：重置 Duration，不改变层数
    Refreshed,
    /// 叠加：层数 +1
    Stacked { new_count: u32 },
    /// 忽略：已达到上限，不执行任何操作
    Ignored { max_reached: bool },
}

/// RON 反序列化用的叠层策略定义
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum StackingRuleDef {
    Replace,
    RefreshDuration,
    StackAdd,
    StackMax { max_stack: u32 },
}

impl Default for StackingRuleDef {
    fn default() -> Self {
        StackingRuleDef::Replace
    }
}

impl From<StackingRuleDef> for StackingRule {
    fn from(def: StackingRuleDef) -> Self {
        match def {
            StackingRuleDef::Replace => StackingRule::Replace,
            StackingRuleDef::RefreshDuration => StackingRule::RefreshDuration,
            StackingRuleDef::StackAdd => StackingRule::StackAdd,
            StackingRuleDef::StackMax { max_stack } => StackingRule::StackMax(max_stack.max(1)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 叠层规则_最大层数() {
        assert_eq!(StackingRule::Replace.max_stacks(), 1);
        assert_eq!(StackingRule::RefreshDuration.max_stacks(), 1);
        assert_eq!(StackingRule::StackAdd.max_stacks(), u32::MAX);
        assert_eq!(StackingRule::StackMax(5).max_stacks(), 5);
    }

    #[test]
    fn 叠层规则_允许叠加() {
        assert!(!StackingRule::Replace.allows_stacking());
        assert!(!StackingRule::RefreshDuration.allows_stacking());
        assert!(StackingRule::StackAdd.allows_stacking());
        assert!(StackingRule::StackMax(5).allows_stacking());
    }

    #[test]
    fn 叠层规则_是否替换() {
        assert!(StackingRule::Replace.is_replace());
        assert!(!StackingRule::RefreshDuration.is_replace());
    }

    #[test]
    fn 叠层规则_是否刷新() {
        assert!(StackingRule::RefreshDuration.is_refresh());
        assert!(!StackingRule::Replace.is_refresh());
    }

    #[test]
    fn 叠层规则定义_转换() {
        let def = StackingRuleDef::StackMax { max_stack: 3 };
        let rule: StackingRule = def.into();
        assert_eq!(rule, StackingRule::StackMax(3));
    }

    #[test]
    fn 叠层规则定义_零上限变为一() {
        let def = StackingRuleDef::StackMax { max_stack: 0 };
        let rule: StackingRule = def.into();
        assert_eq!(rule, StackingRule::StackMax(1));
    }
}
