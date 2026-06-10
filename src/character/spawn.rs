// 单位生成系统：从模板生成初始单位

use crate::assets::CnFont;
use crate::buff::ActiveBuffs;
use crate::core::attribute::Attributes;
use crate::core::tag::{GameplayTag, GameplayTags};
use crate::map::GameMap;
use crate::map::LevelRegistry;
use crate::skill::{SkillCooldowns, SkillSlots};
use bevy::prelude::*;
use bevy::sprite::Anchor;

use super::components::*;
use super::template::UnitTemplateRegistry;
use super::traits::*;

/// 生成初始单位（从 LevelConfig 加载单位部署）
pub fn spawn_units(
    mut commands: Commands,
    map: Res<GameMap>,
    cn_font: Res<CnFont>,
    template_registry: Res<UnitTemplateRegistry>,
    level_registry: Res<LevelRegistry>,
    trait_registry: Res<TraitRegistry>,
    effect_handlers: Res<TraitEffectHandlerRegistry>,
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
                    &mut commands,
                    world_pos,
                    template,
                    coord,
                    tile_size,
                    bar_width,
                    bar_height,
                    &cn_font.handle,
                    &trait_registry,
                    &effect_handlers,
                );
            } else {
                bevy::log::error!(target: "character", template = %deploy.template, "单位模板不存在，该单位将被跳过");
            }
        }
        for deploy in &level.enemy_units {
            let coord = IVec2::new(deploy.coord.0, deploy.coord.1);
            if let Some(template) = template_registry.get(&deploy.template) {
                let world_pos = map.coord_to_world(coord);
                spawn_unit_from_template(
                    &mut commands,
                    world_pos,
                    template,
                    coord,
                    tile_size,
                    bar_width,
                    bar_height,
                    &cn_font.handle,
                    &trait_registry,
                    &effect_handlers,
                );
            } else {
                bevy::log::error!(target: "character", template = %deploy.template, "单位模板不存在，该单位将被跳过");
            }
        }
    }
}

/// 行动顺序数字标记
#[derive(Component)]
pub struct TurnOrderLabel;

fn spawn_unit_from_template(
    commands: &mut Commands,
    world_pos: Vec2,
    template: &super::template::UnitTemplate,
    coord: IVec2,
    tile_size: f32,
    bar_width: f32,
    bar_height: f32,
    font: &Handle<Font>,
    trait_registry: &TraitRegistry,
    effect_handlers: &TraitEffectHandlerRegistry,
) {
    let label: String = template.name.chars().take(1).collect();
    let unit_font = TextFont {
        font: font.clone(),
        font_size: 18.0,
        ..default()
    };

    // 构建 Attributes：从模板设置核心属性基础值
    let mut attributes = Attributes::default();
    for (kind, value) in &template.base_attributes {
        attributes.set_base(*kind, *value);
    }
    attributes.set_base_attack_range(template.base_attack_range);
    attributes.fill_vital_resources();

    // 构建 GameplayTags（class 标签由 Trait 授予，不再从模板直接添加）
    let mut gameplay_tags = GameplayTags::default();

    // 构建 TraitCollection（先创建，再应用被动效果）
    let trait_collection = TraitCollection::new(template.trait_ids.clone());

    // 应用 trait 被动效果
    let (trait_tags, trait_modifiers) =
        apply_passive_traits(&trait_collection, trait_registry, effect_handlers);
    // 合并 trait 授予的标签
    for i in 0..64 {
        let bit = 1u64 << i;
        if trait_tags.0 & bit != 0 {
            gameplay_tags.add(GameplayTag(bit));
        }
    }
    // 保存 trait 授予的标签（用于 rebuild_tags 恢复）
    let persistent_tags = PersistentTags {
        from_traits: trait_tags,
        from_equipment: GameplayTags::default(),
    };
    // 应用 trait 属性修饰符
    for modifier in trait_modifiers {
        attributes.add_modifier(modifier);
    }

    // 构建 SkillSlots
    let skill_slots = SkillSlots::new(template.skill_ids.clone());

    // 构建 AiBehaviorId
    let ai_behavior_id = AiBehaviorId(template.ai_behavior.clone());

    commands
        .spawn((
            Sprite::from_color(
                crate::ui::theme::faction_color(template.faction),
                Vec2::splat(tile_size * 0.6),
            ),
            Transform::from_xyz(world_pos.x, world_pos.y, 1.0),
            Unit {
                faction: template.faction,
                acted: false,
            },
            UnitName(template.name.clone()),
            UnitRace(template.race.clone()),
            UnitClass(template.class.clone()),
            GridPosition { coord },
            attributes,
            gameplay_tags,
            persistent_tags,
            skill_slots,
            trait_collection,
            ai_behavior_id,
            SkillCooldowns::default(),
            ActiveBuffs::default(),
        ))
        .with_children(|parent| {
            // 棋子名称标注（中央）
            parent.spawn((
                Text2d::new(label),
                unit_font,
                TextColor(Color::WHITE),
                TextLayout::new_with_no_wrap(),
                Transform::from_xyz(0.0, 0.0, 0.3),
            ));
            // 行动顺序数字（左上角，稍向右下偏移避免与地形坐标重叠）
            parent.spawn((
                Text2d::new("1"),
                TextFont {
                    font: font.clone(),
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.4)),
                TextLayout::new_with_no_wrap(),
                Transform::from_xyz(-tile_size * 0.2, tile_size * 0.2, 0.3),
                TurnOrderLabel,
            ));
            // HP 条背景（红色）
            parent.spawn((
                Sprite::from_color(Color::srgb(0.6, 0.1, 0.1), Vec2::new(bar_width, bar_height)),
                Transform::from_xyz(-bar_width / 2.0, tile_size * 0.4, 0.1),
                Anchor::CENTER_LEFT,
                HpBarBg,
            ));
            // HP 条前景（绿色）
            parent.spawn((
                Sprite::from_color(Color::srgb(0.1, 0.8, 0.1), Vec2::new(bar_width, bar_height)),
                Transform::from_xyz(-bar_width / 2.0, tile_size * 0.4, 0.2),
                Anchor::CENTER_LEFT,
                HpBarFg,
            ));
        });
}

/// 单位管理插件
pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::{AppState, GameSet};
        app.add_systems(OnEnter(AppState::InGame), spawn_units.in_set(GameSet::Unit));
    }
}
