// 单位模块：角色身份、阵营、生成
// 属性移至 Attributes 组件，标签移至 GameplayTags 组件，技能移至 SkillSlots 组件

use crate::assets::CnFont;
use crate::core::attribute::{AttributeKind, Attributes};
use crate::core::tag::{GameplayTag, GameplayTags};
use crate::data::buff_data::ActiveBuffs;
use crate::data::skill_data::{SkillCooldowns, SkillSlots};
use crate::map::GameMap;
use bevy::prelude::*;
use bevy::sprite::Anchor;

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

    // 玩家单位（名称, 坐标, 技能IDs, 阵营标签, HP, MaxHP, ATK, DEF, MOV, 攻击范围）
    let player_units: [(&str, IVec2, Vec<&str>, GameplayTag, i32, i32, i32, i32, u32, u32); 3] = [
        ("战士", IVec2::new(4, 3), vec!["basic_attack", "charge"], GameplayTag::WARRIOR, 30, 30, 10, 5, 5, 1),
        ("弓手", IVec2::new(3, 4), vec!["basic_attack", "pierce"], GameplayTag::ARCHER, 20, 20, 8, 3, 4, 3),
        ("法师", IVec2::new(2, 5), vec!["basic_attack", "fireball"], GameplayTag::MAGE, 18, 18, 12, 2, 3, 2),
    ];

    for (name, coord, skill_ids, class_tag, hp, max_hp, atk, def, mov, attack_range) in player_units {
        let world_pos = map.coord_to_world(coord);
        spawn_unit(
            &mut commands,
            world_pos,
            Faction::Player,
            name,
            coord,
            skill_ids,
            class_tag,
            hp,
            max_hp,
            atk,
            def,
            mov,
            attack_range,
            tile_size,
            bar_width,
            bar_height,
            &cn_font.handle,
        );
    }

    // 敌方单位
    let enemy_units: [(&str, IVec2, Vec<&str>, GameplayTag, i32, i32, i32, i32, u32, u32); 3] = [
        ("哥布林", IVec2::new(7, 5), vec!["basic_attack"], GameplayTag::WARRIOR, 20, 20, 7, 3, 4, 1),
        ("哥布林", IVec2::new(8, 3), vec!["basic_attack"], GameplayTag::WARRIOR, 20, 20, 7, 3, 4, 1),
        ("暗骑士", IVec2::new(6, 6), vec!["basic_attack", "charge"], GameplayTag::WARRIOR, 35, 35, 12, 6, 3, 1),
    ];

    for (name, coord, skill_ids, class_tag, hp, max_hp, atk, def, mov, attack_range) in enemy_units {
        let world_pos = map.coord_to_world(coord);
        spawn_unit(
            &mut commands,
            world_pos,
            Faction::Enemy,
            name,
            coord,
            skill_ids,
            class_tag,
            hp,
            max_hp,
            atk,
            def,
            mov,
            attack_range,
            tile_size,
            bar_width,
            bar_height,
            &cn_font.handle,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_unit(
    commands: &mut Commands,
    world_pos: Vec2,
    faction: Faction,
    name: &str,
    coord: IVec2,
    skill_ids: Vec<&str>,
    class_tag: GameplayTag,
    hp: i32,
    max_hp: i32,
    atk: i32,
    def: i32,
    mov: u32,
    attack_range: u32,
    tile_size: f32,
    bar_width: f32,
    bar_height: f32,
    font: &Handle<Font>,
) {
    let label: String = name.chars().take(1).collect();
    let unit_font = TextFont {
        font: font.clone(),
        font_size: 18.0,
        ..default()
    };

    // 构建 Attributes
    let mut attributes = Attributes::default();
    attributes.set_base(AttributeKind::Hp, hp as f32);
    attributes.set_base(AttributeKind::MaxHp, max_hp as f32);
    attributes.set_base(AttributeKind::Atk, atk as f32);
    attributes.set_base(AttributeKind::Def, def as f32);
    attributes.set_base(AttributeKind::Mov, mov as f32);
    attributes.set_base(AttributeKind::AttackRange, attack_range as f32);

    // 构建 GameplayTags
    let mut gameplay_tags = GameplayTags::default();
    gameplay_tags.add(class_tag);

    // 构建 SkillSlots
    let skill_slots = SkillSlots::new(skill_ids.into_iter().map(String::from).collect());

    commands.spawn((
        Sprite::from_color(faction.unit_color(), Vec2::splat(tile_size * 0.6)),
        Transform::from_xyz(world_pos.x, world_pos.y, 1.0),
        Unit {
            faction,
            acted: false,
        },
        UnitName(name.to_string()),
        GridPosition { coord },
        attributes,
        gameplay_tags,
        skill_slots,
        SkillCooldowns::default(),
        ActiveBuffs::default(),
        children![
            // 棋子名称标注（中央）
            (
                Text2d::new(label),
                unit_font,
                TextColor(Color::WHITE),
                TextLayout::new_with_no_wrap(),
                Transform::from_xyz(0.0, 0.0, 0.3),
            ),
            // HP 条背景（红色）
            (
                Sprite::from_color(Color::srgb(0.6, 0.1, 0.1), Vec2::new(bar_width, bar_height)),
                Transform::from_xyz(-bar_width / 2.0, tile_size * 0.4, 0.1),
                Anchor::CENTER_LEFT,
                HpBarBg,
            ),
            // HP 条前景（绿色）
            (
                Sprite::from_color(Color::srgb(0.1, 0.8, 0.1), Vec2::new(bar_width, bar_height)),
                Transform::from_xyz(-bar_width / 2.0, tile_size * 0.4, 0.2),
                Anchor::CENTER_LEFT,
                HpBarFg,
            ),
        ],
    ));
}

/// 单位管理插件
pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::{AppState, GameSet};
        app.add_systems(OnEnter(AppState::InGame), spawn_units.in_set(GameSet::Unit));
    }
}
