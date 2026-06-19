//! rules — Narrative 域纯业务规则（零 ECS 依赖）
//!
//! 对话分支过滤、StoryFlag 一致性检查。
//! 详见 docs/02-domain/domains/narrative_domain.md §5

use crate::core::domains::narrative::components::{
    ChoiceDef, ChoiceOption, DialogueNodeDef, StoryFlags,
};

/// 过滤节点中在当前条件下可见的分支。
///
/// 不变量 3.2：相同状态下，同一分支的条件判定结果必须一致。
pub fn filter_visible_choices(node: &DialogueNodeDef, flags: &StoryFlags) -> Vec<ChoiceOption> {
    node.choices
        .iter()
        .map(|choice| {
            let visible = evaluate_choice_condition(choice, flags);
            ChoiceOption {
                choice_id: choice.id.clone(),
                text: choice.text.clone(),
                visible,
            }
        })
        .collect()
}

/// 评估分支条件是否满足。
///
/// 当前简化实现：如果 choice 有 condition_ref，检查对应 StoryFlag。
/// 完整实现将在内容系统接入后使用 Condition 领域。
fn evaluate_choice_condition(choice: &ChoiceDef, flags: &StoryFlags) -> bool {
    match &choice.condition_ref {
        None => true, // 无条件，始终可见
        Some(cond) => {
            // 条件格式: "flag_id=expected_value"
            if let Some(eq_pos) = cond.find('=') {
                let flag_id = &cond[..eq_pos];
                let expected = &cond[eq_pos + 1..];
                flags.check(flag_id, expected)
            } else {
                // 无等号，视为检查 flag 是否存在
                flags.has(cond)
            }
        }
    }
}

/// 检查对话树结构是否无环（不变量 3.1）。
///
/// 使用 DFS 检测反向边（循环引用）。
/// nodes: node_id → next_node_id 映射（从 ChoiceDef 提取）。
pub fn validate_no_cycles(
    entry_node_id: &str,
    edges: &[(&str, Option<&str>)],
) -> Result<(), String> {
    use std::collections::HashSet;

    let mut visited = HashSet::new();
    let mut stack = HashSet::new();

    fn dfs(
        current: &str,
        edges: &[(&str, Option<&str>)],
        visited: &mut HashSet<String>,
        stack: &mut HashSet<String>,
    ) -> Result<(), String> {
        if stack.contains(current) {
            return Err(format!("cycle detected at node '{}'", current));
        }
        if visited.contains(current) {
            return Ok(());
        }
        visited.insert(current.to_string());
        stack.insert(current.to_string());

        for (from, to) in edges {
            if *from == current
                && let Some(next) = to
            {
                dfs(next, edges, visited, stack)?;
            }
        }

        stack.remove(current);
        Ok(())
    }

    dfs(entry_node_id, edges, &mut visited, &mut stack)
}
