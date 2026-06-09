// 行动菜单模块：弹出式行动菜单，使用 SkillSlots 动态生成按钮
// 使用 Widget 库构建，按钮交互通过 UiCommand Message 发出

use crate::character::Unit;
use crate::skill::{SkillRegistry, SkillSlots};
use crate::ui::events::UiCommand;
use crate::ui::theme::UiTheme;
use crate::ui::widgets::layout::*;
use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;

/// 行动类型
#[derive(Clone, Debug)]
pub enum ActionKind {
    Attack,
    Skill(String),
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
    theme: &UiTheme,
) {
    despawn_action_menu(commands, menu_entity);

    let mut children_entities: Vec<Entity> = Vec::new();

    // 攻击按钮
    let attack_btn = commands
        .spawn((
            Button,
            Node {
                padding: theme.button_padding,
                ..default()
            },
            ActionMenuButton {
                kind: ActionKind::Attack,
            },
        ))
        .with_children(|parent| {
            parent.spawn(label("攻击", theme.font_menu, theme.text_primary));
        })
        .id();
    children_entities.push(attack_btn);

    // 特殊技能按钮
    if let Some(skill_id) = skill_slots.special_skill() {
        if let Some(skill_data) = skill_registry.get(skill_id) {
            let skill_btn = commands
                .spawn((
                    Button,
                    Node {
                        padding: theme.button_padding,
                        ..default()
                    },
                    ActionMenuButton {
                        kind: ActionKind::Skill(skill_id.to_string()),
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(label(&skill_data.name, theme.font_menu, theme.text_skill));
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
                padding: theme.button_padding,
                ..default()
            },
            ActionMenuButton {
                kind: ActionKind::Wait,
            },
        ))
        .with_children(|parent| {
            parent.spawn(label("待机", theme.font_menu, theme.text_primary));
        })
        .id();
    children_entities.push(wait_btn);

    // 取消按钮
    let cancel_btn = commands
        .spawn((
            Button,
            Node {
                padding: theme.button_padding,
                ..default()
            },
            ActionMenuButton {
                kind: ActionKind::Cancel,
            },
        ))
        .with_children(|parent| {
            parent.spawn(label("取消", theme.font_menu, theme.text_cancel));
        })
        .id();
    children_entities.push(cancel_btn);

    // 根节点（使用 panel 样式）
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(x),
                top: Val::Px(y),
                flex_direction: FlexDirection::Column,
                padding: theme.panel_padding,
                ..default()
            },
            BackgroundColor(theme.panel_bg),
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

/// 处理行动菜单交互
pub fn handle_action_menu_interaction(
    interaction_query: Query<(&Interaction, &ActionMenuButton), Changed<Interaction>>,
    mut ui_commands: MessageWriter<UiCommand>,
) {
    for (interaction, button) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let cmd = match &button.kind {
            ActionKind::Attack => UiCommand::Attack,
            ActionKind::Skill(skill_id) => UiCommand::Skill {
                skill_id: skill_id.clone(),
            },
            ActionKind::Wait => UiCommand::Wait,
            ActionKind::Cancel => UiCommand::Cancel,
        };
        ui_commands.write(cmd);
    }
}

/// 进入行动菜单阶段时自动弹出菜单
fn on_enter_action_menu(
    mut commands: Commands,
    map: Res<crate::map::GameMap>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    selected_query: Query<
        (
            Entity,
            &crate::character::Unit,
            &crate::character::GridPosition,
            &crate::skill::SkillSlots,
        ),
        With<crate::character::Selected>,
    >,
    mut menu_entity: ResMut<ActionMenuEntity>,
    skill_registry: Res<SkillRegistry>,
    theme: Res<UiTheme>,
) {
    if let Ok((_, unit, gp, skill_slots)) = selected_query.single() {
        let unit_world = map.coord_to_world(gp.coord);
        if let Ok((camera, cam_transform)) = camera_query.single() {
            if let Ok(screen_pos) = camera.world_to_viewport(cam_transform, unit_world.extend(1.0))
            {
                spawn_action_menu(
                    &mut commands,
                    screen_pos.x,
                    screen_pos.y,
                    unit,
                    skill_slots,
                    &mut menu_entity,
                    &skill_registry,
                    &theme,
                );
            }
        }
    }
}

/// 行动菜单插件
pub struct ActionMenuPlugin;

impl Plugin for ActionMenuPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::{AppState, TurnPhase};
        app.init_resource::<ActionMenuEntity>()
            .add_systems(
                Update,
                handle_action_menu_interaction.run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnEnter(TurnPhase::ActionMenu), on_enter_action_menu);
    }
}
