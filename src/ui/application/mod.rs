//! Module Name: Application — UI 应用层（输入意图、命令、事件）
//!
//! UI Application 层是 UI 层与 Domain 层的桥梁，负责：
//! - 将原始输入抽象为语义化意图 (UiIntent)
//! - Widget 交互输出声明 (UiAction)
//! - UI 到 Domain 的命令转换 (UiCommand -> GameCommand)
//! - UI 内部事件广播 (UiEvent)
//!
//! ScreenType 由 `crate::ui::navigation::ScreenType` 提供。
//! 参见 docs/06-ui/01-architecture/application-layer.md

pub mod action;
pub mod command;
pub mod event;
pub mod intent;

pub use crate::ui::navigation::ScreenType;
pub use action::UiAction;
pub use command::UiCommand;
pub use event::UiEvent;
pub use intent::UiIntent;
