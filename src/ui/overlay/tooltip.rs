//! TooltipOverlay — 工具提示系统

use bevy::prelude::*;

/// Tooltip 数据
#[derive(Debug, Clone)]
pub struct TooltipVm {
    pub content_key: &'static str,
    pub position: Vec2,
}

/// Tooltip 服务 — 300ms 延迟显示
#[derive(Resource, Debug)]
pub struct TooltipService {
    pub delay: Timer,
    pub active: bool,
    pub content_key: Option<&'static str>,
    pub position: Vec2,
}

impl Default for TooltipService {
    fn default() -> Self {
        Self {
            delay: Timer::from_seconds(0.3, TimerMode::Once),
            active: false,
            content_key: None,
            position: Vec2::ZERO,
        }
    }
}

impl TooltipService {
    pub fn show(&mut self, key: &'static str, pos: Vec2) {
        self.content_key = Some(key);
        self.position = pos;
        self.delay.reset();
        self.active = true;
    }

    pub fn hide(&mut self) {
        self.active = false;
        self.content_key = None;
    }
}
