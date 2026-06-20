//! Module Name: rules — 战斗领域纯函数业务规则
//!
//! 包含战斗核心逻辑的策略实现，零 ECS 依赖。
//! 所有函数均为纯函数，输入输出为值类型。
//!
//! # 策略模块
//!
//! | 模块 | 职责 |
//! |------|------|
//! | `damage_policy` | 伤害计算：公式选择、暴击、减免 |
//! | `target_policy` | 目标选择：合法性判定、范围/视线/标签过滤 |

pub(crate) mod damage_policy;
pub(crate) mod target_policy;
