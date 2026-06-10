// 角色组件定义：单位身份、阵营、位置、标记等

use super::traits::TraitCollection;
use crate::buff::ActiveBuffs;
use crate::core::attribute::Attributes;
use crate::core::tag::GameplayTags;
use crate::equipment::EquipmentSlots;
use crate::inventory::container::Container;
use crate::skill::{SkillCooldowns, SkillSlots};
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;

/// 阵营
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Faction {
    Player,
    Enemy,
}

/// 战斗单位组件（身份与回合状态）
/// Required Components：生成 Unit 时自动插入默认组件，防止遗漏
#[derive(Component)]
#[require(
    Attributes,
    SkillSlots,
    SkillCooldowns,
    ActiveBuffs,
    GameplayTags,
    PersistentTags,
    TraitCollection,
    EquipmentSlots,
    Container,
    GridPosition
)]
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

impl Default for GridPosition {
    fn default() -> Self {
        Self { coord: IVec2::ZERO }
    }
}

/// 选中标记
#[derive(Component)]
pub struct Selected;

/// 死亡标记：HP 降为 0 时添加，Hook 自动清理固有状态
#[derive(Component)]
#[component(on_add = Dead::on_add_dead)]
pub struct Dead;

impl Dead {
    /// 死亡 Hook：标记已行动，移除选中状态
    fn on_add_dead(mut world: DeferredWorld, context: HookContext) {
        let entity = context.entity;
        bevy::log::trace!(target: "character", entity=?entity, "Dead hook triggered");
        // 标记已行动，防止死亡单位继续行动
        if let Some(mut unit) = world.get_mut::<Unit>(entity) {
            unit.acted = true;
        }
        // 移除选中标记
        world.commands().entity(entity).remove::<Selected>();
    }
}

/// 持久化标签（不被 rebuild 丢失，支持 Trait + Equipment 两层）
#[derive(Component, Default, Debug, Clone)]
pub struct PersistentTags {
    /// Trait 授予的标签（种族/职业/天赋，最持久）
    pub from_traits: GameplayTags,
    /// 装备授予的标签（穿脱变化）
    pub from_equipment: GameplayTags,
}

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
    use crate::core::attribute::Attributes;
    use crate::skill::SkillSlots;
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

    // ── Dead Hook 测试 ──

    #[test]
    fn dead_hook_标记已行动() {
        let mut world = World::new();
        let entity = world
            .spawn((Unit {
                faction: Faction::Player,
                acted: false,
            },))
            .id();
        world.entity_mut(entity).insert(Dead);
        let unit = world.get::<Unit>(entity).unwrap();
        assert!(unit.acted);
    }

    #[test]
    fn dead_hook_移除selected() {
        let mut world = World::new();
        let entity = world
            .spawn((
                Unit {
                    faction: Faction::Player,
                    acted: false,
                },
                Selected,
            ))
            .id();
        world.entity_mut(entity).insert(Dead);
        assert!(world.get::<Selected>(entity).is_none());
    }

    #[test]
    fn dead_hook_无selected时不报错() {
        let mut world = World::new();
        let entity = world
            .spawn((Unit {
                faction: Faction::Player,
                acted: false,
            },))
            .id();
        world.entity_mut(entity).insert(Dead);
        assert!(world.get::<Unit>(entity).unwrap().acted);
    }

    #[test]
    fn unit_必需组件_自动生成() {
        let mut world = World::new();
        let entity = world
            .spawn(Unit {
                faction: Faction::Player,
                acted: false,
            })
            .id();
        assert!(world.get::<Attributes>(entity).is_some());
        assert!(world.get::<SkillSlots>(entity).is_some());
        assert!(world.get::<GridPosition>(entity).is_some());
        assert!(world.get::<ActiveBuffs>(entity).is_some());
    }
}
