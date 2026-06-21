//! UiStore — 统一 ViewModel 容器
//!
//! 架构防火墙：Widget 只能通过 UiStore 消费数据，禁止直接 Query Domain。
//! Projection 纯函数写入 UiStore 并标记 Dirty，Widget 系统检测后按需刷新。
//!
//! 详见 `docs/06-ui/04-data-flow/projection-viewmodel.md` §5

use bevy::prelude::*;

use self::battle_hud::BattleHudVm;
use self::character_panel::CharacterPanelVm;
use self::skill_panel::SkillPanelVm;

pub mod battle_hud;
pub mod character_panel;
pub mod skill_panel;

/// UiStore —— 统一 ViewModel 容器
///
/// # 架构原则
/// - Widget 的唯一数据源（禁止 Query<&DomainComponent>）
/// - Projection 写入 + mark_dirty()
/// - Widget 系统 consume() → 刷新
#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
pub struct UiStore {
    /// 战斗 HUD 数据
    pub battle_hud: BattleHudVm,
    /// 角色面板数据
    pub character_panel: CharacterPanelVm,
    /// 技能面板数据
    pub skill_panel: SkillPanelVm,
}

impl Default for UiStore {
    fn default() -> Self {
        Self {
            battle_hud: BattleHudVm::default(),
            character_panel: CharacterPanelVm::default(),
            skill_panel: SkillPanelVm::default(),
        }
    }
}
