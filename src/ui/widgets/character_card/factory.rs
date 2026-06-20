//! CharacterCard Factory — 角色卡片复合控件的唯一创建入口
//!
//! 遵循 Factory 模式，禁止直接通过 commands.spawn 创建 UI 节点。
//! 输入 Props + Theme → 输出 Entity。所有子控件通过 Primitives 工厂函数创建。
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §9

use bevy::prelude::*;

use crate::ui::primitives::button::{
    components::ButtonVariant,
    factory::spawn_button,
};
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

use super::components::{CharacterAction, CharacterCardState};

/// 工厂函数：生成一个完整的角色卡片控件
///
/// # UI 树结构
///
/// ```text
/// Panel (Card)
///   ├── Text (name, Caption, primary color)
///   ├── Text (level, Caption)
///   ├── ProgressBar (Hp, show_label)
///   ├── ProgressBar (Mp, show_label)
///   ├── Button ("Attack", Primary) — CharacterAction::Attack
///   ├── Button ("Defend", Secondary) — CharacterAction::Defend
///   └── Button ("Skill", Primary) — CharacterAction::Skill
/// ```
///
/// # 参数
/// - `commands`: ECS 命令
/// - `asset_server`: 资源管理器（传递给文本工厂）
/// - `theme`: 主题 Resource（提供颜色/间距令牌）
/// - `name`: 角色显示名称
/// - `level`: 角色等级
/// - `hp_current`: 当前 HP
/// - `hp_max`: 最大 HP
/// - `mp_current`: 当前 MP
/// - `mp_max`: 最大 MP
///
/// # 返回
/// 角色卡片容器实体的 Entity
///
/// # 用法
/// ```ignore
/// let card = spawn_character_card(
///     &mut commands, &asset_server, &theme,
///     "Aria", 5, 80.0, 100.0, 40.0, 50.0,
/// );
/// ```
pub fn spawn_character_card(
    commands: &mut Commands,
    asset_server: &AssetServer,
    theme: &Theme,
    name: impl Into<String>,
    level: u32,
    hp_current: f32,
    hp_max: f32,
    mp_current: f32,
    mp_max: f32,
) -> Entity {
    let name_str: String = name.into();
    let level_str = format!("Lv.{}", level);

    // ── 1. Container panel ──
    // Card variant provides a rounded, padded column layout
    let container = spawn_panel(commands, theme, PanelVariant::Card);

    // Attach CharacterCardState and an identifiable Name
    commands.entity(container).insert((
        CharacterCardState {
            name: name_str.clone(),
            level,
            hp_current,
            hp_max,
            mp_current,
            mp_max,
        },
        Name::new(format!("CharacterCard({})", name_str)),
    ));

    // ── 2. Character name text (Caption variant, primary color) ──
    let name_text = spawn_text(commands, asset_server, theme, &name_str, TextVariant::Caption);
    commands.entity(name_text).insert(TextColor(theme.colors.text_primary));
    commands.entity(name_text).set_parent_in_place(container);

    // ── 3. Level text (Caption variant) ──
    let level_text = spawn_text(commands, asset_server, theme, &level_str, TextVariant::Caption);
    commands.entity(level_text).set_parent_in_place(container);

    // ── 4. HP progress bar (Hp variant, show label) ──
    let hp_bar = spawn_progress_bar(
        commands,
        theme,
        ProgressBarVariant::Hp,
        hp_current,
        hp_max,
        true,
        Val::Px(theme.spacing.sm),
    );
    commands.entity(hp_bar).set_parent_in_place(container);

    // ── 5. MP progress bar (Mp variant, show label) ──
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

    // ── 6. Action buttons ──
    // Attack button (Primary variant)
    let attack_btn = spawn_button(commands, theme, "Attack", ButtonVariant::Primary);
    commands.entity(attack_btn).insert(CharacterAction::Attack);
    commands.entity(attack_btn).set_parent_in_place(container);

    // Defend button (Secondary variant)
    let defend_btn = spawn_button(commands, theme, "Defend", ButtonVariant::Secondary);
    commands.entity(defend_btn).insert(CharacterAction::Defend);
    commands.entity(defend_btn).set_parent_in_place(container);

    // Skill button (Primary variant)
    let skill_btn = spawn_button(commands, theme, "Skill", ButtonVariant::Primary);
    commands.entity(skill_btn).insert(CharacterAction::Skill);
    commands.entity(skill_btn).set_parent_in_place(container);

    container
}
