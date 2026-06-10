// Grid Viewer：运行时查看地形网格和占用情况
// 遵循铁律：复杂系统必须有可视化调试工具

use crate::character::{Faction, GridPosition, Unit, UnitName};
use crate::map::TerrainRegistry;
use crate::map::runtime::{OccupancyGrid, TerrainGrid};
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContext;
use bevy_inspector_egui::egui;

/// Grid Viewer 调试面板
pub fn grid_viewer_system(
    mut egui_ctx: Query<&mut EguiContext, With<bevy::window::PrimaryWindow>>,
    terrain_grid: Res<TerrainGrid>,
    terrain_registry: Res<TerrainRegistry>,
    occupancy: Res<OccupancyGrid>,
    units: Query<(Entity, &Unit, &UnitName, &GridPosition)>,
) {
    let Ok(mut ctx) = egui_ctx.single_mut() else {
        return;
    };
    let ctx = ctx.get_mut();

    egui::Window::new("Grid Viewer")
        .default_pos([10.0, 400.0])
        .default_size([400.0, 300.0])
        .show(ctx, |ui| {
            ui.label(format!(
                "地图尺寸: {}x{}",
                terrain_grid.width, terrain_grid.height
            ));

            ui.separator();

            // 地形概览
            ui.heading("地形");
            egui::Grid::new("terrain_grid")
                .striped(true)
                .show(ui, |ui| {
                    for y in 0..terrain_grid.height {
                        for x in 0..terrain_grid.width {
                            let coord = IVec2::new(x as i32, y as i32);
                            let terrain_id = terrain_grid.get(coord).unwrap_or("?");
                            let terrain_name = terrain_registry
                                .get(terrain_id)
                                .map(|t| t.name.as_str())
                                .unwrap_or(terrain_id);

                            let occupied_by = units
                                .iter()
                                .find(|(_, _, _, gp)| gp.coord == coord)
                                .map(|(_, u, n, _)| {
                                    let faction = match u.faction {
                                        Faction::Player => "友",
                                        Faction::Enemy => "敌",
                                    };
                                    format!("{}{}", faction, n.0)
                                })
                                .unwrap_or_default();

                            let cell_text = if occupied_by.is_empty() {
                                terrain_name.to_string()
                            } else {
                                format!("{}:{}", terrain_name, occupied_by)
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
