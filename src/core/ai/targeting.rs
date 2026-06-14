use crate::core::character::Faction;
use bevy::prelude::*;

use super::strategy::TargetSelector;

/// 单位快照（避免借用冲突）
#[derive(Clone)]
pub(crate) struct UnitSnapshot {
    pub entity: Entity,
    pub faction: Faction,
    pub coord: IVec2,
    pub atk: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub mov: u32,
    pub attack_range: u32,
    pub acted: bool,
    pub skill_ids: Vec<String>,
    pub cooldowns: crate::core::skill::SkillCooldowns,
    pub ai_behavior_id: String,
    /// 单位标签（用于解析地形成本计算器）
    pub tags: crate::core::tag::GameplayTags,
}

/// 根据目标策略选择目标坐标
/// 通过 trait 对象分发，替代原来的 enum+match 模式
pub(crate) fn select_target_coord(
    snapshots: &[UnitSnapshot],
    my_coord: IVec2,
    selector: &dyn TargetSelector,
) -> IVec2 {
    let players: Vec<UnitSnapshot> = snapshots
        .iter()
        .filter(|s| s.faction == Faction::Player)
        .cloned()
        .collect();

    selector.select(&players, my_coord).unwrap_or(my_coord)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ai::strategy::{
        AiStrategyRegistry, LowestHpPercentTarget, MostDangerousTarget, NearestTarget,
        WeakestTarget,
    };
    use crate::core::skill::SkillCooldowns;

    fn make_snapshot(
        entity: Entity,
        faction: Faction,
        coord: IVec2,
        atk: f32,
        hp: f32,
        max_hp: f32,
    ) -> UnitSnapshot {
        UnitSnapshot {
            entity,
            faction,
            coord,
            atk,
            hp,
            max_hp,
            mov: 5,
            attack_range: 1,
            acted: false,
            skill_ids: vec![],
            cooldowns: SkillCooldowns::default(),
            ai_behavior_id: "default".into(),
            tags: crate::core::tag::GameplayTags::default(),
        }
    }

    #[test]
    fn 目标策略_最近() {
        let p1 = make_snapshot(
            Entity::PLACEHOLDER,
            Faction::Player,
            IVec2::new(3, 0),
            10.0,
            20.0,
            20.0,
        );
        let p2 = make_snapshot(
            Entity::PLACEHOLDER,
            Faction::Player,
            IVec2::new(10, 10),
            10.0,
            20.0,
            20.0,
        );
        let snapshots = vec![p1, p2];
        let selector = NearestTarget;
        let result = select_target_coord(&snapshots, IVec2::ZERO, &selector);
        assert_eq!(result, IVec2::new(3, 0));
    }

    #[test]
    fn 目标策略_最弱() {
        let p1 = make_snapshot(
            Entity::PLACEHOLDER,
            Faction::Player,
            IVec2::new(3, 0),
            10.0,
            5.0,
            20.0,
        );
        let p2 = make_snapshot(
            Entity::PLACEHOLDER,
            Faction::Player,
            IVec2::new(5, 0),
            10.0,
            18.0,
            20.0,
        );
        let snapshots = vec![p1, p2];
        let selector = WeakestTarget;
        let result = select_target_coord(&snapshots, IVec2::ZERO, &selector);
        assert_eq!(result, IVec2::new(3, 0));
    }

    #[test]
    fn 目标策略_最危险() {
        let p1 = make_snapshot(
            Entity::PLACEHOLDER,
            Faction::Player,
            IVec2::new(3, 0),
            5.0,
            20.0,
            20.0,
        );
        let p2 = make_snapshot(
            Entity::PLACEHOLDER,
            Faction::Player,
            IVec2::new(5, 0),
            15.0,
            20.0,
            20.0,
        );
        let snapshots = vec![p1, p2];
        let selector = MostDangerousTarget;
        let result = select_target_coord(&snapshots, IVec2::ZERO, &selector);
        assert_eq!(result, IVec2::new(5, 0));
    }

    #[test]
    fn 目标策略_最低血量百分比() {
        let p1 = make_snapshot(
            Entity::PLACEHOLDER,
            Faction::Player,
            IVec2::new(3, 0),
            10.0,
            2.0,
            20.0,
        );
        let p2 = make_snapshot(
            Entity::PLACEHOLDER,
            Faction::Player,
            IVec2::new(5, 0),
            10.0,
            15.0,
            20.0,
        );
        let snapshots = vec![p1, p2];
        let selector = LowestHpPercentTarget;
        let result = select_target_coord(&snapshots, IVec2::ZERO, &selector);
        assert_eq!(result, IVec2::new(3, 0));
    }

    #[test]
    fn 目标策略_无玩家时返回自身位置() {
        let snapshots = vec![];
        let selector = NearestTarget;
        let result = select_target_coord(&snapshots, IVec2::new(7, 7), &selector);
        assert_eq!(result, IVec2::new(7, 7));
    }

    #[test]
    fn 目标策略_通过注册表分发() {
        let registry = AiStrategyRegistry::default();
        let p1 = make_snapshot(
            Entity::PLACEHOLDER,
            Faction::Player,
            IVec2::new(3, 0),
            10.0,
            5.0,
            20.0,
        );
        let p2 = make_snapshot(
            Entity::PLACEHOLDER,
            Faction::Player,
            IVec2::new(5, 0),
            15.0,
            18.0,
            20.0,
        );
        let snapshots = vec![p1, p2];

        // 通过注册表查找 Weakest 策略
        let selector = registry.target_selector("Weakest");
        let result = select_target_coord(&snapshots, IVec2::ZERO, selector);
        assert_eq!(result, IVec2::new(3, 0));
    }
}
