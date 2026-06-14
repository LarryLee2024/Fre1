//! MOD Validators — 校验器
//!
//! 在 MOD 加载前执行多层校验：
//! - Schema 校验：RON 格式与字段完整性
//! - 引用完整性：所有引用的 ID 必须存在
//! - 冲突检测：不同 MOD 之间的定义冲突
//!
//! 校验失败时提供清晰的错误信息，阻止有问题的 MOD 加载。
//!
//! 参见 `docs/architecture/modding-design.md` §MOD Validators。
