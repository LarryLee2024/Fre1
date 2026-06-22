//! UiTextKey — UI 文本本地化 Key 常量
//!
//! 所有用户可见文本必须引用此模块中的 Key，禁止硬编码字符串。
//! Key 定义对应 `assets/localization/*/ui.ftl` 中的条目。
//!
//! 这是 `infra::localization::generated::loc` 中 UI 相关 Key 的便捷重导出，
//! 加上 UI 层独有的额外 Key。
//!
//! 详见 `docs/06-ui/02-design-system/theme-localization.md` §4

// 从生成的 localization keys 中重导出 UI 相关 Key
pub use crate::infra::localization::generated::loc::core::{
    BACK as CORE_BACK, CANCEL as CORE_CANCEL, CONFIRM as CORE_CONFIRM, EXIT as CORE_EXIT,
    LOAD as CORE_LOAD, NEXT as CORE_NEXT, NO as CORE_NO, OK as CORE_OK, SAVE as CORE_SAVE,
    YES as CORE_YES,
};
pub use crate::infra::localization::generated::loc::ui::*;

/// "Player Phase" — BattleScreen Z2 phase display (player's turn)
pub const BATTLE_PHASE_PLAYER: &str = "ui.battle.phase_player";

/// "Enemy Phase" — BattleScreen Z2 phase display (enemy's turn)
pub const BATTLE_PHASE_ENEMY: &str = "ui.battle.phase_enemy";

/// "Turn {number}" — BattleScreen Z2 turn number display (format: "Turn 1")
pub const BATTLE_TURN_NUMBER: &str = "ui.battle.turn_number";
