//! 插值函数
//!
//! 提供常用插值工具函数，适用于动画、过渡和数值平滑。

/// 线性插值。
///
/// 计算 `a` 和 `b` 之间在参数 `t` 处的插值：
/// `result = a + (b - a) * t`
///
/// 参数 `t` 通常在 `[0, 1]` 范围内，但不会钳制。
///
/// # 示例
///
/// ```ignore
/// assert!((lerp(0.0, 10.0, 0.5) - 5.0).abs() < 1e-6);
/// assert!((lerp(0.0, 10.0, 0.0) - 0.0).abs() < 1e-6);
/// assert!((lerp(0.0, 10.0, 1.0) - 10.0).abs() < 1e-6);
/// ```
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// 反线性插值。
///
/// 计算值 `v` 在范围 `[a, b]` 中的归一化位置：
/// `t = (v - a) / (b - a)`
///
/// 当 `a == b` 时返回 `0.0`，避免除零。
///
/// # 示例
///
/// ```ignore
/// assert!((inv_lerp(0.0, 10.0, 5.0) - 0.5).abs() < 1e-6);
/// assert!((inv_lerp(0.0, 10.0, 0.0) - 0.0).abs() < 1e-6);
/// assert!((inv_lerp(0.0, 10.0, 10.0) - 1.0).abs() < 1e-6);
/// assert_eq!(inv_lerp(5.0, 5.0, 5.0), 0.0); // 零范围返回 0
/// ```
pub fn inv_lerp(a: f32, b: f32, v: f32) -> f32 {
    let denom = b - a;
    if denom == 0.0 {
        return 0.0;
    }
    (v - a) / denom
}

/// Hermite 平滑插值 (smoothstep)。
///
/// 使用 Hermite 三次插值在 `edge0` 和 `edge1` 之间平滑过渡：
/// 在 `edge0` 处斜率为 0，在 `edge1` 处斜率为 0。
///
/// 返回 `[0, 1]` 范围内的值，并在边界外钳制。
///
/// 公式：`t = clamp((x - edge0) / (edge1 - edge0), 0, 1)`，
/// `result = t * t * (3 - 2 * t)`
///
/// # 示例
///
/// ```ignore
/// assert!((smoothstep(0.0, 1.0, 0.5) - 0.5).abs() < 1e-6);
/// assert_eq!(smoothstep(0.0, 1.0, 0.0), 0.0);
/// assert_eq!(smoothstep(0.0, 1.0, 1.0), 1.0);
/// ```
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    // 零范围：edge0 == edge1 时直接返回 0.0，避免除零
    if (edge1 - edge0).abs() <= f32::EPSILON {
        return 0.0;
    }
    // 钳制 t 到 [0, 1]
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    // Hermite 三次：t^2 * (3 - 2t)
    t * t * (3.0 - 2.0 * t)
}
