//! Stacking 模块 — 效果堆叠规则中心
//!
//! ADR-026 §六：Stacking 升级为 4-enum 模型
//! - Replace：替换旧实例
//! - RefreshDuration：仅刷新 Duration，不叠加
//! - StackAdd：叠加层数，无上限
//! - StackMax(u32)：叠加层数，上限为参数值

pub mod resolver;
pub mod types;

pub use resolver::*;
pub use types::*;

use bevy::prelude::*;

/// Stacking 模块插件
pub struct StackingPlugin;

impl Plugin for StackingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<StackingRule>()
            .register_type::<StackingResult>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stacking_plugin_registers_types() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StackingPlugin);

        // 验证插件注册成功（不会 panic）
    }

    #[test]
    fn full_stack_scenario() {
        // 场景：中毒叠层，最多5层
        let rule = StackingRule::StackMax(5);

        // 第一次施加
        let result = resolve_stacking(None, rule);
        assert_eq!(result, StackingResult::NewlyApplied);

        // 第二次施加（当前1层）
        let ctx = StackingContext {
            current_stacks: 1,
            rule,
        };
        let result = resolve_stacking(Some(&ctx), rule);
        assert_eq!(result, StackingResult::Stacked { new_count: 2 });

        // 第五次施加（当前4层）
        let ctx = StackingContext {
            current_stacks: 4,
            rule,
        };
        let result = resolve_stacking(Some(&ctx), rule);
        assert_eq!(result, StackingResult::Stacked { new_count: 5 });

        // 第六次施加（当前5层，已满）
        let ctx = StackingContext {
            current_stacks: 5,
            rule,
        };
        let result = resolve_stacking(Some(&ctx), rule);
        assert_eq!(result, StackingResult::Ignored { max_reached: true });
    }

    #[test]
    fn replace_scenario() {
        let rule = StackingRule::Replace;

        // 第一次施加
        let result = resolve_stacking(None, rule);
        assert_eq!(result, StackingResult::NewlyApplied);

        // 重复施加（替换）
        let ctx = StackingContext {
            current_stacks: 3,
            rule,
        };
        let result = resolve_stacking(Some(&ctx), rule);
        assert_eq!(result, StackingResult::Replaced);
    }

    #[test]
    fn refresh_scenario() {
        let rule = StackingRule::RefreshDuration;

        // 第一次施加
        let result = resolve_stacking(None, rule);
        assert_eq!(result, StackingResult::NewlyApplied);

        // 重复施加（刷新）
        let ctx = StackingContext {
            current_stacks: 1,
            rule,
        };
        let result = resolve_stacking(Some(&ctx), rule);
        assert_eq!(result, StackingResult::Refreshed);
    }
}
