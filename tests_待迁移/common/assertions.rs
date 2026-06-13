// 自定义断言宏

/// 断言属性值（整数比较）
#[macro_export]
macro_rules! assert_attr_eq {
    ($attrs:expr, $kind:expr, $expected:expr) => {
        let actual = $attrs.get($kind) as i32;
        let expected = $expected as i32;
        assert_eq!(
            actual, expected,
            "属性 {:?} 期望 {} 实际 {}",
            $kind, expected, actual
        );
    };
    ($attrs:expr, $kind:expr, $expected:expr, $tolerance:expr) => {
        let actual = $attrs.get($kind);
        let expected = $expected as f32;
        assert!(
            (actual - expected).abs() < $tolerance,
            "属性 {:?} 期望 {} 实际 {} (容差 {})",
            $kind,
            expected,
            actual,
            $tolerance
        );
    };
}

/// 断言 Buff 存在
#[macro_export]
macro_rules! assert_has_buff {
    ($buffs:expr, $buff_id:expr) => {
        assert!(
            $buffs.iter().any(|b| b.buff_id == $buff_id),
            "期望存在 Buff '{}'",
            $buff_id
        );
    };
}

/// 断言标签存在
#[macro_export]
macro_rules! assert_has_tag {
    ($tags:expr, $tag:expr) => {
        assert!($tags.has($tag), "期望存在标签 {:?}", $tag);
    };
}

/// 断言标签不存在
#[macro_export]
macro_rules! assert_not_has_tag {
    ($tags:expr, $tag:expr) => {
        assert!(!$tags.has($tag), "期望不存在标签 {:?}", $tag);
    };
}
