//! L0: Shared — 原子层
//!
//! 零业务语义、零技术语义、零框架语义的通用编程原子工具。
//! 依赖: 无（最底层）
//!
//! 详见 `docs/01-architecture/README.md` §3.1

// [ADR-045] pub(crate) — 通用集合扩展，crate 内共享
pub(crate) mod collections;
// [ADR-045] pub(crate) — 错误上下文工具，crate 内共享
pub(crate) mod error;
// [ADR-045] pub(crate) — 非加密高速哈希，crate 内共享
pub(crate) mod hashing;
// [ADR-045] pub(crate) — 强类型 ID，crate 内共享
pub(crate) mod ids;
// [ADR-045] pub(crate) — 纯数学工具，crate 内共享
pub(crate) mod math;
// [ADR-045] pub(crate) — 路径工具，crate 内共享
pub(crate) mod path;
// [ADR-045] pub — 统一导出，对外可见
pub mod prelude;
// [ADR-045] pub(crate) — 确定性随机数，crate 内共享
pub(crate) mod random;
// [ADR-045] pub — 插件入口，对外可见
pub mod shared_plugin;
// [ADR-045] #[cfg(test)] — 测试构建工具，仅测试可见
#[cfg(test)]
pub mod testing;
// [ADR-045] pub(crate) — GameTime, TurnCount，crate 内共享
pub(crate) mod time;
// [ADR-045] pub(crate) — 横切能力抽象，crate 内共享
pub(crate) mod traits;
// [ADR-045] pub(crate) — 链式校验器，crate 内共享
pub(crate) mod validation;
// [ADR-045] pub(crate) — 日志诊断基础设施，crate 内共享
pub(crate) mod diagnostics;
// [ADR-045] pub(crate) — 全局常量，crate 内共享
pub(crate) mod constants;
// [ADR-045] pub(crate) — 本地化 Key 类型，crate 内共享
pub(crate) mod localization_key;
// [ADR-045] pub(crate) — 通用宏定义，crate 内共享
pub(crate) mod macros;
// [ADR-050] pub(crate) — 游戏状态枚举，所有层共享
pub(crate) mod game_state;

pub use shared_plugin::SharedPlugin;
