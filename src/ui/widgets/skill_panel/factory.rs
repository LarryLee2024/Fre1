//! SkillPanel Factory — 技能面板复合控件的唯一创建入口
//!
//! 遵循 Factory 模式，禁止直接通过 commands.spawn 创建 UI 节点。
//! 组合多个 SkillSlot 为一个技能面板容器。
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §9

use bevy::prelude::*;

use crate::ui::primitives::panel::{components::PanelVariant, factory::spawn_panel};
use crate::ui::theme::Theme;
use crate::ui::widgets::skill_slot::components::SkillSlotState;
use crate::ui::widgets::skill_slot::factory::spawn_skill_slot;

use super::components::{SkillPanel, SkillSlotIndex};

/// 工厂函数：生成一个完整的技能面板控件（包含 3 个技能槽）
///
/// # UI 树结构
///
/// ```text
/// Panel (Group) — SkillPanel
///   ├── Panel (Card) — SkillSlot #1 (Attack, id=1, cd=0)
///   │     ├── Text (skill name, Caption)
///   │     ├── ProgressBar (cooldown, Generic)
///   │     └── Button ("Use", Primary)
///   ├── Panel (Card) — SkillSlot #2 (Fireball, id=2, cd=3)
///   │     ├── Text (skill name, Caption)
///   │     ├── ProgressBar (cooldown, Generic)
///   │     └── Button ("Use", Primary)
///   └── Panel (Card) — SkillSlot #3 (Heal, id=3, cd=2)
///         ├── Text (skill name, Caption)
///         ├── ProgressBar (cooldown, Generic)
///         └── Button ("Use", Primary)
/// ```
///
/// # 参数
/// - `commands`: ECS 命令
/// - `asset_server`: 资源管理器（传递给 SkillSlot 工厂）
/// - `theme`: 主题 Resource（提供颜色/间距令牌）
///
/// # 返回
/// 技能面板容器实体的 Entity
///
/// # 用法
/// ```ignore
/// let panel = spawn_skill_panel(&mut commands, &asset_server, &theme);
/// ```
pub fn spawn_skill_panel(
    commands: &mut Commands,
    asset_server: &AssetServer,
    theme: &Theme,
) -> Entity {
    // ── 1. Container panel (Group variant) ──
    // Group 提供分组容器样式，适合包裹多个子卡片
    let container = spawn_panel(commands, theme, PanelVariant::Group);
    commands
        .entity(container)
        .insert((SkillPanel, Name::new("SkillPanel")));

    // ── 2. Skill slots — matching SkillPanelVm::default() ──
    // skill_id=1: "ui.skill.attack"   cooldown_max=0 (no cooldown)
    // skill_id=2: "ui.skill.fireball" cooldown_max=3
    // skill_id=3: "ui.skill.heal"     cooldown_max=2
    //
    // SkillPanelVm::default() 按此顺序提供默认技能数据，
    // 进入战斗后由投影函数 (on_effect_applied, on_turn_started_for_skills) 更新。
    let specs: [(u32, &str, u32); 3] = [
        (1, "ui.skill.attack", 0),
        (2, "ui.skill.fireball", 3),
        (3, "ui.skill.heal", 2),
    ];

    for &(skill_id, name, cooldown_max) in &specs {
        let slot = spawn_skill_slot(commands, asset_server, theme, name, cooldown_max);
        // Override SkillSlotState.skill_id from default 0 to match SkillPanelVm keys,
        // so that refresh_skill_slot_from_vm can find the correct VM entry.
        commands.entity(slot).insert(SkillSlotState {
            name: name.to_string(),
            skill_id,
            cooldown_max,
            cooldown_current: cooldown_max,
            is_ready: cooldown_max == 0,
        });
        commands
            .entity(slot)
            .insert(SkillSlotIndex(skill_id as usize));
        commands.entity(slot).set_parent_in_place(container);
    }

    container
}
