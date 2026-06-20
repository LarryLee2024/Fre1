//! sinks — 日志输出后端
//!
//! 提供文件输出与 JSON 格式化能力。
//! - `file_sink`: 可轮转的文件写入器 + tracing-subscriber Layer
//! - console: 控制台输出（由 tracing-subscriber 默认处理）

pub(crate) mod file_sink;

pub use file_sink::{FileSink, FileSinkConfig, FileSinkLayer};
