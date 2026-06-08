// 角色组件定义：单位身份、阵营、位置、标记等

use crate::gameplay::tag::GameplayTags;
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

/// 可移动范围标记
#[derive(Component)]
pub struct MovableRange;

/// 可攻击范围标记
#[derive(Component)]
pub struct AttackRange;

/// HP 条背景
#[derive(Component)]
pub struct HpBarBg;

/// HP 条前景
#[derive(Component)]
pub struct HpBarFg;

/// 选中高亮（独立实体）
#[derive(Component)]
pub struct SelectionHighlight;

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

/// 阵营颜色
impl Faction {
    pub fn unit_color(&self) -> Color {
        match self {
            Faction::Player => Color::srgb(0.2, 0.5, 1.0),
            Faction::Enemy => Color::srgb(1.0, 0.3, 0.2),
        }
    }
}

/// 清除范围标记和高亮（不含 Selected 移除）
pub fn clear_markers(
    commands: &mut Commands,
    range_entities: &Query<(Entity, Option<&GridPosition>), Or<(With<MovableRange>, With<AttackRange>)>>,
    highlights: &Query<Entity, With<SelectionHighlight>>,
) {
    for (marker, _) in range_entities {
        commands.entity(marker).try_despawn();
    }
    for h in highlights {
        commands.entity(h).try_despawn();
    }
}
