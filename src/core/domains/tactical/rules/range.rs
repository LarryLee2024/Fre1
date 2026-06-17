//! 范围计算 — 纯函数
//!
//! 用于移动范围、攻击范围、技能范围的网格计算。

use super::super::components::GridPos;

/// BFS 移动范围计算。
///
/// 从 center 开始，限制在 max_cost 内，返回所有可达的 GridPos。
/// cost_fn 接受 (from, to) 返回移动消耗。
/// 假设四连通网格。
pub fn bfs_reachable_positions<F>(
    center: GridPos,
    max_cost: f32,
    in_bounds: &dyn Fn(GridPos) -> bool,
    passable: &dyn Fn(GridPos) -> bool,
    cost_fn: &F,
) -> Vec<GridPos>
where
    F: Fn(GridPos, GridPos) -> f32,
{
    use std::collections::HashMap;

    let mut costs: HashMap<GridPos, f32> = HashMap::new();
    let mut queue = std::collections::VecDeque::new();

    costs.insert(center, 0.0);
    queue.push_back(center);

    while let Some(pos) = queue.pop_front() {
        let current_cost = costs[&pos];

        for neighbor in pos.neighbors_4() {
            if !in_bounds(neighbor) || !passable(neighbor) {
                continue;
            }

            let step_cost = cost_fn(pos, neighbor);
            let new_cost = current_cost + step_cost;

            if new_cost <= max_cost {
                let entry = costs.entry(neighbor).or_insert(f32::MAX);
                if new_cost < *entry {
                    *entry = new_cost;
                    queue.push_back(neighbor);
                }
            }
        }
    }

    // 排除起点自身
    costs
        .into_iter()
        .filter(|(p, _)| *p != center)
        .map(|(p, _)| p)
        .collect()
}

/// 攻击范围（武器/技能射程）。
///
/// 返回以 center 为中心，min_range <= 距离 <= max_range 的所有 GridPos。
pub fn attack_range(center: GridPos, min_range: u32, max_range: u32) -> Vec<GridPos> {
    let mut result = Vec::new();
    let max = max_range as i32;

    for dx in -max..=max {
        for dy in -max..=max {
            let dist = dx.unsigned_abs().max(dy.unsigned_abs());
            if dist >= min_range && dist <= max_range {
                let pos = GridPos::new(center.x + dx, center.y + dy);
                if pos != center {
                    result.push(pos);
                }
            }
        }
    }

    result
}
