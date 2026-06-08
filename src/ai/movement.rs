use std::collections::HashMap;

use crate::battle::manhattan_distance;
use bevy::prelude::*;

use super::behavior::MoveStrategy;

/// 根据移动策略选择移动坐标
pub(crate) fn select_move_coord(
    reachable: &HashMap<IVec2, u32>,
    my_coord: IVec2,
    target_coord: IVec2,
    attack_range: u32,
    strategy: MoveStrategy,
) -> IVec2 {
    if reachable.is_empty() {
        return my_coord;
    }

    match strategy {
        MoveStrategy::Aggressive => {
            // 贪心靠近目标
            reachable
                .keys()
                .min_by_key(|coord| manhattan_distance(**coord, target_coord))
                .copied()
                .unwrap_or(my_coord)
        }
        MoveStrategy::Cautious => {
            // 保持攻击距离，不靠近超过攻击范围
            let at_range: Vec<_> = reachable
                .keys()
                .filter(|coord| manhattan_distance(**coord, target_coord) <= attack_range)
                .collect();

            if at_range.is_empty() {
                // 没有在攻击范围内的位置，靠近
                reachable
                    .keys()
                    .min_by_key(|coord| manhattan_distance(**coord, target_coord))
                    .copied()
                    .unwrap_or(my_coord)
            } else {
                // 选择最远的（保持距离）
                at_range
                    .iter()
                    .max_by_key(|coord| manhattan_distance(***coord, target_coord))
                    .map(|c| **c)
                    .unwrap_or(my_coord)
            }
        }
        MoveStrategy::Support => {
            // 优先靠近友军（暂用最近目标逻辑）
            reachable
                .keys()
                .min_by_key(|coord| manhattan_distance(**coord, target_coord))
                .copied()
                .unwrap_or(my_coord)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 移动策略_激进_贪心靠近目标() {
        let mut reachable = HashMap::new();
        reachable.insert(IVec2::new(2, 0), 1);
        reachable.insert(IVec2::new(4, 0), 1);
        reachable.insert(IVec2::new(6, 0), 1);
        let result = select_move_coord(
            &reachable,
            IVec2::ZERO,
            IVec2::new(10, 0),
            1,
            MoveStrategy::Aggressive,
        );
        assert_eq!(result, IVec2::new(6, 0));
    }

    #[test]
    fn 移动策略_谨慎_保持攻击距离() {
        let mut reachable = HashMap::new();
        reachable.insert(IVec2::new(8, 0), 1); // distance 2, in range
        reachable.insert(IVec2::new(3, 0), 1); // distance 7, out of range
        reachable.insert(IVec2::new(10, 0), 1); // distance 0, in range
        // Cautious picks in-range position with MAX distance to target
        // in-range: (8,0) dist=2, (10,0) dist=0 → picks (8,0)
        let result = select_move_coord(
            &reachable,
            IVec2::ZERO,
            IVec2::new(10, 0),
            3,
            MoveStrategy::Cautious,
        );
        assert_eq!(result, IVec2::new(8, 0));
    }

    #[test]
    fn 移动策略_谨慎_无范围内位置时靠近() {
        let mut reachable = HashMap::new();
        reachable.insert(IVec2::new(2, 0), 1);
        reachable.insert(IVec2::new(5, 0), 1);
        // 目标在 (10,0)，攻击范围 1，所有位置都不在范围内
        let result = select_move_coord(
            &reachable,
            IVec2::ZERO,
            IVec2::new(10, 0),
            1,
            MoveStrategy::Cautious,
        );
        assert_eq!(result, IVec2::new(5, 0));
    }

    #[test]
    fn 移动策略_空可移动范围返回自身() {
        let reachable = HashMap::new();
        let result = select_move_coord(
            &reachable,
            IVec2::new(3, 3),
            IVec2::new(10, 0),
            1,
            MoveStrategy::Aggressive,
        );
        assert_eq!(result, IVec2::new(3, 3));
    }
}
