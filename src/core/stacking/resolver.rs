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
    match new_rule {
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
    fn 解析_无已有效果_返回新施加() {
        let result = resolve_stacking(None, StackingRule::Replace);
        assert_eq!(result, StackingResult::NewlyApplied);
    }

    #[test]
    fn 解析_替换策略_返回已替换() {
        let ctx = StackingContext {
            current_stacks: 3,
            rule: StackingRule::Replace,
        };
        let result = resolve_stacking(Some(&ctx), StackingRule::Replace);
        assert_eq!(result, StackingResult::Replaced);
    }

    #[test]
    fn 解析_刷新策略_返回已刷新() {
        let ctx = StackingContext {
            current_stacks: 1,
            rule: StackingRule::RefreshDuration,
        };
        let result = resolve_stacking(Some(&ctx), StackingRule::RefreshDuration);
        assert_eq!(result, StackingResult::Refreshed);
    }

    #[test]
    fn 解析_叠加策略_层数递增() {
        let ctx = StackingContext {
            current_stacks: 2,
            rule: StackingRule::StackAdd,
        };
        let result = resolve_stacking(Some(&ctx), StackingRule::StackAdd);
        assert_eq!(result, StackingResult::Stacked { new_count: 3 });
    }

    #[test]
    fn 解析_上限叠加_低于上限() {
        let ctx = StackingContext {
            current_stacks: 2,
            rule: StackingRule::StackMax(5),
        };
        let result = resolve_stacking(Some(&ctx), StackingRule::StackMax(5));
        assert_eq!(result, StackingResult::Stacked { new_count: 3 });
    }

    #[test]
    fn 解析_上限叠加_达到上限忽略() {
        let ctx = StackingContext {
            current_stacks: 5,
            rule: StackingRule::StackMax(5),
        };
        let result = resolve_stacking(Some(&ctx), StackingRule::StackMax(5));
        assert_eq!(result, StackingResult::Ignored { max_reached: true });
    }

    #[test]
    fn 解析_上限叠加_超过上限忽略() {
        let ctx = StackingContext {
            current_stacks: 10,
            rule: StackingRule::StackMax(5),
        };
        let result = resolve_stacking(Some(&ctx), StackingRule::StackMax(5));
        assert_eq!(result, StackingResult::Ignored { max_reached: true });
    }
}
