// 战斗日志模块：日志数据、颜色常量、日志面板显示

use bevy::prelude::*;
use std::collections::VecDeque;

/// 最大日志条数
const MAX_LOG_LINES: usize = 50;

/// 日志片段（文字 + 颜色）
#[derive(Clone, Reflect)]
pub struct LogSegment {
    pub text: String,
    pub color: Color,
}

/// 战斗日志资源
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct CombatLog {
    /// 每条日志由多个片段组成，片段间拼接显示
    /// VecDeque 未实现 Reflect，用 #[reflect(ignore)] 跳过
    #[reflect(ignore)]
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
    pub const TURN: Color = Color::srgb(1.0, 1.0, 0.4);
    pub const TERRAIN: Color = Color::srgb(0.5, 0.8, 0.5);
}

/// 战斗日志面板标记
#[derive(Component)]
pub struct CombatLogPanel;

/// 战斗日志折叠按钮
#[derive(Component)]
pub struct CombatLogToggle;

/// 战斗日志内容容器（可滚动）
#[derive(Component)]
pub struct CombatLogContent;

use crate::shared::resettable::ResettableResource;

/// 战斗日志折叠状态
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct CombatLogCollapsed(pub bool);

impl ResettableResource for CombatLogCollapsed {}

/// 战斗日志面板尺寸（可拖动调整）
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct CombatLogSize {
    pub width: f32,
    pub height: f32,
    pub is_dragging: bool,
    pub drag_start: Option<(f32, f32, f32, f32)>, // (mouse_x, mouse_y, orig_w, orig_h)
}

impl Default for CombatLogSize {
    fn default() -> Self {
        Self {
            width: 420.0,
            height: 280.0,
            is_dragging: false,
            drag_start: None,
        }
    }
}

/// 日志面板右下角拖拽手柄
#[derive(Component)]
pub struct CombatLogResizeHandle;

/// 更新战斗日志面板显示（多色文本 + 折叠支持）
pub fn update_combat_log(
    combat_log: Res<CombatLog>,
    collapsed: Res<CombatLogCollapsed>,
    content_query: Query<Entity, With<CombatLogContent>>,
    mut commands: Commands,
    theme: Res<crate::ui::theme::UiTheme>,
) {
    if !combat_log.is_changed() && !collapsed.is_changed() {
        return;
    }

    for content_entity in &content_query {
        // 清除旧内容
        commands.entity(content_entity).despawn_children();

        if collapsed.0 {
            // 折叠状态：只显示最新一条
            if let Some(segments) = combat_log.entries.back() {
                spawn_log_line(&mut commands, content_entity, segments, &theme);
            }
        } else {
            // 展开状态：显示所有日志
            for segments in &combat_log.entries {
                spawn_log_line(&mut commands, content_entity, segments, &theme);
            }
        }
    }
}

/// 生成一条日志行（多色文本，横向排列）
fn spawn_log_line(
    commands: &mut Commands,
    parent_entity: Entity,
    segments: &[LogSegment],
    theme: &crate::ui::theme::UiTheme,
) {
    commands.entity(parent_entity).with_children(|parent| {
        // 每条日志用一个横向 Node 包裹，确保内容横向排列
        parent
            .spawn((Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                ..default()
            },))
            .with_children(|line| {
                for segment in segments {
                    line.spawn((
                        Text::new(&segment.text),
                        TextFont {
                            font_size: theme.font_log,
                            ..default()
                        },
                        TextColor(segment.color),
                    ));
                }
            });
    });
}

/// 回合切换时添加回合日志（AGI驱动，不再分阵营阶段）
/// 只在回合数变化时才添加日志（即整个队列行动完一轮后）
pub fn log_turn_change(
    turn_state: Res<crate::core::turn::TurnState>,
    mut combat_log: ResMut<CombatLog>,
    mut last_turn: Local<u32>,
) {
    if turn_state.turn_number != *last_turn {
        *last_turn = turn_state.turn_number;
        combat_log.push(vec![LogSegment {
            text: format!("── 第{}回合 ──", turn_state.turn_number),
            color: log_color::TURN,
        }]);
    }
}

/// 拖拽调整日志面板大小
pub fn drag_resize_log_panel(
    mouse_button: Res<bevy::input::ButtonInput<bevy::input::mouse::MouseButton>>,
    windows: Query<&Window>,
    mut log_size: ResMut<CombatLogSize>,
    handle_query: Query<&Interaction, With<CombatLogResizeHandle>>,
    mut panel_query: Query<&mut Node, With<CombatLogPanel>>,
) {
    // 检测拖拽手柄按下
    for interaction in &handle_query {
        if *interaction == Interaction::Pressed
            && mouse_button.pressed(bevy::input::mouse::MouseButton::Left)
        {
            if !log_size.is_dragging {
                if let Ok(window) = windows.single() {
                    if let Some(cursor) = window.cursor_position() {
                        log_size.is_dragging = true;
                        log_size.drag_start =
                            Some((cursor.x, cursor.y, log_size.width, log_size.height));
                    }
                }
            }
        }
    }

    // 拖拽中
    if log_size.is_dragging {
        if let Some(start) = log_size.drag_start {
            if let Ok(window) = windows.single() {
                if let Some(cursor) = window.cursor_position() {
                    let dx = start.0 - cursor.x; // 面板在右侧，向左拖增大宽度
                    let dy = cursor.y - start.1; // 向下拖增大高度
                    log_size.width = (start.2 + dx).clamp(200.0, 800.0);
                    log_size.height = (start.3 + dy).clamp(100.0, 600.0);

                    // 实时更新面板尺寸
                    for mut node in &mut panel_query {
                        node.width = Val::Px(log_size.width);
                        node.height = Val::Px(log_size.height);
                    }
                }
            }
        }

        // 松开鼠标结束拖拽
        if !mouse_button.pressed(bevy::input::mouse::MouseButton::Left) {
            log_size.is_dragging = false;
            log_size.drag_start = None;
        }
    }
}

/// 战斗日志插件
pub struct CombatLogPlugin;

impl Plugin for CombatLogPlugin {
    fn build(&self, app: &mut App) {
        use crate::core::turn::AppState;
        app.init_resource::<CombatLog>()
            .init_resource::<CombatLogSize>()
            // 注册 Reflect 类型
            .register_type::<LogSegment>()
            .register_type::<CombatLog>()
            .register_type::<CombatLogCollapsed>()
            .register_type::<CombatLogSize>()
            .add_systems(Update, update_combat_log.run_if(in_state(AppState::InGame)))
            .add_systems(Update, log_turn_change.run_if(in_state(AppState::InGame)))
            .add_systems(
                Update,
                drag_resize_log_panel.run_if(in_state(AppState::InGame)),
            );
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
        for i in 0..60 {
            log.push(vec![make_segment(&format!("日志{}", i))]);
        }
        assert_eq!(log.entries.len(), MAX_LOG_LINES);
    }

    #[test]
    fn 战斗日志_刚好等于最大条数不截断() {
        let mut log = CombatLog::default();
        for i in 0..MAX_LOG_LINES {
            log.push(vec![make_segment(&format!("日志{}", i))]);
        }
        assert_eq!(log.entries.len(), MAX_LOG_LINES);
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
