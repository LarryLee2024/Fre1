// 战斗工具模块：距离计算等纯函数
// 伤害计算已移至 core/effect.rs 的 Effect Pipeline

use bevy::prelude::*;

/// 计算曼哈顿距离
pub fn manhattan_distance(a: IVec2, b: IVec2) -> u32 {
    (a.x - b.x).unsigned_abs() as u32 + (a.y - b.y).unsigned_abs() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 曼哈顿距离_相邻格子() {
        assert_eq!(manhattan_distance(IVec2::new(0, 0), IVec2::new(1, 0)), 1);
        assert_eq!(manhattan_distance(IVec2::new(0, 0), IVec2::new(0, 1)), 1);
    }

    #[test]
    fn 曼哈顿距离_对角线() {
        assert_eq!(manhattan_distance(IVec2::new(0, 0), IVec2::new(3, 4)), 7);
    }

    #[test]
    fn 曼哈顿距离_同一位置() {
        assert_eq!(manhattan_distance(IVec2::new(2, 3), IVec2::new(2, 3)), 0);
    }

    #[test]
    fn 曼哈顿距离_负坐标() {
        assert_eq!(manhattan_distance(IVec2::new(-1, -2), IVec2::new(1, 2)), 6);
    }
}
