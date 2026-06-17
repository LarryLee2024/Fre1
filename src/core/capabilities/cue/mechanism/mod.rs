//! Cue Mechanism — 信号管理与分发逻辑

pub mod components;
pub mod dispatch;

pub use components::CueContainerComponent;
pub use dispatch::{DispatchResult, DispatchTarget, can_trigger, collect_cues, dispatch_cue};
