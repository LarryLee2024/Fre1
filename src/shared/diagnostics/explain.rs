//! Human-readable calculation breakdown for diagnostics
//!
//! Provides the `Explain` trait and `CalcBreakdown` struct for generating
//! human-readable traces of how a calculation result was derived.
//! See docs/11-refactor/explain-research-report.md

use std::fmt;

/// Human-readable calculation breakdown.
///
/// Documents each step of a calculation with input parameters,
/// intermediate steps, and final output for diagnostics and debugging.
///
/// # Design
///
/// - `formula_expr` — plain-text formula (e.g. "Price = Base * Reputation * Supply * Stolen")
/// - `inputs` — named parameter list with human-readable values
/// - `steps` — labeled intermediate results with operation descriptions
/// - `output` — final computed value (f32 for universal compatibility)
/// - `source_doc` — document reference, satisfying SS9 (every calc has a doc reference)
#[derive(Debug, Clone)]
pub struct CalcBreakdown {
    /// Formula expression (e.g. "Price = Base * Reputation * Supply * Stolen").
    pub formula_expr: String,

    /// Input parameters used in the calculation.
    pub inputs: Vec<BreakdownInput>,

    /// Intermediate calculation steps.
    pub steps: Vec<BreakdownStep>,

    /// Final output value.
    pub output: f32,

    /// Optional document reference (satisfies SS9: every calc has a doc reference).
    pub source_doc: Option<String>,
}

/// Named input parameter to a calculation.
#[derive(Debug, Clone)]
pub struct BreakdownInput {
    /// Parameter name (e.g. "base", "reputation_modifier").
    pub name: String,

    /// Human-readable value representation (e.g. "100", "0.9 (Friendly)").
    pub value: String,
}

/// A single intermediate step in a calculation.
#[derive(Debug, Clone)]
pub struct BreakdownStep {
    /// Step label (e.g. "after_reputation_discount").
    pub label: String,

    /// Operation description (e.g. "base * 0.9 (Friendly)").
    pub operation: String,

    /// Result value after this step.
    pub output: f32,
}

/// Provides a human-readable breakdown of a calculation result.
///
/// Implement on any value object that performs multi-step calculations
/// to enable diagnostic tracing and debug display.
///
/// # Example
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
pub trait Explain: fmt::Debug {
    /// Returns a structured breakdown of how this value was calculated.
    fn explain(&self) -> CalcBreakdown;
}
