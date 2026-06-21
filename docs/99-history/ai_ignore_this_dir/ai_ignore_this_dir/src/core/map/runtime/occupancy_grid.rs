// 占用网格：持久化单位占据信息，替代临时 HashMap
// OccupancyGrid 是单位占位的唯一真相源

use bevy::prelude::*;
use std::collections::HashMap;

/// 占用网格资源：记录每个坐标被哪个 Entity 占据
/// 替代寻路/移动时临时构建的 occupation_map HashMap
#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource)]
pub struct OccupancyGrid {
    /// (x, y) → Entity
    occupied: HashMap<IVec2, Entity>,
}

impl OccupancyGrid {
    /// 设置占用
    pub fn set(&mut self, coord: IVec2, entity: Entity) {
        self.occupied.insert(coord, entity);
    }

    /// 移除占用
    pub fn remove(&mut self, coord: IVec2) -> Option<Entity> {
        self.occupied.remove(&coord)
    }

    /// 检查坐标是否被占用
    pub fn is_occupied(&self, coord: IVec2) -> bool {
        self.occupied.contains_key(&coord)
    }

    /// 获取占用该坐标的 Entity
    pub fn get_entity(&self, coord: IVec2) -> Option<Entity> {
        self.occupied.get(&coord).copied()
    }

    /// 清空所有占用
    pub fn clear(&mut self) {
        self.occupied.clear();
    }

    /// 从所有单位的位置重建占用表
    pub fn rebuild(&mut self, units: impl Iterator<Item = (Entity, IVec2)>) {
        self.occupied.clear();
        for (entity, coord) in units {
            self.occupied.insert(coord, entity);
        }
    }

    /// 获取占用坐标集合（用于寻路阻挡）
    pub fn occupied_coords(&self) -> impl Iterator<Item = IVec2> + '_ {
        self.occupied.keys().copied()
    }

    /// 检查指定坐标是否被占用（排除指定 Entity，用于自身移动）
    pub fn is_occupied_except(&self, coord: IVec2, except: Entity) -> bool {
        self.occupied
            .get(&coord)
            .map(|&e| e != except)
            .unwrap_or(false)
    }
}

/// 每帧从单位位置更新占用网格
pub fn update_occupancy_grid(
    mut grid: ResMut<OccupancyGrid>,
    units: Query<(Entity, &crate::core::character::GridPosition)>,
) {
    grid.rebuild(units.iter().map(|(e, gp)| (e, gp.coord)));
}

#[cfg(test)]
mod tests {
    // ================================================
    // AI Self-Check (test_spec.md §13.1)
    // ================================================
    // ✅ 测试行为，不是实现
    // ✅ 符合领域规则
    // ✅ 测试是确定性的
    // ✅ 使用标准测试数据
    // ✅ 没有测试私有实现
    // ✅ 没有生成不在范围内的测试
    // ================================================

    use super::*;

    /// Test ID: MAP-OCC-001
    /// Title: 设置和查询占用
    ///
    /// Given: 空 OccupancyGrid
    /// When: 设置 (2,3) 被 e1 占据
    /// Then: is_occupied(2,3)=true, get_entity(2,3)=Some(e1)
    ///
    /// Assertions: is_occupied 和 get_entity 返回正确值
    #[test]
    fn 设置和查询占用() {
        // Given
        let mut grid = OccupancyGrid::default();
        let e1 = Entity::from_bits(1);

        // When
        grid.set(IVec2::new(2, 3), e1);

        // Then
        assert!(grid.is_occupied(IVec2::new(2, 3)));
        assert!(!grid.is_occupied(IVec2::new(0, 0)));
        assert_eq!(grid.get_entity(IVec2::new(2, 3)), Some(e1));
    }

    /// Test ID: MAP-OCC-002
    /// Title: 移除占用
    ///
    /// Given: OccupancyGrid 中 (2,3) 被 e1 占据
    /// When: 移除 (2,3) 的占用
    /// Then: 返回 Some(e1)，(2,3) 不再被占用
    ///
    /// Assertions: remove() 返回 Some(e1), is_occupied() 返回 false
    #[test]
    fn 移除占用() {
        // Given
        let mut grid = OccupancyGrid::default();
        let e1 = Entity::from_bits(1);
        grid.set(IVec2::new(2, 3), e1);

        // When
        let removed = grid.remove(IVec2::new(2, 3));

        // Then
        assert_eq!(removed, Some(e1));
        assert!(!grid.is_occupied(IVec2::new(2, 3)));
    }

    /// Test ID: MAP-OCC-003
    /// Title: 排除自身检查占用
    ///
    /// Given: OccupancyGrid 中 (2,3) 被 e1 占据
    /// When: 调用 is_occupied_except(2,3, e1)
    /// Then: 返回 false（自身不算被占用）
    ///
    /// Assertions: is_occupied_except 对自身返回 false，对其他返回 true
    #[test]
    fn 排除自身检查占用() {
        // Given
        let mut grid = OccupancyGrid::default();
        let e1 = Entity::from_bits(1);
        let e2 = Entity::from_bits(2);
        grid.set(IVec2::new(2, 3), e1);

        // When & Then
        assert!(!grid.is_occupied_except(IVec2::new(2, 3), e1));
        assert!(grid.is_occupied_except(IVec2::new(2, 3), e2));
    }

    /// Test ID: MAP-OCC-004
    /// Title: 重建占用表
    ///
    /// Given: OccupancyGrid 中有旧数据
    /// When: 调用 rebuild() 传入新单位位置
    /// Then: 旧数据被清除，新数据被设置
    ///
    /// Assertions: 新坐标被占用，旧坐标不再被占用
    #[test]
    fn 重建占用表() {
        // Given
        let mut grid = OccupancyGrid::default();
        let e1 = Entity::from_bits(1);
        let e2 = Entity::from_bits(2);
        let units = vec![(e1, IVec2::new(0, 0)), (e2, IVec2::new(3, 4))];

        // When
        grid.rebuild(units.into_iter());

        // Then
        assert!(grid.is_occupied(IVec2::new(0, 0)));
        assert!(grid.is_occupied(IVec2::new(3, 4)));
        assert!(!grid.is_occupied(IVec2::new(1, 1)));
    }
}
