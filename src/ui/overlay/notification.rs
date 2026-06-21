//! NotificationOverlay — 非模态通知系统

use bevy::prelude::*;

use crate::ui::primitives::panel::{components::PanelVariant, factory::spawn_panel};
use crate::ui::primitives::text::{components::TextVariant, factory::spawn_localized_text};
use crate::ui::theme::Theme;

/// 通知优先级
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationPriority {
    Normal,
    Important,
    Critical,
}

/// 通知视图模型
#[derive(Debug, Clone)]
pub struct NotificationVm {
    pub message_key: &'static str,
    pub priority: NotificationPriority,
    pub duration_secs: f32,
}

/// 通知实例组件 — 附加到已生成的通知实体上
#[derive(Component, Debug, Clone, Reflect)]
pub struct NotificationInstance {
    pub timer: Timer,
}

/// 通知服务
#[derive(Resource, Debug, Clone)]
pub struct NotificationService {
    pub queue: Vec<NotificationVm>,
    pub active: Vec<Entity>,
}

impl Default for NotificationService {
    fn default() -> Self {
        Self {
            queue: Vec::new(),
            active: Vec::new(),
        }
    }
}

impl NotificationService {
    /// 将通知入队（按时间顺序显示）。
    pub fn push(&mut self, vm: NotificationVm) {
        self.queue.push(vm);
    }
}

/// 生成通知 Widget
pub fn spawn_notification(
    commands: &mut Commands,
    asset_server: &AssetServer,
    theme: &Theme,
    vm: &NotificationVm,
) -> Entity {
    let panel = spawn_panel(commands, theme, PanelVariant::Basic);
    let text = spawn_localized_text(
        commands,
        asset_server,
        theme,
        vm.message_key,
        "",
        TextVariant::Caption,
    );
    commands.entity(text).set_parent_in_place(panel);
    commands.entity(panel).insert(NotificationInstance {
        timer: Timer::from_seconds(vm.duration_secs, TimerMode::Once),
    });
    panel
}

/// 处理通知队列
pub fn process_notification_queue(
    mut service: ResMut<NotificationService>,
    mut commands: Commands,
    theme: Res<Theme>,
    asset_server: Res<AssetServer>,
) {
    let pending: Vec<NotificationVm> = service.queue.drain(..).collect();
    for vm in pending {
        let entity = spawn_notification(&mut commands, &asset_server, &theme, &vm);
        service.active.push(entity);
    }
}

/// 通知生命周期管理
pub fn tick_notifications(
    mut commands: Commands,
    mut query: Query<(Entity, &mut NotificationInstance)>,
    time: Res<Time>,
) {
    for (entity, mut instance) in query.iter_mut() {
        instance.timer.tick(time.delta());
        if instance.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
