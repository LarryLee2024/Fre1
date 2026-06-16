//! L0: Shared — 原子层
//!
//! 零业务语义、零技术语义、零框架语义的通用编程原子工具。
//! 依赖: 无（最底层）
//!
//! 详见 `docs/01-architecture/README.md` §3.1

pub mod collections;
pub mod error;
pub mod hashing;
pub mod ids;
pub mod math;
pub mod path;
pub mod prelude;
pub mod random;
pub mod shared_plugin;
pub mod testing;
pub mod time;
pub mod traits;
pub mod validation;

pub use shared_plugin::SharedPlugin;
