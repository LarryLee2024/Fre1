use std::collections::HashMap;

use bevy::prelude::*;

use super::strategy::MoveSelector;

/// 根据移动策略选择移动坐标
/// 通过 trait 对象分发，替代原来的 enum+match 模式
pub(crate) fn select_move_coord(
    reachable: &HashMap<IVec2, u32>,
    my_coord: IVec2,
    target_coord: IVec2,
    attack_range: u32,
    selector: &dyn MoveSelector,
) -> IVec2 {
    if reachable.is_empty() {
        return my_coord;
    }

    selector.select(reachable, my_coord, target_coord, attack_range)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ai::strategy::{AggressiveMove, AiStrategyRegistry, CautiousMove};

    #[test]
    fn 移动策略_激进_贪心靠近目标() {
        let mut reachable = HashMap::new();
        reachable.insert(IVec2::new(2, 0), 1);
        reachable.insert(IVec2::new(4, 0), 1);
        reachable.insert(IVec2::new(6, 0), 1);
        let selector = AggressiveMove;
        let result = select_move_coord(&reachable, IVec2::ZERO, IVec2::new(10, 0), 1, &selector);
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
        let selector = CautiousMove;
        let result = select_move_coord(&reachable, IVec2::ZERO, IVec2::new(10, 0), 3, &selector);
        assert_eq!(result, IVec2::new(8, 0));
    }

    #[test]
    fn 移动策略_谨慎_无范围内位置时靠近() {
        let mut reachable = HashMap::new();
        reachable.insert(IVec2::new(2, 0), 1);
        reachable.insert(IVec2::new(5, 0), 1);
        // 目标在 (10,0)，攻击范围 1，所有位置都不在范围内
        let selector = CautiousMove;
        let result = select_move_coord(&reachable, IVec2::ZERO, IVec2::new(10, 0), 1, &selector);
        assert_eq!(result, IVec2::new(5, 0));
    }

    #[test]
    fn 移动策略_空可移动范围返回自身() {
        let reachable = HashMap::new();
        let selector = AggressiveMove;
        let result = select_move_coord(
            &reachable,
            IVec2::new(3, 3),
            IVec2::new(10, 0),
            1,
            &selector,
        );
        assert_eq!(result, IVec2::new(3, 3));
    }

    #[test]
    fn 移动策略_通过注册表分发() {
        let registry = AiStrategyRegistry::default();
        let mut reachable = HashMap::new();
        reachable.insert(IVec2::new(2, 0), 1);
        reachable.insert(IVec2::new(6, 0), 1);

        // 通过注册表查找 Aggressive 策略
        let selector = registry.move_selector("Aggressive");
        let result = select_move_coord(&reachable, IVec2::ZERO, IVec2::new(10, 0), 1, selector);
        assert_eq!(result, IVec2::new(6, 0));
    }
}
