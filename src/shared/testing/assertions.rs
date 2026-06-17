//! 自定义测试断言
//!
//! 提供数值容差、状态不变量、错误类型校验等断言宏。

/// 数值容差断言（f32 比较）。
///
/// # Examples
/// ```ignore
/// assert_approx_eq!(result, 30.0, 0.01);
/// ```
#[macro_export]
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr, $epsilon:expr) => {
        let left_val = $left;
        let right_val = $right;
        let eps = $epsilon;
        assert!(
            (left_val - right_val).abs() <= eps,
            "assertion failed: |{} - {}| = {} > {}",
            left_val,
            right_val,
            (left_val - right_val).abs(),
            eps
        );
    };
    ($left:expr, $right:expr) => {
        $crate::assert_approx_eq!($left, $right, 0.001);
    };
}

/// HP 不变量断言：HP >= 0。
#[macro_export]
macro_rules! assert_hp_non_negative {
    ($hp:expr) => {
        assert!($hp >= 0.0, "HP invariant violated: hp = {} < 0", $hp);
    };
}

/// 效果阶段断言。
#[macro_export]
macro_rules! assert_effect_stage {
    ($effect:expr, $expected:expr) => {
        assert_eq!(
            $effect.stage, $expected,
            "Effect stage mismatch: expected {:?}, got {:?}",
            $expected, $effect.stage
        );
    };
}

/// 结果类型断言：成功。
#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {
        assert!($result.is_ok(), "expected Ok, got Err: {:?}", $result.err());
    };
}

/// 结果类型断言：失败。
#[macro_export]
macro_rules! assert_err {
    ($result:expr) => {
        assert!($result.is_err(), "expected Err, got Ok");
    };
}

/// 结果类型断言：失败且匹配特定错误变体。
#[macro_export]
macro_rules! assert_err_matches {
    ($result:expr, $pattern:pat) => {
        match $result {
            Err($pattern) => {}
            Err(other) => panic!(
                "error variant mismatch: expected pattern {:?}, got {:?}",
                stringify!($pattern),
                other
            ),
            Ok(val) => panic!("expected Err, got Ok({:?})", val),
        }
    };
}

/// Tag 位掩码包含断言。
#[macro_export]
macro_rules! assert_tag_contains {
    ($mask:expr, $bit:expr) => {
        assert!(
            $mask & (1u128 << $bit) != 0,
            "tag bit {} not set in mask {:b}",
            $bit,
            $mask
        );
    };
}

/// Tag 位掩码不包含断言。
#[macro_export]
macro_rules! assert_tag_not_contains {
    ($mask:expr, $bit:expr) => {
        assert!(
            $mask & (1u128 << $bit) == 0,
            "tag bit {} unexpectedly set in mask {:b}",
            $bit,
            $mask
        );
    };
}
