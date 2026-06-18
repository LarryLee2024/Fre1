//! 法术业务规则模块
//!
//! 包含纯函数形式的施法校验、专注管理、升环规则。
//! 详见 docs/02-domain/domains/spell_domain.md §3, §5

mod formulas;
mod rules;
pub use formulas::*;
pub use rules::*;
