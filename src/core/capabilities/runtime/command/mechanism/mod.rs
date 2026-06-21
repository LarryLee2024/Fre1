//! Command Mechanism — 命令分发逻辑

pub(crate) mod dispatch;
pub(crate) mod processor;

pub use dispatch::{CommandHandler, dispatch_batch, dispatch_command, validate_command};
