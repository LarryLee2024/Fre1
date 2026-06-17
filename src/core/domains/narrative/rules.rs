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
            if *from == current {
                if let Some(next) = to {
                    dfs(next, edges, visited, stack)?;
                }
            }
        }

        stack.remove(current);
        Ok(())
    }

    dfs(entry_node_id, edges, &mut visited, &mut stack)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::domains::narrative::components::StoryFlags;

    #[test]
    fn unconditional_choice_always_visible() {
        let choice = ChoiceDef {
            id: "c1".into(),
            text: "Hello".into(),
            next_node_id: None,
            set_flags: vec![],
            condition_ref: None,
        };
        let node = DialogueNodeDef {
            id: "n1".into(),
            npc_text: "Hi".into(),
            choices: vec![choice],
            is_important: false,
            condition_ref: None,
        };
        let flags = StoryFlags::new();
        let result = filter_visible_choices(&node, &flags);
        assert_eq!(result.len(), 1);
        assert!(result[0].visible);
    }

    #[test]
    fn conditional_choice_hidden_when_flag_missing() {
        let choice = ChoiceDef {
            id: "c_quest".into(),
            text: "I finished the quest".into(),
            next_node_id: Some("n_reward".into()),
            set_flags: vec![],
            condition_ref: Some("quest_completed=true".into()),
        };
        let node = DialogueNodeDef {
            id: "n1".into(),
            npc_text: "How's the quest?".into(),
            choices: vec![choice],
            is_important: false,
            condition_ref: None,
        };

        // 无标记 → 不可见
        let mut flags = StoryFlags::new();
        let result = filter_visible_choices(&node, &flags);
        assert!(!result[0].visible);

        // 有标记 → 可见
        flags.set_flag("quest_completed", "true");
        let result = filter_visible_choices(&node, &flags);
        assert!(result[0].visible);
    }

    #[test]
    fn cycle_detection_clean() {
        let edges = vec![("n1", Some("n2")), ("n2", Some("n3")), ("n3", None)];
        assert!(validate_no_cycles("n1", &edges).is_ok());
    }

    #[test]
    fn cycle_detection_with_cycle() {
        let edges = vec![
            ("n1", Some("n2")),
            ("n2", Some("n3")),
            ("n3", Some("n1")), // 环！
        ];
        assert!(validate_no_cycles("n1", &edges).is_err());
    }
}
