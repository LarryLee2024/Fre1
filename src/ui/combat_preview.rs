// 战斗预览浮窗：SelectTarget 阶段鼠标悬停敌方时显示预估伤害/命中/暴击

use crate::battle::CombatIntent;
use crate::map::GameMap;
use crate::turn::TurnPhase;
use crate::ui::theme::UiTheme;
use crate::ui::view_models::CombatPreviewView;
use bevy::prelude::*;

/// 战斗预览浮窗根节点标记
#[derive(Component)]
pub struct CombatPreviewRoot;

/// 战斗预览浮窗实体追踪
#[derive(Resource, Default)]
pub struct CombatPreviewEntity {
    pub entity: Option<Entity>,
}

/// 更新战斗预览浮窗
/// 在 SelectTarget 阶段，当 CombatPreviewView.is_visible 为 true 时显示
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
    // 只在 SelectTarget 阶段且预览可见时显示
    let should_show = *turn_phase.get() == TurnPhase::SelectTarget && preview_view.is_visible;

    if !should_show {
        despawn_preview(&mut commands, &mut preview_entity);
        return;
    }

    // 找到鼠标悬停的敌方单位（通过 CombatIntent 的 target_coord）
    let Some(target_coord) = combat_intent.target_coord else {
        despawn_preview(&mut commands, &mut preview_entity);
        return;
    };

    // 找目标单位的世界位置
    let target_world = map.coord_to_world(target_coord);

    // 先销毁旧浮窗
    despawn_preview(&mut commands, &mut preview_entity);

    // 生成新浮窗
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

    let panel_id = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(screen_pos.x + 20.0),
                top: Val::Px(screen_pos.y - 60.0),
                padding: UiRect::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.92)),
            CombatPreviewRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(info_text),
                TextFont {
                    font_size: theme.font_small,
                    ..default()
                },
                TextColor(damage_color),
            ));
        })
        .id();

    preview_entity.entity = Some(panel_id);
}

/// 销毁预览浮窗
fn despawn_preview(commands: &mut Commands, entity: &mut CombatPreviewEntity) {
    if let Some(e) = entity.entity {
        commands.entity(e).try_despawn();
        entity.entity = None;
    }
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
