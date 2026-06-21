//! facade — 本地化的横切编排。
//!
//! 组合存储（数据库 + 缓存）用于缓存解析。

pub(crate) mod resolve;

pub use resolve::resolve_cached;
