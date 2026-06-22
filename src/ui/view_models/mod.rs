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
            // 使用合理的非零默认值，使 UI 在首次投影触发前能正常渲染。
            // turn_number 从 0 开始，以便 on_turn_started 在首次激活时递增到 1；
            // phase_key 初始为空，避免在任何战斗事件前产生误导性显示。
            battle_hud: BattleHudVm {
                hp: 100.0,
                max_hp: 100.0,
                mp: 50.0,
                max_mp: 50.0,
                ap: 1.0,
                max_ap: 1.0,
                turn_number: 0,
                phase_key: "",
                current_unit_id: 0,
                is_player_controlled: false,
                is_in_battle: false,
            },
            character_panel: CharacterPanelVm::default(),
            skill_panel: SkillPanelVm::default(),
        }
    }
}
