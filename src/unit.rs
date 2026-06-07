// 单位模块：角色属性、阵营、生成

use bevy::prelude::*;
use crate::map::GameMap;

/// 阵营
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Faction {
    /// 玩家方
    Player,
    /// 敌方
    Enemy,
}

/// 战斗单位组件
#[derive(Component)]
pub struct Unit {
    /// 阵营
    pub faction: Faction,
    /// 移动力
    pub mov: u32,
    /// 生命值
    pub hp: i32,
    /// 最大生命值
    pub max_hp: i32,
    /// 攻击力
    pub atk: i32,
    /// 防御力
    pub def: i32,
    /// 攻击范围（曼哈顿距离）
    pub attack_range: u32,
    /// 本回合是否已行动
    pub acted: bool,
}

/// 单位所在格子坐标
#[derive(Component)]
pub struct GridPosition {
    pub coord: IVec2,
}

/// 选中标记
#[derive(Component)]
pub struct Selected;

/// 可移动范围标记
#[derive(Component)]
pub struct MovableRange;

/// 可攻击范围标记
#[derive(Component)]
pub struct AttackRange;

/// 阵营颜色
impl Faction {
    pub fn unit_color(&self) -> Color {
        match self {
            Faction::Player => Color::srgb(0.2, 0.5, 1.0),
            Faction::Enemy => Color::srgb(1.0, 0.3, 0.2),
        }
    }
}

/// 生成初始单位
pub fn spawn_units(
    mut commands: Commands,
    map: Res<GameMap>,
) {
    let tile_size = map.tile_size;

    // 玩家单位
    let player_units = [
        (IVec2::new(2, 2), "战士", 5, 30, 30, 10, 5, 1),
        (IVec2::new(3, 4), "弓手", 4, 20, 20, 8, 3, 3),
        (IVec2::new(2, 5), "法师", 3, 18, 18, 12, 2, 2),
    ];

    for (coord, _name, mov, hp, max_hp, atk, def, attack_range) in player_units {
        let world_pos = map.coord_to_world(coord);
        commands.spawn((
            Sprite::from_color(Faction::Player.unit_color(), Vec2::splat(tile_size * 0.6)),
            Transform::from_xyz(world_pos.x, world_pos.y, 1.0),
            Unit {
                faction: Faction::Player,
                mov,
                hp,
                max_hp,
                atk,
                def,
                attack_range,
                acted: false,
            },
            GridPosition { coord },
        ));
    }

    // 敌方单位
    let enemy_units = [
        (IVec2::new(7, 5), "哥布林", 4, 20, 20, 7, 3, 1),
        (IVec2::new(8, 3), "哥布林", 4, 20, 20, 7, 3, 1),
        (IVec2::new(6, 6), "暗骑士", 3, 35, 35, 12, 6, 1),
    ];

    for (coord, _name, mov, hp, max_hp, atk, def, attack_range) in enemy_units {
        let world_pos = map.coord_to_world(coord);
        commands.spawn((
            Sprite::from_color(Faction::Enemy.unit_color(), Vec2::splat(tile_size * 0.6)),
            Transform::from_xyz(world_pos.x, world_pos.y, 1.0),
            Unit {
                faction: Faction::Enemy,
                mov,
                hp,
                max_hp,
                atk,
                def,
                attack_range,
                acted: false,
            },
            GridPosition { coord },
        ));
    }
}
