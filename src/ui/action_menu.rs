// 行动菜单模块：弹出式行动菜单，使用 SkillSlots 动态生成按钮

use crate::battle::{CombatIntent, PrevPosition};
use crate::gameplay::attribute::{AttributeKind, Attributes};
use crate::skill::{effective_skill_range, BASIC_ATTACK_ID, SkillRegistry, SkillSlots};
use crate::input;
use crate::map::GameMap;
use crate::turn::TurnPhase;
use crate::character::{
    AttackRange, GridPosition, MovableRange, Selected, SelectionHighlight, Unit,
};
use bevy::prelude::*;

/// 行动类型
#[derive(Clone, Debug)]
pub enum ActionKind {
    Attack,
    Skill(String), // 技能 ID
    Wait,
    Cancel,
}

/// 菜单容器标记
#[derive(Component)]
pub struct ActionMenuRoot;

/// 菜单按钮标记
#[derive(Component)]
pub struct ActionMenuButton {
    pub kind: ActionKind,
}

/// 追踪菜单实体防止重复
#[derive(Resource, Default)]
pub struct ActionMenuEntity {
    pub entity: Option<Entity>,
}

/// 生成行动菜单
pub fn spawn_action_menu(
    commands: &mut Commands,
    x: f32,
    y: f32,
    _unit: &Unit,
    skill_slots: &SkillSlots,
    menu_entity: &mut ActionMenuEntity,
    skill_registry: &SkillRegistry,
) {
    despawn_action_menu(commands, menu_entity);

    let mut children_entities: Vec<Entity> = Vec::new();

    // 基础攻击按钮
    let attack_btn = commands
        .spawn((
            Button,
            Node {
                padding: UiRect::px(8.0, 8.0, 4.0, 4.0),
                ..default()
            },
            ActionMenuButton {
                kind: ActionKind::Attack,
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("攻击"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::WHITE),
            ));
        })
        .id();
    children_entities.push(attack_btn);

    // 特殊技能按钮（如果有）
    if let Some(skill_id) = skill_slots.special_skill() {
        if let Some(skill_data) = skill_registry.get(skill_id) {
            let skill_btn = commands
                .spawn((
                    Button,
                    Node {
                        padding: UiRect::px(8.0, 8.0, 4.0, 4.0),
                        ..default()
                    },
                    ActionMenuButton {
                        kind: ActionKind::Skill(skill_id.to_string()),
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(&skill_data.name),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::srgb(1.0, 0.8, 0.3)),
                    ));
                })
                .id();
            children_entities.push(skill_btn);
        }
    }

    // 待机按钮
    let wait_btn = commands
        .spawn((
            Button,
            Node {
                padding: UiRect::px(8.0, 8.0, 4.0, 4.0),
                ..default()
            },
            ActionMenuButton { kind: ActionKind::Wait },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("待机"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::WHITE),
            ));
        })
        .id();
    children_entities.push(wait_btn);

    // 取消按钮
    let cancel_btn = commands
        .spawn((
            Button,
            Node {
                padding: UiRect::px(8.0, 8.0, 4.0, 4.0),
                ..default()
            },
            ActionMenuButton {
                kind: ActionKind::Cancel,
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("取消"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        })
        .id();
    children_entities.push(cancel_btn);

    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(x),
                top: Val::Px(y),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            ActionMenuRoot,
        ))
        .id();

    for &child in &children_entities {
        commands.entity(root).add_child(child);
    }

    menu_entity.entity = Some(root);
}

/// 安全销毁菜单
pub fn despawn_action_menu(commands: &mut Commands, menu_entity: &mut ActionMenuEntity) {
    if let Some(entity) = menu_entity.entity {
        commands.entity(entity).try_despawn();
        menu_entity.entity = None;
    }
}

/// 取消移动：回退位置 + 清除选中
pub fn cancel_move(
    commands: &mut Commands,
    selected_query: &Query<Entity, With<Selected>>,
    range_entities: &Query<(Entity, Option<&GridPosition>), Or<(With<MovableRange>, With<AttackRange>)>>,
    highlights: &Query<Entity, With<SelectionHighlight>>,
    prev_position: &PrevPosition,
    map: &GameMap,
    menu_entity: &mut ActionMenuEntity,
) {
    despawn_action_menu(commands, menu_entity);

    if let Ok(selected_entity) = selected_query.single() {
        if let Some(prev_coord) = prev_position.coord {
            let world_pos = map.coord_to_world(prev_coord);
            commands
                .entity(selected_entity)
                .insert(Transform::from_xyz(world_pos.x, world_pos.y, 1.0))
                .insert(GridPosition { coord: prev_coord });
        }
    }

    input::clear_selection(commands, selected_query, range_entities, highlights);
}

/// 处理行动菜单交互
pub fn handle_action_menu_interaction(
    mut commands: Commands,
    interaction_query: Query<(&Interaction, &ActionMenuButton), Changed<Interaction>>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    selected_units: Query<(Entity, &Unit, &GridPosition, &Attributes), With<Selected>>,
    mut combat_intent: ResMut<CombatIntent>,
    mut menu_entity: ResMut<ActionMenuEntity>,
    map: Res<GameMap>,
    skill_registry: Res<SkillRegistry>,
) {
    for (interaction, button) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match &button.kind {
            ActionKind::Attack => {
                combat_intent.skill_id = Some(BASIC_ATTACK_ID.to_string());
                if let Ok((_, _, gp, attrs)) = selected_units.single() {
                    let base_range = attrs.get(AttributeKind::AttackRange) as u32;
                    if let Some(skill_data) = skill_registry.get(BASIC_ATTACK_ID) {
                        let range = effective_skill_range(skill_data, base_range);
                        input::show_attack_range(&mut commands, &map, gp.coord, range);
                    }
                }
                despawn_action_menu(&mut commands, &mut menu_entity);
                next_phase.set(TurnPhase::SelectTarget);
            }
            ActionKind::Skill(skill_id) => {
                combat_intent.skill_id = Some(skill_id.clone());
                if let Ok((_, _, gp, attrs)) = selected_units.single() {
                    let base_range = attrs.get(AttributeKind::AttackRange) as u32;
                    if let Some(skill_data) = skill_registry.get(skill_id) {
                        let range = effective_skill_range(skill_data, base_range);
                        input::show_attack_range(&mut commands, &map, gp.coord, range);
                    }
                }
                despawn_action_menu(&mut commands, &mut menu_entity);
                next_phase.set(TurnPhase::SelectTarget);
            }
            ActionKind::Wait => {
                despawn_action_menu(&mut commands, &mut menu_entity);
                next_phase.set(TurnPhase::WaitAction);
            }
            ActionKind::Cancel => {
                // 取消需要回退位置，但 cancel_move 需要 prev_position
                // 这里简化处理：直接回到 SelectUnit
                despawn_action_menu(&mut commands, &mut menu_entity);
                combat_intent.target_coord = None;
                combat_intent.skill_id = None;
                next_phase.set(TurnPhase::SelectUnit);
            }
        }
    }
}

/// 行动菜单插件
pub struct ActionMenuPlugin;

impl Plugin for ActionMenuPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::AppState;
        app.init_resource::<ActionMenuEntity>()
            .add_systems(
                Update,
                handle_action_menu_interaction.run_if(in_state(AppState::InGame)),
            );
    }
}
