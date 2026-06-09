// 角色组件定义：单位身份、阵营、位置、标记等

use crate::core::tag::GameplayTags;
use bevy::prelude::*;

/// 阵营
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Faction {
    Player,
    Enemy,
}

/// 战斗单位组件（身份与回合状态）
#[derive(Component)]
pub struct Unit {
    pub faction: Faction,
    pub acted: bool,
}

/// 单位名称
#[derive(Component)]
pub struct UnitName(pub String);

/// 单位种族
#[derive(Component)]
pub struct UnitRace(pub String);

/// 单位职业
#[derive(Component)]
pub struct UnitClass(pub String);

/// 单位所在格子坐标
#[derive(Component)]
pub struct GridPosition {
    pub coord: IVec2,
}

/// 选中标记
#[derive(Component)]
pub struct Selected;

/// Trait 授予的标签（独立存储，不会被 rebuild_tags_from_buffs 丢失）
#[derive(Component, Default, Debug, Clone)]
pub struct TraitGrantedTags(pub GameplayTags);

/// HP 条背景
#[derive(Component)]
pub struct HpBarBg;

/// HP 条前景
#[derive(Component)]
pub struct HpBarFg;

/// AI 行为 ID（敌方单位使用）
#[derive(Component, Default, Debug, Clone)]
pub struct AiBehaviorId(pub String);

/// 导航箭头标记（路径上的小圆点）
#[derive(Component)]
pub struct PathArrow;

/// 移动动画组件：挂在正在移动的单位上，系统逐格插值
#[derive(Component)]
pub struct MovingUnit {
    /// 路径坐标序列（含终点）
    pub path: Vec<IVec2>,
    /// 当前正在前往的路径索引
    pub current_index: usize,
    /// 每格移动耗时（秒）
    pub speed: f32,
    /// 当前格内已用时间
    pub elapsed: f32,
    /// 移动完成后的回调阶段
    pub next_phase: crate::turn::TurnPhase,
}

impl MovingUnit {
    /// 当前目标坐标
    pub fn target_coord(&self) -> Option<IVec2> {
        self.path.get(self.current_index).copied()
    }

    /// 是否已到达终点
    pub fn is_finished(&self) -> bool {
        self.current_index >= self.path.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::turn::TurnPhase;

    fn make_moving_unit(path: Vec<IVec2>, current_index: usize) -> MovingUnit {
        MovingUnit {
            path,
            current_index,
            speed: 0.1,
            elapsed: 0.0,
            next_phase: TurnPhase::SelectUnit,
        }
    }

    // ── target_coord 测试 ──

    #[test]
    fn 移动单位_目标坐标_在路径中() {
        let unit = make_moving_unit(
            vec![IVec2::new(0, 0), IVec2::new(1, 0), IVec2::new(2, 0)],
            1,
        );
        assert_eq!(unit.target_coord(), Some(IVec2::new(1, 0)));
    }

    #[test]
    fn 移动单位_目标坐标_空路径() {
        let unit = make_moving_unit(vec![], 0);
        assert_eq!(unit.target_coord(), None);
    }

    #[test]
    fn 移动单位_目标坐标_索引越界() {
        let unit = make_moving_unit(vec![IVec2::new(0, 0)], 5);
        assert_eq!(unit.target_coord(), None);
    }

    // ── is_finished 测试 ──

    #[test]
    fn 移动单位_是否完成_未完成() {
        let unit = make_moving_unit(vec![IVec2::new(0, 0), IVec2::new(1, 0)], 0);
        assert!(!unit.is_finished());
    }

    #[test]
    fn 移动单位_是否完成_已完成() {
        let unit = make_moving_unit(vec![IVec2::new(0, 0)], 1);
        assert!(unit.is_finished());
    }

    #[test]
    fn 移动单位_是否完成_空路径() {
        let unit = make_moving_unit(vec![], 0);
        assert!(unit.is_finished());
    }

    #[test]
    fn 移动单位_是否完成_刚到达终点() {
        let unit = make_moving_unit(vec![IVec2::new(0, 0), IVec2::new(1, 0)], 2);
        assert!(unit.is_finished());
    }
}
