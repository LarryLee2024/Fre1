// 背包面板：独立显示选中单位的背包内容

use crate::core::turn::AppState;
use crate::infrastructure::localization::{CurrentLocale, LocalizationService};
use crate::ui::theme::UiTheme;
use crate::ui::view_models::SelectedUnitView;
use crate::ui::widgets::layout::*;
use bevy::prelude::*;

/// 背包面板文本标签
#[derive(Component)]
pub enum InventoryLabel {
    Items,
}

/// 背包面板根节点
#[derive(Component)]
pub struct InventoryPanel;

/// 生成背包面板
pub fn spawn_inventory_panel(
    mut commands: Commands,
    theme: Res<UiTheme>,
    localization: Res<LocalizationService>,
    locale: Res<CurrentLocale>,
) {
    let title_text = localization.resolve("ui.inventory.title", &locale.0, None);
    let empty_text = localization.resolve("ui.inventory.empty", &locale.0, None);
    commands
        .spawn((
            panel_top_right(&theme, theme.gap_large, theme.gap_large, 260.0, 300.0),
            BackgroundColor(theme.panel_bg),
            InventoryPanel,
        ))
        .insert(Name::new("InventoryPanel"))
        .with_children(|parent| {
            parent.spawn((
                Text::new(title_text),
                TextFont {
                    font_size: theme.font_medium,
                    ..default()
                },
                TextColor(theme.text_primary),
            ));
            parent.spawn((
                Text::new(empty_text),
                TextFont {
                    font_size: theme.font_small,
                    ..default()
                },
                TextColor(theme.text_secondary),
                InventoryLabel::Items,
            ));
        });
}

/// 更新背包面板
pub fn update_inventory_panel(
    view: Res<SelectedUnitView>,
    mut query: Query<(&InventoryLabel, &mut Text)>,
    localization: Res<LocalizationService>,
    locale: Res<CurrentLocale>,
) {
    if !view.is_changed() {
        return;
    }

    for (label, mut text) in &mut query {
        match label {
            InventoryLabel::Items => {
                if !view.is_selected || view.inventory.is_empty() {
                    **text = localization.resolve("ui.inventory.empty", &locale.0, None);
                } else {
                    let lines: Vec<String> = view
                        .inventory
                        .iter()
                        .map(|i| {
                            if i.count > 1 {
                                format!("  {} x{} [{}]", i.item_name, i.count, i.rarity)
                            } else {
                                format!("  {} [{}]", i.item_name, i.rarity)
                            }
                        })
                        .collect();
                    **text = lines.join("\n");
                }
            }
        }
    }
}

/// 背包面板插件
pub struct InventoryPanelPlugin;

impl Plugin for InventoryPanelPlugin {
    fn build(&self, app: &mut App) {
        use crate::core::turn::GameSet;
        app.add_systems(
            OnEnter(AppState::InGame),
            spawn_inventory_panel.in_set(GameSet::Ui),
        )
        .add_systems(
            Update,
            update_inventory_panel.run_if(in_state(AppState::InGame)),
        );
    }
}
