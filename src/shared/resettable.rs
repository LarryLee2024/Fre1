/// ResettableResource trait — 统一资源重置机制（Phase 4.4）
///
/// 所有需要随 InGame 状态退出而重置的运行时资源实现此 trait。
/// 替代 `turn/mod.rs` 中 13 个重复的 `insert_resource(Default::default())` 调用。
///
/// 使用方式：
/// ```ignore
/// #[derive(Resource, Default)]
/// struct MyResource { ... }
///
/// impl ResettableResource for MyResource {}
/// ```
pub trait ResettableResource: Default {
    /// 将资源重置为默认值
    fn reset(&mut self) {
        *self = Self::default();
    }
}

/// 批量重置所有实现了 ResettableResource 的资源
///
/// 在 cleanup_ingame 中调用此函数替代手动重置。
pub fn reset_resources<T: ResettableResource>(resource: &mut T) {
    resource.reset();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct TestResource {
        value: i32,
    }

    impl ResettableResource for TestResource {}

    #[test]
    fn resettable_resource_重置为默认值() {
        let mut res = TestResource { value: 42 };
        res.reset();
        assert_eq!(res.value, 0);
    }
}
