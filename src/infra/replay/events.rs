//! Infra 层回放事件
//!
//! 直接 re-export 核心层的回放事件，方便 Infra 层消费者使用。
//! 所有事件定义见 `src/core/capabilities/runtime/replay/events.rs`。

pub use crate::core::capabilities::runtime::replay::events::{
    RecordingCompleted, RecordingStarted, ReplayCompleted, ReplayFrameProcessed,
    ReplayMismatchDetected, ReplayStarted,
};
