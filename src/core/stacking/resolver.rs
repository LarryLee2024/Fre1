//! 叠层判定解析器 — 纯函数实现
//!
//! resolve_stacking() 是 Stacking 模块的核心纯函数，
//! 输入当前效果状态 + 新效果请求，输出叠层判定结果。

use super::types::{StackingResult, StackingRule};

/// 叠层判定上下文
#[derive(Clone, Debug)]
pub struct StackingContext {
    /// 当前层数
    pub current_stacks: u32,
    /// 叠层策略
    pub rule: StackingRule,
}

/// 执行叠层判定
///
/// 纯函数：不修改任何状态，仅根据输入返回判定结果。
///
/// # 参数
/// - `existing`: 当前已存在的效果状态（None 表示没有已有同类型效果）
/// - `new_rule`: 新效果的叠层策略
///
/// # 返回
/// StackingResult 枚举，指示应执行的操作
pub fn resolve_stacking(
    existing: Option<&StackingContext>,
    new_rule: StackingRule,
) -> StackingResult {
    // 没有已有同类型效果，直接添加
    let Some(ctx) = existing else {
        return StackingResult::NewlyApplied;
    };

    // 根据叠层策略判定
    match ctx.rule {
        // Replace: 替换旧实例
        StackingRule::Replace => StackingResult::Replaced,

        // RefreshDuration: 刷新持续时间，不叠加
        StackingRule::RefreshDuration => StackingResult::Refreshed,

        // StackAdd: 叠加，无上限
        StackingRule::StackAdd => {
            let new_count = ctx.current_stacks + 1;
            StackingResult::Stacked { new_count }
        }

        // StackMax(n): 叠加有上限
        StackingRule::StackMax(max) => {
            if ctx.current_stacks >= max {
                // 已达到上限，忽略
                StackingResult::Ignored { max_reached: true }
            } else {
                let new_count = ctx.current_stacks + 1;
                StackingResult::Stacked { new_count }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_no_existing_returns_newly_applied() {
        let result = resolve_stacking(None, StackingRule::Replace);
        assert_eq!(result, StackingResult::NewlyApplied);
    }

    #[test]
    fn resolve_replace_returns_replaced() {
        let ctx = StackingContext {
            current_stacks: 3,
            rule: StackingRule::Replace,
        };
        let result = resolve_stacking(Some(&ctx), StackingRule::Replace);
        assert_eq!(result, StackingResult::Replaced);
    }

    #[test]
    fn resolve_refresh_returns_refreshed() {
        let ctx = StackingContext {
            current_stacks: 1,
            rule: StackingRule::RefreshDuration,
        };
        let result = resolve_stacking(Some(&ctx), StackingRule::RefreshDuration);
        assert_eq!(result, StackingResult::Refreshed);
    }

    #[test]
    fn resolve_stack_add_increments() {
        let ctx = StackingContext {
            current_stacks: 2,
            rule: StackingRule::StackAdd,
        };
        let result = resolve_stacking(Some(&ctx), StackingRule::StackAdd);
        assert_eq!(result, StackingResult::Stacked { new_count: 3 });
    }

    #[test]
    fn resolve_stack_max_below_limit() {
        let ctx = StackingContext {
            current_stacks: 2,
            rule: StackingRule::StackMax(5),
        };
        let result = resolve_stacking(Some(&ctx), StackingRule::StackMax(5));
        assert_eq!(result, StackingResult::Stacked { new_count: 3 });
    }

    #[test]
    fn resolve_stack_max_at_limit_ignores() {
        let ctx = StackingContext {
            current_stacks: 5,
            rule: StackingRule::StackMax(5),
        };
        let result = resolve_stacking(Some(&ctx), StackingRule::StackMax(5));
        assert_eq!(result, StackingResult::Ignored { max_reached: true });
    }

    #[test]
    fn resolve_stack_max_above_limit_ignores() {
        let ctx = StackingContext {
            current_stacks: 10,
            rule: StackingRule::StackMax(5),
        };
        let result = resolve_stacking(Some(&ctx), StackingRule::StackMax(5));
        assert_eq!(result, StackingResult::Ignored { max_reached: true });
    }
}
