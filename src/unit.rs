// 单位模块：角色属性、阵营、生成

use crate::assets::CnFont;
use crate::map::GameMap;
use bevy::prelude::*;
use bevy::sprite::Anchor;

/// 阵营
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Faction {
    /// 玩家方
    Player,
    /// 敌方
    Enemy,
}

/// 技能类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Skill {
    #[default]
    None,
    /// 冲锋：1.5倍伤害，近战
    Charge,
    /// 穿透箭：1.3倍伤害，无视50%防御，远程+1
    Pierce,
    /// 火球：1.8倍伤害，中程
    Fireball,
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
    /// 技能
    pub skill: Skill,
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
pub fn spawn_units(mut commands: Commands, map: Res<GameMap>, cn_font: Res<CnFont>) {
    let tile_size = map.tile_size;
    let bar_width = tile_size * 0.6;
    let bar_height = 4.0;

    // 玩家单位（名称, 坐标, 技能, 移动力, HP, MaxHP, ATK, DEF, 攻击范围）
    let player_units: [(&str, IVec2, Skill, u32, i32, i32, i32, i32, u32); 3] = [
        ("战士", IVec2::new(2, 2), Skill::Charge, 5, 30, 30, 10, 5, 1),
        ("弓手", IVec2::new(3, 4), Skill::Pierce, 4, 20, 20, 8, 3, 3),
        ("法师", IVec2::new(2, 5), Skill::Fireball, 3, 18, 18, 12, 2, 2),
    ];

    for (name, coord, skill, mov, hp, max_hp, atk, def, attack_range) in player_units {
        let world_pos = map.coord_to_world(coord);
        spawn_unit(
            &mut commands,
            world_pos,
            Faction::Player,
            name,
            coord,
            skill,
            mov,
            hp,
            max_hp,
            atk,
            def,
            attack_range,
            tile_size,
            bar_width,
            bar_height,
            &cn_font.handle,
        );
    }

    // 敌方单位
    let enemy_units: [(&str, IVec2, Skill, u32, i32, i32, i32, i32, u32); 3] = [
        ("哥布林", IVec2::new(7, 5), Skill::None, 4, 20, 20, 7, 3, 1),
        ("哥布林", IVec2::new(8, 3), Skill::None, 4, 20, 20, 7, 3, 1),
        ("暗骑士", IVec2::new(6, 6), Skill::Charge, 3, 35, 35, 12, 6, 1),
    ];

    for (name, coord, skill, mov, hp, max_hp, atk, def, attack_range) in enemy_units {
        let world_pos = map.coord_to_world(coord);
        spawn_unit(
            &mut commands,
            world_pos,
            Faction::Enemy,
            name,
            coord,
            skill,
            mov,
            hp,
            max_hp,
            atk,
            def,
            attack_range,
            tile_size,
            bar_width,
            bar_height,
            &cn_font.handle,
        );
    }
}

fn spawn_unit(
    commands: &mut Commands,
    world_pos: Vec2,
    faction: Faction,
    name: &str,
    coord: IVec2,
    skill: Skill,
    mov: u32,
    hp: i32,
    max_hp: i32,
    atk: i32,
    def: i32,
    attack_range: u32,
    tile_size: f32,
    bar_width: f32,
    bar_height: f32,
    font: &Handle<Font>,
) {
    // 取名称首字作为棋子标注
    let label: String = name.chars().take(1).collect();
    let unit_font = TextFont {
        font: font.clone(),
        font_size: 18.0,
        ..default()
    };

    commands.spawn((
        Sprite::from_color(faction.unit_color(), Vec2::splat(tile_size * 0.6)),
        Transform::from_xyz(world_pos.x, world_pos.y, 1.0),
        Unit {
            faction,
            mov,
            hp,
            max_hp,
            atk,
            def,
            attack_range,
            acted: false,
            skill,
        },
        UnitName(name.to_string()),
        GridPosition { coord },
        children![
            // 棋子名称标注（中央）
            (
                Text2d::new(label),
                unit_font,
                TextColor(Color::WHITE),
                TextLayout::new_with_no_wrap(),
                Transform::from_xyz(0.0, 0.0, 0.3),
            ),
            // HP 条背景（红色）- 锚点左对齐
            (
                Sprite::from_color(Color::srgb(0.6, 0.1, 0.1), Vec2::new(bar_width, bar_height)),
                Transform::from_xyz(-bar_width / 2.0, tile_size * 0.4, 0.1),
                Anchor::CENTER_LEFT,
                HpBarBg,
            ),
            // HP 条前景（绿色）- 锚点左对齐，从左端扣血
            (
                Sprite::from_color(Color::srgb(0.1, 0.8, 0.1), Vec2::new(bar_width, bar_height)),
                Transform::from_xyz(-bar_width / 2.0, tile_size * 0.4, 0.2),
                Anchor::CENTER_LEFT,
                HpBarFg,
            ),
        ],
    ));
}
