//! Module Name: UI — 表现层 (L3)
//!
//! 最高层，依赖 Core（L1）和 Infra（L2），但不被任何下层依赖。
//! 职责：将领域状态投影为 UI，管理 Screen/Widget/Overlay。
//!
//! 三层架构：Primitives → Widgets → Screens
//! 详见 `docs/06-ui/` 架构文档

pub mod plugin;
pub mod primitives;
pub mod screens;
pub mod theme;
pub mod widgets;

pub use plugin::UiPlugin;
pub use primitives::{
    button::{
        components::{ButtonInteraction, ButtonState, ButtonVariant},
        events::ButtonClicked,
        factory::spawn_button,
    },
    list::{
        components::{ListState, ListVariant},
        factory::spawn_list,
    },
    modal::{
        components::{ModalState, ModalVariant},
        events::{ModalCancelled, ModalConfirmed},
        factory::spawn_modal,
    },
    panel::{
        components::{PanelState, PanelVariant},
        factory::spawn_panel,
    },
    progress_bar::{
        components::{ProgressBarState, ProgressBarVariant},
        factory::spawn_progress_bar,
    },
    text::{
        components::{TextVariant, TextWidget},
        factory::spawn_text,
    },
};
pub use theme::{Theme, ThemePlugin, UiColors, UiSpacing, UiTypography};
pub use widgets::skill_slot::components::{SkillSlotAction, SkillSlotState};
pub use widgets::skill_slot::factory::spawn_skill_slot;
