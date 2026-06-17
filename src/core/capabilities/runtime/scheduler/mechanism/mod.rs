//! Scheduler Mechanism — 调度执行逻辑

pub mod executor;

pub use executor::{
    TickSequence, advance_to_next_phase, advance_to_next_turn, execute_tick, is_phase, total_frames,
};
