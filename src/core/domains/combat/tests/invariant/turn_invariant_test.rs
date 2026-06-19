//! Invariant tests — Combat 回合系统领域不变量测试
//!
//! 验证 combat_domain.md §3 定义的领域不变量。
//!
//! | 不变量 | 描述 | 测试覆盖 |
//! |--------|------|----------|
//! | 3.2 | 回合严格交替 — 单位必须按先攻顺序依次行动 | `turn_order_is_strict` |
//! | 3.4 | 一回合只能有一个行动态单位 | `only_one_active_unit_per_turn` |
//! | 3.3 | 先攻排序不变性 — 战斗开始后 TurnQueue 不修改排序 | `initiative_order_is_stable` |

use bevy::prelude::Entity;

use crate::core::domains::combat::components::{TeamId, TurnEntry, TurnQueue};

/// 创建指定索引的测试实体。
fn entity(id: u32) -> Entity {
    Entity::from_raw_u32(id).unwrap()
}

fn make_interleaved_entries() -> Vec<TurnEntry> {
    let team_a = TeamId::new("player");
    let team_b = TeamId::new("enemy");
    vec![
        TurnEntry::new(entity(1), team_a.clone(), 25),
        TurnEntry::new(entity(2), team_b.clone(), 22),
        TurnEntry::new(entity(3), team_a.clone(), 18),
        TurnEntry::new(entity(4), team_b.clone(), 14),
        TurnEntry::new(entity(5), team_a, 10),
    ]
}

/// 不变量 3.2: 回合严格交替 — 单位必须按先攻顺序依次行动。
///
/// 验证 TurnQueue.advance() 严格按照初始化时的排列顺序推进，
/// 不会跳过或重排。
#[test]
fn turn_order_is_strict() {
    let mut q = TurnQueue::new(make_interleaved_entries());
    let expected_initiatives = [25, 22, 18, 14, 10];

    for &expected in &expected_initiatives {
        let current = q.current().expect("should have current entry");
        assert_eq!(
            current.initiative, expected,
            "turn order must follow initiative sequence: expected {expected}, got {}",
            current.initiative
        );
        q.advance();
    }

    // 一轮完成后回到开头
    assert_eq!(q.round_number(), 2);
    assert_eq!(q.current().unwrap().initiative, 25);
}

/// 不变量 3.4: 同一时刻最多只有一个单位处于"行动中"状态。
///
/// 验证 current() 返回单一单位，且 advance() 之前不会切换。
#[test]
fn only_one_active_unit_per_turn() {
    let mut q = TurnQueue::new(make_interleaved_entries());

    // 在 advance 之前，current 始终保持不变
    let first = q.current().unwrap().entity;
    assert_eq!(
        q.current().unwrap().entity,
        first,
        "current unit should not change without advance()"
    );

    // advance 后切换到下一个
    q.advance();
    let second = q.current().unwrap().entity;
    assert_ne!(second, first, "advance() should switch to next unit");

    // 同一个单位持续行动直到下次 advance
    assert_eq!(
        q.current().unwrap().entity,
        second,
        "unit should remain active until next advance()"
    );
}

/// 不变量 3.3: 先攻排序不变性 — 战斗开始后 TurnQueue 的 entries 排序不修改。
///
/// 验证 entries() 返回的顺序在 advance 前后一致。
#[test]
fn initiative_order_is_stable() {
    let q = TurnQueue::new(make_interleaved_entries());
    let initial: Vec<_> = q.entries().iter().map(|e| e.initiative).collect();

    // 模拟多次 advance
    let mut q = q;
    for _ in 0..7 {
        q.advance();
    }

    // entries 排序不变
    let after: Vec<_> = q.entries().iter().map(|e| e.initiative).collect();
    assert_eq!(
        initial, after,
        "initiative order must remain unchanged during battle"
    );
}

/// TurnQueue 不支持外部直接修改 entries 顺序。
///
/// entries() 返回只读引用，验证 entries 字段不可通过公开 API 修改。
#[test]
fn turn_queue_entries_is_read_only() {
    let q = TurnQueue::new(make_interleaved_entries());
    let entries = q.entries();
    // entries 为 &[TurnEntry]，只读
    let _ = entries.len(); // 仅编译验证，entries 为不可变引用
    assert!(!entries.is_empty());
}
