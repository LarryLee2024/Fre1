// 地形信息浮窗模块：右键查看地面属性

use crate::map::{GameMap, Tile};
use crate::turn::TurnPhase;
use crate::character::Faction;
use bevy::prelude::*;

/// 地形信息浮窗标记组件
#[derive(Component)]
pub struct TileInfoRoot;

/// 地形信息浮窗实体追踪
#[derive(Resource, Default)]
pub struct TileInfoEntity {
    pub entity: Option<Entity>,
}

/// 处理右键查看地形信息（SelectUnit 阶段）
pub fn handle_tile_info(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    map: Res<GameMap>,
    tiles: Query<&Tile>,
    turn_state: Res<crate::turn::TurnState>,
    turn_phase: Res<State<TurnPhase>>,
    mut commands: Commands,
    mut tile_info_entity: ResMut<TileInfoEntity>,
) {
    if turn_state.current_faction != Faction::Player {
        return;
    }

    let left_clicked = mouse_button.just_pressed(MouseButton::Left);
    let right_clicked = mouse_button.just_pressed(MouseButton::Right);

    // 左键关闭地形信息浮窗
    if left_clicked {
        despawn_tile_info(&mut commands, &mut tile_info_entity);
    }

    // 仅在 SelectUnit 阶段右键显示地形信息
    if !right_clicked || *turn_phase.get() != TurnPhase::SelectUnit {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, cam_transform)) = camera_query.single() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) else { return };
    let coord = map.world_to_coord(world_pos);
    if !map.is_in_bounds(coord) {
        return;
    }

    // 关闭旧浮窗
    despawn_tile_info(&mut commands, &mut tile_info_entity);

    // 查找该坐标的地形
    for tile in &tiles {
        if tile.coord == coord {
            if let Ok(screen_pos) = camera.world_to_viewport(cam_transform, world_pos.extend(0.0)) {
                spawn_tile_info(
                    &mut commands,
                    screen_pos.x,
                    screen_pos.y,
                    tile,
                    &mut tile_info_entity,
                );
            }
            return;
        }
    }
}

/// 安全销毁地形信息浮窗
fn despawn_tile_info(commands: &mut Commands, tile_info_entity: &mut TileInfoEntity) {
    if let Some(entity) = tile_info_entity.entity {
        commands.entity(entity).try_despawn();
        tile_info_entity.entity = None;
    }
}

/// 生成地形信息浮窗
fn spawn_tile_info(
    commands: &mut Commands,
    screen_x: f32,
    screen_y: f32,
    tile: &Tile,
    tile_info_entity: &mut TileInfoEntity,
) {
    let terrain = tile.terrain;
    let move_cost_str = match tile.move_cost {
        Some(c) => format!("{}", c),
        None => "不可通行".to_string(),
    };
    let info_text = format!(
        "坐标: ({}, {})\n地形: {}\n移动消耗: {}\n防御加成: {}",
        tile.coord.x,
        tile.coord.y,
        terrain.label(),
        move_cost_str,
        tile.defense_bonus,
    );

    let panel_id = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(screen_x + 20.0),
                top: Val::Px(screen_y - 40.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.92)),
        ))
        .insert(TileInfoRoot)
        .with_children(|parent| {
            parent.spawn((
                Text::new(info_text),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.95)),
                TextLayout::new_with_no_wrap(),
            ));
        })
        .id();

    tile_info_entity.entity = Some(panel_id);
}

/// 地形信息浮窗插件
pub struct TileInfoPlugin;

impl Plugin for TileInfoPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::AppState;
        app.init_resource::<TileInfoEntity>().add_systems(
            Update,
            handle_tile_info.run_if(in_state(AppState::InGame)),
        );
    }
}
