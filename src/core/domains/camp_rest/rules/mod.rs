//! 营地/休息业务规则模块
//!
//! 包含纯函数形式的休息规则、生命骰计算规则。
//! 详见 docs/02-domain/domains/camp_rest_domain.md §3, §5

mod formulas;
mod rules;
pub use formulas::*;
pub use rules::*;
