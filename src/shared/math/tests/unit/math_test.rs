use crate::shared::math::{hex_distance, inv_lerp, lerp, smoothstep, FloatEq, HexCoord};

// ============================================================================
// HexCoord 测试
// ============================================================================

#[test]
fn hex_distance_zero_is_zero() {
    assert_eq!(hex_distance((0, 0), (0, 0)), 0);
    assert_eq!(hex_distance((5, -3), (5, -3)), 0);
}

#[test]
fn hex_distance_adjacent_is_one() {
    // 相邻六边形距离为 1
    assert_eq!(hex_distance((0, 0), (1, 0)), 1);
    assert_eq!(hex_distance((0, 0), (0, 1)), 1);
    assert_eq!(hex_distance((0, 0), (1, -1)), 1);
}

#[test]
fn hex_distance_known_values() {
    assert_eq!(hex_distance((0, 0), (2, 1)), 3);
    assert_eq!(hex_distance((0, 0), (3, -1)), 4);
    assert_eq!(hex_distance((-3, 2), (1, -1)), 5);
}

#[test]
fn hex_distance_is_symmetric() {
    let a = (4, -2);
    let b = (-1, 3);
    assert_eq!(hex_distance(a, b), hex_distance(b, a));
}

#[test]
fn hex_distance_triangle_inequality() {
    // 三角不等式：d(a,c) <= d(a,b) + d(b,c)
    let a = (0, 0);
    let b = (3, 1);
    let c = (-2, 4);
    let d_ab = hex_distance(a, b);
    let d_bc = hex_distance(b, c);
    let d_ac = hex_distance(a, c);
    // 对于立方体距离，等式成立
    assert!(d_ac <= d_ab + d_bc);
}

#[test]
fn hex_coord_new_and_fields() {
    let coord = HexCoord::new(3, -5);
    assert_eq!(coord.q, 3);
    assert_eq!(coord.r, -5);
}

#[test]
fn hex_coord_from_tuple() {
    let coord = HexCoord::from((2, 4));
    assert_eq!(coord.q, 2);
    assert_eq!(coord.r, 4);
}

#[test]
fn hex_coord_distance_to() {
    let a = HexCoord::new(0, 0);
    let b = HexCoord::new(2, 1);
    assert_eq!(a.distance_to(&b), 3);
    assert_eq!(b.distance_to(&a), 3);
}

#[test]
fn hex_coord_add() {
    let a = HexCoord::new(1, 2);
    let b = HexCoord::new(3, -4);
    let sum = a + b;
    assert_eq!(sum.q, 4);
    assert_eq!(sum.r, -2);
}

#[test]
fn hex_coord_sub() {
    let a = HexCoord::new(5, 3);
    let b = HexCoord::new(2, 1);
    let diff = a - b;
    assert_eq!(diff.q, 3);
    assert_eq!(diff.r, 2);
}

#[test]
fn hex_coord_neighbors_count() {
    let center = HexCoord::new(0, 0);
    let neighbors = center.neighbors();
    assert_eq!(neighbors.len(), 6);
}

#[test]
fn hex_coord_neighbors_all_adjacent() {
    let center = HexCoord::new(0, 0);
    let neighbors = center.neighbors();
    for neighbor in &neighbors {
        assert_eq!(center.distance_to(neighbor), 1);
    }
}

#[test]
fn hex_coord_neighbors_distinct() {
    let center = HexCoord::new(0, 0);
    let neighbors = center.neighbors();
    for i in 0..6 {
        for j in (i + 1)..6 {
            assert_ne!(neighbors[i], neighbors[j]);
        }
    }
}

#[test]
fn hex_coord_neighbors_contains_expected() {
    let center = HexCoord::new(3, -1);
    let neighbors = center.neighbors();
    let expected = [
        HexCoord::new(4, -1),
        HexCoord::new(2, -1),
        HexCoord::new(3, 0),
        HexCoord::new(3, -2),
        HexCoord::new(4, -2),
        HexCoord::new(2, 0),
    ];
    for exp in &expected {
        assert!(neighbors.contains(exp), "Missing neighbor: ({}, {})", exp.q, exp.r);
    }
}

#[test]
fn hex_coord_neighbors_ring_returns_to_origin() {
    // 围绕一个中心六边形走一圈应该回到原点
    let center = HexCoord::new(0, 0);
    let mut pos = HexCoord::new(1, 0); // 从第一个邻居开始
    // 绕中心走一圈：右-上-左-左-下-右 形成一个环
    let path = [
        HexCoord::new(0, 1),   // 右上
        HexCoord::new(-1, 1),  // 左上
        HexCoord::new(-1, 0),  // 左
        HexCoord::new(0, -1),  // 左下
        HexCoord::new(1, -1),  // 右下
        HexCoord::new(1, 0),   // 回到起点
    ];
    for step in &path {
        pos = pos + *step;
    }
    assert_eq!(pos, center);
}

#[test]
fn hex_coord_hash_map_key() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(HexCoord::new(0, 0));
    set.insert(HexCoord::new(1, 2));
    set.insert(HexCoord::new(0, 0)); // 重复
    assert_eq!(set.len(), 2);
}

#[test]
fn hex_coord_debug_output() {
    let coord = HexCoord::new(-1, 5);
    let debug = format!("{:?}", coord);
    assert!(debug.contains("-1"));
    assert!(debug.contains("5"));
}

// ============================================================================
// FloatEq 测试
// ============================================================================

#[test]
fn float_eq_f32_exact() {
    assert!(3.14_f32.float_eq(&3.14_f32, 1e-6));
}

#[test]
fn float_eq_f32_within_epsilon() {
    assert!((1.0_f32 + 2.0_f32).float_eq(&3.0_f32, 1e-6));
}

#[test]
fn float_eq_f32_outside_epsilon() {
    assert!(!1.0_f32.float_eq(&1.001_f32, 1e-6));
}

#[test]
fn float_eq_f32_negative() {
    assert!((-1.5_f32).float_eq(&-1.5_f32, 1e-6));
    assert!(!(-1.5_f32).float_eq(&-1.6_f32, 1e-6));
}

#[test]
fn float_eq_f64_exact() {
    assert!(1.0_f64.float_eq(&1.0_f64, 1e-12));
}

#[test]
fn float_eq_f64_within_epsilon() {
    assert!((1.0_f64 / 3.0_f64 * 3.0_f64).float_eq(&1.0_f64, 1e-6));
}

#[test]
fn float_eq_f64_outside_epsilon() {
    assert!(!1.0_f64.float_eq(&1.000_000_1_f64, 1e-12));
}

// ============================================================================
// 插值函数测试
// ============================================================================

#[test]
fn lerp_midpoint() {
    let result = lerp(0.0, 10.0, 0.5);
    assert!((result - 5.0).abs() < 1e-6);
}

#[test]
fn lerp_start() {
    let result = lerp(0.0, 10.0, 0.0);
    assert!((result - 0.0).abs() < 1e-6);
}

#[test]
fn lerp_end() {
    let result = lerp(0.0, 10.0, 1.0);
    assert!((result - 10.0).abs() < 1e-6);
}

#[test]
fn lerp_negative_t() {
    let result = lerp(0.0, 10.0, -0.5);
    assert!((result - (-5.0)).abs() < 1e-6);
}

#[test]
fn lerp_overshoot() {
    let result = lerp(0.0, 10.0, 1.5);
    assert!((result - 15.0).abs() < 1e-6);
}

#[test]
fn lerp_reversed_range() {
    // 当 a > b 时，lerp 仍然正确
    let result = lerp(10.0, 0.0, 0.5);
    assert!((result - 5.0).abs() < 1e-6);
}

#[test]
fn inv_lerp_midpoint() {
    let result = inv_lerp(0.0, 10.0, 5.0);
    assert!((result - 0.5).abs() < 1e-6);
}

#[test]
fn inv_lerp_start() {
    let result = inv_lerp(0.0, 10.0, 0.0);
    assert!((result - 0.0).abs() < 1e-6);
}

#[test]
fn inv_lerp_end() {
    let result = inv_lerp(0.0, 10.0, 10.0);
    assert!((result - 1.0).abs() < 1e-6);
}

#[test]
fn inv_lerp_outside_range() {
    let result = inv_lerp(0.0, 10.0, 15.0);
    assert!((result - 1.5).abs() < 1e-6);
}

#[test]
fn inv_lerp_zero_range() {
    // a == b 时应当返回 0.0
    let result = inv_lerp(5.0, 5.0, 5.0);
    assert_eq!(result, 0.0);
    let result = inv_lerp(5.0, 5.0, 10.0);
    assert_eq!(result, 0.0);
}

#[test]
fn lerp_inv_lerp_roundtrip() {
    // lerp 和 inv_lerp 互为逆运算
    let a = 2.0;
    let b = 8.0;
    for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let v = lerp(a, b, t);
        let t_back = inv_lerp(a, b, v);
        assert!((t - t_back).abs() < 1e-6);
    }
}

#[test]
fn smoothstep_midpoint() {
    // smoothstep(0, 1, 0.5) = 0.5
    let result = smoothstep(0.0, 1.0, 0.5);
    assert!((result - 0.5).abs() < 1e-6);
}

#[test]
fn smoothstep_below_edge0() {
    let result = smoothstep(0.0, 1.0, -0.5);
    assert_eq!(result, 0.0);
}

#[test]
fn smoothstep_above_edge1() {
    let result = smoothstep(0.0, 1.0, 1.5);
    assert_eq!(result, 1.0);
}

#[test]
fn smoothstep_slope_at_edges() {
    // smoothstep 在 edge0 和 edge1 处的斜率为 0
    // 检查靠近边缘处的值
    let near_0 = smoothstep(0.0, 1.0, 0.001);
    let near_1 = smoothstep(0.0, 1.0, 0.999);

    // 在起始处增长非常慢（斜率为 0）
    assert!(near_0 < 0.0001);

    // 在末尾处增长也非常慢（斜率为 0）
    assert!(near_1 > 0.9999);
}

#[test]
fn smoothstep_monotonic() {
    // smoothstep 在 [0,1] 上单调递增
    let mut prev = smoothstep(0.0, 1.0, 0.0);
    for i in 1..=100 {
        let x = i as f32 / 100.0;
        let curr = smoothstep(0.0, 1.0, x);
        assert!(curr >= prev, "smoothstep not monotonic at x={}", x);
        prev = curr;
    }
}

#[test]
fn smoothstep_negative_range() {
    // 当 edge0 > edge1 时，结果可能异常，但不会 panic
    let result = smoothstep(1.0, 0.0, 0.5);
    assert!((result - 0.5).abs() < 1e-6);
}

#[test]
fn smoothstep_zero_range() {
    // edge0 == edge1 时返回 0
    let result = smoothstep(5.0, 5.0, 5.0);
    assert_eq!(result, 0.0);
}
