//! 反应业务规则模块
//!
//! 包含纯函数形式的反应触发校验、优先级计算、反制判定规则。
//! 详见 docs/02-domain/domains/reaction_domain.md §3, §5

mod rules;
pub use rules::*;
