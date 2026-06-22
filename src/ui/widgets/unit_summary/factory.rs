//! UnitSummary Factory — 单位摘要控件的唯一创建入口
//!
//! 遵循 Factory 模式，禁止直接通过 commands.spawn 创建 UI 节点。
//! 输入 Props + Theme → 输出 Entity。所有子控件通过 Primitives 工厂函数创建。
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §9

use bevy::prelude::*;

use crate::ui::primitives::panel::{components::PanelVariant, factory::spawn_panel};
use crate::ui::primitives::progress_bar::{
    components::ProgressBarVariant, factory::spawn_progress_bar,
};
use crate::ui::primitives::text::{components::TextVariant, factory::spawn_text};
use crate::ui::theme::Theme;

use super::components::UnitSummary;

/// 工厂函数：生成一个完整的单位摘要控件
///
/// # UI 树结构
///
/// ```text
/// Panel (Card) — "UnitSummary"
///   ├── Node (Row) — top row: name + level
///   │   ├── Text (name, Caption, primary)
///   │   └── Text (level, Caption, secondary)
///   └── ProgressBar (Hp, show_label, sm height)
/// ```
///
/// # 参数
/// - `commands`: ECS 命令
/// - `asset_server`: 资源管理器（传递给文本工厂）
/// - `theme`: 主题 Resource（提供颜色/间距令牌）
/// - `name`: 单位显示名称
/// - `level`: 单位等级
/// - `hp_current`: 当前 HP
/// - `hp_max`: 最大 HP
///
/// # 返回
/// 单位摘要容器实体的 Entity
///
/// # 用法
/// ```ignore
/// let summary = spawn_unit_summary(
///     &mut commands, &asset_server, &theme,
///     "Aria", 5, 80.0, 100.0,
/// );
/// ```
pub fn spawn_unit_summary(
    commands: &mut Commands,
    asset_server: &AssetServer,
    theme: &Theme,
    name: &str,
    level: u32,
    hp_current: f32,
    hp_max: f32,
) -> Entity {
    let level_str = format!("Lv.{}", level);
    let hp_str = format!("HP: {:.0}/{:.0}", hp_current, hp_max);

    // ── 1. Container panel (Card variant) ──
    let container = spawn_panel(commands, theme, PanelVariant::Card);

    commands
        .entity(container)
        .insert((UnitSummary, Name::new(format!("UnitSummary({})", name))));

    // ── 2. Top row: name (left) + level (right) ──
    let top_row = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.0),
                ..default()
            },
            Name::new(format!("UnitSummaryTopRow({})", name)),
        ))
        .id();
    commands.entity(top_row).set_parent_in_place(container);

    // 单位名称文本 (left, primary color)
    let name_text = spawn_text(commands, asset_server, theme, name, TextVariant::Caption);
    commands
        .entity(name_text)
        .insert(TextColor(theme.colors.text_primary));
    commands.entity(name_text).set_parent_in_place(top_row);

    // 等级文本 (right, secondary color)
    let level_text = spawn_text(
        commands,
        asset_server,
        theme,
        &level_str,
        TextVariant::Caption,
    );
    commands
        .entity(level_text)
        .insert(TextColor(theme.colors.text_secondary));
    commands.entity(level_text).set_parent_in_place(top_row);

    // ── 3. HP label text ──
    // 显示 HP 数值文本（当前/最大），位于进度条上方
    let hp_label = spawn_text(commands, asset_server, theme, &hp_str, TextVariant::Body);
    commands
        .entity(hp_label)
        .insert(TextColor(theme.colors.text_primary));
    commands.entity(hp_label).set_parent_in_place(container);

    // ── 4. HP progress bar ──
    let hp_bar = spawn_progress_bar(
        commands,
        theme,
        ProgressBarVariant::Hp,
        hp_current,
        hp_max,
        false,
        Val::Px(theme.spacing.sm),
    );
    commands.entity(hp_bar).set_parent_in_place(container);

    container
}
