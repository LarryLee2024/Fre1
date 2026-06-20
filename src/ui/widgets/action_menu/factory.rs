//! ActionMenu Factory — 行动菜单控件的唯一创建入口
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
use crate::ui::primitives::list::{
    components::ListVariant,
    factory::spawn_list,
};
use crate::ui::theme::Theme;

use super::components::{ActionMenuItem, ActionMenuState, ActionType};

/// 工厂函数：生成一个完整的战斗行动菜单控件
///
/// # UI 树结构
///
/// ```text
/// List (Vertical)
///   ├── Button (\"Attack\", Primary)  — ActionType::Attack
///   ├── Button (\"Defend\", Secondary) — ActionType::Defend
///   ├── Button (\"Skill\", Primary)    — ActionType::Skill
///   ├── Button (\"Item\", Secondary)   — ActionType::Item
///   └── Button (\"Wait\", Ghost)       — ActionType::Wait
/// ```
///
/// # 参数
/// - `commands`: ECS 命令
/// - `theme`: 主题 Resource（提供颜色/间距令牌）
///
/// # 返回
/// 行动菜单容器实体的 Entity
///
/// # 用法
/// ```ignore
/// let menu = spawn_action_menu(&mut commands, &theme);
/// ```
pub fn spawn_action_menu(
    commands: &mut Commands,
    theme: &Theme,
) -> Entity {
    // ── 1. Container list (Vertical) ──
    let container = spawn_list(commands, theme, ListVariant::Vertical);

    // Build default action items
    let actions = vec![
        ActionMenuItem {
            label: "Attack".to_string(),
            action_type: ActionType::Attack,
            enabled: true,
        },
        ActionMenuItem {
            label: "Defend".to_string(),
            action_type: ActionType::Defend,
            enabled: true,
        },
        ActionMenuItem {
            label: "Skill".to_string(),
            action_type: ActionType::Skill,
            enabled: true,
        },
        ActionMenuItem {
            label: "Item".to_string(),
            action_type: ActionType::Item,
            enabled: true,
        },
        ActionMenuItem {
            label: "Wait".to_string(),
            action_type: ActionType::Wait,
            enabled: true,
        },
    ];

    // Attach ActionMenuState and an identifiable Name
    commands.entity(container).insert((
        ActionMenuState {
            actions,
        },
        Name::new("ActionMenu"),
    ));

    // ── 2. Attack button (Primary) ──
    let attack_btn = spawn_button(commands, theme, "Attack", ButtonVariant::Primary);
    commands.entity(attack_btn).insert(ActionType::Attack);
    commands.entity(attack_btn).set_parent_in_place(container);

    // ── 3. Defend button (Secondary) ──
    let defend_btn = spawn_button(commands, theme, "Defend", ButtonVariant::Secondary);
    commands.entity(defend_btn).insert(ActionType::Defend);
    commands.entity(defend_btn).set_parent_in_place(container);

    // ── 4. Skill button (Primary) ──
    let skill_btn = spawn_button(commands, theme, "Skill", ButtonVariant::Primary);
    commands.entity(skill_btn).insert(ActionType::Skill);
    commands.entity(skill_btn).set_parent_in_place(container);

    // ── 5. Item button (Secondary) ──
    let item_btn = spawn_button(commands, theme, "Item", ButtonVariant::Secondary);
    commands.entity(item_btn).insert(ActionType::Item);
    commands.entity(item_btn).set_parent_in_place(container);

    // ── 6. Wait button (Ghost) ──
    let wait_btn = spawn_button(commands, theme, "Wait", ButtonVariant::Ghost);
    commands.entity(wait_btn).insert(ActionType::Wait);
    commands.entity(wait_btn).set_parent_in_place(container);

    container
}
