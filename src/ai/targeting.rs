use crate::battle::manhattan_distance;
use crate::character::Faction;
use bevy::prelude::*;

use super::behavior::TargetStrategy;

/// 单位快照（避免借用冲突）
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
    pub cooldowns: crate::skill::SkillCooldowns,
    pub ai_behavior_id: String,
}

/// 根据目标策略选择目标坐标
pub(crate) fn select_target_coord(
    snapshots: &[UnitSnapshot],
    my_coord: IVec2,
    strategy: TargetStrategy,
) -> IVec2 {
    let players: Vec<&UnitSnapshot> = snapshots
        .iter()
        .filter(|s| s.faction == Faction::Player)
        .collect();

    match strategy {
        TargetStrategy::Nearest => players
            .iter()
            .min_by_key(|s| manhattan_distance(my_coord, s.coord))
            .map(|s| s.coord),
        TargetStrategy::Weakest => players.iter().min_by_key(|s| s.hp as i32).map(|s| s.coord),
        TargetStrategy::MostDangerous => {
            players.iter().max_by_key(|s| s.atk as i32).map(|s| s.coord)
        }
        TargetStrategy::LowestHpPercent => players
            .iter()
            .min_by_key(|s| {
                if s.max_hp > 0.0 {
                    (s.hp / s.max_hp * 100.0) as i32
                } else {
                    0
                }
            })
            .map(|s| s.coord),
    }
    .unwrap_or(my_coord)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::SkillCooldowns;

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
        let result = select_target_coord(&snapshots, IVec2::ZERO, TargetStrategy::Nearest);
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
        let result = select_target_coord(&snapshots, IVec2::ZERO, TargetStrategy::Weakest);
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
        let result = select_target_coord(&snapshots, IVec2::ZERO, TargetStrategy::MostDangerous);
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
        let result = select_target_coord(&snapshots, IVec2::ZERO, TargetStrategy::LowestHpPercent);
        assert_eq!(result, IVec2::new(3, 0));
    }

    #[test]
    fn 目标策略_无玩家时返回自身位置() {
        let snapshots = vec![];
        let result = select_target_coord(&snapshots, IVec2::new(7, 7), TargetStrategy::Nearest);
        assert_eq!(result, IVec2::new(7, 7));
    }
}
