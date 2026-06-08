// 特效模块：伤害数字弹出、上浮淡出动画

use bevy::prelude::*;

/// 伤害数字弹出标记
#[derive(Component)]
pub struct DamagePopup {
    /// 剩余时间（秒）
    pub timer: Timer,
}

/// 伤害数字弹出初始配置
#[derive(Component)]
pub struct DamagePopupConfig {
    /// 上浮速度
    pub float_speed: f32,
    /// 淡出开始时间比例
    pub fade_start: f32,
}

impl Default for DamagePopupConfig {
    fn default() -> Self {
        Self {
            float_speed: 40.0,
            fade_start: 0.5,
        }
    }
}

/// 生成伤害数字弹出
pub fn spawn_damage_popup(
    commands: &mut Commands,
    world_pos: Vec2,
    damage: i32,
    font: &Handle<Font>,
    is_crit: bool,
) {
    let color = if is_crit { Color::srgb(1.0, 0.2, 0.2) } else { Color::srgb(1.0, 0.9, 0.3) };
    let font_size = if is_crit { 22.0 } else { 16.0 };
    let text = if is_crit { format!("{}!", damage) } else { format!("{}", damage) };

    commands.spawn((
        Text2d::new(text),
        TextFont {
            font: font.clone(),
            font_size,
            ..default()
        },
        TextColor(color),
        TextLayout::new_with_no_wrap(),
        Transform::from_xyz(world_pos.x, world_pos.y + 20.0, 5.0),
        DamagePopup {
            timer: Timer::from_seconds(1.2, TimerMode::Once),
        },
        DamagePopupConfig::default(),
    ));
}

/// 更新伤害数字弹出（上浮 + 淡出）
pub fn update_damage_popups(
    time: Res<Time>,
    mut commands: Commands,
    mut popups: Query<(
        Entity,
        &mut DamagePopup,
        &mut Transform,
        &mut TextColor,
        &DamagePopupConfig,
    )>,
) {
    for (entity, mut popup, mut transform, mut color, config) in &mut popups {
        popup.timer.tick(time.delta());
        let ratio = popup.timer.fraction();

        // 上浮
        transform.translation.y += config.float_speed * time.delta_secs();

        // 淡出
        if ratio > config.fade_start {
            let fade_ratio = (ratio - config.fade_start) / (1.0 - config.fade_start);
            color.0.set_alpha(1.0 - fade_ratio);
        }

        // 超时移除
        if popup.timer.just_finished() {
            commands.entity(entity).try_despawn();
        }
    }
}

/// VFX 插件
pub struct VfxPlugin;

impl Plugin for VfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_damage_popups);
    }
}
