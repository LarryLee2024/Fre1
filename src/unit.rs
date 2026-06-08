// 单位模块：角色身份、阵营、生成
// 属性移至 Attributes 组件，标签移至 GameplayTags 组件，技能移至 SkillSlots 组件
// 单位定义从 UnitTemplateRegistry 加载，替代硬编码数组

use crate::assets::CnFont;
use crate::core::attribute::Attributes;
use crate::core::tag::{GameplayTag, GameplayTags};
use crate::core::trait_def::{TraitCollection, TraitRegistry, apply_passive_traits};
use crate::data::buff_data::ActiveBuffs;
use crate::data::map_data::LevelRegistry;
use crate::data::skill_data::{SkillCooldowns, SkillSlots};
use crate::data::unit_template::UnitTemplateRegistry;
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

/// AI 行为 ID（敌方单位使用）
#[derive(Component, Default, Debug, Clone)]
pub struct AiBehaviorId(pub String);

/// 生成初始单位（从 LevelConfig 加载单位部署）
pub fn spawn_units(
    mut commands: Commands,
    map: Res<GameMap>,
    cn_font: Res<CnFont>,
    template_registry: Res<UnitTemplateRegistry>,
    level_registry: Res<LevelRegistry>,
    trait_registry: Res<TraitRegistry>,
) {
    let tile_size = map.tile_size;
    let bar_width = tile_size * 0.6;
    let bar_height = 4.0;

    // 从关卡配置获取单位部署
    let level = level_registry.first();
    if let Some(level) = level {
        for deploy in &level.player_units {
            let coord = IVec2::new(deploy.coord.0, deploy.coord.1);
            if let Some(template) = template_registry.get(&deploy.template) {
                let world_pos = map.coord_to_world(coord);
                spawn_unit_from_template(
                    &mut commands, world_pos, template, coord,
                    tile_size, bar_width, bar_height, &cn_font.handle, &trait_registry,
                );
            } else {
                bevy::log::error!("单位模板不存在: {}", deploy.template);
            }
        }
        for deploy in &level.enemy_units {
            let coord = IVec2::new(deploy.coord.0, deploy.coord.1);
            if let Some(template) = template_registry.get(&deploy.template) {
                let world_pos = map.coord_to_world(coord);
                spawn_unit_from_template(
                    &mut commands, world_pos, template, coord,
                    tile_size, bar_width, bar_height, &cn_font.handle, &trait_registry,
                );
            } else {
                bevy::log::error!("单位模板不存在: {}", deploy.template);
            }
        }
    }
}

fn spawn_unit_from_template(
    commands: &mut Commands,
    world_pos: Vec2,
    template: &crate::data::unit_template::UnitTemplate,
    coord: IVec2,
    tile_size: f32,
    bar_width: f32,
    bar_height: f32,
    font: &Handle<Font>,
    trait_registry: &TraitRegistry,
) {
    let label: String = template.name.chars().take(1).collect();
    let unit_font = TextFont {
        font: font.clone(),
        font_size: 18.0,
        ..default()
    };

    // 构建 Attributes
    let mut attributes = Attributes::default();
    for (kind, value) in &template.base_attributes {
        attributes.set_base(*kind, *value);
    }

    // 构建 GameplayTags（class_tag + trait 授予的标签）
    let mut gameplay_tags = GameplayTags::default();
    gameplay_tags.add(template.class_tag);

    // 应用 trait 被动效果
    let (trait_tags, trait_modifiers) = apply_passive_traits(&template.trait_ids, trait_registry);
    // 合并 trait 授予的标签
    for i in 0..64 {
        let bit = 1u64 << i;
        if trait_tags.0 & bit != 0 {
            gameplay_tags.add(GameplayTag(bit));
        }
    }
    // 应用 trait 属性修饰符
    for modifier in trait_modifiers {
        attributes.add_modifier(modifier);
    }

    // 构建 SkillSlots
    let skill_slots = SkillSlots::new(template.skill_ids.clone());

    // 构建 TraitCollection
    let trait_collection = TraitCollection::new(template.trait_ids.clone());

    // 构建 AiBehaviorId
    let ai_behavior_id = AiBehaviorId(template.ai_behavior.clone());

    commands.spawn((
        Sprite::from_color(template.faction.unit_color(), Vec2::splat(tile_size * 0.6)),
        Transform::from_xyz(world_pos.x, world_pos.y, 1.0),
        Unit {
            faction: template.faction,
            acted: false,
        },
        UnitName(template.name.clone()),
        GridPosition { coord },
        attributes,
        gameplay_tags,
        skill_slots,
        trait_collection,
        ai_behavior_id,
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
