// 行动菜单模块：弹出式行动菜单，使用 SkillSlots 动态生成按钮
// 使用 Widget 库构建，按钮交互通过 UiCommand Message 发出

use crate::buff::ActiveBuffs;
use crate::character::{GridPosition, Selected, TraitCollection, TraitRegistry, UnitName};
use crate::core::attribute::{AttributeKind, Attributes};
use crate::skill::{SkillRegistry, SkillSlots};
use crate::ui::events::UiCommand;
use crate::ui::focus::BlocksGameInput;
use crate::ui::theme::UiTheme;
use crate::ui::view_models::SelectedUnitView;
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
    view: &SelectedUnitView,
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
            button(theme),
            ActionMenuButton {
                kind: ActionKind::Attack,
            },
        ))
        .with_children(|parent| {
            parent.spawn(label("攻击", theme.font_menu, theme.text_primary));
        })
        .id();
    children_entities.push(attack_btn);

    // 技能按钮（从 ViewModel 读取）
    for skill_entry in &view.skills {
        if let Some(skill_data) = skill_registry.get(&skill_entry.id) {
            let skill_btn = commands
                .spawn((
                    Button,
                    button(theme),
                    ActionMenuButton {
                        kind: ActionKind::Skill(skill_entry.id.clone()),
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
            button(theme),
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
            button(theme),
            ActionMenuButton {
                kind: ActionKind::Cancel,
            },
        ))
        .with_children(|parent| {
            parent.spawn(label("取消", theme.font_menu, theme.text_cancel));
        })
        .id();
    children_entities.push(cancel_btn);

    // 根节点（使用 panel 样式，BlocksGameInput 阻止游戏输入）
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
            BlocksGameInput,
        ))
        .insert(Name::new("ActionMenu"))
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
    view: Res<SelectedUnitView>,
    mut menu_entity: ResMut<ActionMenuEntity>,
    skill_registry: Res<SkillRegistry>,
    theme: Res<UiTheme>,
    selected_query: Query<Entity, With<Selected>>,
    units: Query<(
        Entity,
        &UnitName,
        &Attributes,
        &SkillSlots,
        &ActiveBuffs,
        Option<&TraitCollection>,
        Option<&crate::equipment::EquipmentSlots>,
        Option<&crate::inventory::container::Container>,
        &GridPosition,
    )>,
    trait_registry: Res<TraitRegistry>,
    _equipment_registry: Res<crate::equipment::EquipmentRegistry>,
    _item_registry: Res<crate::inventory::definition::ItemRegistry>,
) {
    // 优先使用 SelectedUnitView，如果为空则从 Selected 实体构建
    let mut fallback_view = SelectedUnitView::default();
    let (grid_coord, view_ref) = if !view.name.is_empty() {
        (view.grid_coord, view.into_inner())
    } else if let Ok(entity) = selected_query.single() {
        if let Ok((
            _entity,
            name,
            attrs,
            skill_slots,
            buffs,
            trait_collection,
            _equipment_slots,
            _container,
            grid_pos,
        )) = units.get(entity)
        {
            fallback_view.name = name.0.clone();
            fallback_view.grid_coord = grid_pos.coord;
            fallback_view.is_selected = true;
            fallback_view.hp = attrs.get(AttributeKind::Hp) as i32;
            fallback_view.max_hp = attrs.get(AttributeKind::MaxHp) as i32;
            fallback_view.mp = attrs.get(AttributeKind::Mp) as i32;
            fallback_view.max_mp = attrs.get(AttributeKind::MaxMp) as i32;
            fallback_view.stamina = attrs.get(AttributeKind::Stamina) as i32;
            fallback_view.max_stamina = attrs.get(AttributeKind::MaxStamina) as i32;
            fallback_view.skills = skill_slots
                .skill_ids
                .iter()
                .filter_map(|id| {
                    skill_registry.get(id).map(|sd| crate::ui::view_models::SkillEntry {
                        name: sd.name.clone(),
                        id: id.to_string(),
                        cost_mp: sd.cost_mp,
                        range: sd.range,
                        cooldown: sd.cooldown,
                        description: sd.description.clone(),
                    })
                })
                .collect();
            fallback_view.traits = trait_collection
                .map(|tc| {
                    tc.trait_ids()
                        .iter()
                        .filter_map(|tid| {
                            trait_registry.get(tid).map(|td| crate::ui::view_models::TraitEntry {
                                name: td.name.clone(),
                                description: td.description.clone(),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();
            fallback_view.buffs = buffs
                .iter()
                .map(|inst| crate::ui::view_models::BuffEntry {
                    name: inst.name.clone(),
                    remaining_turns: inst.remaining_turns,
                    is_buff: inst.is_buff,
                })
                .collect();
            (grid_pos.coord, &fallback_view)
        } else {
            return;
        }
    } else {
        return;
    };

    let unit_world = map.coord_to_world(grid_coord);
    if let Ok((camera, cam_transform)) = camera_query.single() {
        if let Ok(screen_pos) = camera.world_to_viewport(cam_transform, unit_world.extend(1.0)) {
            spawn_action_menu(
                &mut commands,
                screen_pos.x,
                screen_pos.y,
                view_ref,
                &mut menu_entity,
                &skill_registry,
                &theme,
            );
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
