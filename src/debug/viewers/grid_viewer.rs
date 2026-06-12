// Grid Viewer：运行时查看地形网格和占用情况
// 遵循铁律：复杂系统必须有可视化调试工具

use crate::character::{Faction, GridPosition, Unit, UnitName};
use crate::map::TerrainRegistry;
use crate::map::runtime::{OccupancyGrid, TerrainGrid};
use bevy::prelude::*;
use bevy_inspector_egui::egui;
use std::collections::HashMap;

/// Grid Viewer 可视状态
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

/// 渲染 Grid 视图内容
pub fn render(
    ui: &mut egui::Ui,
    terrain_grid: &TerrainGrid,
    terrain_registry: &TerrainRegistry,
    _occupancy: &OccupancyGrid,
    units: &Query<(
        Entity,
        &Unit,
        &UnitName,
        &GridPosition,
        &crate::core::attribute::Attributes,
        &crate::equipment::EquipmentSlots,
        &crate::character::TraitCollection,
        &crate::skill::SkillSlots,
        &crate::skill::SkillCooldowns,
        &crate::core::tag::GameplayTags,
        Option<&crate::character::AiBehaviorId>,
        Option<&crate::buff::ActiveBuffs>,
    )>,
    viewer_state: &mut GridViewerState,
) {
    ui.heading("Grid Viewer");

    // 预计算 unit 位置
    let unit_map: HashMap<IVec2, String> = units
        .iter()
        .map(|(_, u, n, gp, ..)| {
            let faction = match u.faction {
                Faction::Player => "友",
                Faction::Enemy => "敌",
            };
            (gp.coord, format!("{}{}", faction, n.0))
        })
        .collect();

    let total_rows = terrain_grid.height as i32;
    let total_cols = terrain_grid.width as i32;

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

    // 地形概览
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
        .filter(|(_, u, ..)| u.faction == Faction::Player)
        .count();
    let enemy_count = units
        .iter()
        .filter(|(_, u, ..)| u.faction == Faction::Enemy)
        .count();
    ui.label(format!("友方: {}  敌方: {}", player_count, enemy_count));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_viewer_state_default_values() {
        let state = GridViewerState::default();
        assert_eq!(state.scroll_row, 0);
        assert_eq!(state.page_rows, 20);
    }

    #[test]
    fn pagination_first_page() {
        let mut state = GridViewerState {
            scroll_row: 60,
            page_rows: 20,
        };
        state.scroll_row = 0;
        assert_eq!(state.scroll_row, 0);
    }

    #[test]
    fn pagination_prev_page() {
        let mut state = GridViewerState {
            scroll_row: 40,
            page_rows: 20,
        };
        state.scroll_row = (state.scroll_row - state.page_rows).max(0);
        assert_eq!(state.scroll_row, 20);
    }

    #[test]
    fn pagination_prev_page_at_first_page() {
        let mut state = GridViewerState {
            scroll_row: 0,
            page_rows: 20,
        };
        state.scroll_row = (state.scroll_row - state.page_rows).max(0);
        assert_eq!(state.scroll_row, 0);
    }

    #[test]
    fn pagination_next_page() {
        let mut state = GridViewerState {
            scroll_row: 0,
            page_rows: 20,
        };
        let total_rows = 100;
        state.scroll_row = (state.scroll_row + state.page_rows).min(total_rows - 1);
        assert_eq!(state.scroll_row, 20);
    }

    #[test]
    fn pagination_next_page_clamped_to_last() {
        let mut state = GridViewerState {
            scroll_row: 85,
            page_rows: 20,
        };
        let total_rows = 100;
        state.scroll_row = (state.scroll_row + state.page_rows).min(total_rows - 1);
        assert_eq!(state.scroll_row, 99);
    }

    #[test]
    fn pagination_last_page() {
        let mut state = GridViewerState {
            scroll_row: 0,
            page_rows: 20,
        };
        let total_rows = 100;
        state.scroll_row = (total_rows - state.page_rows).max(0);
        assert_eq!(state.scroll_row, 80);
    }

    #[test]
    fn pagination_last_page_map_smaller_than_page() {
        let mut state = GridViewerState {
            scroll_row: 0,
            page_rows: 20,
        };
        let total_rows = 10;
        state.scroll_row = (total_rows - state.page_rows).max(0);
        assert_eq!(state.scroll_row, 0);
    }

    #[test]
    fn viewport_range_calculation() {
        let state = GridViewerState {
            scroll_row: 40,
            page_rows: 20,
        };
        let total_rows = 100;
        let start_row = state.scroll_row;
        let end_row = (start_row + state.page_rows).min(total_rows);
        assert_eq!(start_row, 40);
        assert_eq!(end_row, 60);
    }

    #[test]
    fn viewport_range_last_page_partial() {
        let state = GridViewerState {
            scroll_row: 85,
            page_rows: 20,
        };
        let total_rows = 100;
        let start_row = state.scroll_row;
        let end_row = (start_row + state.page_rows).min(total_rows);
        assert_eq!(start_row, 85);
        assert_eq!(end_row, 100);
    }
}
