//! Command Mechanism — 命令分发逻辑

pub mod dispatch;

pub use dispatch::{CommandHandler, dispatch_batch, dispatch_command, validate_command};
