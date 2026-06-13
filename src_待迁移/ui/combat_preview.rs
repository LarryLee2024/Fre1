// 战斗预览浮窗：SelectTarget 阶段鼠标悬停敌方时显示预估伤害/命中/暴击
// 使用通用 Popup Widget

use crate::battle::CombatIntent;
use crate::map::GameMap;
use crate::turn::TurnPhase;
use crate::ui::theme::UiTheme;
use crate::ui::view_models::CombatPreviewView;
use crate::ui::widgets::popup::{add_popup_text, despawn_popup, spawn_popup};
use bevy::prelude::*;

/// 战斗预览浮窗实体追踪
#[derive(Resource, Default)]
pub struct CombatPreviewEntity {
    pub entity: Option<Entity>,
}

/// 更新战斗预览浮窗
pub fn update_combat_preview_popup(
    preview_view: Res<CombatPreviewView>,
    turn_phase: Res<State<TurnPhase>>,
    mut preview_entity: ResMut<CombatPreviewEntity>,
    mut commands: Commands,
    theme: Res<UiTheme>,
    map: Res<GameMap>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    combat_intent: Res<CombatIntent>,
) {
    let should_show = *turn_phase.get() == TurnPhase::SelectTarget && preview_view.is_visible;

    if !should_show {
        preview_entity.entity = despawn_popup(&mut commands, preview_entity.entity);
        return;
    }

    let Some(target_coord) = combat_intent.target_coord else {
        preview_entity.entity = despawn_popup(&mut commands, preview_entity.entity);
        return;
    };

    let target_world = map.coord_to_world(target_coord);
    preview_entity.entity = despawn_popup(&mut commands, preview_entity.entity);

    let Ok((camera, cam_transform)) = camera_query.single() else {
        return;
    };
    let Ok(screen_pos) = camera.world_to_viewport(cam_transform, target_world.extend(2.0)) else {
        return;
    };

    let damage_color = if preview_view.is_lethal {
        theme.crit_color
    } else {
        theme.damage_color
    };

    let info_text = format!(
        "伤害: {}\n命中: {}%\n暴击: {}%{}",
        preview_view.estimated_damage,
        preview_view.hit_rate,
        preview_view.crit_rate,
        if preview_view.is_lethal {
            "\n⚠ 致命!"
        } else {
            ""
        },
    );

    let panel_id = spawn_popup(&mut commands, &theme, screen_pos.x, screen_pos.y, None);
    commands.entity(panel_id).insert(Name::new("CombatPreview"));
    add_popup_text(
        &mut commands,
        panel_id,
        &info_text,
        theme.font_small,
        damage_color,
    );
    preview_entity.entity = Some(panel_id);
}

/// 战斗预览插件
pub struct CombatPreviewPlugin;

impl Plugin for CombatPreviewPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::AppState;
        app.init_resource::<CombatPreviewEntity>().add_systems(
            Update,
            update_combat_preview_popup.run_if(in_state(AppState::InGame)),
        );
    }
}
