//! 纯数学工具（距离 / 插值 / 网格坐标）
//!
//! 提供共享层零语义数学工具：
//!
//! - **HexGrid**: 六边形网格立方体坐标，距离计算，邻居查询
//! - **FloatEq**: 带 epsilon 的浮点比较 trait
//! - **插值函数**: lerp / inv_lerp / smoothstep

mod hex;
mod interpolation;

pub use hex::{hex_distance, HexCoord};
pub use interpolation::{inv_lerp, lerp, smoothstep};

/// 浮点比较 trait —— 在 epsilon 容差内比较。
///
/// 主要用于测试和数值稳定性检查，避免直接 `==` 比较浮点数。
///
/// # 默认 epsilon
///
/// | 类型 | 默认 epsilon |
/// |------|-------------|
/// | f32  | `1e-6`      |
/// | f64  | `1e-12`     |
///
/// # 示例
///
/// ```ignore
/// use crate::shared::math::FloatEq;
///
/// assert!(0.1_f32 + 0.2_f32).float_eq(&0.3_f32, 1e-6));
/// assert!(1.0_f64.float_eq(&1.0_f64, 1e-12));
/// assert!(!1.0_f64.float_eq(&1.0001_f64, 1e-12));
/// ```
pub trait FloatEq {
    /// 在指定 epsilon 容差内比较两个值是否相等。
    fn float_eq(&self, other: &Self, epsilon: f32) -> bool;
}

impl FloatEq for f32 {
    fn float_eq(&self, other: &Self, epsilon: f32) -> bool {
        (self - other).abs() <= epsilon
    }
}

impl FloatEq for f64 {
    fn float_eq(&self, other: &Self, epsilon: f32) -> bool {
        (self - other).abs() <= epsilon as f64
    }
}

#[cfg(test)]
mod tests;
