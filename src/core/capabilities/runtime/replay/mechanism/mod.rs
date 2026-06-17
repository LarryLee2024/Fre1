//! Replay Mechanism — 录制与回放逻辑

pub mod player;
pub mod recorder;

pub use player::{PlaybackSession, fast_forward};
pub use recorder::{
    RecordingSession, calculate_frame_checksum, validate_frame_sequence, validate_version,
};
