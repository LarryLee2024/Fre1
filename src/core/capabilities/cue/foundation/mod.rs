//! Cue Foundation — 表现信号基础类型与值对象

pub(crate) mod types;
pub(crate) mod values;

pub use types::{
    AnimationParams, CueData, CueDef, CueError, CueTag, CueType, PopupDirection, PopupParams,
    SFXParams, ShakeFalloff, ShakeParams, VFXParams,
};
pub use values::{CueBinding, CueContainer};
