//! MOD API — 稳定公开接口
//!
//! MOD 作者唯一需要了解的部分。提供安全的添加/扩展能力：
//! - 添加新技能、Buff、装备
//! - 注册新内容到 Registry
//!
//! ## 设计原则
//!
//! - API 必须保持向后兼容（SemVer）
//! - 所有公开函数必须是纯数据操作（无副作用）
//! - 禁止暴露 Core 内部类型
//!
//! 参见 `docs/architecture/modding-design.md` §MOD API。
