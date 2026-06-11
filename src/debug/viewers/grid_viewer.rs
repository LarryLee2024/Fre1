// Grid Viewer：运行时查看地形网格和占用情况
// 遵循铁律：复杂系统必须有可视化调试工具
// 优化：HashMap 预计算 unit 位置 + 视口裁剪，避免每帧 O(cells×units) 扫描

use crate::character::{Faction, GridPosition, Unit, UnitName};
use crate::map::TerrainRegistry;
use crate::map::runtime::{OccupancyGrid, TerrainGrid};
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContext;
use bevy_inspector_egui::egui;
use std::collections::HashMap;

/// Grid Viewer 可视状态：仅渲染视口范围内的行
#[derive(Resource)]
pub struct GridViewerState {
    /// 视口起始行（包含）
    pub scroll_row: i32,
    /// 每次滚动的行数
    pub page_rows: i32,
}

impl Default for GridViewerState {
    fn default() -> Self {
        Self {
            scroll_row: 0,
            page_rows: 20,
        }
    }
}

/// Grid Viewer 调试面板
pub fn grid_viewer_system(
    mut egui_ctx: Query<&mut EguiContext, With<bevy::window::PrimaryWindow>>,
    terrain_grid: Res<TerrainGrid>,
    terrain_registry: Res<TerrainRegistry>,
    occupancy: Res<OccupancyGrid>,
    units: Query<(Entity, &Unit, &UnitName, &GridPosition)>,
    mut viewer_state: ResMut<GridViewerState>,
) {
    let Ok(mut ctx) = egui_ctx.single_mut() else {
        return;
    };
    let ctx = ctx.get_mut();

    // 预计算 unit 位置 → O(1) 查找
    let unit_map: HashMap<IVec2, String> = units
        .iter()
        .map(|(_, u, n, gp)| {
            let faction = match u.faction {
                Faction::Player => "友",
                Faction::Enemy => "敌",
            };
            (gp.coord, format!("{}{}", faction, n.0))
        })
        .collect();

    let total_rows = terrain_grid.height as i32;
    let total_cols = terrain_grid.width as i32;

    egui::Window::new("Grid Viewer")
        .default_pos([10.0, 640.0])
        .default_size([400.0, 300.0])
        .show(ctx, |ui| {
            ui.label(format!(
                "地图尺寸: {}x{}",
                terrain_grid.width, terrain_grid.height
            ));

            // 视口导航
            ui.horizontal(|ui| {
                if ui.button("◀ 首页").clicked() {
                    viewer_state.scroll_row = 0;
                }
                if ui.button("▲ 上页").clicked() {
                    viewer_state.scroll_row =
                        (viewer_state.scroll_row - viewer_state.page_rows).max(0);
                }
                if ui.button("▼ 下页").clicked() {
                    viewer_state.scroll_row =
                        (viewer_state.scroll_row + viewer_state.page_rows).min(total_rows - 1);
                }
                if ui.button("末页 ▶").clicked() {
                    viewer_state.scroll_row = (total_rows - viewer_state.page_rows).max(0);
                }
                ui.label(format!("行 {}/{}", viewer_state.scroll_row + 1, total_rows));
            });

            ui.separator();

            // 地形概览（仅渲染视口行）
            ui.heading("地形");
            let start_row = viewer_state.scroll_row;
            let end_row = (start_row + viewer_state.page_rows).min(total_rows);

            egui::Grid::new("terrain_grid")
                .striped(true)
                .show(ui, |ui| {
                    for y in start_row..end_row {
                        for x in 0..total_cols {
                            let coord = IVec2::new(x, y);
                            let terrain_id = terrain_grid.get(coord).unwrap_or("?");
                            let terrain_name = terrain_registry
                                .get(terrain_id)
                                .map(|t| t.name.as_str())
                                .unwrap_or(terrain_id);

                            let cell_text = match unit_map.get(&coord) {
                                Some(occupied) => format!("{}:{}", terrain_name, occupied),
                                None => terrain_name.to_string(),
                            };

                            ui.label(&cell_text);
                        }
                        ui.end_row();
                    }
                });

            ui.separator();

            // 占用统计
            ui.heading("占用");
            let player_count = units
                .iter()
                .filter(|(_, u, _, _)| u.faction == Faction::Player)
                .count();
            let enemy_count = units
                .iter()
                .filter(|(_, u, _, _)| u.faction == Faction::Enemy)
                .count();
            ui.label(format!("友方: {}  敌方: {}", player_count, enemy_count));
        });
}
