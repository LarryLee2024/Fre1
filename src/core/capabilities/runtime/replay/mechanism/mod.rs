//! Replay Mechanism — 录制与回放逻辑

pub(crate) mod player;
pub(crate) mod recorder;
pub(crate) mod resources;

pub use player::{PlaybackSession, fast_forward};
pub use recorder::{
    RecordingSession, calculate_frame_checksum, validate_frame_sequence, validate_version,
};
pub use resources::{FrameCounter, ReplayModeGuard as EcsReplayModeGuard};
