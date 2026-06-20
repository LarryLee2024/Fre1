//! BuffIcon Factory — Buff 图标控件的唯一创建入口
//!
//! 遵循 Factory 模式，禁止直接通过 commands.spawn 创建 UI 节点。
//! 输入 Props + Theme → 输出 Entity。所有子控件通过 Primitives 工厂函数创建。
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §9

use bevy::prelude::*;

use crate::ui::primitives::panel::{
    components::PanelVariant,
    factory::spawn_panel,
};
use crate::ui::primitives::progress_bar::{
    components::ProgressBarVariant,
    factory::spawn_progress_bar,
};
use crate::ui::primitives::text::{
    components::TextVariant,
    factory::spawn_text,
};
use crate::ui::theme::Theme;

use super::components::BuffIconState;

/// 工厂函数：生成一个完整的 Buff/Debuff 图标控件
///
/// # UI 树结构
///
/// ```text
/// Panel (Card, colored border)
///   ├── Text (remaining turns, Caption, centered)
///   ├── Text (buff name, Caption)
///   └── ProgressBar (duration, Generic, thin)
/// ```
///
/// # 参数
/// - `commands`: ECS 命令
/// - `asset_server`: 资源管理器（传递给文本工厂）
/// - `theme`: 主题 Resource（提供颜色/间距令牌）
/// - `name`: Buff 显示名称
/// - `remaining_turns`: 剩余持续回合数
/// - `max_turns`: 最大持续回合数
/// - `is_debuff`: 是否为减益效果
///
/// # 返回
/// BuffIcon 容器实体的 Entity
///
/// # 用法
/// ```ignore
/// let icon = spawn_buff_icon(
///     &mut commands, &asset_server, &theme,
///     "Poison", 3, 5, true,
/// );
/// ```
pub fn spawn_buff_icon(
    commands: &mut Commands,
    asset_server: &AssetServer,
    theme: &Theme,
    name: impl Into<String>,
    remaining_turns: u32,
    max_turns: u32,
    is_debuff: bool,
) -> Entity {
    let name_str: String = name.into();
    let border_color = if is_debuff {
        theme.colors.feedback_negative
    } else {
        theme.colors.feedback_positive
    };

    // ── 1. Container panel (Card variant, compact square) ──
    let container = spawn_panel(commands, theme, PanelVariant::Card);

    // Override border with buff/debuff color and attach BuffIconState
    commands.entity(container).insert((
        BorderColor::all(border_color),
        BuffIconState {
            name: name_str.clone(),
            remaining_turns,
            max_turns,
            is_debuff,
        },
        Name::new(format!("BuffIcon({})", name_str)),
    ));

    // ── 2. Remaining turns text (centered, emphasis) ──
    let turns_str = remaining_turns.to_string();
    let turns_text = spawn_text(commands, asset_server, theme, &turns_str, TextVariant::Caption);
    // Override to primary color for visibility
    commands.entity(turns_text).insert(TextColor(theme.colors.text_primary));
    commands.entity(turns_text).set_parent_in_place(container);

    // ── 3. Buff name text (secondary color, small) ──
    let name_text = spawn_text(commands, asset_server, theme, &name_str, TextVariant::Caption);
    commands.entity(name_text).set_parent_in_place(container);

    // ── 4. Duration progress bar (Generic, thin, hidden label) ──
    let duration_bar = spawn_progress_bar(
        commands,
        theme,
        ProgressBarVariant::Generic,
        remaining_turns as f32,
        max_turns as f32,
        false,
        Val::Px(theme.spacing.xs),
    );
    commands.entity(duration_bar).set_parent_in_place(container);

    container
}
