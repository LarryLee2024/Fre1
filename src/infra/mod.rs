//! L2: Infra — 技术实现层
//!
//! 渲染/持久化/输入等"脏活"。
//! 依赖: Core (L1) + Shared (L0)
//!
//! 详见 `docs/01-architecture/README.md` §3.4

pub mod camera;
pub mod input;
pub mod localization;
pub mod logging;
pub mod map;
pub mod picking;
pub mod pipeline;
pub mod registry;
pub mod replay;
pub mod save;
