//! MOD Loaders — 内容加载器
//!
//! 负责从 MOD 包中加载内容并合并到游戏：
//! - 读取 MOD 的 RON 配置文件
//! - 合并基础内容与 MOD 内容（MOD 内容优先级更高）
//! - 确保加载过程不影响运行中游戏状态
//!
//! 参见 `docs/architecture/modding-design.md` §MOD Loaders。
