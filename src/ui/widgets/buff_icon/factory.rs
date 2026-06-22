//! BuffIcon Factory — Buff 图标控件的唯一创建入口
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

use super::components::{BuffIconState, BuffType};

/// 根据 BuffType 返回对应的语义边框颜色
fn border_color_for_buff_type(buff_type: BuffType, theme: &Theme) -> Color {
    match buff_type {
        BuffType::Buff => theme.colors.feedback_positive,
        BuffType::Debuff => theme.colors.feedback_negative,
        BuffType::Neutral => theme.colors.feedback_warning,
    }
}

/// 工厂函数：生成一个完整的 Buff/Debuff 图标控件
///
/// # UI 树结构
///
/// ```text
/// Panel (Card, colored border based on buff_type)
///   ├── Text (remaining turns, Caption, centered)
///   ├── Text (buff name, Caption)
///   ├── Text (stacks badge, Caption, top-right) — 仅 stacks > 0 时
///   └── ProgressBar (duration, Generic, thin)
/// ```
///
/// # 参数
/// - `commands`: ECS 命令
/// - `asset_server`: 资源管理器（传递给文本工厂）
/// - `theme`: 主题 Resource（提供颜色/间距令牌）
/// - `name`: Buff 显示名称
/// - `buff_type`: Buff 类型（增益/减益/中性）
/// - `remaining_turns`: 剩余持续回合数
/// - `max_turns`: 最大持续回合数
/// - `stacks`: 叠加层数（0 = 不显示徽章）
/// - `tooltip_key`: 悬浮提示的本地化 key（MVP 阶段可空）
///
/// # 返回
/// BuffIcon 容器实体的 Entity
///
/// # 用法
/// ```ignore
/// let icon = spawn_buff_icon(
///     &mut commands, &asset_server, &theme,
///     "Poison", BuffType::Debuff, 3, 5, 2, "",
/// );
/// ```
pub fn spawn_buff_icon(
    commands: &mut Commands,
    asset_server: &AssetServer,
    theme: &Theme,
    name: impl Into<String>,
    buff_type: BuffType,
    remaining_turns: u32,
    max_turns: u32,
    stacks: u32,
    tooltip_key: impl Into<String>,
) -> Entity {
    let name_str: String = name.into();
    let tooltip_key_str: String = tooltip_key.into();
    let border_color = border_color_for_buff_type(buff_type, theme);

    // ── 1. Container panel (Card variant, compact square) ──
    let container = spawn_panel(commands, theme, PanelVariant::Card);

    // 根据 Buff 类型设置边框颜色并挂载 BuffIconState
    commands.entity(container).insert((
        BorderColor::all(border_color),
        BuffIconState {
            name: name_str.clone(),
            buff_type,
            remaining_turns,
            max_turns,
            stacks,
            tooltip_key: tooltip_key_str,
        },
        Name::new(format!("BuffIcon({})", name_str)),
    ));

    // ── 2. Remaining turns text (centered, emphasis) ──
    let turns_str = remaining_turns.to_string();
    let turns_text = spawn_text(
        commands,
        asset_server,
        theme,
        &turns_str,
        TextVariant::Caption,
    );
    // 覆盖为主色调以提高可见性
    commands
        .entity(turns_text)
        .insert(TextColor(theme.colors.text_primary));
    commands.entity(turns_text).set_parent_in_place(container);

    // ── 3. Buff name text (secondary color, small) ──
    let name_text = spawn_text(
        commands,
        asset_server,
        theme,
        &name_str,
        TextVariant::Caption,
    );
    commands.entity(name_text).set_parent_in_place(container);

    // ── 4. Stacks badge (top-right corner, only if stacks > 0) ──
    if stacks > 0 {
        let stacks_str = stacks.to_string();
        let stacks_text = spawn_text(
            commands,
            asset_server,
            theme,
            &stacks_str,
            TextVariant::Caption,
        );
        commands.entity(stacks_text).insert((
            TextColor(theme.colors.text_accent),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(2.0),
                top: Val::Px(2.0),
                ..default()
            },
        ));
        commands.entity(stacks_text).set_parent_in_place(container);
    }

    // ── 5. Duration progress bar (Generic, thin, hidden label) ──
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
