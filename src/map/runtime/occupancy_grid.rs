// 占用网格：持久化单位占据信息，替代临时 HashMap
// OccupancyGrid 是单位占位的唯一真相源

use bevy::prelude::*;
use std::collections::HashMap;

/// 占用网格资源：记录每个坐标被哪个 Entity 占据
/// 替代寻路/移动时临时构建的 occupation_map HashMap
#[derive(Resource, Debug, Default)]
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
    pub fn rebuild(
        &mut self,
        units: impl Iterator<Item = (Entity, IVec2)>,
    ) {
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
    units: Query<(Entity, &crate::character::GridPosition)>,
) {
    grid.rebuild(units.iter().map(|(e, gp)| (e, gp.coord)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 设置和查询占用() {
        let mut grid = OccupancyGrid::default();
        let e1 = Entity::from_bits(1);
        grid.set(IVec2::new(2, 3), e1);
        assert!(grid.is_occupied(IVec2::new(2, 3)));
        assert!(!grid.is_occupied(IVec2::new(0, 0)));
        assert_eq!(grid.get_entity(IVec2::new(2, 3)), Some(e1));
    }

    #[test]
    fn 移除占用() {
        let mut grid = OccupancyGrid::default();
        let e1 = Entity::from_bits(1);
        grid.set(IVec2::new(2, 3), e1);
        let removed = grid.remove(IVec2::new(2, 3));
        assert_eq!(removed, Some(e1));
        assert!(!grid.is_occupied(IVec2::new(2, 3)));
    }

    #[test]
    fn 排除自身检查占用() {
        let mut grid = OccupancyGrid::default();
        let e1 = Entity::from_bits(1);
        let e2 = Entity::from_bits(2);
        grid.set(IVec2::new(2, 3), e1);
        // e1 自身位置不算被占用
        assert!(!grid.is_occupied_except(IVec2::new(2, 3), e1));
        // 其他单位算被占用
        assert!(grid.is_occupied_except(IVec2::new(2, 3), e2));
    }

    #[test]
    fn 重建占用表() {
        let mut grid = OccupancyGrid::default();
        let e1 = Entity::from_bits(1);
        let e2 = Entity::from_bits(2);
        let units = vec![(e1, IVec2::new(0, 0)), (e2, IVec2::new(3, 4))];
        grid.rebuild(units.into_iter());
        assert!(grid.is_occupied(IVec2::new(0, 0)));
        assert!(grid.is_occupied(IVec2::new(3, 4)));
        assert!(!grid.is_occupied(IVec2::new(1, 1)));
    }
}
