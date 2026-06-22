//! CharacterStatusPanel Factory — 角色状态面板复合控件的唯一创建入口
//!
//! 遵循 Factory 模式，禁止直接通过 commands.spawn 创建 UI 节点。
//! 输入 Props + Theme → 输出 Entity。所有子控件通过 Primitives 工厂函数创建。
//!
//! 组合 CharacterPortrait / Text / ProgressBar 为完整的角色状态展示区域。
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md` §3.2

use bevy::prelude::*;

use crate::ui::binding::Dirty;
use crate::ui::primitives::panel::{components::PanelVariant, factory::spawn_panel};
use crate::ui::primitives::progress_bar::{
    components::ProgressBarVariant, factory::spawn_progress_bar,
};
use crate::ui::primitives::text::{components::TextVariant, factory::spawn_text};
use crate::ui::theme::Theme;
use crate::ui::view_models::character_panel::CharacterPanelVm;
use crate::ui::widgets::character_portrait::PortraitBorder;
use crate::ui::widgets::character_portrait::spawn_character_portrait;

use super::components::{
    CharacterStatusPanel, CharacterStatusPanelNameLabel, CharacterStatusPanelState,
    CharacterStatusPanelStatusLabel,
};

/// 工厂函数：生成一个完整的角色状态面板控件
///
/// # UI 树结构
///
/// ```text
/// Panel (Card) — CharacterStatusPanel
///   ├── Panel (Basic, Row) — Top section
///   │   ├── CharacterPortrait (placeholder avatar + border)
///   │   └── Panel (Basic, Column) — Info column
///   │       ├── Text (name, Body, primary color)
///   │       └── ProgressBar (HP, Hp variant, show_label)
///   ├── ProgressBar (MP, Mp variant, show_label)
///   ├── ProgressBar (AP, Generic variant, show_label) — 仅 ap_max > 0 时
///   └── Text (status_text, Caption, centered) — 仅 Some 时
/// ```
///
/// # 参数
/// - `commands`: ECS 命令
/// - `asset_server`: 资源管理器（传递给子工厂）
/// - `theme`: 主题 Resource（提供颜色/间距令牌）
/// - `name`: 角色显示名称
/// - `hp_current`: 当前 HP
/// - `hp_max`: 最大 HP
/// - `mp_current`: 当前 MP
/// - `mp_max`: 最大 MP
/// - `ap_current`: 当前 AP（0 时隐藏 AP 条）
/// - `ap_max`: 最大 AP（0 时隐藏 AP 条）
/// - `status_text`: 可选状态文本（如"待机中""移动中"），None 时不渲染
/// - `is_active`: 是否为当前行动角色（影响肖像边框样式）
///
/// # 返回
/// 角色状态面板容器实体的 Entity
///
/// # 用法
/// ```ignore
/// let panel = spawn_character_status_panel(
///     &mut commands, &asset_server, &theme,
///     "Aria", 80.0, 100.0, 40.0, 50.0, 3.0, 5.0,
///     Some("待机中"), true,
/// );
/// ```
#[allow(clippy::too_many_arguments)]
pub fn spawn_character_status_panel(
    commands: &mut Commands,
    asset_server: &AssetServer,
    theme: &Theme,
    name: impl Into<String>,
    hp_current: f32,
    hp_max: f32,
    mp_current: f32,
    mp_max: f32,
    ap_current: f32,
    ap_max: f32,
    status_text: Option<impl Into<String>>,
    is_active: bool,
) -> Entity {
    let name_str: String = name.into();
    let status_text_str: Option<String> = status_text.map(|s| s.into());

    // ── 1. Container panel (Card variant) ──
    // Card 变体提供大圆角、带内边距的垂直列布局
    let container = spawn_panel(commands, theme, PanelVariant::Card);

    // 挂载 CharacterStatusPanelState、Dirty<CharacterPanelVm>（用于 ViewModel 刷新）
    // 以及可识别的 Name
    commands.entity(container).insert((
        CharacterStatusPanel,
        CharacterStatusPanelState {
            name: name_str.clone(),
            hp_current,
            hp_max,
            mp_current,
            mp_max,
            ap_current,
            ap_max,
            status_text: status_text_str.clone(),
            is_active,
        },
        Dirty::<CharacterPanelVm>::default(),
        Name::new(format!("CharacterStatusPanel({})", name_str)),
    ));

    // ── 2. Top section (horizontal row: portrait + info column) ──
    let top_section = spawn_panel(commands, theme, PanelVariant::Basic);
    commands.entity(top_section).insert((
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(theme.spacing.sm),
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
    ));
    commands.entity(top_section).set_parent_in_place(container);

    // ── 2a. CharacterPortrait (placeholder block with border) ──
    let portrait_border = if is_active {
        PortraitBorder::Active
    } else {
        PortraitBorder::Inactive
    };
    // 使用主题强调色作为默认头像占位色
    let portrait = spawn_character_portrait(
        commands,
        theme,
        portrait_border,
        theme.colors.accent_primary,
    );
    commands.entity(portrait).set_parent_in_place(top_section);

    // ── 2b. Info column (vertical: name + HP bar) ──
    let info_column = spawn_panel(commands, theme, PanelVariant::Basic);
    commands.entity(info_column).insert((
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(theme.spacing.xs),
            flex_grow: 1.0,
            ..default()
        },
        BackgroundColor(Color::NONE),
    ));
    commands
        .entity(info_column)
        .set_parent_in_place(top_section);

    // ── 2c. Character name text (Body variant, primary color) ──
    let name_text = spawn_text(commands, asset_server, theme, &name_str, TextVariant::Body);
    commands.entity(name_text).insert((
        TextColor(theme.colors.text_primary),
        CharacterStatusPanelNameLabel,
    ));
    commands.entity(name_text).set_parent_in_place(info_column);

    // ── 2d. HP progress bar (Hp variant, show label) ──
    let hp_bar = spawn_progress_bar(
        commands,
        theme,
        ProgressBarVariant::Hp,
        hp_current,
        hp_max,
        true,
        Val::Px(theme.spacing.sm),
    );
    commands.entity(hp_bar).set_parent_in_place(info_column);

    // ── 3. MP progress bar (Mp variant, show label) ──
    let mp_bar = spawn_progress_bar(
        commands,
        theme,
        ProgressBarVariant::Mp,
        mp_current,
        mp_max,
        true,
        Val::Px(theme.spacing.sm),
    );
    commands.entity(mp_bar).set_parent_in_place(container);

    // ── 4. AP progress bar (Generic variant, show label) — only if ap_max > 0 ──
    if ap_max > 0.0 {
        let ap_bar = spawn_progress_bar(
            commands,
            theme,
            ProgressBarVariant::Generic,
            ap_current,
            ap_max,
            true,
            Val::Px(theme.spacing.sm),
        );
        commands.entity(ap_bar).set_parent_in_place(container);
    }

    // ── 5. Status text (Caption variant, centered) — only if present ──
    if let Some(ref status) = status_text_str {
        let status_txt = spawn_text(commands, asset_server, theme, status, TextVariant::Caption);
        commands.entity(status_txt).insert((
            TextColor(theme.colors.text_secondary),
            Node {
                align_self: AlignSelf::Center,
                ..default()
            },
            CharacterStatusPanelStatusLabel,
        ));
        commands.entity(status_txt).set_parent_in_place(container);
    }

    container
}
