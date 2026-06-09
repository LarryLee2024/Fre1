// 战斗日志模块：日志数据、颜色常量、日志面板显示

use bevy::prelude::*;
use std::collections::VecDeque;

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
    pub entries: VecDeque<Vec<LogSegment>>,
}

impl CombatLog {
    /// 添加一条日志
    pub fn push(&mut self, segments: Vec<LogSegment>) {
        self.entries.push_back(segments);
        if self.entries.len() > MAX_LOG_LINES {
            self.entries.pop_front();
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

/// 战斗日志插件
pub struct CombatLogPlugin;

impl Plugin for CombatLogPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::AppState;
        app.init_resource::<CombatLog>()
            .add_systems(Update, update_combat_log.run_if(in_state(AppState::InGame)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_segment(text: &str) -> LogSegment {
        LogSegment {
            text: text.into(),
            color: log_color::NORMAL,
        }
    }

    #[test]
    fn 战斗日志_添加一条() {
        let mut log = CombatLog::default();
        log.push(vec![make_segment("测试")]);
        assert_eq!(log.entries.len(), 1);
    }

    #[test]
    fn 战斗日志_多条日志() {
        let mut log = CombatLog::default();
        for i in 0..5 {
            log.push(vec![make_segment(&format!("日志{}", i))]);
        }
        assert_eq!(log.entries.len(), 5);
    }

    #[test]
    fn 战斗日志_超过最大条数截断最旧() {
        let mut log = CombatLog::default();
        for i in 0..10 {
            log.push(vec![make_segment(&format!("日志{}", i))]);
        }
        // 超过 MAX_LOG_LINES(8)，应该只保留最新 8 条
        assert_eq!(log.entries.len(), 8);
        // 最旧的两条（0, 1）被移除，最新的是 2..9
        assert_eq!(log.entries[0][0].text, "日志2");
        assert_eq!(log.entries[7][0].text, "日志9");
    }

    #[test]
    fn 战斗日志_刚好等于最大条数不截断() {
        let mut log = CombatLog::default();
        for i in 0..8 {
            log.push(vec![make_segment(&format!("日志{}", i))]);
        }
        assert_eq!(log.entries.len(), 8);
        assert_eq!(log.entries[0][0].text, "日志0");
    }

    #[test]
    fn 战斗日志_多片段拼接() {
        let mut log = CombatLog::default();
        log.push(vec![
            LogSegment {
                text: "[".into(),
                color: log_color::NORMAL,
            },
            LogSegment {
                text: "战士".into(),
                color: log_color::PLAYER,
            },
            LogSegment {
                text: "] 攻击 ".into(),
                color: log_color::NORMAL,
            },
            LogSegment {
                text: "哥布林".into(),
                color: log_color::ENEMY,
            },
        ]);
        assert_eq!(log.entries.len(), 1);
        assert_eq!(log.entries[0].len(), 4);
    }
}
