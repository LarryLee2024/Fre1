//! Cue Foundation — 表现信号基础类型与值对象

pub(crate) mod error;
pub(crate) mod types;
pub(crate) mod values;

pub use types::{
    AnimationParams, CueData, CueDef, CueTag, CueType, PopupDirection, PopupParams, SFXParams,
    ShakeFalloff, ShakeParams, VFXParams,
};
pub use values::{CueBinding, CueContainer};
