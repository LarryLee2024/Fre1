// 地形信息浮窗模块：右键查看地面属性
// 使用通用 Popup Widget

use crate::core::character::Faction;
use crate::core::map::GameMap;
use crate::core::map::TerrainRegistry;
use crate::core::map::runtime::TerrainGrid;
use crate::core::turn::TurnPhase;
use crate::ui::theme::UiTheme;
use crate::ui::widgets::popup::{add_popup_text, despawn_popup, spawn_popup};
use bevy::prelude::*;

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
    terrain_grid: Res<TerrainGrid>,
    terrain_registry: Res<TerrainRegistry>,
    turn_state: Res<crate::core::turn::TurnState>,
    turn_phase: Res<State<TurnPhase>>,
    mut commands: Commands,
    mut tile_info_entity: ResMut<TileInfoEntity>,
    theme: Res<UiTheme>,
) {
    if turn_state.current_faction != Faction::Player {
        return;
    }

    let left_clicked = mouse_button.just_pressed(MouseButton::Left);
    let right_clicked = mouse_button.just_pressed(MouseButton::Right);

    // 左键关闭地形信息浮窗
    if left_clicked {
        tile_info_entity.entity = despawn_popup(&mut commands, tile_info_entity.entity);
    }

    if !right_clicked || *turn_phase.get() != TurnPhase::SelectUnit {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((camera, cam_transform)) = camera_query.single() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) else {
        return;
    };
    let coord = map.world_to_coord(world_pos);
    if !map.is_in_bounds(coord) {
        return;
    }

    tile_info_entity.entity = despawn_popup(&mut commands, tile_info_entity.entity);

    if let Some(terrain_id) = terrain_grid.get(coord) {
        if let Ok(screen_pos) = camera.world_to_viewport(cam_transform, world_pos.extend(0.0)) {
            let terrain_def = terrain_registry.get(terrain_id);
            let terrain_name = terrain_def
                .map(|def| def.name.as_str())
                .unwrap_or(terrain_id);
            let move_cost_str = terrain_def
                .and_then(|def| def.move_cost)
                .map(|c| format!("{}", c))
                .unwrap_or_else(|| "不可通行".to_string());
            let defense_bonus = terrain_def.map(|def| def.defense_bonus).unwrap_or(0);
            let info_text = format!(
                "坐标: ({}, {})\n地形: {}\n移动消耗: {}\n防御加成: {}",
                coord.x, coord.y, terrain_name, move_cost_str, defense_bonus,
            );

            let panel_id = spawn_popup(&mut commands, &theme, screen_pos.x, screen_pos.y, None);
            add_popup_text(
                &mut commands,
                panel_id,
                &info_text,
                theme.font_small,
                theme.tile_info_text,
            );
            tile_info_entity.entity = Some(panel_id);
        }
    }
}

/// 地形信息浮窗插件
pub struct TileInfoPlugin;

impl Plugin for TileInfoPlugin {
    fn build(&self, app: &mut App) {
        use crate::core::turn::AppState;
        app.init_resource::<TileInfoEntity>()
            .add_systems(Update, handle_tile_info.run_if(in_state(AppState::InGame)));
    }
}
