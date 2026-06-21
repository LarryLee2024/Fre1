//! 面向诊断的可读计算分解
//!
//! 提供 `Explain` trait 和 `CalcBreakdown` 结构体，用于生成
//! 计算结果推导过程的可读追踪。
//! 参见 docs/11-refactor/explain-research-report.md

use std::fmt;

/// 可读的计算分解。
///
/// 记录计算的每个步骤，包含输入参数、
/// 中间步骤和最终输出，用于诊断和调试。
///
/// # 设计
///
/// - `formula_expr` — 纯文本公式（如 "Price = Base * Reputation * Supply * Stolen"）
/// - `inputs` — 带可读值的命名参数列表
/// - `steps` — 带操作描述的标记中间结果
/// - `output` — 最终计算值（f32 保证通用兼容性）
/// - `source_doc` — 文档引用，满足 SS9（每次计算都有文档引用）
#[derive(Debug, Clone)]
pub struct CalcBreakdown {
    /// 公式表达式（如 "Price = Base * Reputation * Supply * Stolen"）。
    pub formula_expr: String,

    /// 计算中使用的输入参数。
    pub inputs: Vec<BreakdownInput>,

    /// 中间计算步骤。
    pub steps: Vec<BreakdownStep>,

    /// 最终输出值。
    pub output: f32,

    /// 可选的文档引用（满足 SS9：每次计算都有文档引用）。
    pub source_doc: Option<String>,
}

/// 计算的命名输入参数。
#[derive(Debug, Clone)]
pub struct BreakdownInput {
    /// 参数名（如 "base"、"reputation_modifier"）。
    pub name: String,

    /// 人类可读的值表示（如 "100"、"0.9 (Friendly)"）。
    pub value: String,
}

/// 计算中的单个中间步骤。
#[derive(Debug, Clone)]
pub struct BreakdownStep {
    /// 步骤标签（如 "after_reputation_discount"）。
    pub label: String,

    /// 操作描述（如 "base * 0.9 (Friendly)"）。
    pub operation: String,

    /// 此步骤后的结果值。
    pub output: f32,
}

/// 提供计算结果的可读分解。
///
/// 在任何执行多步计算的值对象上实现此 trait，
/// 以启用诊断追踪和调试显示。
///
/// # 示例
///
/// ```ignore
/// impl Explain for MyPrice {
///     fn explain(&self) -> CalcBreakdown {
///         CalcBreakdown {
///             formula_expr: "Price = Base * Markup".into(),
///             inputs: vec![BreakdownInput { name: "base".into(), value: "100".into() }],
///             steps: vec![BreakdownStep {
///                 label: "after_markup".into(),
///                 operation: "100 * 1.2".into(),
///                 output: 120.0,
///             }],
///             output: 120.0,
///             source_doc: Some("docs/02-domain/domains/my_domain.md".into()),
///         }
///     }
/// }
/// ```
/// 计算过程解释 trait。
///
/// 存在原因：伤害/治疗/经验等核心公式需要可追溯的计算过程，
/// CalcBreakdown 记录输入、中间步骤、最终结果，供 UI 伤害预览和 Debug 工具消费。
pub trait Explain: fmt::Debug {
    /// 返回该值计算过程的结构化分解。
    fn explain(&self) -> CalcBreakdown;
}
