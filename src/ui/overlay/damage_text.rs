//! DamageTextOverlay — 伤害浮字系统

use bevy::prelude::*;

/// 伤害数字组件
#[derive(Component, Debug, Clone, Reflect)]
pub struct DamageNumber {
    pub value: i32,
    pub timer: Timer,
}

/// 生成伤害数字浮字
pub fn spawn_damage_number(
    commands: &mut Commands,
    theme: &crate::ui::Theme,
    value: i32,
) -> Entity {
    let text = commands
        .spawn((
            Text::new(format!("{}", value)),
            TextFont {
                font_size: FontSize::Px(24.0),
                ..default()
            },
            TextColor(if value < 0 {
                theme.colors.feedback_negative
            } else {
                theme.colors.feedback_positive
            }),
            DamageNumber {
                value,
                timer: Timer::from_seconds(1.5, TimerMode::Once),
            },
        ))
        .id();
    text
}

/// 伤害数字生命周期
pub fn tick_damage_numbers(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DamageNumber)>,
    time: Res<Time>,
) {
    for (entity, mut dn) in query.iter_mut() {
        dn.timer.tick(time.delta());
        if dn.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
