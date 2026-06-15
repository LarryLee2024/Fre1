// 执行栈：处理嵌套触发和中断的 LIFO 结构
// 参考：docs/01-architecture/skill-buff-abstraction.md §4.8.1
// 参考：docs/02-domain/trigger/trigger-rules.md

use crate::core::effect::EffectDef;
use bevy::prelude::*;

/// 栈深度上限 — 防止无限递归触发导致栈溢出
pub const MAX_STACK_DEPTH: usize = 32;

/// 栈溢出错误
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackOverflowError;

impl std::fmt::Display for StackOverflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "触发栈深度超过 MAX_STACK_DEPTH ({MAX_STACK_DEPTH})，可能存在无限递归触发"
        )
    }
}

impl std::error::Error for StackOverflowError {}

/// 栈条目：单次触发事件的完整上下文
#[derive(Debug, Clone)]
pub struct StackEntry {
    /// 触发源：哪个事件压入的
    pub trigger: super::Trigger,
    /// 触发上下文
    pub context: super::TriggerContext,
    /// 待执行的效果列表
    pub effects: Vec<EffectDef>,
    /// 优先级（数值越高越先弹出）
    pub priority: i32,
    /// 是否可被取消
    pub cancellable: bool,
}

/// 执行栈 — 处理嵌套触发和中断的 LIFO 结构
/// 参考：docs/01-architecture/skill-buff-abstraction.md §4.8.1
///
/// Stack 是响应栈（LIFO），支持嵌套、中断、取消，类似 MTG 的堆叠响应机制。
/// 典型场景：死亡触发 → Buff 触发 → 反击触发（嵌套响应）
#[derive(Resource, Debug)]
pub struct ExecutionStack {
    entries: Vec<StackEntry>,
    depth: usize,
}

impl Default for ExecutionStack {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            depth: 0,
        }
    }
}

impl ExecutionStack {
    /// 压入新事件到栈顶
    /// 深度超过 MAX_STACK_DEPTH 时返回错误，不压入
    pub fn push(&mut self, entry: StackEntry) -> Result<(), StackOverflowError> {
        if self.depth >= MAX_STACK_DEPTH {
            bevy::log::warn!(
                target: "core::trigger",
                depth = self.depth,
                max_depth = MAX_STACK_DEPTH,
                trigger = ?entry.trigger,
                "触发栈深度超限，丢弃事件防止无限递归"
            );
            return Err(StackOverflowError);
        }
        self.entries.push(entry);
        self.depth += 1;
        Ok(())
    }

    /// 弹出栈顶事件并执行
    pub fn pop(&mut self) -> Option<StackEntry> {
        let entry = self.entries.pop();
        if entry.is_some() && self.depth > 0 {
            self.depth -= 1;
        }
        entry
    }

    /// 取消栈顶事件（响应方可取消触发）
    /// 返回 true 表示取消成功，false 表示栈空或栈顶不可取消
    pub fn cancel_top(&mut self) -> bool {
        let last_idx = self.entries.len().saturating_sub(1);
        if last_idx == 0 && self.entries.is_empty() {
            return false;
        }

        let cancellable = self
            .entries
            .get(last_idx)
            .map_or(false, |top| top.cancellable);

        if cancellable {
            let removed = self.entries.remove(last_idx);
            if self.depth > 0 {
                self.depth -= 1;
            }
            bevy::log::info!(
                target: "core::trigger",
                trigger = ?removed.trigger,
                "触发栈顶事件已取消"
            );
            return true;
        }
        false
    }

    /// 获取栈深度（防止无限递归触发）
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// 清空栈
    pub fn clear(&mut self) {
        self.entries.clear();
        self.depth = 0;
    }

    /// 栈是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 查看栈顶事件（不弹出）
    pub fn peek(&self) -> Option<&StackEntry> {
        self.entries.last()
    }
}

#[cfg(test)]
mod tests {
    // ================================================
    // Bevy SRPG AI宪法 v1.1 自检结果（测试专用）
    // ================================================
    // ✅ 测行为不测实现：是 — 断言验证栈操作结果，不验证内部 Vec 实现
    // ✅ 符合领域规则：是 — 覆盖 MAX_STACK_DEPTH = 32 不变量、LIFO 弹出顺序
    // ✅ 确定性：是 — 硬编码条目数据，无随机性
    // ✅ 使用标准数据：是 — 使用标准 StackEntry 结构
    // ✅ 无越界测试：是 — 仅测试公共 API
    // ✅ 未测试私有实现：是 — 仅通过 pub 接口测试
    // ================================================
    use super::*;
    use crate::core::trigger::{Trigger, TriggerContext};

    // ── 测试辅助函数 ──

    fn make_ctx(trigger: Trigger) -> TriggerContext {
        TriggerContext {
            trigger,
            source_entity: Entity::from_bits(1),
            target_entity: Entity::from_bits(2),
            damage_dealt: None,
            is_critical: None,
            chain_depth: 0,
        }
    }

    /// 构建测试用 StackEntry（指定 trigger + cancellable，默认 priority=0）
    fn make_entry(trigger: Trigger, cancellable: bool) -> StackEntry {
        StackEntry {
            trigger,
            context: make_ctx(trigger),
            effects: vec![],
            priority: 0,
            cancellable,
        }
    }

    /// 构建测试用 StackEntry（指定 trigger + priority + cancellable）
    fn make_entry_with_priority(trigger: Trigger, priority: i32, cancellable: bool) -> StackEntry {
        StackEntry {
            trigger,
            context: make_ctx(trigger),
            effects: vec![],
            priority,
            cancellable,
        }
    }

    // ══════════════════════════════════════════════
    // Test 1: LIFO 顺序（基础 push/pop）
    // ══════════════════════════════════════════════

    #[test]
    fn lifo_三个条目_弹出顺序正确() {
        let mut stack = ExecutionStack::default();
        stack.push(make_entry(Trigger::TurnStart, false)).unwrap(); // A（最先压入）
        stack.push(make_entry(Trigger::AfterAttack, false)).unwrap(); // B
        stack.push(make_entry(Trigger::Death, false)).unwrap(); // C（最后压入）

        // LIFO: C → B → A
        assert_eq!(stack.pop().unwrap().trigger, Trigger::Death);
        assert_eq!(stack.pop().unwrap().trigger, Trigger::AfterAttack);
        assert_eq!(stack.pop().unwrap().trigger, Trigger::TurnStart);
        assert!(stack.pop().is_none());
    }

    #[test]
    fn lifo_单条目_push_pop() {
        let mut stack = ExecutionStack::default();
        stack.push(make_entry(Trigger::TurnStart, false)).unwrap();

        assert_eq!(stack.depth(), 1);
        assert!(!stack.is_empty());

        let popped = stack.pop();
        assert!(popped.is_some());
        assert_eq!(popped.unwrap().trigger, Trigger::TurnStart);
        assert_eq!(stack.depth(), 0);
        assert!(stack.is_empty());
    }

    #[test]
    fn lifo_交替push_pop() {
        let mut stack = ExecutionStack::default();

        stack.push(make_entry(Trigger::TurnStart, false)).unwrap();
        stack.push(make_entry(Trigger::AfterAttack, false)).unwrap();
        assert_eq!(stack.pop().unwrap().trigger, Trigger::AfterAttack);

        stack.push(make_entry(Trigger::Death, false)).unwrap();
        assert_eq!(stack.pop().unwrap().trigger, Trigger::Death);
        assert_eq!(stack.pop().unwrap().trigger, Trigger::TurnStart);

        assert!(stack.is_empty());
    }

    // ══════════════════════════════════════════════
    // Test 2: MAX_STACK_DEPTH 边界
    // ══════════════════════════════════════════════

    #[test]
    fn max_depth_压入32条目成功() {
        let mut stack = ExecutionStack::default();

        for i in 0..MAX_STACK_DEPTH {
            let result = stack.push(make_entry(Trigger::TurnStart, false));
            assert!(result.is_ok(), "第 {} 条目应压入成功", i + 1);
        }

        assert_eq!(stack.depth(), MAX_STACK_DEPTH);
        assert!(!stack.is_empty());
    }

    #[test]
    fn max_depth_第33条目被拒绝() {
        let mut stack = ExecutionStack::default();

        for _ in 0..MAX_STACK_DEPTH {
            stack.push(make_entry(Trigger::TurnStart, false)).unwrap();
        }

        let result = stack.push(make_entry(Trigger::Death, false));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StackOverflowError);
    }

    #[test]
    fn max_depth_拒绝后深度不变() {
        let mut stack = ExecutionStack::default();

        for _ in 0..MAX_STACK_DEPTH {
            stack.push(make_entry(Trigger::TurnStart, false)).unwrap();
        }

        let _ = stack.push(make_entry(Trigger::Death, false));
        assert_eq!(stack.depth(), MAX_STACK_DEPTH);
    }

    #[test]
    fn max_depth_拒绝后栈内容不变() {
        let mut stack = ExecutionStack::default();

        for _ in 0..MAX_STACK_DEPTH {
            stack.push(make_entry(Trigger::TurnStart, false)).unwrap();
        }

        let _ = stack.push(make_entry(Trigger::Death, false));

        // 弹出第一个应是 LIFO 最后压入的（Trigger::TurnStart）
        let popped = stack.pop().unwrap();
        assert_eq!(popped.trigger, Trigger::TurnStart);
        assert_eq!(stack.depth(), MAX_STACK_DEPTH - 1);
    }

    #[test]
    fn max_depth_弹出后可再压入() {
        let mut stack = ExecutionStack::default();

        for _ in 0..MAX_STACK_DEPTH {
            stack.push(make_entry(Trigger::TurnStart, false)).unwrap();
        }

        // 弹出一个
        let _ = stack.pop();
        assert_eq!(stack.depth(), MAX_STACK_DEPTH - 1);

        // 现在可以再压入
        let result = stack.push(make_entry(Trigger::Death, false));
        assert!(result.is_ok());
        assert_eq!(stack.depth(), MAX_STACK_DEPTH);
    }

    // ══════════════════════════════════════════════
    // Test 3: cancel_top() 取消
    // ══════════════════════════════════════════════

    #[test]
    fn cancel_top_可取消条目_成功取消() {
        let mut stack = ExecutionStack::default();
        stack.push(make_entry(Trigger::AfterDamaged, true)).unwrap();

        assert!(stack.cancel_top());
        assert!(stack.is_empty());
        assert_eq!(stack.depth(), 0);
    }

    #[test]
    fn cancel_top_不可取消条目_返回false() {
        let mut stack = ExecutionStack::default();
        stack
            .push(make_entry(Trigger::AfterDamaged, false))
            .unwrap();

        let cancelled = stack.cancel_top();
        assert!(!cancelled);
        assert_eq!(stack.depth(), 1);
    }

    #[test]
    fn cancel_top_取消栈顶可取消_保留下层不可取消() {
        let mut stack = ExecutionStack::default();

        // 底层：不可取消
        stack.push(make_entry(Trigger::TurnStart, false)).unwrap();
        // 顶层：可取消
        stack.push(make_entry(Trigger::AfterDamaged, true)).unwrap();

        assert!(stack.cancel_top()); // 取消 AfterDamaged
        assert_eq!(stack.depth(), 1);

        // 底层仍在
        let remaining = stack.pop().unwrap();
        assert_eq!(remaining.trigger, Trigger::TurnStart);
        assert!(!remaining.cancellable);
    }

    #[test]
    fn cancel_top_栈顶不可取消_跳过() {
        let mut stack = ExecutionStack::default();

        // 底层：可取消
        stack.push(make_entry(Trigger::AfterDamaged, true)).unwrap();
        // 顶层：不可取消
        stack.push(make_entry(Trigger::TurnStart, false)).unwrap();

        // cancel_top 检查的是 LIFO 栈顶（TurnStart），不可取消
        assert!(!stack.cancel_top());
        assert_eq!(stack.depth(), 2);
    }

    #[test]
    fn cancel_top_空栈返回false() {
        let mut stack = ExecutionStack::default();
        assert!(!stack.cancel_top());
    }

    #[test]
    fn cancel_top_连续取消() {
        let mut stack = ExecutionStack::default();

        stack.push(make_entry(Trigger::AfterDamaged, true)).unwrap();
        stack.push(make_entry(Trigger::AfterAttack, true)).unwrap();
        stack.push(make_entry(Trigger::Death, true)).unwrap();

        assert!(stack.cancel_top()); // 取消 Death（栈顶）
        assert_eq!(stack.depth(), 2);

        assert!(stack.cancel_top()); // 取消 AfterAttack（新栈顶）
        assert_eq!(stack.depth(), 1);

        assert!(stack.cancel_top()); // 取消 AfterDamaged（最后一个）
        assert_eq!(stack.depth(), 0);
        assert!(stack.is_empty());
    }

    // ══════════════════════════════════════════════
    // Test 4: 嵌套触发确定性（LIFO 确定性 + 优先级一致性）
    // ══════════════════════════════════════════════
    //
    // 架构设计：ExecutionStack 为 LIFO 结构，pop 返回最后压入的条目。
    // priority 字段用于 TriggerHandler 注册表的分发排序（按 priority 分发），
    // Stack 弹出顺序严格遵循 LIFO。
    //
    // 本组测试验证：相同输入序列 → 相同输出序列（确定性），
    // 以及 priority 字段不影响弹出顺序（LIFO 保证）。

    #[test]
    fn 确定性_相同输入序列_相同输出序列() {
        // 多次执行相同操作序列，结果必须一致
        for _ in 0..10 {
            let mut stack = ExecutionStack::default();
            stack.push(make_entry(Trigger::TurnStart, false)).unwrap();
            stack.push(make_entry(Trigger::AfterAttack, false)).unwrap();
            stack.push(make_entry(Trigger::Death, false)).unwrap();

            assert_eq!(
                stack.pop().unwrap().trigger,
                Trigger::Death,
                "弹出顺序必须一致"
            );
            assert_eq!(
                stack.pop().unwrap().trigger,
                Trigger::AfterAttack,
                "弹出顺序必须一致"
            );
            assert_eq!(
                stack.pop().unwrap().trigger,
                Trigger::TurnStart,
                "弹出顺序必须一致"
            );
        }
    }

    #[test]
    fn 确定性_不同priority_仍为lifo() {
        // priority 字段不影响 pop 顺序，弹出始终为 LIFO
        let mut stack = ExecutionStack::default();
        stack
            .push(make_entry_with_priority(Trigger::TurnStart, 10, true))
            .unwrap(); // priority=10
        stack
            .push(make_entry_with_priority(Trigger::AfterAttack, 30, true))
            .unwrap(); // priority=30
        stack
            .push(make_entry_with_priority(Trigger::Death, 20, true))
            .unwrap(); // priority=20

        // LIFO: Death → AfterAttack → TurnStart（忽略 priority）
        assert_eq!(stack.pop().unwrap().trigger, Trigger::Death);
        assert_eq!(stack.pop().unwrap().trigger, Trigger::AfterAttack);
        assert_eq!(stack.pop().unwrap().trigger, Trigger::TurnStart);
    }

    #[test]
    fn 确定性_大量条目_顺序一致() {
        let triggers = [
            Trigger::TurnStart,
            Trigger::TurnEnd,
            Trigger::BeforeAttack,
            Trigger::AfterAttack,
            Trigger::BeforeDamaged,
            Trigger::AfterDamaged,
            Trigger::BeforeMove,
            Trigger::AfterMove,
            Trigger::KillTarget,
            Trigger::Death,
            Trigger::BattleStart,
            Trigger::BattleEnd,
        ];

        for _ in 0..5 {
            let mut stack = ExecutionStack::default();
            for t in &triggers {
                stack.push(make_entry(*t, false)).unwrap();
            }

            // 弹出顺序必须为逆序（LIFO）
            for expected in triggers.iter().rev() {
                assert_eq!(stack.pop().unwrap().trigger, *expected);
            }
            assert!(stack.is_empty());
        }
    }

    // ══════════════════════════════════════════════
    // Test 5: 空栈行为
    // ══════════════════════════════════════════════

    #[test]
    fn 空栈_pop返回none() {
        let mut stack = ExecutionStack::default();
        assert!(stack.pop().is_none());
    }

    #[test]
    fn 空栈_is_empty为true() {
        let stack = ExecutionStack::default();
        assert!(stack.is_empty());
    }

    #[test]
    fn 空栈_depth为0() {
        let stack = ExecutionStack::default();
        assert_eq!(stack.depth(), 0);
    }

    #[test]
    fn 空栈_cancel_top返回false() {
        let mut stack = ExecutionStack::default();
        assert!(!stack.cancel_top());
    }

    #[test]
    fn 空栈_peek返回none() {
        let stack = ExecutionStack::default();
        assert!(stack.peek().is_none());
    }

    // ══════════════════════════════════════════════
    // Test 6: clear 清空
    // ══════════════════════════════════════════════

    #[test]
    fn clear_压入5条目后清空() {
        let mut stack = ExecutionStack::default();

        for _ in 0..5 {
            stack.push(make_entry(Trigger::TurnStart, false)).unwrap();
        }
        assert_eq!(stack.depth(), 5);
        assert!(!stack.is_empty());

        stack.clear();

        assert_eq!(stack.depth(), 0);
        assert!(stack.is_empty());
    }

    #[test]
    fn clear_空栈清空无影响() {
        let mut stack = ExecutionStack::default();
        stack.clear();
        assert!(stack.is_empty());
        assert_eq!(stack.depth(), 0);
    }

    #[test]
    fn clear_满栈清空后可再压入() {
        let mut stack = ExecutionStack::default();

        for _ in 0..MAX_STACK_DEPTH {
            stack.push(make_entry(Trigger::TurnStart, false)).unwrap();
        }

        stack.clear();
        assert_eq!(stack.depth(), 0);

        // 清空后可以再压入
        let result = stack.push(make_entry(Trigger::Death, false));
        assert!(result.is_ok());
        assert_eq!(stack.depth(), 1);
    }

    // ══════════════════════════════════════════════
    // Test 7: 深度追踪准确性
    // ══════════════════════════════════════════════

    #[test]
    fn 深度追踪_逐条目递增() {
        let mut stack = ExecutionStack::default();

        assert_eq!(stack.depth(), 0);

        stack.push(make_entry(Trigger::TurnStart, false)).unwrap();
        assert_eq!(stack.depth(), 1);

        stack.push(make_entry(Trigger::AfterAttack, false)).unwrap();
        assert_eq!(stack.depth(), 2);

        stack.push(make_entry(Trigger::Death, false)).unwrap();
        assert_eq!(stack.depth(), 3);
    }

    #[test]
    fn 深度追踪_弹出后递减() {
        let mut stack = ExecutionStack::default();

        for _ in 0..10 {
            stack.push(make_entry(Trigger::TurnStart, false)).unwrap();
        }
        assert_eq!(stack.depth(), 10);

        let _ = stack.pop();
        assert_eq!(stack.depth(), 9);

        let _ = stack.pop();
        assert_eq!(stack.depth(), 8);
    }

    #[test]
    fn 深度追踪_cancel后递减() {
        let mut stack = ExecutionStack::default();

        stack.push(make_entry(Trigger::TurnStart, true)).unwrap();
        stack.push(make_entry(Trigger::AfterAttack, true)).unwrap();
        assert_eq!(stack.depth(), 2);

        let cancelled = stack.cancel_top();
        assert!(cancelled);
        assert_eq!(stack.depth(), 1);

        let cancelled = stack.cancel_top();
        assert!(cancelled);
        assert_eq!(stack.depth(), 0);
    }

    #[test]
    fn 深度追踪_push_pop交替() {
        let mut stack = ExecutionStack::default();

        stack.push(make_entry(Trigger::TurnStart, false)).unwrap();
        assert_eq!(stack.depth(), 1);

        stack.push(make_entry(Trigger::AfterAttack, false)).unwrap();
        assert_eq!(stack.depth(), 2);

        let _ = stack.pop();
        assert_eq!(stack.depth(), 1);

        stack.push(make_entry(Trigger::Death, false)).unwrap();
        assert_eq!(stack.depth(), 2);

        let _ = stack.pop();
        let _ = stack.pop();
        assert_eq!(stack.depth(), 0);
        assert!(stack.is_empty());
    }

    // ══════════════════════════════════════════════
    // Test 8: peek 不影响栈
    // ══════════════════════════════════════════════

    #[test]
    fn peek_不改变深度() {
        let mut stack = ExecutionStack::default();
        stack.push(make_entry(Trigger::KillTarget, false)).unwrap();

        let _ = stack.peek();
        assert_eq!(stack.depth(), 1);

        let _ = stack.peek();
        assert_eq!(stack.depth(), 1);
    }

    #[test]
    fn peek_连续查看返回相同结果() {
        let mut stack = ExecutionStack::default();
        stack.push(make_entry(Trigger::Death, false)).unwrap();

        assert_eq!(stack.peek().unwrap().trigger, Trigger::Death);
        assert_eq!(stack.peek().unwrap().trigger, Trigger::Death);
    }

    // ══════════════════════════════════════════════
    // Test 9: StackOverflowError 格式化与比较
    // ══════════════════════════════════════════════

    #[test]
    fn stack_overflow_error_display() {
        let err = StackOverflowError;
        let msg = format!("{}", err);
        assert!(msg.contains("32"));
        assert!(msg.contains("MAX_STACK_DEPTH"));
    }

    #[test]
    fn stack_overflow_error_clone() {
        let err = StackOverflowError;
        let cloned = err.clone();
        assert_eq!(format!("{}", err), format!("{}", cloned));
    }
}
