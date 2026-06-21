//! SkillSlot Factory — 技能槽控件的唯一创建入口
//!
//! 遵循 Factory 模式，禁止直接通过 commands.spawn 创建 UI 节点。
//! 输入 Props + Theme → 输出 Entity。所有子控件通过 Primitives 工厂函数创建。
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §9

use bevy::prelude::*;

use crate::infra::localization::generated::loc;
use crate::ui::primitives::button::{components::ButtonVariant, factory::spawn_localized_button};
use crate::ui::primitives::panel::{components::PanelVariant, factory::spawn_panel};
use crate::ui::primitives::progress_bar::{
    components::ProgressBarVariant, factory::spawn_progress_bar,
};
use crate::ui::primitives::text::{components::TextVariant, factory::spawn_text};
use crate::ui::theme::Theme;

use super::components::{SkillSlotAction, SkillSlotState};

/// 工厂函数：生成一个完整的技能槽控件
///
/// # UI 树结构
///
/// ```text
/// Panel (Card)
///   ├── Text (skill name, Caption)
///   ├── ProgressBar (cooldown, Generic)
///   └── Button ("Use", Primary) — SkillSlotAction::Use
/// ```
///
/// # 参数
/// - `commands`: ECS 命令
/// - `asset_server`: 资源管理器（传递给文本工厂）
/// - `theme`: 主题 Resource（提供颜色/间距令牌）
/// - `name`: 技能显示名称
/// - `cooldown_max`: 冷却最大值（初始时技能处于满冷却状态）
///
/// # 返回
/// 技能槽容器实体的 Entity
///
/// # 用法
/// ```ignore
/// let slot = spawn_skill_slot(
///     &mut commands, &asset_server, &theme,
///     "Fire Bolt", 12,
/// );
/// ```
pub fn spawn_skill_slot(
    commands: &mut Commands,
    asset_server: &AssetServer,
    theme: &Theme,
    name: impl Into<String>,
    cooldown_max: u32,
) -> Entity {
    let name_str: String = name.into();

    // ── 1. Container panel ──
    // Card variant provides a rounded, padded column layout
    let container = spawn_panel(commands, theme, PanelVariant::Card);

    // Attach SkillSlotState and an identifiable Name
    commands.entity(container).insert((
        SkillSlotState {
            name: name_str.clone(),
            cooldown_max,
            // Start at max cooldown (just used)
            cooldown_current: cooldown_max,
            is_ready: cooldown_max == 0,
        },
        Name::new(format!("SkillSlot({})", name_str)),
    ));

    // ── 2. Skill name text (Caption variant, secondary color) ──
    let name_text = spawn_text(
        commands,
        asset_server,
        theme,
        &name_str,
        TextVariant::Caption,
    );
    commands.entity(name_text).set_parent_in_place(container);

    // ── 3. Cooldown progress bar (Generic variant, starts full) ──
    let cooldown_bar = spawn_progress_bar(
        commands,
        theme,
        ProgressBarVariant::Generic,
        cooldown_max as f32,
        cooldown_max as f32,
        true,
        Val::Px(theme.spacing.sm),
    );
    commands.entity(cooldown_bar).set_parent_in_place(container);

    // ── 4. Use button (Primary variant, small) ──
    let use_button = spawn_localized_button(commands, theme, loc::ui::USE, "Use", ButtonVariant::Primary);
    // Attach SkillSlotAction marker for event routing
    commands.entity(use_button).insert(SkillSlotAction::Use);
    commands.entity(use_button).set_parent_in_place(container);

    container
}
