//! MOD Compatibility — 兼容性管理
//!
//! 确保 MOD 之间以及 MOD 与游戏版本之间的兼容性：
//! - 版本检查：MOD 要求的最低游戏版本
//! - 兼容性矩阵：已知 MOD 之间的冲突记录
//! - 自动降级：当 MOD 冲突时自动禁用低优先级 MOD
//!
//! 参见 `docs/architecture/modding-design.md` §MOD Compatibility。
