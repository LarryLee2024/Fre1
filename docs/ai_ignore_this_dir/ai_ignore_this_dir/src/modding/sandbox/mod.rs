//! MOD Sandbox — 沙箱环境
//!
//! 限制 MOD 执行权限，防止恶意 MOD 破坏游戏：
//! - 文件系统访问限制（仅允许 MOD 自己的目录）
//! - 网络访问限制（默认禁止）
//! - 内存与性能上限（防止 MOD 导致 OOM）
//! - API 调用审计（记录所有 MOD 的 API 调用）
//!
//! 参见 `docs/architecture/modding-design.md` §MOD Sandbox。
