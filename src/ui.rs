// UI 模块：信息面板、行动菜单、回合提示、HP 条、战斗日志
// 使用 Bevy 0.19 bsn! 宏 + FontSource/FontSize API
// bsn! 中 TextFont.font 使用默认值，通过 setup_ui_font 系统后置更新

use crate::turn::{AppState, TurnPhase, TurnState};
use crate::unit::{Faction, HpBarFg, Selected, Unit};
use bevy::prelude::*;

/// 中文字体路径
const CN_FONT: &str = "fonts/Arial Unicode.ttf";

/// 最大日志条数
const MAX_LOG_LINES: usize = 8;

// ── 战斗日志系统 ──

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
    pub const HEAL: Color = Color::srgb(0.3, 1.0, 0.4);
    pub const KILL: Color = Color::srgb(1.0, 0.2, 0.8);
    pub const PLAYER: Color = Color::srgb(0.4, 0.7, 1.0);
    pub const ENEMY: Color = Color::srgb(1.0, 0.6, 0.3);
    pub const TURN: Color = Color::srgb(1.0, 1.0, 0.4);
    pub const TERRAIN: Color = Color::srgb(0.5, 0.8, 0.5);
}

/// 战斗日志面板标记
#[derive(Component, Default, Clone)]
pub struct CombatLogPanel;

// ── UI 组件标记 ──

/// 回合提示文本
#[derive(Component, Default, Clone)]
pub struct TurnIndicator;

/// 选中单位信息文本
#[derive(Component, Default, Clone)]
pub struct UnitInfoPanel;

/// 行动菜单提示文本
#[derive(Component, Default, Clone)]
pub struct ActionMenuText;

/// 标记需要设置中文字体的 UI 实体
#[derive(Component, Default, Clone)]
pub struct NeedsFont;

/// 生成 UI（使用 bsn! 宏声明式场景）
/// TextFont.font 字段因 bsn! FromTemplate 枚举限制，使用默认值
/// 通过 setup_ui_font 系统后置覆盖字体
pub fn spawn_ui(mut commands: Commands) {
    // 回合提示
    commands.spawn_scene(bsn! {
        TurnIndicator
        NeedsFont
        Text("第 1 回合 - 玩家阶段")
        TextFont {
            font_size: FontSize::Px(24.0),
        }
        TextColor(Color::WHITE)
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
        }
    });

    // 操作提示
    commands.spawn_scene(bsn! {
        NeedsFont
        Text("左键选择/移动 | 右键取消 | E 结束回合")
        TextFont {
            font_size: FontSize::Px(14.0),
        }
        TextColor(Color::srgb(0.7, 0.7, 0.7))
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            left: Val::Px(10.0),
        }
    });

    // 单位信息面板
    commands.spawn_scene(bsn! {
        UnitInfoPanel
        NeedsFont
        Text("选择一个单位")
        TextFont {
            font_size: FontSize::Px(18.0),
        }
        TextColor(Color::WHITE)
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
        }
    });

    // 行动菜单提示（默认隐藏）
    commands.spawn_scene(bsn! {
        ActionMenuText
        NeedsFont
        Text("点击敌方单位攻击 | 右键/点击空地待机")
        TextFont {
            font_size: FontSize::Px(16.0),
        }
        TextColor(Color::srgb(1.0, 1.0, 0.5))
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(40.0),
            left: Val::Px(10.0),
        }
        Visibility::Hidden
    });

    // 战斗日志面板（右侧）
    commands.spawn_scene(bsn! {
        CombatLogPanel
        NeedsFont
        Text("")
        TextFont {
            font_size: FontSize::Px(13.0),
        }
        TextColor(Color::WHITE)
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            width: Val::Px(240.0),
        }
    });
}

/// 后置设置中文字体（bsn! FromTemplate 枚举限制的变通方案）
/// 一次性系统：找到所有 NeedsFont 实体，设置字体后移除标记
pub fn setup_ui_font(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut query: Query<(Entity, &mut TextFont), With<NeedsFont>>,
) {
    let font: Handle<Font> = asset_server.load(CN_FONT);
    for (entity, mut text_font) in &mut query {
        text_font.font = FontSource::Handle(font.clone());
        commands.entity(entity).remove::<NeedsFont>();
    }
}

/// 更新回合提示
pub fn update_turn_indicator(
    turn_state: Res<TurnState>,
    mut query: Query<&mut Text, With<TurnIndicator>>,
) {
    if turn_state.is_changed() {
        for mut text in &mut query {
            let faction_name = match turn_state.current_faction {
                Faction::Player => "玩家",
                Faction::Enemy => "敌方",
            };
            **text = format!("第 {} 回合 - {}阶段", turn_state.turn_number, faction_name);
        }
    }
}

/// 更新单位信息面板
pub fn update_unit_info(
    selected_units: Query<(&Unit, &crate::unit::UnitName), With<Selected>>,
    mut query: Query<
        &mut Text,
        (
            With<UnitInfoPanel>,
            Without<TurnIndicator>,
            Without<ActionMenuText>,
        ),
    >,
) {
    for mut text in &mut query {
        if let Ok((unit, name)) = selected_units.single() {
            **text = format!(
                "{}  HP: {}/{}  ATK: {}  DEF: {}  MOV: {}  Range: {}",
                name.0, unit.hp, unit.max_hp, unit.atk, unit.def, unit.mov, unit.attack_range
            );
        } else {
            **text = "选择一个单位".to_string();
        }
    }
}

/// 更新行动菜单可见性
pub fn update_action_menu(
    turn_phase: Res<State<TurnPhase>>,
    turn_state: Res<TurnState>,
    mut query: Query<&mut Visibility, With<ActionMenuText>>,
) {
    for mut vis in &mut query {
        *vis = if *turn_phase.get() == TurnPhase::SelectAction
            && turn_state.current_faction == Faction::Player
        {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// 更新 HP 条宽度
pub fn update_hp_bars(
    units: Query<(&Unit, &Children), With<Unit>>,
    mut hp_fgs: Query<&mut Sprite, With<HpBarFg>>,
    map: Res<crate::map::GameMap>,
) {
    let bar_width = map.tile_size * 0.6;
    for (unit, children) in &units {
        let ratio = (unit.hp as f32 / unit.max_hp as f32).max(0.0);
        for child in children.iter() {
            if let Ok(mut sprite) = hp_fgs.get_mut(child) {
                sprite.custom_size = Some(Vec2::new(bar_width * ratio, 4.0));
            }
        }
    }
}

/// 检查胜负条件
pub fn check_game_over(
    units: Query<&Unit>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut turn_indicator: Query<&mut Text, With<TurnIndicator>>,
) {
    let has_player = units.iter().any(|u| u.faction == Faction::Player);
    let has_enemy = units.iter().any(|u| u.faction == Faction::Enemy);

    if !has_enemy {
        for mut text in &mut turn_indicator {
            **text = "胜利！".to_string();
        }
        next_app_state.set(AppState::GameOver);
    } else if !has_player {
        for mut text in &mut turn_indicator {
            **text = "失败...".to_string();
        }
        next_app_state.set(AppState::GameOver);
    }
}

/// 更新战斗日志面板显示
/// 由于 Text 是 newtype 不支持富文本，用符号标记突出核心数据
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
