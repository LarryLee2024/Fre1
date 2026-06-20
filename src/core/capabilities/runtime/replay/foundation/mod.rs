//! Replay Foundation — 回放基础类型与值对象

pub(crate) mod error;
pub(crate) mod traits;
pub(crate) mod types;
pub(crate) mod values;

pub use error::ReplayError;
pub use traits::{ReplayAction, Replayable};
pub use types::{AbilityTarget, ReplayCommand, ReplayFrame, ReplayHeader, RngSeeds, RngStream};
pub use values::{
    DeterministicRng, ReplayLog, ReplayMismatch, ReplayMode, ReplayModeGuard, ReplayPlayer,
    ReplayRecorder, ReplayValidator,
};
