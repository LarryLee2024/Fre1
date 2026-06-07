// 战斗日志模块：日志数据、颜色常量、日志面板显示

use bevy::prelude::*;

/// 最大日志条数
const MAX_LOG_LINES: usize = 8;

/// 日志片段（文字 + 颜色）
#[derive(Clone)]
pub struct LogSegment {
    pub text: String,
    pub color: Color,
}

/// 战斗日志资源
#[derive(Resource, Default)]
pub struct CombatLog {
    /// 每条日志由多个片段组成，片段间拼接显示
    pub entries: Vec<Vec<LogSegment>>,
}

impl CombatLog {
    /// 添加一条日志
    pub fn push(&mut self, segments: Vec<LogSegment>) {
        self.entries.push(segments);
        if self.entries.len() > MAX_LOG_LINES {
            self.entries.remove(0);
        }
    }
}

/// 日志颜色常量
pub mod log_color {
    use bevy::prelude::Color;
    pub const NORMAL: Color = Color::srgb(0.8, 0.8, 0.8);
    pub const DAMAGE: Color = Color::srgb(1.0, 0.4, 0.3);
    #[allow(dead_code)]
    pub const HEAL: Color = Color::srgb(0.3, 1.0, 0.4);
    pub const KILL: Color = Color::srgb(1.0, 0.2, 0.8);
    pub const PLAYER: Color = Color::srgb(0.4, 0.7, 1.0);
    pub const ENEMY: Color = Color::srgb(1.0, 0.6, 0.3);
    #[allow(dead_code)]
    pub const TURN: Color = Color::srgb(1.0, 1.0, 0.4);
    pub const TERRAIN: Color = Color::srgb(0.5, 0.8, 0.5);
}

/// 战斗日志面板标记
#[derive(Component)]
pub struct CombatLogPanel;

/// 更新战斗日志面板显示
pub fn update_combat_log(
    combat_log: Res<CombatLog>,
    mut query: Query<&mut Text, With<CombatLogPanel>>,
) {
    if combat_log.is_changed() {
        for mut text in &mut query {
            let display: String = combat_log
                .entries
                .iter()
                .map(|segments| {
                    segments
                        .iter()
                        .map(|s| s.text.as_str())
                        .collect::<Vec<&str>>()
                        .join("")
                })
                .collect::<Vec<String>>()
                .join("\n");
            **text = display;
        }
    }
}
